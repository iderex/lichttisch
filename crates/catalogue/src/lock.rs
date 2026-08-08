// SPDX-License-Identifier: AGPL-3.0-only
//! One writer per catalogue, refused rather than serialised (#35).
//!
//! Two instances writing one catalogue is a corruption bug that presents as a
//! mystery: the damage is done by the second writer and discovered by whoever
//! opens the catalogue next. The cheap answer is to make the second one refuse
//! and say who is holding it, rather than to make concurrent writing correct.
//!
//! The mechanism is the operating system's advisory lock on one file, taken
//! through `std::fs::File::try_lock`. That choice is what answers the case
//! which produces most of the reports elsewhere: a previous instance that died
//! without releasing anything. A lock held by the kernel against an open file
//! is released when the process holding it ends, however it ends, so a stale
//! lock is not a state this module has to detect and recover from. It is a
//! state that cannot arise. Nothing here reads a process identifier to decide
//! whether a holder is still alive, and no operator ever deletes a file by hand
//! to get back in.
//!
//! What the lock does not do is decide anything about the storage engine
//! underneath. Which engine that is has not been chosen; issue #5 measures the
//! candidates and issue #6 chooses one. This module locks the catalogue's
//! directory and holds no opinion about what is inside it.

use std::fmt;
use std::fs::{File, OpenOptions, TryLockError};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The file whose lock is the right to write this catalogue.
///
/// It carries no catalogue data and is never read for anything but its lock.
const LOCK_FILE: &str = "catalogue.lock";

/// Where the holder describes itself for the benefit of whoever is refused.
///
/// Deliberately a second file rather than the body of the lock file. An
/// exclusive lock on Windows refuses a read of the locked range by any other
/// process, so a holder that wrote its description into the file it locked
/// would have written it where the only reader who needs it cannot reach it.
const HOLDER_FILE: &str = "catalogue.lock.holder";

/// The right to write a catalogue, held for as long as this value lives.
///
/// Dropping it releases the lock. So does the process ending, including ending
/// abruptly, which is the whole reason the lock is the kernel's rather than a
/// file this module writes and later has to judge the freshness of.
#[derive(Debug)]
pub struct WriteLock {
    /// Held open because the lock is a property of this open file, not of the
    /// path. Closing the file releases the lock.
    file: File,
    holder: PathBuf,
}

/// The right to read a catalogue nobody is writing.
#[derive(Debug)]
pub struct ReadLock {
    /// Held open for the same reason as in `WriteLock`.
    file: File,
}

/// What a refused open knows about the process that is holding the catalogue.
///
/// Every field is what the holder wrote about itself. None of it is verified
/// against the operating system, so it names a holder for a person to act on
/// rather than proving anything about one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder {
    /// The process identifier the holder reported.
    pub process: u32,
    /// The program the holder reported running as.
    pub program: String,
    /// When the holder took the lock, in seconds since the Unix epoch.
    pub taken_at: u64,
}

/// Why an open did not produce a lock.
#[derive(Debug)]
pub enum Refusal {
    /// Another process holds the write lock, and it described itself.
    HeldBy(Holder),
    /// Another process holds the write lock and did not leave a usable
    /// description of itself.
    ///
    /// This is a separate answer from `HeldBy` rather than a `Holder` with
    /// empty fields, because "somebody is holding this and I cannot tell you
    /// who" and "this process is holding it" are different things to tell an
    /// operator, and collapsing them would let the second sentence be printed
    /// when only the first was true.
    HeldByAnUnidentifiedProcess,
    /// The lock could not be taken or released for a reason that is not
    /// contention.
    Unavailable(io::Error),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeldBy(holder) => write!(
                f,
                "this catalogue is open for writing by process {} ({}), which took it at {} seconds past the Unix epoch. Close that instance, or wait for it to finish, and open this one again.",
                holder.process, holder.program, holder.taken_at
            ),
            Self::HeldByAnUnidentifiedProcess => write!(
                f,
                "this catalogue is open for writing by another process, which left no readable description of itself. Close the other instance, or wait for it to finish, and open this one again."
            ),
            Self::Unavailable(error) => write!(
                f,
                "the catalogue lock could not be taken: {error}. Nothing was opened."
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// What was established about the lock's strength at the location it was taken.
///
/// The lock is the operating system's, so it is exactly as strong as the
/// filesystem holding the file. On a file server that answers a lock request
/// without honouring it, two writers would both be admitted and neither would
/// be told. That is the one case where this module's whole promise fails
/// silently, so an open says what it established rather than leaving the
/// caller to assume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    /// The path was recognised as naming another machine's filesystem, so the
    /// single-writer promise rests on that server rather than on this kernel
    /// and is not established here.
    Network,
    /// The path was not recognised as a network location.
    ///
    /// This is not the same statement as "the path is local", and it is a
    /// separate variant rather than a `Local` one for that reason. What this
    /// module recognises exactly is a Windows UNC path. A mapped drive letter,
    /// a Unix mount of a remote filesystem and a bind mount over one all reach
    /// this answer, so an operator reading it learns that nothing was found,
    /// not that nothing is there.
    NotRecognisedAsNetwork,
}

/// A write lock and what the open established about where it was taken.
///
/// `#[must_use]` on the open functions is what makes the location statement
/// reach a caller: the whole point of `Location::Network` is that somebody is
/// told, and a value nobody is obliged to bind is a statement nobody has to
/// read.
#[derive(Debug)]
pub struct Opened<T> {
    /// The lock itself. Holding this value is holding the lock.
    pub lock: T,
    /// What the open established about the location, for the caller to state.
    pub location: Location,
}

/// Take the right to write the catalogue in `directory`.
///
/// The directory must already exist. The lock file and the holder file are
/// created inside it if they are not there.
///
/// # Errors
///
/// Returns `Refusal::HeldBy` or `Refusal::HeldByAnUnidentifiedProcess` when
/// another process is writing this catalogue, and `Refusal::Unavailable` when
/// the lock could not be attempted at all.
#[must_use = "the location this reports is the caller's to state, and a network location is where the single-writer promise is not established"]
pub fn open_for_writing(directory: &Path) -> Result<Opened<WriteLock>, Refusal> {
    let lock_path = directory.join(LOCK_FILE);
    let holder_path = directory.join(HOLDER_FILE);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(Refusal::Unavailable)?;

    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Err(refusal_naming_the_holder(&holder_path)),
        Err(TryLockError::Error(error)) => return Err(Refusal::Unavailable(error)),
    }

