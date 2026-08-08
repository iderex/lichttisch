// SPDX-License-Identifier: AGPL-3.0-only
//! The lock file drift guard (#14).
//!
//! A lock file that no longer matches the manifests is a build that resolves
//! differently on the next machine, and the failure is silent: an ordinary
//! `cargo build` repairs the lock in place and carries on, so the drift is
//! discovered later as an unexplained line in somebody else's diff.
//!
//! The obvious guard does not work here, and it was written before it was
//! measured. `cargo metadata --locked` refuses to rewrite the lock and fails
//! instead, which is the right instrument in a workflow step; inside a test it
//! cannot fail, because `cargo test` builds the test first and the build has
//! already repaired the lock by the time the test runs. Measured rather than
//! supposed: with the workspace version bumped and the lock left alone, the
//! `--locked` form of this test passed and `git status --porcelain -- Cargo.lock`
//! reported the file as modified in the same run.
//!
//! So the guard asks the question the build cannot hide from: after the build,
//! does the lock file on disk still match the one git stores. A rewrite during
//! the build is exactly the drift, and it leaves that trace whether or not
//! anybody was watching.
//!
//! It repairs nothing. The repair is a command a person runs, printed in the
//! failure, because a guard that fixes what it finds proves nothing on the next
//! run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[allow(clippy::expect_used, reason = "a guard that cannot find its tree stops")]
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

// Fail closed. A guard that could not run is not a guard that found nothing,
// and treating the two the same is how a check comes to pass a tree it never
// read.
fn git(args: &[&str]) -> Output {
    let out = Command::new("git")
        .current_dir(workspace_root())
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("could not run git {args:?}: {err}"));
    assert!(
        out.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

#[test]
#[allow(clippy::expect_used, reason = "no git means the guard could not run")]
fn lock_file_is_tracked() {
    // Without this leg the guard below has a hole it cannot see: an untracked
    // lock file differs from nothing, so every comparison passes and the tree
    // records no resolution at all.
    let out = Command::new("git")
        .current_dir(workspace_root())
        .args(["ls-files", "--error-unmatch", "--", "Cargo.lock"])
        .output()
        .expect("could not run git");
    assert!(
        out.status.success(),
        "Cargo.lock is not tracked, so nothing records how this workspace \
         resolves:\n\n    git add Cargo.lock\n"
    );
}

#[test]
fn lock_file_matches_what_git_will_record() {
    let out = git(&["diff", "--name-only", "--", "Cargo.lock"]);
    let changed = String::from_utf8_lossy(&out.stdout);
    assert!(
        changed.trim().is_empty(),
        "Cargo.lock on disk differs from what git will record, so this build \
         resolved to something the tree does not hold. That is the drift: the \
         build repaired the lock in place and said nothing.\n\n\
         Read what moved, then keep it or undo it:\n\n\
         \x20   git diff -- Cargo.lock\n\
         \x20   git add Cargo.lock\n"
    );
}
