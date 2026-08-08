// SPDX-License-Identifier: AGPL-3.0-only
//! The licence identifier guard (#110).
//!
//! The licence file landed first and it is the easy half. The half that goes
//! wrong later is everything that has to agree with it: a source file copied
//! out of this tree and into another one carries no statement of its terms
//! unless the file itself carries it, and a manifest that writes its own
//! `license` field is a second place holding one fact.
//!
//! So there is one identifier here, it is the spelling SPDX gives, and this
//! file refuses every departure from it that a reading of the tracked tree can
//! see. Four subjects, one rule each:
//!
//! - every tracked Rust source opens with the identifier,
//! - every workspace member takes its terms from the workspace rather than
//!   declaring them again,
//! - the workspace declares the identifier, and the dependency policy permits
//!   it, so this project's own terms are on the allow list its own gate reads,
//! - the licence file is the licence the identifier names.
//!
//! Every judgement is a pure function over text, which is what makes the
//! near-misses in `bites` possible: each one feeds the function the mistake
//! somebody actually makes and the neighbour it differs from by almost
//! nothing. A guard proven only against the tree it already passes is a guard
//! proven against nothing.
//!
//! ## Why a file added tomorrow is covered
//!
//! The subjects come from `git ls-files` at the commit under test rather than
//! from a list in this file. A module arriving without a header is a tracked
//! source with no header, so it fails; it does not pass by being absent from
//! something somebody forgot to extend. Both legs that read a list assert they
//! read something, so a listing that failed cannot be mistaken for a tree with
//! nothing in it.
//!
//! ## What this cannot judge
//!
//! Whether the terms are the right terms for a given file. A file copied in
//! from somewhere else under somebody else's licence passes the moment it
//! carries this header, and it passes wrongly. Nothing in a reading of the
//! text sees where a file came from, so that one is held by whoever reviews
//! the change that adds it.
//!
//! It reads tracked text. An untracked file is invisible to it, which is the
//! correct blind spot: an untracked file is not part of what is distributed.
//!
//! And it judges the identifier rather than the notice. The identifier is the
//! machine-readable short form; the full grant, the warranty disclaimer and
//! the address of the licence itself are in `LICENSE`, which the identifier
//! points at and which this guard checks is still the licence being pointed
//! at.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The terms of this tree, in the spelling SPDX gives them.
///
/// Not the spelling the hosting platform's API returns, which is `AGPL-3.0`
/// and is the older identifier for the same licence. The two are one licence
/// and two strings, and a tree carrying both has one fact in two spellings.
const IDENTIFIER: &str = "AGPL-3.0-only";

/// The first line of every source file.
const HEADER: &str = "// SPDX-License-Identifier: AGPL-3.0-only";

/// What a member manifest carries instead of terms of its own.
const INHERITS: &str = "license.workspace = true";

/// The licence file the identifier points at, and the two lines it opens with.
const LICENCE_FILE: &str = "LICENSE";
const LICENCE_TITLE: &str = "GNU AFFERO GENERAL PUBLIC LICENSE";
const LICENCE_VERSION: &str = "Version 3, 19 November 2007";

/// The dependency policy, which decides which terms may enter this tree.
const POLICY_FILE: &str = "deny.toml";

/// The workspace manifest, which declares the terms every member takes.
const ROOT_MANIFEST: &str = "Cargo.toml";

/// The older identifier for the same licence, which is what the hosting
/// platform's API answers with and what a document written from a listing
/// therefore says.
const OLDER_SPELLING: &str = "AGPL-3.0";

/// The highest decision record present when the prose rule landed.
///
/// A record is added rather than edited once it is accepted, so a rule
/// reaching backwards would make a landed record permanently red with no legal
/// repair. `docs/decisions/0004-where-tethering-sits.md` introduces a pasted
/// answer with the spelling that answer uses, which is correct writing and is
/// below this line. The rule applies above it, which is what "a new document"
/// means here. The same device, for the same reason, is in
/// `crates/lichttisch/tests/architecture_rules.rs`.
const RECORD_WATERMARK: u32 = 10;

