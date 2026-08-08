// SPDX-License-Identifier: AGPL-3.0-only
//! One writer per catalogue, proven against real processes (#35).
//!
//! The unit tests beside the module judge the parts that are pure: which paths
//! are recognised as naming another machine, and which holder descriptions
//! parse. Neither of those is the property the module exists for. The property
//! is about two operating system processes, and it is not provable inside one
//! of them: a single process taking a lock it already holds is a question about
//! that process's own file handles, and every operating system answers it
//! differently from the question the module is about.
//!
//! So each test here starts a second process. The second process is this test
//! binary run again, with an environment variable naming the role it is to
//! play. That keeps the whole arrangement inside one compilation unit and adds
//! no target to the workspace, at the cost of two functions below that are
//! `#[test]` in name only and return at once when their variable is absent.
//! They are the child, and they are marked as such.
//!
//! What each test proves, and the mistake it is written against:
//!
//! - a live holder is refused, and the refusal names the process holding it,
//!   against a second instance being admitted and corrupting the catalogue,
//! - a holder that was killed leaves nothing behind, against the recovery that
//!   reads a description file and refuses because one is there, which turns
//!   every crash into a support request,
//! - several processes opening at once produce exactly one winner, against a
//!   check that reads before it takes and lets two through the window between.
//!
//! Nothing here needs elevation, a network, a display or a device. The only
//! thing it needs is a writable scratch directory, and it takes that from the
//! one Cargo sets aside for an integration test rather than from the system
//! temporary directory.

#![allow(
    clippy::expect_used,
    reason = "a test that cannot set its fixture up should stop rather than pass"
)]

use catalogue::lock::{self, Refusal};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Names the directory a child is to hold the write lock on.
const HOLD: &str = "LICHTTISCH_TEST_HOLD";

/// Names the directory a child is to race the others for.
const RACE: &str = "LICHTTISCH_TEST_RACE";

/// How long a wait for another process may take before the test gives up.
///
/// Generous rather than tight. This is the bound on a test hanging, not a
/// measurement of anything, and a slow machine under load is not a failure of
/// the property under test.
const PATIENCE: Duration = Duration::from_secs(30);

/// How often a wait looks again.
const GLANCE: Duration = Duration::from_millis(20);

/// How many processes the race starts.
const RACERS: usize = 4;

/// Distinguishes the directories of tests running side by side in one process.
static NEXT: AtomicU32 = AtomicU32::new(0);

