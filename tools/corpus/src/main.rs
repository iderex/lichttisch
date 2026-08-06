//! The measurement corpus (#4).
//!
//! The readme claims a hundred thousand-plus images. Nothing on this board may
//! assert a number about that scale without a corpus that produced it, and a
//! corpus of real photographs cannot be shipped: they are large, and they are
//! other people's pictures, which is the thing this project exists not to move
//! around. So it is generated from a seed rather than collected, and a second
//! machine reproduces it rather than downloading it.
//!
//! What is in the tree is the generator, the seed and the recorded sizes. The
//! corpus itself is never committed: the default output sits under `target/`,
//! which `.gitignore` excludes, and a test in this crate holds that.
//!
//! This corpus holds no photograph. Nothing measured on it says anything about
//! culling quality, only about catalogue, metadata and input-output behaviour.

mod plan;
mod write;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use plan::{Defect, Params, civil_from_unix};

/// The seed a run uses when nobody chose one. Stated here so a run with no
/// arguments is still a run somebody else can repeat.
const DEFAULT_SEED: u64 = 4_242_424_242;
/// A size that fits anywhere. The hundred thousand file run is asked for on
/// the command line, so nobody fills a disk by typing the command without
/// reading it.
const DEFAULT_FILES: usize = 5_000;
const DEFAULT_OUT: &str = "target/corpus";

const USAGE: &str = "\
usage: cargo run --locked --release -p corpus -- [options]

  --out <path>               where the corpus goes (default target/corpus)
  --files <n>                how many files (default 5000)
  --seed <n>                 the seed everything comes from (default 4242424242)
  --cards <n>                how many cards the frames came off (default 6)
  --years <n>                how many years the timestamps span (default 4)
  --malformed-permille <n>   damaged files per thousand (default 10)
  --byte-divisor <n>         divide every planned length by this (default 1)
  --dry-run                  say what it would cost and write nothing
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("corpus: {message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let Some(asked) = read_arguments(std::env::args().skip(1)) else {
        eprint!("{USAGE}");
        return Ok(ExitCode::from(2));
    };

    let root = PathBuf::from(&asked.out);
    if !asked.dry {
        refuse_an_occupied_directory(&root)?;
    }

    let started = Instant::now();
    let plan = plan::build(asked.params);
    let report = write::put(&plan, &root, asked.dry)
        .map_err(|error| format!("writing under {}: {error}", root.display()))?;
    let elapsed = started.elapsed();

    print_conditions(&asked);
    print_corpus(&plan, &report, elapsed.as_secs_f64());
    Ok(ExitCode::SUCCESS)
}

/// Everything one run was asked for.
struct Asked {
    params: Params,
    out: String,
    dry: bool,
}

/// Read the arguments, or `None` where one of them was not understood.
///
/// An argument nobody understood is refused rather than ignored. A run that
/// silently dropped `--files 100000` would report a corpus of five thousand
/// under a heading somebody read as a hundred thousand.
fn read_arguments(arguments: impl Iterator<Item = String>) -> Option<Asked> {
    let mut asked = Asked {
        params: Params {
            files: DEFAULT_FILES,
            seed: DEFAULT_SEED,
            cards: 6,
            years: 4,
            malformed_permille: 10,
            byte_divisor: 1,
        },
        out: DEFAULT_OUT.to_owned(),
        dry: false,
    };

    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        let mut value = || arguments.next();
        match argument.as_str() {
            "--dry-run" => asked.dry = true,
            "--out" => asked.out = value()?,
            "--files" => asked.params.files = value()?.parse().ok()?,
            "--seed" => asked.params.seed = value()?.parse().ok()?,
            "--cards" => asked.params.cards = value()?.parse().ok()?,
            "--years" => asked.params.years = value()?.parse().ok()?,
            "--malformed-permille" => asked.params.malformed_permille = value()?.parse().ok()?,
            "--byte-divisor" => asked.params.byte_divisor = value()?.parse().ok()?,
            _ => return None,
        }
    }

    if asked.params.files == 0 || asked.params.malformed_permille > 1000 {
        return None;
    }
    Some(asked)
}

/// Refuse to write into a directory that already holds something.
///
/// Nothing here removes anything. An operator who pointed this at a directory
/// with their own files in it gets a refusal naming the directory, not a
/// generator deciding on their behalf what may go.
fn refuse_an_occupied_directory(root: &Path) -> Result<(), String> {
    let Ok(mut entries) = std::fs::read_dir(root) else {
        return Ok(());
    };
    if entries.next().is_some() {
        return Err(format!(
            "{} is not empty. Remove it yourself and run again; this never deletes anything.",
            root.display()
        ));
    }
    Ok(())
}