#[allow(clippy::expect_used, reason = "a guard that cannot find its tree stops")]
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

/// Fail closed. A guard that could not list the tree is not a guard that read
/// it and found nothing.
#[allow(clippy::expect_used, reason = "no git means the guard could not run")]
fn tracked(pattern: &str) -> Vec<String> {
    let out = Command::new("git")
        .current_dir(workspace_root())
        .args(["ls-files", "--", pattern])
        .output()
        .expect("could not run git ls-files");
    assert!(
        out.status.success(),
        "git ls-files -- {pattern} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .filter(|line| !line.is_empty())
        .collect()
}

#[allow(clippy::expect_used, reason = "a guard that cannot read its subject stops")]
fn read(path: &str) -> String {
    let full = workspace_root().join(path);
    std::fs::read_to_string(&full)
        .unwrap_or_else(|why| panic!("could not read {}: {why}", full.display()))
}

/// Why this source file does not carry the identifier, or `None`.
///
/// The rule is the first line and not merely somewhere in the file. A licence
/// scanner reads the head of a file, a reader opening it sees the head first,
/// and one position has one answer, where "in the first few lines" has as many
/// answers as there are readers.
fn source_finding(text: &str) -> Option<String> {
    let Some(first) = text.lines().next() else {
        return Some("the file is empty, so it states no terms at all".to_owned());
    };
    if first == HEADER {
        return None;
    }
    if let Some(at) = text.lines().position(|line| line == HEADER) {
        let number = at + 1;
        return Some(format!(
            "carries the identifier on line {number} rather than on the first line, where it \
             reads as one comment among others"
        ));
    }
    if first.contains("SPDX-License-Identifier") {
        return Some(format!(
            "opens with `{first}`, and the terms of this tree are `{IDENTIFIER}`"
        ));
    }
    Some(format!(
        "opens with `{first}` and states no terms, so a copy of it carries none"
    ))
}

/// Why this member manifest does not take its terms from the workspace, or
/// `None`.
///
/// A member writing the identifier out is refused even when the identifier is
/// right. Two homes for one fact is the state that lets them disagree, and the
/// disagreement is invisible: a member declaring the older spelling, or a
/// permissive licence copied in with the rest of a manifest, builds and
/// publishes exactly like a member that inherited.
fn manifest_finding(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with("license") && line != INHERITS {
            return Some(format!(
                "declares terms of its own in `{line}`, and a member takes them from the \
                 workspace with `{INHERITS}`"
            ));
        }
    }
    if text.lines().any(|line| line.trim() == INHERITS) {
        return None;
    }
    Some(format!(
        "carries no `{INHERITS}`, so it states no terms and inherits none"
    ))
}

/// Why the workspace does not declare the identifier, or `None`.
fn root_finding(text: &str) -> Option<String> {
    let declared = format!("license = \"{IDENTIFIER}\"");
    if text.lines().any(|line| line.trim() == declared) {
        return None;
    }
    Some(format!(
        "carries no `{declared}`, so the members inheriting from it inherit something else"
    ))
}

/// Why the dependency policy does not permit this project's own terms, or
/// `None`.
///
/// The gate reading that policy judges what may enter the tree. A tree whose
/// own terms are absent from its own allow list is a tree that would refuse
/// itself as a dependency, which is a contradiction worth failing on rather
/// than a curiosity.
fn policy_finding(text: &str) -> Option<String> {
    let entry = format!("\"{IDENTIFIER}\",");
    if text.lines().any(|line| line.trim() == entry) {
        return None;
    }
    Some(format!(
        "does not list {entry} so this project's own terms are not on the list its own \
         dependency gate reads"
    ))
}

