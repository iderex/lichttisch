//! Putting the plan on disk, and saying exactly what went there.
//!
//! The bytes of a file are a function of its planned content seed and its
//! length, so two runs of one seed write the same bytes. The digest below is
//! how that is checked without comparing two trees by hand: it is folded over
//! every path and every byte, in path order, while the files are being
//! written.
//!
//! It is a Fowler-Noll-Vo fold and not a cryptographic digest. It is enough to
//! notice that two runs differ and it is not evidence against anybody who is
//! trying to make two different trees agree. Nothing here needs the second
//! property, and claiming it would be the more expensive mistake.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use crate::plan::{Defect, HEADER_BUDGET, Plan, PlannedFile};

/// What a written file opens with, so a reader can tell the corpus from a
/// photograph without opening a photograph.
const MAGIC: &[u8] = b"LICHTTISCH-CORPUS-V1";
/// The same number of bytes, and not the magic. A file that carries this is
/// the damage a reader has to survive rather than a file it can read.
const BROKEN_MAGIC: &[u8] = b"????????????????????";

const CHUNK: usize = 64 * 1024;

/// What a run put on disk.
pub(crate) struct Written {
    pub files: usize,
    pub bytes: u64,
    pub directories: usize,
    /// `None` where nothing was written and no byte was generated.
    pub digest: Option<u64>,
}

/// Write the whole plan under `root`.
///
/// With `dry` set, nothing is created and no content byte is generated. That
/// is the only way to ask what a full-size corpus would cost without paying
/// for it, and it is why the digest is absent from such a run rather than
/// invented for it.
pub(crate) fn put(plan: &Plan, root: &Path, dry: bool) -> io::Result<Written> {
    let mut bytes = 0_u64;
    let mut made: HashSet<String> = HashSet::new();
    let mut digest = FNV_OFFSET;

    for file in &plan.files {
        bytes += file.written_len;
        let directory = file.path.rsplit_once('/').map_or("", |(head, _)| head);
        if !made.contains(directory) {
            if !dry && !directory.is_empty() {
                fs::create_dir_all(root.join(directory))?;
            }
            made.insert(directory.to_owned());
        }
        if dry {
            continue;
        }
        digest = fold(digest, file.path.as_bytes());
        digest = fold(digest, &file.written_len.to_le_bytes());
        digest = write_one(root, file, digest)?;
    }

    Ok(Written {
        files: plan.files.len(),
        bytes,
        directories: made.len(),
        digest: if dry { None } else { Some(digest) },
    })
}

/// One file: its header, its filler, its length and its timestamp.
fn write_one(root: &Path, file: &PlannedFile, mut digest: u64) -> io::Result<u64> {
    let target = root.join(&file.path);
    let handle = File::create(&target)?;
    let mut out = BufWriter::with_capacity(CHUNK, handle);

    let header = header_of(file);
    let mut remaining = file.written_len;
    let budget = u64::try_from(HEADER_BUDGET).unwrap_or(0);
    let head = usize::try_from(remaining.min(budget)).unwrap_or(0);
    out.write_all(&header[..head])?;
    digest = fold(digest, &header[..head]);
    remaining = remaining.saturating_sub(u64::try_from(head).unwrap_or(remaining));

    let mut filler = Filler::new(file.content_seed);
    let mut buffer = vec![0_u8; CHUNK];
    while remaining > 0 {
        let take = usize::try_from(remaining).unwrap_or(CHUNK).min(CHUNK);
        filler.fill(&mut buffer[..take]);
        out.write_all(&buffer[..take])?;
        digest = fold(digest, &buffer[..take]);
        remaining = remaining.saturating_sub(u64::try_from(take).unwrap_or(remaining));
    }

    out.flush()?;
    // The timestamp is part of the shape of the load: an importer that reads
    // it is reading something clustered the way a shoot clusters rather than
    // the moment this corpus was generated.
    out.get_ref()
        .set_modified(UNIX_EPOCH + Duration::from_secs(file.captured_unix))?;
    Ok(digest)
}

/// The opening bytes, padded to the budget so every reader can rely on where
/// the filler starts.
fn header_of(file: &PlannedFile) -> [u8; HEADER_BUDGET] {
    let magic = if file.defect == Defect::BadHeader {
        BROKEN_MAGIC
    } else {
        MAGIC
    };
    let model = file.model;
    let captured = file.captured_unix;
    let declared = file.declared_len;
    let session = file.session;
    let burst = file.burst;
    let text = format!(
        " model={model} captured={captured} declared={declared} session={session} burst={burst}"
    );

    let mut header = [b' '; HEADER_BUDGET];
    let magic_len = magic.len().min(header.len());
    header[..magic_len].copy_from_slice(&magic[..magic_len]);
    let room = header.len() - magic_len - 1;
    let body = text.as_bytes();
    let body_len = body.len().min(room);
    header[magic_len..magic_len + body_len].copy_from_slice(&body[..body_len]);
    let last = header.len() - 1;
    header[last] = b'\n';
    header
}

