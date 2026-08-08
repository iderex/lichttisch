// SPDX-License-Identifier: AGPL-3.0-only
//! The structural rules, as things that fail (#105).
//!
//! The decision records state rules about shape rather than about behaviour:
//! where a foreign declaration may live, which module may reach which, what
//! this tree may depend on. Each one is load-bearing and each one was a
//! sentence. A sentence is not a rule, and the record that says so says it
//! about itself: `docs/decisions/0001-means.md` records that nothing in this
//! tree opens a decision record and names this issue as the gap.
//!
//! What this file holds is the enforced half. The rules that cannot be
//! enforced today are recorded in `docs/architecture-rules.md` with the reason
//! and what would enforce each one, because a rule quietly left out reads
//! exactly like a rule nobody stated.
//!
//! The enforced list is not written down anywhere. It is the names of the
//! tests in the `rule` module below, printed by
//!
//!     cargo test -p lichttisch --test architecture_rules -- --list rule::
//!
//! so a rule that is added, renamed or deleted moves the list by construction
//! and no second copy can disagree with it.
//!
//! Every rule is a pure function over text, and the test that judges the tree
//! feeds it the tracked files. That is what makes a near-miss possible: the
//! `bites` module feeds each function the one-character mistake somebody will
//! actually make, and the neighbour that must stay green. A guard proven only
//! against the tree it already passes is a guard proven against nothing.
//!
//! Three bounds, stated rather than left to be found. The stripper below
//! removes a line comment and the contents of a double-quoted string literal
//! and nothing else, so a block comment or a raw string could hide a violation
//! from it; the tree uses neither today. The record rule matches a heading
//! line exactly, so a heading written with different words fails and a line
//! inside a fenced block passes. And the module direction is refused by
//! `crates/lichttisch/tests/module_boundaries.rs` rather than here, so the
//! entry for it in the list is a check that its guard is still in the tree,
//! not a second implementation of it.
//!
//! A fourth bound used to be here without being written down. The two rules
//! about foreign declarations and about the keyword read a fixed area, and
//! that area was `crates` alone, so the four workspace members under `tools`
//! were judged by nothing. Both rules exist because the workspace lint table
//! reaches only a member that opted into it, and a member under `tools` can
//! omit those two lines exactly as a member under `crates` can. The area is
//! now every directory the workspace declares a member in, and
//! `rule::every_workspace_member_sits_under_an_area_these_rules_read` is what
//! keeps it that way: a member added under a new directory reddens rather
//! than arriving unjudged.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The highest decision record present when the record rule landed.
///
/// A record is added rather than edited once it is accepted, so a field
/// required of every record would make seven landed records permanently red
/// with no legal repair. The rule therefore applies above this number, which
/// is what "a new decision record" means here.
const RECORD_WATERMARK: u32 = 10;

/// The heading a record above the watermark carries, matched as a whole line.
const RECORD_SECTION: &str = "## Structural rules";

/// The register holding the rules nothing refuses yet.
const UNENFORCED_REGISTER: &str = "docs/architecture-rules.md";

/// The note that has to name the command printing the enforced list.
const LAYOUT_NOTE: &str = "docs/layout.md";

/// The declaration the module-direction guard reads.
const BOUNDARY_DECLARATION: &str = "crates/module-boundaries.txt";

/// The guard that reads it.
const BOUNDARY_GUARD: &str = "crates/lichttisch/tests/module_boundaries.rs";

/// The manifest declaring which members the workspace has.
const WORKSPACE_MANIFEST: &str = "Cargo.toml";

/// The directories the source rules below read, as pathspecs for git.
///
/// Two rather than one, because a workspace member is a member wherever it
/// sits. Adding a member under a third directory without adding it here is
/// what the area rule refuses.
const JUDGED_AREAS: [&str; 2] = ["crates", "tools"];