    // Written after the lock is held and not before, so a description can only
    // exist alongside a lock somebody actually took. It is rewritten whole on
    // every acquisition, so the description a refused process reads is this
    // holder's rather than a dead predecessor's.
    describe_this_process(&holder_path).map_err(Refusal::Unavailable)?;

    Ok(Opened {
        lock: WriteLock {
            file,
            holder: holder_path,
        },
        location: classify(directory),
    })
}

/// Take the right to read the catalogue in `directory`.
///
/// A reader is admitted only while no process is writing. That is a real
/// restriction and it is stated in `docs/catalogue-locking.md` rather than
/// worked around here: admitting a reader alongside a writer would be a claim
/// about what the storage engine underneath does to a file mid-write, and that
/// engine has not been chosen yet.
///
/// # Errors
///
/// The same refusals as `open_for_writing`.
#[must_use = "the location this reports is the caller's to state, and a network location is where the single-writer promise is not established"]
pub fn open_for_reading(directory: &Path) -> Result<Opened<ReadLock>, Refusal> {
    let lock_path = directory.join(LOCK_FILE);
    let holder_path = directory.join(HOLDER_FILE);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(Refusal::Unavailable)?;

    match file.try_lock_shared() {
        Ok(()) => Ok(Opened {
            lock: ReadLock { file },
            location: classify(directory),
        }),
        Err(TryLockError::WouldBlock) => Err(refusal_naming_the_holder(&holder_path)),
        Err(TryLockError::Error(error)) => Err(Refusal::Unavailable(error)),
    }
}

impl WriteLock {
    /// Give up the lock and say whether the release itself failed.
    ///
    /// Dropping the value releases the lock too. This exists for a caller that
    /// wants to know rather than to assume, because a release that failed on a
    /// filesystem which answered the request without honouring it is exactly
    /// the case `Location::Network` is about.
    ///
    /// # Errors
    ///
    /// Returns the operating system's error when the release failed. The lock
    /// is released by the file closing in any case.
    pub fn release(self) -> io::Result<()> {
        // Best effort, and deliberately not an error: the description is a
        // courtesy to the next process, and failing to remove it leaves a file
        // that the next acquisition overwrites anyway.
        let _ = std::fs::remove_file(&self.holder);
        self.file.unlock()
    }
}

impl ReadLock {
    /// Give up the read lock and say whether the release itself failed.
    ///
    /// # Errors
    ///
    /// Returns the operating system's error when the release failed.
    pub fn release(self) -> io::Result<()> {
        self.file.unlock()
    }
}

/// Say what is known about the filesystem holding `directory`.
///
/// The one thing recognised exactly is a Windows UNC path, which names another
/// machine in the path itself. Everything else answers
/// `Location::NotRecognisedAsNetwork`, which is a statement about this
/// function and not about the path.
#[must_use]
pub fn classify(directory: &Path) -> Location {
    // Matched on the string form because a UNC path is a fact about the
    // spelling: `\\server\share` and its verbatim form `\\?\UNC\server\share`
    // both name a machine that is not this one. The check is on both separators
    // so a path assembled with forward slashes, which Windows accepts, is not
    // read as local.
    let text = directory.to_string_lossy();
    let bytes = text.as_bytes();
    let starts_with_two_separators =
        matches!(bytes, [a, b, ..] if is_separator(*a) && is_separator(*b));

    if !starts_with_two_separators {
        return Location::NotRecognisedAsNetwork;
    }

    // `\\?\C:\...` and `\\.\...` are the verbatim and device forms and name
    // this machine. `\\?\UNC\...` is the verbatim form of a share and does not.
    let rest = &text[2..];
    let is_verbatim_or_device = rest.starts_with('?') || rest.starts_with('.');
    if is_verbatim_or_device {
        let upper = rest.to_ascii_uppercase();
        if upper.starts_with("?/UNC/") || upper.starts_with("?\\UNC\\") {
            return Location::Network;
        }
        return Location::NotRecognisedAsNetwork;
    }

    Location::Network
}