/// Why the licence file is not the licence the identifier names, or `None`.
///
/// The identifier is a pointer and this is the one leg that reads what it
/// points at. An identifier naming one licence over the text of another is the
/// failure that survives every other check here, because every other check
/// compares the tree against the identifier rather than against the licence.
fn licence_file_finding(text: &str) -> Option<String> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let title = lines.next().unwrap_or_default();
    let version = lines.next().unwrap_or_default();
    if title == LICENCE_TITLE && version == LICENCE_VERSION {
        return None;
    }
    Some(format!(
        "opens with `{title}` and `{version}`, and `{IDENTIFIER}` names `{LICENCE_TITLE}` \
         `{LICENCE_VERSION}`"
    ))
}

/// The document a reader meets first, and the one statement about terms most
/// people will ever read.
const README: &str = "README.md";

/// Why the readme's statement about terms does not agree with the licence
/// file, or `None`.
///
/// It has to name the licence by the title the file itself carries, and it has
/// to point at that file. A readme naming the wrong licence is the statement
/// somebody acts on without opening anything else.
fn readme_finding(text: &str) -> Option<String> {
    if !text.to_lowercase().contains(&LICENCE_TITLE.to_lowercase()) {
        return Some(format!(
            "does not name `{LICENCE_TITLE}`, which is the licence `{IDENTIFIER}` identifies"
        ));
    }
    if !text.contains(LICENCE_FILE) {
        return Some(format!(
            "names the licence without pointing at {LICENCE_FILE}, so a reader cannot reach the \
             terms from it"
        ));
    }
    None
}

/// Whether this document is one the prose rule reaches.
///
/// A record at or below the watermark is not, for the reason the watermark
/// carries. Everything else is, including a document that is not a record at
/// all, because those are edited rather than superseded.
fn prose_rule_reaches(path: &str) -> bool {
    let Some(name) = path.strip_prefix("docs/decisions/") else {
        return true;
    };
    let number: String = name.chars().take_while(char::is_ascii_digit).collect();
    match number.parse::<u32>() {
        Ok(number) => number > RECORD_WATERMARK,
        // A record whose name does not open with a number is not a record this
        // exemption was written for, so it is judged. Failing towards judging
        // is the direction a guard fails in.
        Err(_) => true,
    }
}

/// Every line of prose naming the older spelling, with its line number.
///
/// Prose only. Inside a block the spelling is what a command answered, and a
/// rule refusing it there would refuse a correct paste, which is the shape
/// that teaches somebody to delete the evidence rather than fix the claim.
///
/// The block detection here is the simple one: a fence opens and closes on a
/// line of its own, and a line indented by four spaces or a tab is a block.
/// `tools/docs-lint` reads blocks too and reads them more carefully, and the
/// two are not one implementation because they judge different things. The
/// bound this leaves is a fence opened inside a list item, which nothing in
/// this tree writes today.
fn prose_findings(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut inside_a_fence = false;
    for (index, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            inside_a_fence = !inside_a_fence;
            continue;
        }
        if inside_a_fence || line.starts_with("    ") || line.starts_with('\t') {
            continue;
        }
        let mut at = 0;
        while let Some(next) = line[at..].find(OLDER_SPELLING) {
            let starts = at + next;
            let ends = starts + OLDER_SPELLING.len();
            if !line[ends..].starts_with("-only") {
                let number = index + 1;
                found.push(format!("{number}: {}", line.trim()));
                break;
            }
            at = ends;
        }
    }
    found
}

#[test]
fn the_header_and_the_identifier_cannot_drift_apart() {
    // Two constants holding one fact, which is the state every rule in this
    // file refuses elsewhere. It is allowed here only because this leg refuses
    // the drift the moment either one is edited alone.
    assert!(
        HEADER.ends_with(IDENTIFIER),
        "the header `{HEADER}` does not end in the identifier `{IDENTIFIER}`, so a source file \
         passing this guard may carry terms the rest of the tree does not declare"
    );
}