/// The opening of a foreign block, as it survives the stripper below.
///
/// The stripper empties a string literal and keeps its quotes, so a real
/// declaration reaches this needle as `extern ""` and this file's own fixtures
/// reach it as `""`. That is what lets the rule judge the file it is written
/// in instead of exempting it.
const FOREIGN_BLOCK: &str = "extern \"";

#[allow(clippy::expect_used, reason = "a guard that cannot find its tree stops")]
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

/// Every tracked path under `area`, asked of git.
///
/// Fail closed in both directions. A git that could not run is not a tree with
/// no files in it, and an area that answers with nothing is a pathspec that
/// stopped matching rather than a rule with nothing left to judge.
#[allow(clippy::expect_used, reason = "no git means the guard could not run")]
fn tracked(area: &str) -> Vec<PathBuf> {
    let root = workspace_root();
    let out = Command::new("git")
        .current_dir(&root)
        .arg("ls-files")
        .arg("-z")
        .arg("--")
        .arg(area)
        .output()
        .expect("could not run git ls-files");
    assert!(
        out.status.success(),
        "git ls-files -- {area} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let listing = String::from_utf8_lossy(&out.stdout);
    let paths: Vec<PathBuf> = listing
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(|entry| root.join(entry))
        .collect();
    assert!(
        !paths.is_empty(),
        "no tracked path under {area}, so this rule judged nothing. A guard \
         that read an empty set is not a guard that found no violation."
    );
    paths
}

#[allow(clippy::expect_used, reason = "a file git tracks must be readable")]
fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("could not read {}: {why}", path.display()))
}

/// The path as the reader will type it, relative to the workspace root.
fn shown(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// One line with its line comment removed and its string literals emptied.
///
/// The comment goes because a rule about what the code does must not fire on a
/// doc comment describing the rule. The contents of a string literal go for
/// the same reason and one more: this file holds the needles it searches for,
/// and emptying a literal rather than exempting this file is what keeps the
/// rule judging the file it is written in. A guard that skips itself is a
/// guard nobody is judging.
///
/// The quotes themselves are kept. Dropping them as well was the first shape
/// of this function and it silently disarmed the foreign-block rule, whose
/// needle ends in one: every declaration in the tree reached the rule with the
/// quote already deleted and nothing could ever match. The near-miss in
/// `bites` is what caught it, which is the whole argument for having one.
fn code_only(line: &str) -> String {
    let mut kept = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    while let Some(c) = chars.next() {
        if in_string {
            if c == '\\' {
                chars.next();
            } else if c == '"' {
                in_string = false;
                kept.push(c);
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            kept.push(c);
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        kept.push(c);
    }
    kept
}

/// Every line of `text` whose code contains `needle` as a whole word.
fn word_hits(text: &str, needle: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            let code = code_only(line);
            code.match_indices(needle).any(|(at, _)| {
                let before = code[..at].chars().next_back();
                let after = code[at + needle.len()..].chars().next();
                !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
            })
        })
        .map(|(index, line)| (index + 1, line.trim().to_owned()))
        .collect()
}

/// Every line of `text` whose code contains `needle` anywhere.
fn substring_hits(text: &str, needle: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| code_only(line).contains(needle))
        .map(|(index, line)| (index + 1, line.trim().to_owned()))
        .collect()
}

/// Lines declaring a foreign function.
///
/// `extern crate` is not one of them and must stay green: it names a package
/// in this workspace rather than a symbol on the far side of a boundary, and
/// it is the innocent line one token away from the mistake.
fn foreign_declarations(text: &str) -> Vec<(usize, String)> {
    substring_hits(text, FOREIGN_BLOCK)
}

/// Lines using the keyword that turns off the compiler's guarantees.
fn unchecked_blocks(text: &str) -> Vec<(usize, String)> {
    word_hits(text, "unsafe")
}

/// Lines of a manifest naming `package` outside a comment.
fn manifest_mentions(text: &str, package: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            let code = line.split('#').next().unwrap_or("");
            code.contains(package)
        })
        .map(|(index, line)| (index + 1, line.trim().to_owned()))
        .collect()
}