fn is_separator(byte: u8) -> bool {
    byte == b'\\' || byte == b'/'
}

/// Read the holder's description, and say plainly when it could not be read.
///
/// Every failure here reaches the same answer: a holder that cannot be
/// described is reported as unidentified rather than as a `Holder` with
/// invented fields. A description can be absent legitimately, in the window
/// between another process taking the lock and writing its description.
fn refusal_naming_the_holder(holder_path: &Path) -> Refusal {
    let Ok(mut file) = File::open(holder_path) else {
        return Refusal::HeldByAnUnidentifiedProcess;
    };
    let mut text = String::new();
    if file.read_to_string(&mut text).is_err() {
        return Refusal::HeldByAnUnidentifiedProcess;
    }
    parse_holder(&text).map_or(Refusal::HeldByAnUnidentifiedProcess, Refusal::HeldBy)
}

/// Parse a description, or answer that there is none.
///
/// A missing field, a field that does not parse and a description written by a
/// newer version that this one does not understand all reach `None`. That is
/// the fail-closed direction for this value: the cost of answering `None` is a
/// less helpful message, and the cost of guessing is a message naming a process
/// that is not the holder.
fn parse_holder(text: &str) -> Option<Holder> {
    let mut process = None;
    let mut program = None;
    let mut taken_at = None;

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "process" => process = value.trim().parse::<u32>().ok(),
            "program" => program = Some(value.trim().to_owned()),
            "taken_at" => taken_at = value.trim().parse::<u64>().ok(),
            _ => {}
        }
    }

    Some(Holder {
        process: process?,
        program: program?,
        taken_at: taken_at?,
    })
}

/// Write what this process can say about itself for a refused process to read.
fn describe_this_process(holder_path: &Path) -> io::Result<()> {
    let program = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned());

    // A newline in either field would forge a second key, so both are written
    // through a form that cannot carry one. The program name comes from the
    // filesystem and the epoch seconds from the clock, so neither is attacker
    // supplied today; the escaping is here because the reader is a parser and
    // the writer is the only thing keeping its input well formed.
    let program = program.replace(['\r', '\n'], " ");

    let taken_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(holder_path)?;
    write!(
        file,
        "process={}\nprogram={}\ntaken_at={}\n",
        std::process::id(),
        program,
        taken_at
    )?;
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::{Holder, Location, classify, parse_holder};
    use std::path::Path;

    #[test]
    fn a_unc_path_is_recognised_as_a_network_location() {
        assert_eq!(
            classify(Path::new(r"\\server\share\pictures")),
            Location::Network
        );
        assert_eq!(
            classify(Path::new("//server/share/pictures")),
            Location::Network
        );
        assert_eq!(
            classify(Path::new(r"\\?\UNC\server\share")),
            Location::Network
        );
    }

    #[test]
    fn an_ordinary_path_is_not_recognised_as_one() {
        assert_eq!(
            classify(Path::new(r"C:\pictures")),
            Location::NotRecognisedAsNetwork
        );
        assert_eq!(
            classify(Path::new("/home/photographer/pictures")),
            Location::NotRecognisedAsNetwork
        );
        assert_eq!(
            classify(Path::new(r"\\?\C:\pictures")),
            Location::NotRecognisedAsNetwork
        );
        assert_eq!(
            classify(Path::new(r"\\.\PhysicalDrive0")),
            Location::NotRecognisedAsNetwork
        );
    }

    #[test]
    fn a_description_that_parses_names_its_holder() {
        let parsed = parse_holder("process=41\nprogram=lichttisch\ntaken_at=1786000000\n");
        assert_eq!(
            parsed,
            Some(Holder {
                process: 41,
                program: "lichttisch".to_owned(),
                taken_at: 1_786_000_000,
            })
        );
    }

    #[test]
    fn a_description_missing_a_field_names_nobody() {
        assert_eq!(parse_holder("process=41\nprogram=lichttisch\n"), None);
        assert_eq!(parse_holder("program=lichttisch\ntaken_at=1\n"), None);
        assert_eq!(parse_holder(""), None);
    }

    #[test]
    fn a_description_that_does_not_parse_names_nobody() {
        assert_eq!(
            parse_holder("process=not a number\nprogram=lichttisch\ntaken_at=1\n"),
            None
        );
    }
}
