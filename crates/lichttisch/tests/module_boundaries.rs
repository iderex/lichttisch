//! The module boundary guard (#19).
//!
//! The catalogue answers queries. Decoding parses files that arrived from a
//! card, a camera or a stranger. Those are different risk profiles and
//! different dependency sets, and until this test existed the boundary between
//! them was a paragraph in `docs/layout.md` that nothing refused.
//!
//! What the manifests already give is thin. Cargo refuses a dependency that is
//! not declared, so no module reaches another one by accident, and that is the
//! whole of it: a manifest edit adding the wrong dependency on purpose builds
//! and passes. The edit that matters is not even a module edit. An image
//! library added to the catalogue's own manifest, or pulled in behind an
//! optional feature of something else it already has, puts a parser of
//! stranger bytes inside the module whose whole argument is that it holds
//! none.
//!
//! So the direction is declared in `crates/module-boundaries.txt` and this
//! test compares the declaration against what Cargo resolves. Three questions,
//! one test each:
//!
//! - is every workspace member declared, so a new module cannot arrive
//!   unplaced,
//! - does every declared name still name a member, so a removed module leaves
//!   no permission behind,
//! - and does any member reach a package its line does not carry.
//!
//! Reachability comes from `cargo tree` rather than from reading the manifests
//! here. Cargo's resolver is the thing that decides what a build actually
//! contains, and a second implementation of it in a test file would be a
//! second answer that disagrees on the day one of them is wrong. Three flags
//! matter and each closes a hole a simpler command leaves open:
//!
//! - `--edges normal` keeps dev and build dependencies out, because a test
//!   helper is not a thing the shipped module depends on,
//! - `--all-features` reaches the dependency that is optional today and turned
//!   on by whoever adds a feature tomorrow,
//! - `--target all` reaches the dependency declared for one platform only,
//!   which is otherwise invisible on every machine except that one.
//!
//! What this does not do: it names no forbidden library. The catalogue's line
//! is empty, so every package is refused there until somebody writes one down,
//! and the argument happens at the declaration rather than at a review of a
//! lock file diff. The bound on that is the other direction. A line may permit
//! a package that nothing reaches any more, and nothing here refuses a
//! permission that has gone stale.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The declaration this test is the enforcement of, relative to the root.
const DECLARATION: &str = "crates/module-boundaries.txt";

#[allow(clippy::expect_used, reason = "a guard that cannot find its tree stops")]
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

/// Fail closed. A resolver that could not run is not a resolver that found
/// nothing, and treating the two the same is how a guard comes to pass a tree
/// it never read.
#[allow(clippy::expect_used, reason = "no cargo means the guard could not run")]
fn cargo_tree(args: &[&str]) -> String {
    // Cargo sets CARGO to the binary running this test, so a machine with more
    // than one toolchain does not get asked which one it meant.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let out = Command::new(&cargo)
        .current_dir(workspace_root())
        .arg("tree")
        .args(args)
        .output()
        .expect("could not run cargo tree");
    assert!(
        out.status.success(),
        "cargo tree {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The package names in one `cargo tree` run, in the order they were printed.
///
/// Each line is `name version (source)`, and a line cargo repeats carries a
/// trailing `(*)` instead of its subtree. The first token is the name either
/// way.
fn package_names(tree: &str) -> Vec<String> {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|token| *token != "(*)")
        .map(ToOwned::to_owned)
        .collect()
}

/// Every workspace member, asked of cargo rather than read out of the root
/// manifest, so a member list that drifts is not this test's blind spot.
fn workspace_members() -> BTreeSet<String> {
    let tree = cargo_tree(&[
        "--workspace",
        "--depth",
        "0",
        "--edges",
        "normal",
        "--all-features",
        "--target",
        "all",
        "--prefix",
        "none",
        "--locked",
    ]);
    package_names(&tree).into_iter().collect()
}

/// Every package `member` reaches through its normal dependencies, directly or
/// through anything else, with the member itself removed.
fn reached_by(member: &str) -> BTreeSet<String> {
    let tree = cargo_tree(&[
        "--package",
        member,
        "--edges",
        "normal",
        "--all-features",
        "--target",
        "all",
        "--prefix",
        "none",
        "--locked",
    ]);
    let mut reached: BTreeSet<String> = package_names(&tree).into_iter().collect();
    reached.remove(member);
    reached
}

/// The declaration, parsed.
///
/// A line that is neither blank, nor a comment, nor of the declared shape
/// fails here rather than being skipped. A declaration a reader can mistype
/// into silence is not a declaration.
fn declaration() -> BTreeMap<String, BTreeSet<String>> {
    let path = workspace_root().join(DECLARATION);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            panic!("{DECLARATION} could not be read, so nothing declares the direction: {err}")
        }
    };

    let mut declared = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let number = index + 1;
        let Some((member, reaches)) = line.split_once("->") else {
            panic!(
                "{DECLARATION}:{number} is not a declaration and was not \
                 skipped. Every line that is not blank and not a comment reads \
                 `<member> -> <package> <package> ...`, and this one reads:\n\n\
                 \x20   {line}\n"
            );
        };
        let member = member.trim().to_owned();
        assert!(
            !member.is_empty(),
            "{DECLARATION}:{number} declares an arrow with nothing on the left of it"
        );
        let reaches: BTreeSet<String> = reaches.split_whitespace().map(ToOwned::to_owned).collect();
        assert!(
            declared.insert(member.clone(), reaches).is_none(),
            "{DECLARATION}:{number} declares `{member}` a second time, and the \
             two lines cannot both be the rule"
        );
    }
    declared
}

#[test]
fn every_member_declares_where_it_sits() {
    let declared = declaration();
    let undeclared: Vec<String> = workspace_members()
        .into_iter()
        .filter(|member| !declared.contains_key(member))
        .collect();
    assert!(
        undeclared.is_empty(),
        "these workspace members declare nothing about what they may reach, so \
         the boundary rule does not cover them:\n\n    {}\n\n\
         Add a line for each to {DECLARATION}. A module that says nothing about \
         where it sits passes by default, and passing by default is what this \
         leg refuses.\n",
        undeclared.join("\n    ")
    );
}

#[test]
fn every_declared_name_is_a_member() {
    let members = workspace_members();
    let dangling: Vec<String> = declaration()
        .into_keys()
        .filter(|member| !members.contains(member))
        .collect();
    assert!(
        dangling.is_empty(),
        "{DECLARATION} declares these, and the workspace has no such member:\n\n\
         \x20   {}\n\n\
         A permission outliving the module it was written for is a permission \
         nobody is watching. Remove the line or restore the member.\n",
        dangling.join("\n    ")
    );
}

#[test]
fn no_module_reaches_further_than_it_declares() {
    let declared = declaration();
    let mut offending = Vec::new();
    for (member, allowed) in &declared {
        for reached in reached_by(member) {
            if !allowed.contains(&reached) {
                offending.push(format!("{member} -> {reached}"));
            }
        }
    }
    assert!(
        offending.is_empty(),
        "these edges are in the resolved dependency graph and not in \
         {DECLARATION}:\n\n    {}\n\n\
         An edge is listed by the module it starts at and the package it \
         reaches, whether it is a direct dependency or arrives through another \
         one. Either the dependency does not belong there, or the line that \
         permits it is a change somebody argues.\n",
        offending.join("\n    ")
    );
}