/// The workspace members a manifest declares, in the order it declares them.
///
/// Only the array is read. `[workspace.dependencies]` further down the same
/// file names the same directories on lines of its own, and a rule reading the
/// whole manifest would count those as membership and then agree with itself
/// about an area nobody declared.
///
/// A `#` comment is dropped first, inside the array as well as outside it,
/// because a member somebody commented out is not a member and an area kept
/// alive by a commented line is an area the rules would read for no reason.
fn declared_members(manifest: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let code = line.split('#').next().unwrap_or("");
        let scanning = if inside {
            code
        } else if code.trim_start().starts_with("members") && code.contains('[') {
            inside = true;
            code.split_once('[').map_or("", |(_, after)| after)
        } else {
            continue;
        };
        let closes = scanning.contains(']');
        let upto = scanning.split(']').next().unwrap_or("");
        for (index, piece) in upto.split('"').enumerate() {
            if index % 2 == 1 {
                members.push(piece.to_owned());
            }
        }
        if closes {
            break;
        }
    }
    members
}

/// Every declared member lying under none of `areas`.
///
/// The comparison is by path component and not by prefix. An area written
/// `tool` is the mistake to expect, and a prefix test would accept it, read
/// nothing under it, and leave four members judged by nobody while the rule
/// stayed green.
fn members_outside(members: &[String], areas: &[&str]) -> Vec<String> {
    members
        .iter()
        .filter(|member| {
            let member = member.replace('\\', "/");
            !areas.iter().any(|area| {
                member == *area
                    || member
                        .strip_prefix(area)
                        .is_some_and(|rest| rest.starts_with('/'))
            })
        })
        .cloned()
        .collect()
}

/// The record number a decision record's filename carries, if it carries one.
fn record_number(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() == 4 {
        digits.parse().ok()
    } else {
        None
    }
}

/// Whether a record says where each structural rule it states is held.
///
/// The heading is matched as a whole line rather than as a substring, so a
/// record mentioning the section in a sentence has not carried it.
fn declares_where_its_rules_are_held(text: &str) -> bool {
    text.lines().any(|line| line.trim_end() == RECORD_SECTION)
}

/// Whether a guard still names the declaration it is supposed to read.
///
/// The whole file is searched rather than its code, because the guard names
/// its declaration in a string literal and the stripper would empty it.
fn guard_names(guard: &str, declaration: &str) -> bool {
    guard.contains(declaration)
}

/// Whether a tracked path is Rust source, however the filesystem cased it.
fn is_rust_source(shown: &str) -> bool {
    Path::new(shown)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
}

/// The message a rule fails with: the rule, then the offending line.
#[allow(clippy::expect_used, reason = "writing into a String cannot fail")]
fn refusal(rule: &str, path: &str, hits: &[(usize, String)]) -> String {
    use std::fmt::Write as _;
    let mut message = format!("{rule}\n\nrefused in {path}:\n");
    for (line, text) in hits {
        write!(message, "\n    {path}:{line}: {text}").expect("a String never fails to grow");
    }
    message.push('\n');
    message
}

/// Every rule this tree refuses a violation of. The names of these tests are
/// the list; nothing else holds it.
mod rule {
    use super::{
        BOUNDARY_DECLARATION, BOUNDARY_GUARD, JUDGED_AREAS, LAYOUT_NOTE, RECORD_SECTION,
        RECORD_WATERMARK, UNENFORCED_REGISTER, WORKSPACE_MANIFEST, declared_members,
        declares_where_its_rules_are_held, foreign_declarations, guard_names, is_rust_source,
        manifest_mentions, members_outside, read, record_number, refusal, shown, tracked,
        unchecked_blocks, workspace_root,
    };

    const FOREIGN_HOME: &str = "crates/foreign/";