#[test]
fn every_tracked_source_carries_the_identifier() {
    let sources = tracked("*.rs");
    assert!(
        !sources.is_empty(),
        "no tracked Rust source was listed, so this guard judged nothing and would have passed a \
         tree it never opened"
    );

    let mut offending = Vec::new();
    for path in &sources {
        if let Some(why) = source_finding(&read(path)) {
            offending.push(format!("{path}: {why}"));
        }
    }

    assert!(
        offending.is_empty(),
        "these tracked sources do not open with the identifier:\n\n    {}\n\n\
         Add `{HEADER}` as the first line. A file copied out of this tree states its terms only \
         if it carries them itself.\n\n\
         {} source(s) were examined.\n",
        offending.join("\n    "),
        sources.len()
    );
}

#[test]
fn every_member_takes_its_terms_from_the_workspace() {
    let mut manifests = tracked("crates/*/Cargo.toml");
    manifests.extend(tracked("tools/*/Cargo.toml"));
    assert!(
        !manifests.is_empty(),
        "no member manifest was listed, so this guard judged nothing"
    );

    let mut offending = Vec::new();
    for path in &manifests {
        if let Some(why) = manifest_finding(&read(path)) {
            offending.push(format!("{path}: {why}"));
        }
    }

    assert!(
        offending.is_empty(),
        "these member manifests do not take their terms from the workspace:\n\n    {}\n\n\
         Write `{INHERITS}` and delete the field. The workspace is where the identifier is \
         declared once.\n\n\
         {} manifest(s) were examined.\n",
        offending.join("\n    "),
        manifests.len()
    );
}

#[test]
fn the_workspace_declares_the_identifier() {
    assert!(
        root_finding(&read(ROOT_MANIFEST)).is_none(),
        "{ROOT_MANIFEST} {}",
        root_finding(&read(ROOT_MANIFEST)).unwrap_or_default()
    );
}

#[test]
fn the_dependency_policy_permits_this_projects_own_terms() {
    assert!(
        policy_finding(&read(POLICY_FILE)).is_none(),
        "{POLICY_FILE} {}",
        policy_finding(&read(POLICY_FILE)).unwrap_or_default()
    );
}

#[test]
fn the_licence_file_is_the_one_the_identifier_names() {
    assert!(
        licence_file_finding(&read(LICENCE_FILE)).is_none(),
        "{LICENCE_FILE} {}",
        licence_file_finding(&read(LICENCE_FILE)).unwrap_or_default()
    );
}

#[test]
fn the_readme_names_the_licence_the_identifier_identifies() {
    assert!(
        readme_finding(&read(README)).is_none(),
        "{README} {}",
        readme_finding(&read(README)).unwrap_or_default()
    );
}

#[test]
fn no_document_states_the_terms_in_the_older_spelling() {
    let documents = tracked("*.md");
    assert!(
        !documents.is_empty(),
        "no tracked document was listed, so this leg judged nothing"
    );

    let mut judged = 0_usize;
    let mut offending = Vec::new();
    for path in &documents {
        if !prose_rule_reaches(path) {
            continue;
        }
        judged += 1;
        for finding in prose_findings(&read(path)) {
            offending.push(format!("{path}:{finding}"));
        }
    }

    assert!(
        offending.is_empty(),
        "these lines state the terms as `{OLDER_SPELLING}` outside a block:\n\n    {}\n\n\
         Write `{IDENTIFIER}`, or name the licence, or move the string into the block under the \
         command that answered with it. One licence with two names in the prose is how the two \
         come to disagree.\n\n\
         {judged} of {} document(s) were judged; the rest are records at or below \
         {RECORD_WATERMARK}, which are added rather than edited.\n",
        offending.join("\n    "),
        documents.len()
    );
}

