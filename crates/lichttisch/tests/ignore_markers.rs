//! The convention for a test that cannot run here (#17).
//!
//! Some tests will need a camera on the end of a cable, or an accelerator, or
//! a person to watch what happens. None of those exist in the automated
//! checks, so those tests are marked and skipped there.
//!
//! Rust already supplies the marker. What it does not supply is a reason, and
//! a bare `#[ignore]` is the failure this guard is about: it reads as "skipped"
//! in the run output with nothing saying what would have to be true for it to
//! run, so a test disabled while somebody debugged it looks exactly like a
//! test waiting for hardware, and the first one is never turned back on.
//!
//! So every ignore carries a reason and the reason opens with what kind of
//! thing is missing. Two prefixes exist because two different things are being
//! said, and a run that skipped either of them cannot be read as a full run:
//!
//! - `hardware:` for a test that needs a device the checks do not have,
//! - `manual:` for a test a person has to watch or judge.
//!
//! The guard exists before the first such test does, which is the point. The
//! first person to need one finds the convention rather than inventing it, and
//! the second one is not choosing between two conventions.
//!
//! What this does not do: it reads the tracked sources as text, so it sees the
//! attribute as written. An ignore produced by a macro is invisible to it, and
//! so is one in a file that is not tracked.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The prefixes a reason may open with, and what each one claims.
const PREFIXES: [&str; 2] = ["hardware:", "manual:"];

#[allow(clippy::expect_used, reason = "a guard that cannot find its tree stops")]
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

/// Fail closed. A guard that could not list the sources is not a guard that
/// read them and found nothing.
#[allow(clippy::expect_used, reason = "no git means the guard could not run")]
fn tracked_rust_sources() -> Vec<PathBuf> {
    let root = workspace_root();
    let out = Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "--", "*.rs"])
        .output()
        .expect("could not run git ls-files");
    assert!(
        out.status.success(),
        "git ls-files failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| root.join(line))
        .collect()
}

#[test]
fn every_ignored_test_says_what_it_is_waiting_for() {
    let mut bare = Vec::new();
    let mut sources_read = 0_usize;

    for path in tracked_rust_sources() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        sources_read += 1;
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if !line.starts_with("#[ignore") {
                continue;
            }
            let acceptable = PREFIXES
                .iter()
                .any(|prefix| line.starts_with(&format!("#[ignore = \"{prefix}")));
            if !acceptable {
                let shown = path.display();
                let number = index + 1;
                bare.push(format!("{shown}:{number}: {line}"));
            }
        }
    }

    assert!(
        sources_read > 0,
        "no tracked Rust source was read, so this guard judged nothing and \
         would have passed a tree it never opened"
    );

    assert!(
        bare.is_empty(),
        "these ignored tests do not say what they are waiting for:\n\n    {}\n\n\
         Write the reason into the attribute, opening with one of {PREFIXES:?}:\n\n\
         \x20   #[ignore = \"hardware: needs a camera on the other end of a cable\"]\n\
         \x20   #[ignore = \"manual: a person has to look at the result\"]\n\n\
         A skip with no reason is indistinguishable from a test somebody turned \
         off while debugging, and that is the one that never comes back on.\n",
        bare.join("\n    ")
    );
}