    /// Every tracked Rust source in the judged areas, minus the one module the
    /// two rules below exempt.
    fn judged_sources() -> Vec<(String, String)> {
        let mut sources = Vec::new();
        for area in JUDGED_AREAS {
            for path in tracked(area) {
                let shown = shown(&path);
                if !is_rust_source(&shown) || shown.starts_with(FOREIGN_HOME) {
                    continue;
                }
                let text = read(&path);
                sources.push((shown, text));
            }
        }
        sources
    }

    #[test]
    fn no_module_outside_foreign_declares_a_foreign_function() {
        let rule = "Every declaration of a foreign function lives in \
                    crates/foreign and nowhere else. A parser of stranger bytes \
                    reached from anywhere else is a boundary that exists only \
                    in a document.";
        for (shown, text) in judged_sources() {
            let hits = foreign_declarations(&text);
            assert!(hits.is_empty(), "{}", refusal(rule, &shown, &hits));
        }
    }

    #[test]
    fn no_module_outside_foreign_turns_off_the_compilers_guarantees() {
        let rule = "The keyword that turns off the compiler's guarantees is \
                    written in crates/foreign and nowhere else. The workspace \
                    lint table denies it, but only for a member that opted \
                    into the table, and a member may omit those two lines.";
        for (shown, text) in judged_sources() {
            let hits = unchecked_blocks(&text);
            assert!(hits.is_empty(), "{}", refusal(rule, &shown, &hits));
        }
    }

    #[test]
    fn every_workspace_member_sits_under_an_area_these_rules_read() {
        let rule = "The two rules above read a fixed set of directories, so a \
                    workspace member outside that set is compiled, shipped and \
                    judged by nothing. This is what makes their reach follow \
                    the workspace rather than the directory somebody happened \
                    to start in: a member under a new directory reddens here \
                    until that directory is added to the areas they read.";
        let manifest = read(&workspace_root().join(WORKSPACE_MANIFEST));
        let members = declared_members(&manifest);
        assert!(
            !members.is_empty(),
            "{rule}\n\nno member read out of {WORKSPACE_MANIFEST}, so this rule \
             judged nothing. A guard that read an empty set is not a guard that \
             found no violation.\n"
        );
        let outside = members_outside(&members, &JUDGED_AREAS);
        assert!(
            outside.is_empty(),
            "{rule}\n\nread by no rule above: {}\n\nthe areas read are: {}\n",
            outside.join(", "),
            JUDGED_AREAS.join(", ")
        );
    }

    #[test]
    fn no_manifest_here_depends_on_the_neighbouring_project() {
        let rule = "docs/decisions/0007-scope-boundary.md states no code \
                    dependency in either direction between this project and \
                    the generative editing project. This is the direction this \
                    tree can refuse.";
        for path in tracked("*Cargo.toml") {
            let shown = shown(&path);
            let hits = manifest_mentions(&read(&path), "retusche");
            assert!(hits.is_empty(), "{}", refusal(rule, &shown, &hits));
        }
    }

    #[test]
    fn the_module_dependency_direction_is_refused_by_its_own_guard() {
        let rule = "The direction between the modules is declared in \
                    crates/module-boundaries.txt and refused by \
                    crates/lichttisch/tests/module_boundaries.rs. This entry \
                    exists so the derived list is complete; deleting either \
                    file removes a rule from the tree and reddens this test \
                    rather than passing in silence.";
        let root = workspace_root();
        for owed in [BOUNDARY_DECLARATION, BOUNDARY_GUARD] {
            assert!(root.join(owed).is_file(), "{rule}\n\nmissing: {owed}\n");
        }
        assert!(
            guard_names(&read(&root.join(BOUNDARY_GUARD)), BOUNDARY_DECLARATION),
            "{rule}\n\nthe guard no longer names the declaration it reads\n"
        );
    }