/// The near-misses. Each pair is the mistake somebody makes and the neighbour
/// it differs from by almost nothing, so a rule that stopped biting is a red
/// test here rather than a green tree.
mod bites {
    use super::{
        HEADER, licence_file_finding, manifest_finding, policy_finding, prose_findings,
        prose_rule_reaches, readme_finding, root_finding, source_finding,
    };

    /// The spelling the hosting platform's API returns for this repository.
    /// Somebody reads it out of a listing, writes it into a header, and the
    /// tree now carries two identifiers for one licence.
    #[test]
    fn a_source_carrying_the_platform_spelling_is_refused() {
        let near_miss = "// SPDX-License-Identifier: AGPL-3.0\n//! A module.\n";
        assert!(source_finding(near_miss).is_some());
    }

    #[test]
    fn the_same_source_with_the_declared_spelling_is_read() {
        let neighbour = format!("{HEADER}\n//! A module.\n");
        assert!(source_finding(&neighbour).is_none());
    }

    /// Every file in this tree opens with a module documentation comment, so
    /// the identifier arrives after one as often as before it. It is a comment
    /// among comments there rather than the first thing anything reads.
    #[test]
    fn an_identifier_under_the_module_documentation_is_refused() {
        let near_miss = format!("//! A module.\n{HEADER}\n");
        let why = source_finding(&near_miss);
        assert!(why.is_some());
        assert!(
            why.unwrap_or_default().contains("line 2"),
            "the finding names where the identifier actually is, or a reader cannot act on it"
        );
    }

    #[test]
    fn a_source_with_no_identifier_at_all_is_refused() {
        assert!(source_finding("//! A module.\n").is_some());
    }

    #[test]
    fn an_empty_source_is_refused_rather_than_passing_for_having_no_first_line() {
        assert!(source_finding("").is_some());
    }

    /// A manifest copied from another member and edited by hand. The value is
    /// the older spelling, it is plausible, and nothing else in the tree would
    /// notice.
    #[test]
    fn a_member_declaring_the_older_spelling_is_refused() {
        let near_miss = "[package]\nname = \"signals\"\nlicense = \"AGPL-3.0\"\n";
        assert!(manifest_finding(near_miss).is_some());
    }

    /// The same field with the right value. Still refused, because the failure
    /// this rule prevents is the second home rather than the wrong string.
    #[test]
    fn a_member_declaring_the_right_value_in_the_wrong_place_is_refused() {
        let near_miss = "[package]\nname = \"signals\"\nlicense = \"AGPL-3.0-only\"\n";
        assert!(manifest_finding(near_miss).is_some());
    }

    #[test]
    fn the_same_manifest_inheriting_is_read() {
        let neighbour = "[package]\nname = \"signals\"\nlicense.workspace = true\n";
        assert!(manifest_finding(neighbour).is_none());
    }

    #[test]
    fn a_manifest_saying_nothing_about_terms_is_refused() {
        assert!(manifest_finding("[package]\nname = \"signals\"\n").is_some());
    }

    /// A commented-out field is not a declaration. Refusing one would push
    /// somebody into deleting the comment that explains why the line is not
    /// there.
    #[test]
    fn a_commented_field_is_not_read_as_a_declaration() {
        let neighbour = "[package]\n# license = \"MIT\" was here once\nlicense.workspace = true\n";
        assert!(manifest_finding(neighbour).is_none());
    }

    #[test]
    fn a_workspace_declaring_a_neighbouring_licence_is_refused() {
        assert!(root_finding("[workspace.package]\nlicense = \"AGPL-3.0\"\n").is_some());
    }

    #[test]
    fn a_workspace_declaring_the_identifier_is_read() {
        assert!(root_finding("[workspace.package]\nlicense = \"AGPL-3.0-only\"\n").is_none());
    }

    /// The allow list carrying the older spelling. The gate would then refuse
    /// this project's own terms while appearing to permit them.
    #[test]
    fn a_policy_listing_the_older_spelling_is_refused() {
        assert!(policy_finding("allow = [\n    \"AGPL-3.0\",\n]\n").is_some());
    }