/// The filler bytes, from the content seed and nothing else.
struct Filler(u64);

impl Filler {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn fill(&mut self, buffer: &mut [u8]) {
        for chunk in buffer.chunks_mut(8) {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let word = self.0.rotate_right(23).to_le_bytes();
            let take = chunk.len().min(word.len());
            chunk[..take].copy_from_slice(&word[..take]);
        }
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

fn fold(mut digest: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{MAGIC, put};
    use crate::plan::{Defect, Params, build};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn params(files: usize, seed: u64) -> Params {
        Params {
            files,
            seed,
            cards: 6,
            years: 4,
            malformed_permille: 20,
            byte_divisor: 200_000,
        }
    }

    /// A directory of this run's own, under the system temporary directory.
    /// No dependency, and no two tests sharing a path.
    fn scratch(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("lichttisch-corpus-{}-{label}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("a scratch directory");
        path
    }

    /// Every file under `root`, as (relative path, bytes).
    fn walk(root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(here) = stack.pop() {
            let Ok(entries) = fs::read_dir(&here) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let (Ok(relative), Ok(bytes)) = (path.strip_prefix(root), fs::read(&path))
                {
                    found.push((relative.to_string_lossy().replace('\\', "/"), bytes));
                }
            }
        }
        found.sort_by(|left, right| left.0.cmp(&right.0));
        found
    }

    /// The digest of one exact corpus, measured once and written down.
    ///
    /// Two runs in one process agreeing proves the writer holds no state
    /// between them. It does not prove that a second MACHINE produces the same
    /// bytes, because both runs are on this one. This constant is what carries
    /// that half: every machine that runs the suite folds the same corpus and
    /// compares against the same number, so a difference in word size, in byte
    /// order or in a path separator arrives as a red test on the machine that
    /// has it rather than as a corpus nobody can reproduce.
    ///
    /// It is tied to the parameters below and to the layout of a written file.
    /// A deliberate change to either is a change to this number, argued in the
    /// commit that makes it.
    const RECORDED_DIGEST: u64 = 12_179_757_594_739_747_953;

    #[test]
    fn two_runs_of_one_seed_write_the_same_bytes() {
        // This is the whole claim the corpus rests on. A generator that came
        // out differently on a second run would make every number measured
        // against it a number about one machine on one day.
        let plan = build(params(300, 41));
        let first = scratch("same-a");
        let second = scratch("same-b");
        let left = put(&plan, &first, false).expect("the first run writes");
        let right = put(&plan, &second, false).expect("the second run writes");

        assert_eq!(left.digest, right.digest, "the digests disagree");
        assert_eq!(
            left.digest,
            Some(RECORDED_DIGEST),
            "this machine folded a different corpus from the one written down"
        );
        assert_eq!(walk(&first), walk(&second), "the trees disagree");
        assert_eq!(left.files, 300);

        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }

    #[test]
    fn a_different_seed_writes_a_different_corpus() {
        let here = scratch("seed-a");
        let there = scratch("seed-b");
        let left = put(&build(params(300, 41)), &here, false).expect("writes");
        let right = put(&build(params(300, 42)), &there, false).expect("writes");
        assert_ne!(left.digest, right.digest);

        let _ = fs::remove_dir_all(&here);
        let _ = fs::remove_dir_all(&there);
    }

    #[test]
    fn a_file_is_as_long_as_the_plan_says_and_damaged_where_the_plan_says() {
        let plan = build(params(300, 43));
        let root = scratch("lengths");
        put(&plan, &root, false).expect("writes");

        let mut checked_bad_header = 0;
        for file in &plan.files {
            let bytes = fs::read(root.join(&file.path)).expect("the file is there");
            assert_eq!(
                u64::try_from(bytes.len()).unwrap_or(0),
                file.written_len,
                "{} is the wrong length",
                file.path
            );
            match file.defect {
                Defect::BadHeader => {
                    assert!(
                        !bytes.starts_with(MAGIC),
                        "{} claims to be readable and is the damaged case",
                        file.path
                    );
                    checked_bad_header += 1;
                }
                Defect::None => assert!(bytes.starts_with(MAGIC), "{} lost its header", file.path),
                Defect::Truncated => {}
            }
        }
        assert!(checked_bad_header > 0, "no damaged header to check");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_dry_run_creates_nothing_and_claims_no_digest() {
        // The only way to ask what a full-size corpus costs without paying for
        // it. A dry run that reported a digest would be reporting one it did
        // not compute.
        let plan = build(params(300, 47));
        let root = scratch("dry");
        let report = put(&plan, &root, true).expect("a dry run");
        assert!(report.digest.is_none());
        assert_eq!(report.files, 300);
        assert!(report.bytes > 0);
        assert!(walk(&root).is_empty(), "a dry run put something on disk");

        let _ = fs::remove_dir_all(&root);
    }
}