    #[test]
    fn a_new_decision_record_says_where_each_structural_rule_it_states_is_held() {
        let rule = "A decision record added above the watermark carries a \
                    section headed exactly '## Structural rules', naming for \
                    every structural rule it states the test that refuses a \
                    violation or the entry in docs/architecture-rules.md that \
                    records it as unenforced. A record stating no structural \
                    rule writes that in the section rather than omitting it, \
                    because a record that says neither leaves a sentence \
                    nothing reads.";
        for path in tracked("docs/decisions") {
            let shown = shown(&path);
            let Some(number) = record_number(&path) else {
                continue;
            };
            if number <= RECORD_WATERMARK {
                continue;
            }
            assert!(
                declares_where_its_rules_are_held(&read(&path)),
                "{rule}\n\n{shown} carries no {RECORD_SECTION} heading\n"
            );
        }
    }

    #[test]
    fn the_unenforced_half_is_registered_and_the_enforced_half_stays_derived() {
        let rule = "Every structural rule is enforced by a test here or \
                    recorded in docs/architecture-rules.md with the reason and \
                    what would enforce it. The register has to exist for the \
                    second half to mean anything, and docs/layout.md has to \
                    name the command that prints the first half, because a \
                    derived list nobody can print is a list somebody will \
                    write out by hand.";
        let root = workspace_root();
        assert!(
            root.join(UNENFORCED_REGISTER).is_file(),
            "{rule}\n\nmissing: {UNENFORCED_REGISTER}\n"
        );
        let listing = "--list rule::";
        assert!(
            guard_names(&read(&root.join(LAYOUT_NOTE)), listing),
            "{rule}\n\n{LAYOUT_NOTE} no longer names the command that prints \
             the enforced rules\n"
        );
    }
}

/// The proof that each rule bites, and that its neighbour does not.
///
/// Each pair is one change apart. The first is the mistake somebody makes; the
/// second is the innocent line one character away from it, which has to stay
/// green or the rule is refusing the wrong thing.
mod bites {
    use super::{
        BOUNDARY_DECLARATION, declared_members, declares_where_its_rules_are_held,
        foreign_declarations, guard_names, manifest_mentions, members_outside, record_number,
        unchecked_blocks,
    };
    use std::path::Path;

    #[test]
    fn the_foreign_declaration_rule_refuses_a_declaration() {
        let near_miss = "mod inner {\n    unsafe extern \"C\" {\n        fn decode();\n    }\n}\n";
        assert_eq!(foreign_declarations(near_miss).len(), 1);
        assert_eq!(foreign_declarations(near_miss)[0].0, 2);
    }

    #[test]
    fn the_foreign_declaration_rule_passes_a_line_that_only_names_one() {
        let neighbour = "// every extern \"C\" block lives in crates/foreign\n";
        assert!(foreign_declarations(neighbour).is_empty());
    }

    #[test]
    fn the_foreign_declaration_rule_passes_a_package_named_the_same_way() {
        let neighbour = "extern crate foreign;\n";
        assert!(foreign_declarations(neighbour).is_empty());
    }

    #[test]
    fn the_foreign_declaration_rule_passes_a_string_holding_its_own_needle() {
        let neighbour = "const NEEDLE: &str = \"extern \\\"\";\n";
        assert!(foreign_declarations(neighbour).is_empty());
    }

    #[test]
    fn the_guarantee_rule_refuses_the_keyword() {
        let near_miss = "fn read() {\n    unsafe { *pointer }\n}\n";
        assert_eq!(unchecked_blocks(near_miss).len(), 1);
        assert_eq!(unchecked_blocks(near_miss)[0].0, 2);
    }

    #[test]
    fn the_guarantee_rule_passes_a_word_that_merely_starts_the_same_way() {
        let neighbour = "fn unsafely_named_helper() {}\nlet unsafe_count = 0;\n";
        assert!(unchecked_blocks(neighbour).is_empty());
    }

    #[test]
    fn the_guarantee_rule_passes_a_comment_and_a_string() {
        let neighbour = "// no unsafe block here\nlet needle = \"unsafe\";\n";
        assert!(unchecked_blocks(neighbour).is_empty());
    }