    #[test]
    fn a_policy_listing_the_identifier_is_read() {
        assert!(policy_finding("allow = [\n    \"AGPL-3.0-only\",\n]\n").is_none());
    }

    /// The two licences differ by one word in the title and by the date. A
    /// tree carrying the plain General Public Licence under an Affero
    /// identifier makes the network clause a claim nothing supports, and the
    /// network clause is the reason for the choice.
    #[test]
    fn a_licence_file_one_word_short_of_the_named_licence_is_refused() {
        let near_miss = "        GNU GENERAL PUBLIC LICENSE\n           Version 3, 29 June 2007\n";
        assert!(licence_file_finding(near_miss).is_some());
    }

    #[test]
    fn the_named_licence_is_read() {
        let neighbour =
            "     GNU AFFERO GENERAL PUBLIC LICENSE\n       Version 3, 19 November 2007\n";
        assert!(licence_file_finding(neighbour).is_none());
    }

    /// A readme naming the licence one word short of the one this project is
    /// under. The word is the network clause, which is the whole reason for
    /// the choice.
    #[test]
    fn a_readme_naming_a_neighbouring_licence_is_refused() {
        let near_miss = "See [LICENSE](LICENSE) for the GNU General Public License version 3.\n";
        assert!(readme_finding(near_miss).is_some());
    }

    #[test]
    fn a_readme_naming_the_licence_without_pointing_at_it_is_refused() {
        let near_miss = "This project is under the GNU Affero General Public License version 3.\n";
        assert!(readme_finding(near_miss).is_some());
    }

    #[test]
    fn the_readme_sentence_this_tree_carries_is_read() {
        let neighbour = "See [LICENSE](LICENSE) for the terms, the GNU Affero General Public License \
             version 3.\n";
        assert!(readme_finding(neighbour).is_none());
    }

    /// The sentence somebody writes after reading the identifier out of a
    /// listing. It is one string away from the one the tree declares and no
    /// other rule here sees a document.
    #[test]
    fn prose_stating_the_older_spelling_is_refused() {
        let near_miss = "This repository is AGPL-3.0, so a linked library has to agree.\n";
        assert_eq!(prose_findings(near_miss).len(), 1);
    }

    #[test]
    fn the_same_sentence_in_the_declared_spelling_is_read() {
        let neighbour = "This repository is AGPL-3.0-only, so a linked library has to agree.\n";
        assert!(prose_findings(neighbour).is_empty());
    }

    /// The same string as the answer a command gave. Refusing it here would
    /// refuse the evidence rather than the claim.
    #[test]
    fn the_answer_a_command_gave_is_not_prose() {
        let indented = "    gh api repos/iderex/lichttisch --jq .license.spdx_id\n    AGPL-3.0\n";
        assert!(prose_findings(indented).is_empty());
        let fenced = "```\ngh api repos/iderex/lichttisch --jq .license.spdx_id\nAGPL-3.0\n```\n";
        assert!(prose_findings(fenced).is_empty());
    }

    /// A fence that closed and prose after it. Without this the rule could be
    /// satisfied by opening a fence at the top of a document.
    #[test]
    fn prose_after_a_closed_fence_is_still_prose() {
        let after = "```\nAGPL-3.0\n```\n\nThis repository is AGPL-3.0.\n";
        assert_eq!(prose_findings(after).len(), 1);
    }

    #[test]
    fn a_record_at_the_watermark_is_not_reached_and_one_above_it_is() {
        assert!(!prose_rule_reaches(
            "docs/decisions/0004-where-tethering-sits.md"
        ));
        assert!(!prose_rule_reaches(
            "docs/decisions/0010-catalogue-schema.md"
        ));
        assert!(prose_rule_reaches("docs/decisions/0011-a-later-record.md"));
        assert!(prose_rule_reaches("README.md"));
        assert!(prose_rule_reaches("docs/licensing.md"));
    }
}