/// Every condition the numbers below were produced under.
///
/// The wall clock is a fact about a machine, so the machine is printed beside
/// it. `host_identity` is absent on purpose and says so: two result files can
/// be compared without either of them naming a host, and that is a limit of
/// this tool rather than a fact about the runs.
fn print_conditions(asked: &Asked) {
    let params = asked.params;
    let cpus = std::thread::available_parallelism().map_or_else(
        |_| "not readable".to_owned(),
        |count| count.get().to_string(),
    );
    println!("conditions");
    println!("    arch={}", std::env::consts::ARCH);
    println!("    cpus={cpus}");
    println!("    host_identity=not recorded");
    println!("    os={}", std::env::consts::OS);
    println!(
        "    profile={}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    println!("    byte_divisor={}", params.byte_divisor.max(1));
    println!("    cards={}", params.cards.max(1));
    println!("    dry_run={}", asked.dry);
    println!("    files_asked_for={}", params.files);
    println!("    malformed_permille={}", params.malformed_permille);
    println!("    out={}", asked.out);
    println!("    seed={}", params.seed);
    println!("    years={}", params.years.max(1));
    println!();
}

fn print_corpus(plan: &plan::Plan, report: &write::Written, seconds: f64) {
    let declared: u64 = plan.files.iter().map(|file| file.declared_len).sum();
    let truncated = plan
        .files
        .iter()
        .filter(|file| file.defect == Defect::Truncated)
        .count();
    let bad_header = plan
        .files
        .iter()
        .filter(|file| file.defect == Defect::BadHeader)
        .count();

    let mut names: Vec<&str> = plan
        .files
        .iter()
        .map(|file| file.path.rsplit_once('/').map_or("", |(_, name)| name))
        .collect();
    names.sort_unstable();
    let mut distinct = 0_usize;
    // Files that share their filename with at least one other file, counted as
    // files rather than as names, because the case an import path has to
    // survive is a file arriving under a name that is already taken.
    let mut sharing = 0_usize;
    let mut index = 0;
    while index < names.len() {
        let mut run = index + 1;
        while run < names.len() && names[run] == names[index] {
            run += 1;
        }
        distinct += 1;
        if run - index > 1 {
            sharing += run - index;
        }
        index = run;
    }

    let deepest = plan
        .files
        .iter()
        .map(|file| file.path.matches('/').count())
        .max()
        .unwrap_or_default();
    let earliest = plan
        .files
        .iter()
        .map(|file| file.captured_unix)
        .min()
        .unwrap_or_default();
    let latest = plan
        .files
        .iter()
        .map(|file| file.captured_unix)
        .max()
        .unwrap_or_default();

    println!("corpus");
    println!("    bursts={}", plan.bursts);
    println!("    bytes_declared={declared}");
    println!("    bytes_written={}", report.bytes);
    println!("    damaged_bad_header={bad_header}");
    println!("    damaged_truncated={truncated}");
    println!("    deepest_path={deepest}");
    println!("    directories={}", report.directories);
    println!("    distinct_filenames={distinct}");
    println!("    earliest={}", as_date(earliest));
    println!("    files={}", report.files);
    println!("    files_sharing_a_filename={sharing}");
    println!("    latest={}", as_date(latest));
    println!("    sessions={}", plan.sessions);
    println!();

    match report.digest {
        Some(digest) => println!("digest={digest:016x}"),
        // A dry run generated no byte, so it has no digest to report. Printing
        // one it had not computed is the failure this line exists against.
        None => println!("digest=not computed: a dry run writes nothing and generates nothing"),
    }
    println!("elapsed={seconds:.3} s");
    println!();
    println!("This corpus holds no photograph. Nothing measured on it says anything about culling");
    println!("quality, only about catalogue, metadata and input-output behaviour.");
}

fn as_date(unix: u64) -> String {
    let (year, month, day) = civil_from_unix(unix);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{DEFAULT_OUT, read_arguments};

    fn arguments(words: &[&str]) -> Option<super::Asked> {
        read_arguments(words.iter().map(|word| (*word).to_owned()))
    }

    #[test]
    fn an_argument_nobody_understood_is_refused() {
        // A run that silently dropped an option would report a corpus under a
        // heading somebody read as a different one.
        assert!(arguments(&["--flies", "100"]).is_none());
        assert!(arguments(&["--files", "not-a-number"]).is_none());
        assert!(arguments(&["--files"]).is_none());
        assert!(arguments(&["--files", "0"]).is_none());
        assert!(arguments(&["--malformed-permille", "1001"]).is_none());
    }

    #[test]
    fn the_defaults_are_the_ones_the_usage_states() {
        let asked = arguments(&[]).expect("no arguments is a valid run");
        assert_eq!(asked.out, DEFAULT_OUT);
        assert_eq!(asked.params.files, 5_000);
        assert!(!asked.dry);
    }

    #[test]
    fn the_default_output_is_somewhere_git_ignores() {
        // The corpus is never committed. This reads the declaration rather
        // than trusting it: remove `/target` from .gitignore and this goes
        // red, which is the only reason the sentence above is worth writing.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf);
        let Some(root) = root else {
            panic!("the workspace root is two levels above this crate");
        };
        let Ok(ignored) = std::fs::read_to_string(root.join(".gitignore")) else {
            panic!("the tree carries a .gitignore");
        };
        let head = DEFAULT_OUT.split('/').next().unwrap_or(DEFAULT_OUT);
        assert!(
            ignored
                .lines()
                .any(|line| line.trim() == format!("/{head}")),
            "the default output {DEFAULT_OUT} is not under an ignored directory"
        );
    }

    #[test]
    fn every_option_is_read() {
        let asked = arguments(&[
            "--out",
            "somewhere",
            "--files",
            "11",
            "--seed",
            "5",
            "--cards",
            "2",
            "--years",
            "3",
            "--malformed-permille",
            "250",
            "--byte-divisor",
            "7",
            "--dry-run",
        ])
        .expect("every option is understood");
        assert_eq!(asked.out, "somewhere");
        assert_eq!(asked.params.files, 11);
        assert_eq!(asked.params.seed, 5);
        assert_eq!(asked.params.cards, 2);
        assert_eq!(asked.params.years, 3);
        assert_eq!(asked.params.malformed_permille, 250);
        assert_eq!(asked.params.byte_divisor, 7);
        assert!(asked.dry);
    }
}