    #[test]
    fn the_area_rule_refuses_a_member_under_a_directory_nothing_reads() {
        let near_miss = [String::from("crates/catalogue"), String::from("xtask")];
        assert_eq!(members_outside(&near_miss, &["crates", "tools"]), ["xtask"]);
    }

    #[test]
    fn the_area_rule_refuses_an_area_one_letter_short() {
        let near_miss = [String::from("tools/bench")];
        assert_eq!(
            members_outside(&near_miss, &["crates", "tool"]),
            ["tools/bench"]
        );
    }

    #[test]
    fn the_area_rule_passes_a_member_under_an_area_that_is_read() {
        let neighbour = [String::from("tools/bench"), String::from("crates/foreign")];
        assert!(members_outside(&neighbour, &["crates", "tools"]).is_empty());
    }

    #[test]
    fn the_member_list_stops_at_the_end_of_the_array() {
        let manifest = concat!(
            "[workspace]\n",
            "members = [\n",
            "    \"crates/catalogue\",\n",
            "    \"tools/bench\",\n",
            "]\n",
            "\n",
            "[workspace.dependencies]\n",
            "surface = { path = \"crates/surface\" }\n",
        );
        assert_eq!(
            declared_members(manifest),
            ["crates/catalogue", "tools/bench"]
        );
    }

    #[test]
    fn the_member_list_passes_over_a_member_somebody_commented_out() {
        let manifest = concat!(
            "[workspace]\n",
            "members = [\n",
            "    \"crates/catalogue\",\n",
            "#   \"xtask\",\n",
            "]\n",
        );
        assert_eq!(declared_members(manifest), ["crates/catalogue"]);
    }

    #[test]
    fn the_dependency_rule_refuses_a_manifest_entry() {
        let near_miss = "[dependencies]\nretusche = { path = \"../retusche\" }\n";
        assert_eq!(manifest_mentions(near_miss, "retusche").len(), 1);
    }

    #[test]
    fn the_dependency_rule_passes_a_comment_naming_it() {
        let neighbour = "# nothing here depends on retusche, see 0007\n";
        assert!(manifest_mentions(neighbour, "retusche").is_empty());
    }

    #[test]
    fn the_record_rule_reads_a_number_out_of_a_filename() {
        assert_eq!(
            record_number(Path::new("docs/decisions/0011-raw-decoder.md")),
            Some(11)
        );
        assert_eq!(record_number(Path::new("docs/decisions/README.md")), None);
    }

    #[test]
    fn the_record_rule_refuses_a_heading_one_letter_short() {
        let near_miss = "# 0011 A record\n\n## Structural rule\n\nHeld by nothing yet.\n";
        assert!(!declares_where_its_rules_are_held(near_miss));
    }

    #[test]
    fn the_record_rule_refuses_a_record_that_only_mentions_the_section() {
        let near_miss = "# 0011 A record\n\nSee the ## Structural rules section of 0010.\n";
        assert!(!declares_where_its_rules_are_held(near_miss));
    }

    #[test]
    fn the_record_rule_passes_a_record_that_declares() {
        let neighbour = "# 0011 A record\n\n## Structural rules\n\nHeld by nothing yet.\n";
        assert!(declares_where_its_rules_are_held(neighbour));
    }

    #[test]
    fn the_boundary_entry_refuses_a_guard_that_lost_its_declaration() {
        let near_miss = "const DECLARATION: &str = \"crates/module_boundaries.txt\";\n";
        assert!(!guard_names(near_miss, BOUNDARY_DECLARATION));
    }

    #[test]
    fn the_boundary_entry_passes_a_guard_that_still_reads_it() {
        let neighbour = "const DECLARATION: &str = \"crates/module-boundaries.txt\";\n";
        assert!(guard_names(neighbour, BOUNDARY_DECLARATION));
    }
}