/// Where the scratch directories go.
///
/// Cargo sets this for an integration test and the compiler resolves it, so
/// nothing here reads the running process's environment for it. That is the
/// difference that matters rather than a tidier location: the local threat
/// model the code scanning gate is configured with treats a value read out of
/// the environment as untrusted input, so `std::env::temp_dir()` standing here
/// made every path this fixture built a tainted one and carried the taint into
/// `catalogue::lock` through the calls below.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// A directory of its own for one test, removed when the test ends.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = Path::new(SCRATCH_ROOT)
            .join(format!("lichttisch-{name}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).expect("a scratch directory has to be creatable");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Best effort. A directory left behind is untidy; a test failing in the
        // cleanup would report the wrong thing.
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// A child process, killed when the handle is dropped.
///
/// Held so that a test failing part way through does not leave a process
/// holding a lock on a directory the next run wants.
struct Role {
    child: Child,
}

impl Role {
    /// Start this test binary again, running `test_name` and nothing else.
    fn start(test_name: &str, variable: &str, directory: &Path) -> Self {
        let binary = env::current_exe().expect("a test binary knows its own path");
        let child = Command::new(binary)
            .arg(test_name)
            .arg("--exact")
            .arg("--nocapture")
            .env(variable, directory)
            .spawn()
            .expect("the test binary has to be startable again");
        Self { child }
    }

    fn id(&self) -> u32 {
        self.child.id()
    }

    /// End the process abruptly, the way a crash ends one.
    ///
    /// This is the whole of the dead-holder fixture. Nothing asks the child to
    /// release anything and nothing gives it the chance to.
    fn kill(&mut self) {
        self.child
            .kill()
            .expect("a running child has to be killable");
        self.child
            .wait()
            .expect("a killed child has to be reapable");
    }
}

impl Drop for Role {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Wait until `path` exists, and say plainly when it never did.
fn wait_for(path: &Path) {
    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        sleep(GLANCE);
    }
    panic!("{} never appeared", path.display());
}

/// Wait until `directory` holds `count` files whose name starts with `prefix`.
fn wait_for_count(directory: &Path, prefix: &str, count: usize) -> Vec<String> {
    let deadline = Instant::now() + PATIENCE;
    loop {
        let found = named(directory, prefix);
        if found.len() >= count {
            return found;
        }
        assert!(
            Instant::now() < deadline,
            "only {} of {count} files named {prefix}* appeared in {}",
            found.len(),
            directory.display()
        );
        sleep(GLANCE);
    }
}

/// The contents of every file in `directory` whose name starts with `prefix`.
fn named(directory: &Path, prefix: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(prefix)
            && let Ok(text) = fs::read_to_string(entry.path())
        {
            found.push(text);
        }
    }
    found.sort();
    found
}

/// Keep asking for the write lock until it is given or the patience runs out.
///
/// The kernel releases a dead process's locks as that process is torn down, and
/// the tear-down is not instantaneous. Retrying is the difference between
/// testing the property and testing the scheduler.
fn open_for_writing_within_patience(directory: &Path) -> lock::Opened<lock::WriteLock> {
    let deadline = Instant::now() + PATIENCE;
    loop {
        match lock::open_for_writing(directory) {
            Ok(opened) => return opened,
            Err(refusal) => {
                assert!(
                    Instant::now() < deadline,
                    "the lock a dead process held was never released: {refusal}"
                );
                sleep(GLANCE);
            }
        }
    }
}

#[test]
fn a_second_process_is_refused_and_the_refusal_names_the_holder() {
    let scratch = Scratch::new("live-holder");
    let mut holder = Role::start("child_holds_the_write_lock", HOLD, scratch.path());
    wait_for(&scratch.path().join("ready"));

    let refusal = lock::open_for_writing(scratch.path())
        .expect_err("a catalogue another process is writing must not open for writing");

    match refusal {
        Refusal::HeldBy(ref who) => {
            assert_eq!(
                who.process,
                holder.id(),
                "the refusal named a process that is not the holder"
            );
            let said = refusal.to_string();
            assert!(
                said.contains(&holder.id().to_string()),
                "the refusal does not name the holder to the operator: {said}"
            );
            assert!(
                said.contains("Close that instance"),
                "the refusal does not say what to do about it: {said}"
            );
        }
        other => panic!("expected a named holder, got {other}"),
    }

    // The read-only open is refused too, which is the restriction
    // docs/catalogue-locking.md states rather than works around. It is asserted
    // here so that admitting a reader becomes a decision somebody makes against
    // a red test rather than a behaviour that drifts in.
    let reading = lock::open_for_reading(scratch.path());
    assert!(
        reading.is_err(),
        "a reader was admitted while a writer held the catalogue, which the note says cannot happen"
    );

    holder.kill();
}

#[test]
fn a_holder_that_was_killed_leaves_nothing_to_clean_up_by_hand() {
    let scratch = Scratch::new("dead-holder");
    let mut holder = Role::start("child_holds_the_write_lock", HOLD, scratch.path());
    wait_for(&scratch.path().join("ready"));

    holder.kill();

    // Both files the module writes are still on disk. This is what makes the
    // acquisition below a proof rather than a tautology: the recovery that
    // reads a description and refuses because one is present would fail here,
    // and that recovery is the one this module deliberately does not have.
    let description = scratch.path().join("catalogue.lock.holder");
    assert!(
        description.exists(),
        "the fixture is not the case under test: the dead holder left no description behind"
    );

    let opened = open_for_writing_within_patience(scratch.path());

    assert!(
        description.exists(),
        "the catalogue opened only because something deleted the dead holder's description"
    );
    opened
        .lock
        .release()
        .expect("releasing a lock this process holds cannot fail");
}

#[test]
fn several_processes_opening_at_once_produce_one_winner() {
    let scratch = Scratch::new("race");
    let mut racers = Vec::new();
    for _ in 0..RACERS {
        racers.push(Role::start(
            "child_races_for_the_write_lock",
            RACE,
            scratch.path(),
        ));
    }

    wait_for_count(scratch.path(), "ready-", RACERS);
    fs::write(scratch.path().join("start"), b"go").expect("the starting file has to be writable");

    let results = wait_for_count(scratch.path(), "result-", RACERS);
    let winners = results.iter().filter(|text| *text == "acquired").count();
    let losers = results.iter().filter(|text| *text == "refused").count();

    assert_eq!(
        winners, 1,
        "{RACERS} processes opened one catalogue at once and {winners} of them were admitted: {results:?}"
    );
    assert_eq!(
        losers,
        RACERS - 1,
        "a process neither won nor was refused for contention: {results:?}"
    );
}

/// The child that holds the lock until it is killed.
///
/// A test in name only. Without its variable it is the harness running a
/// function that returns, which is what lets the parent start this binary
/// again without a second target existing.
#[test]
fn child_holds_the_write_lock() {
    let Ok(directory) = env::var(HOLD) else {
        return;
    };
    let directory = PathBuf::from(directory);

    let opened = lock::open_for_writing(&directory).expect("the child could not take the lock");
    fs::write(directory.join("ready"), b"held").expect("the child could not report itself ready");

    // Held until the parent kills this process. The bound is the parent's
    // patience plus a margin, so a parent that died leaves nothing running
    // forever.
    sleep(PATIENCE + Duration::from_secs(30));
    drop(opened);
}

/// The child that waits for the start and then opens once.
///
/// A test in name only, for the same reason as the one above.
#[test]
fn child_races_for_the_write_lock() {
    let Ok(directory) = env::var(RACE) else {
        return;
    };
    let directory = PathBuf::from(directory);
    let me = std::process::id();

    fs::write(directory.join(format!("ready-{me}")), b"waiting")
        .expect("the child could not report itself ready");
    wait_for(&directory.join("start"));

    let (outcome, held) = match lock::open_for_writing(&directory) {
        Ok(opened) => ("acquired", Some(opened)),
        Err(Refusal::HeldBy(_) | Refusal::HeldByAnUnidentifiedProcess) => ("refused", None),
        Err(Refusal::Unavailable(_)) => ("unavailable", None),
    };
    fs::write(directory.join(format!("result-{me}")), outcome)
        .expect("the child could not report its outcome");

    // A winner holds long enough that every other racer has had its answer, so
    // the count the parent reads is the count at one moment rather than a
    // sequence of handovers.
    if held.is_some() {
        sleep(Duration::from_secs(3));
    }
}
