// SPDX-License-Identifier: AGPL-3.0-only
//! The judgement, as pure functions over text.
//!
//! Every rule here takes strings and returns findings. Nothing reaches the
//! filesystem, the network or the clock, which is what lets each rule carry a
//! near-miss fixture beside it in the tests at the bottom of this file: the
//! document that violates a rule and the document one character away from it
//! are two strings rather than two branches.

use std::collections::{BTreeSet, HashSet};

/// The rule names. They appear in the failure output, so a reader knows which
/// declaration or which paragraph of the issue a refusal came from.
pub const REFERENCES: &str = "references";
pub const COMMANDS: &str = "commands";
pub const SHAPE: &str = "shape";
pub const RECORD: &str = "record";
pub const DECLARATIONS: &str = "declarations";

/// The file extensions that make a token without a slash still a path. A token
/// carrying a slash is a candidate whatever it ends with.
const EXTENSIONS: [&str; 7] = [".md", ".rs", ".toml", ".yml", ".yaml", ".txt", ".lock"];

/// What a link target may start with and still not be a path in this tree.
const NOT_A_PATH: [&str; 4] = ["http://", "https://", "mailto:", "#"];

/// One thing a document owes and does not carry.
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub rule: &'static str,
    pub what: String,
    pub quoted: String,
}

/// What the tree declares about itself, read from one file so that a reader
/// asking "why is that allowed" has one place to look.
pub struct Declarations {
    pub programs: Vec<Declared>,
    pub references: Vec<Declared>,
}

pub struct Declared {
    pub value: String,
    pub reason: String,
    pub line: usize,
}

/// Everything one document produced: what it owes, and what it used out of the
/// declarations. The second half is what makes an unused declaration a failure
/// rather than a comment nobody removed.
#[derive(Default)]
pub struct Judgement {
    pub findings: Vec<Finding>,
    pub programs_used: BTreeSet<String>,
    pub references_used: BTreeSet<String>,
}

/// Read the declarations file. A line that is neither blank, nor a comment, nor
/// one of the two declared shapes fails the parse rather than being skipped: a
/// declaration a reader can mistype into silence is not a declaration.
///
/// # Errors
///
/// Returns the line number and what was wrong with it.
pub fn read_declarations(text: &str) -> Result<Declarations, String> {
    let mut programs = Vec::new();
    let mut references = Vec::new();
    for (at, raw) in text.lines().enumerate() {
        let line = at + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut words = trimmed.splitn(3, char::is_whitespace);
        let kind = words.next().unwrap_or_default();
        let value = words.next().unwrap_or_default().trim();
        let reason = words.next().unwrap_or_default().trim();
        if value.is_empty() {
            return Err(format!("line {line}: `{kind}` names nothing"));
        }
        if reason.is_empty() {
            return Err(format!(
                "line {line}: `{kind} {value}` carries no reason, and a declaration without one \
                 is a hole nobody can argue with"
            ));
        }
        let declared = Declared {
            value: value.to_owned(),
            reason: reason.to_owned(),
            line,
        };
        match kind {
            "program" => programs.push(declared),
            "reference" => references.push(declared),
            other => {
                return Err(format!(
                    "line {line}: `{other}` is not a declaration kind. The kinds are `program` \
                     and `reference`."
                ));
            }
        }
    }
    Ok(Declarations {
        programs,
        references,
    })
}

/// What a line is, once the block structure has been read.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Prose, a heading, a list item: anything a reader reads as text.
    Prose,
    /// Inside a code block, or the fence line of one.
    Code,
}

/// Split a document into prose and code.
///
/// Two block shapes exist in this tree. A fenced block runs between two lines
/// whose first non-blank characters are three backticks. An indented block is
/// a run of lines indented past a threshold that began after a blank line, and
/// it ends at the first non-blank line indented less than that. The blank-line
/// condition is what keeps a wrapped list item from being read as code.
///
/// The threshold moves inside a list, and it has to. A paragraph continuing a
/// list item is written four spaces in and is a paragraph, not code, so a rule
/// reading it as a block would refuse ordinary Markdown for looking like a
/// command. Inside a list the threshold is eight, which is where a code block
/// under a list item actually starts.
#[must_use]
pub fn classify(text: &str) -> Vec<Kind> {
    let mut kinds = Vec::new();
    let mut fenced = false;
    let mut code_from: Option<usize> = None;
    let mut after_blank = true;
    let mut in_list = false;
    for raw in text.lines() {
        if raw.trim_start().starts_with("```") {
            fenced = !fenced;
            kinds.push(Kind::Code);
            after_blank = false;
            continue;
        }
        if fenced {
            kinds.push(Kind::Code);
            after_blank = false;
            continue;
        }
        if raw.trim().is_empty() {
            kinds.push(if code_from.is_some() {
                Kind::Code
            } else {
                Kind::Prose
            });
            after_blank = true;
            continue;
        }
        let indent = raw.len() - raw.trim_start().len();
        if let Some(threshold) = code_from {
            if indent >= threshold {
                kinds.push(Kind::Code);
                after_blank = false;
                continue;
            }
            code_from = None;
        }
        let threshold = if in_list { 8 } else { 4 };
        if indent >= threshold && after_blank {
            code_from = Some(threshold);
            kinds.push(Kind::Code);
        } else {
            if indent == 0 {
                in_list = starts_a_list_item(raw);
            }
            kinds.push(Kind::Prose);
        }
        after_blank = false;
    }
    kinds
}

/// Whether a line at column zero opens a list item. Only the column-zero case
/// matters: it is what decides whether the lines under it are continuations.
fn starts_a_list_item(line: &str) -> bool {
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return true;
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    digits > 0
        && line
            .chars()
            .nth(digits)
            .is_some_and(|one| one == '.' || one == ')')
        && line.chars().nth(digits + 1) == Some(' ')
}

/// The first line of every code block, with its zero-based line index.
///
/// The first line is the whole of what the command rule reads. What follows it
/// in a block is the output of the command above, and telling output from a
/// second command is not something a reading of the text does.
#[must_use]
pub fn block_openings(text: &str) -> Vec<(usize, String)> {
    let kinds = classify(text);
    let mut openings = Vec::new();
    let mut inside = false;
    for (at, raw) in text.lines().enumerate() {
        let code = kinds.get(at).copied().unwrap_or(Kind::Prose) == Kind::Code;
        if !code {
            inside = false;
            continue;
        }
        if raw.trim().is_empty() || raw.trim_start().starts_with("```") {
            continue;
        }
        if !inside {
            inside = true;
            openings.push((at, raw.trim_start().to_owned()));
        }
    }
    openings
}

/// The program a line invokes, where the line is shaped like an invocation.
///
/// Shaped like an invocation means: a first word of lower-case ASCII, digits
/// and the punctuation a program name carries, followed by at least one more
/// word. That shape is what separates `git ls-files .github/workflows` from
/// the output lines around it, which begin with a capital, with punctuation,
/// with a `key=value` pair, or are a single word.
///
/// It is a shape and not a parse, so it reads two things wrongly in opposite
/// directions, and both are held by the marker: an English sentence indented as
/// a block reads as an invocation, and a command written after the first line
/// of its block is not read at all.
#[must_use]
pub fn invoked_program(line: &str) -> Option<&str> {
    let mut words = line.split_whitespace();
    let program = words.next()?;
    words.next()?;
    let mut characters = program.chars();
    if !characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
    {
        return None;
    }
    if !program
        .chars()
        .all(|one| one.is_ascii_lowercase() || one.is_ascii_digit() || "_.+-".contains(one))
    {
        return None;
    }
    Some(program)
}

/// The marker that takes a block out of the command rule, and the reason it
/// carries. It sits on its own line above the block, invisible when the
/// document is rendered and unmissable when it is read as text.
const MARKER: &str = "<!-- docs-lint: illustrative,";

#[must_use]
fn marker_reason(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix(MARKER)?;
    let reason = rest.strip_suffix("-->")?.trim();
    if reason.is_empty() {
        None
    } else {
        Some(reason)
    }
}

/// Every reference a document makes to a path in this tree, from its prose.
///
/// Two shapes are read. A Markdown link whose target is not a URL and not a
/// bare anchor, and a backticked token shaped like a path. Code blocks are not
/// read, which is a bound rather than an oversight: a path inside a command is
/// an argument to that command, and this rule has no way to tell one the
/// command creates from one it expects to find.
#[must_use]
pub fn references(text: &str) -> Vec<(usize, String)> {
    let kinds = classify(text);
    let mut found = Vec::new();
    for (at, raw) in text.lines().enumerate() {
        if kinds.get(at).copied().unwrap_or(Kind::Prose) == Kind::Code {
            continue;
        }
        for target in link_targets(raw) {
            found.push((at, target));
        }
        for token in backticked(raw) {
            if looks_like_a_path(&token) {
                found.push((at, token));
            }
        }
    }
    found
}

fn link_targets(line: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let bytes: Vec<char> = line.chars().collect();
    let mut at = 0;
    while at + 1 < bytes.len() {
        if bytes[at] == ']' && bytes[at + 1] == '(' {
            let mut end = at + 2;
            while end < bytes.len() && bytes[end] != ')' {
                end += 1;
            }
            if end < bytes.len() {
                let inside: String = bytes[at + 2..end].iter().collect();
                let target = inside.split_whitespace().next().unwrap_or_default();
                let target = target.split('#').next().unwrap_or_default();
                if !target.is_empty() && !NOT_A_PATH.iter().any(|skip| inside.starts_with(skip)) {
                    targets.push(target.to_owned());
                }
                at = end;
            }
        }
        at += 1;
    }
    targets
}

fn backticked(line: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        spans.push(after[..close].to_owned());
        rest = &after[close + 1..];
    }
    spans
}

fn looks_like_a_path(token: &str) -> bool {
    if token.is_empty() || token.starts_with('/') || token.contains("//") {
        return false;
    }
    if !token
        .chars()
        .all(|one| one.is_ascii_alphanumeric() || "._-/".contains(one))
    {
        return false;
    }
    let body = token.strip_suffix('/').unwrap_or(token);
    if body.is_empty() {
        return false;
    }
    body.contains('/') || EXTENSIONS.iter().any(|extension| body.ends_with(extension))
}

/// Whether a reference names something git tracks, as a file or as a directory
/// holding tracked files.
#[must_use]
pub fn resolves(reference: &str, tracked: &HashSet<String>) -> bool {
    let body = reference.strip_suffix('/').unwrap_or(reference);
    if tracked.contains(body) {
        return true;
    }
    let directory = format!("{body}/");
    tracked.iter().any(|path| path.starts_with(&directory))
}

/// Judge one document against the tracked tree and the declarations.
#[must_use]
pub fn judge_document(
    path: &str,
    text: &str,
    tracked: &HashSet<String>,
    declared: &Declarations,
) -> Judgement {
    let mut judgement = reference_findings(path, text, tracked, declared);
    let commands = command_findings(path, text, declared);
    judgement.findings.extend(commands.findings);
    judgement.programs_used.extend(commands.programs_used);
    judgement.findings.extend(shape_findings(path, text));
    judgement.findings.extend(record_findings(path, text));
    judgement
}

fn quoted_line(text: &str, at: usize) -> String {
    text.lines().nth(at).unwrap_or_default().trim().to_owned()
}

/// Every reference a document makes, judged against the tracked tree.
fn reference_findings(
    path: &str,
    text: &str,
    tracked: &HashSet<String>,
    declared: &Declarations,
) -> Judgement {
    let mut judgement = Judgement::default();
    let quote = |at: usize| quoted_line(text, at);
    let exempt: HashSet<&str> = declared
        .references
        .iter()
        .map(|one| one.value.as_str())
        .collect();
    for (at, reference) in references(text) {
        if exempt.contains(reference.as_str()) {
            judgement.references_used.insert(reference);
            continue;
        }
        if !resolves(&reference, tracked) {
            judgement.findings.push(Finding {
                path: path.to_owned(),
                line: at + 1,
                rule: REFERENCES,
                what: format!(
                    "`{reference}` names nothing git tracks. Write a path that resolves, or \
                     declare it with its reason in the declarations file."
                ),
                quoted: quote(at),
            });
        }
    }
    judgement
}

/// Every command a document shows, judged against the roster and the markers.
fn command_findings(path: &str, text: &str, declared: &Declarations) -> Judgement {
    let mut judgement = Judgement::default();
    let quote = |at: usize| quoted_line(text, at);
    let roster: HashSet<&str> = declared
        .programs
        .iter()
        .map(|one| one.value.as_str())
        .collect();
    let markers = marker_lines(text);
    for (at, opening) in block_openings(text) {
        let marked = marker_above(&markers, at);
        let Some(program) = invoked_program(&opening) else {
            if let Some(marker_at) = marked {
                judgement.findings.push(Finding {
                    path: path.to_owned(),
                    line: marker_at + 1,
                    rule: COMMANDS,
                    what: String::from(
                        "this block is not read as an invocation, so the marker exempts it from \
                         nothing. Remove the marker.",
                    ),
                    quoted: quote(marker_at),
                });
            }
            continue;
        };
        if roster.contains(program) {
            judgement.programs_used.insert(program.to_owned());
            if let Some(marker_at) = marked {
                judgement.findings.push(Finding {
                    path: path.to_owned(),
                    line: marker_at + 1,
                    rule: COMMANDS,
                    what: format!(
                        "`{program}` is in the roster, so this block needs no marker. Remove it, \
                         or the next reader cannot tell a marker that is load-bearing from one \
                         that is decoration."
                    ),
                    quoted: quote(marker_at),
                });
            }
            continue;
        }
        if marked.is_some() {
            continue;
        }
        judgement.findings.push(Finding {
            path: path.to_owned(),
            line: at + 1,
            rule: COMMANDS,
            what: format!(
                "`{program}` is not a program this tree declares it has, so this command cannot \
                 be run from a clone. Declare it in the declarations file with where it comes \
                 from, or mark the block illustrative with the reason."
            ),
            quoted: quote(at),
        });
    }
    for (at, reason) in &markers {
        if reason.is_empty() {
            judgement.findings.push(Finding {
                path: path.to_owned(),
                line: at + 1,
                rule: COMMANDS,
                what: String::from("this marker carries no reason after the comma."),
                quoted: quote(*at),
            });
        } else if !block_openings(text).iter().any(|(start, _)| {
            marker_above(&markers, *start).is_some_and(|marker_at| marker_at == *at)
        }) {
            judgement.findings.push(Finding {
                path: path.to_owned(),
                line: at + 1,
                rule: COMMANDS,
                what: String::from(
                    "this marker is followed by no block, so it exempts nothing and reads as \
                     though it does.",
                ),
                quoted: quote(*at),
            });
        }
    }
    judgement
}

/// Every marker line in a document, with the reason it carries. A marker whose
/// reason is empty is kept here rather than dropped, so it is refused rather
/// than ignored.
fn marker_lines(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .filter(|(_, raw)| raw.trim().starts_with(MARKER))
        .map(|(at, raw)| (at, marker_reason(raw).unwrap_or_default().to_owned()))
        .collect()
}

/// The marker that governs a block, which is the one directly above it with
/// only blank lines between.
fn marker_above(markers: &[(usize, String)], block: usize) -> Option<usize> {
    markers
        .iter()
        .map(|(at, _)| *at)
        .filter(|at| *at < block)
        .max()
        .filter(|at| block - at <= 2)
}

/// The formatting rules. Each one is a property of the bytes rather than a
/// judgement about the writing, and each was measured against this tree before
/// it was written down.
///
/// Line width is deliberately absent. The documents wrap prose at eighty
/// columns by hand, and the exceptions are tables and links that cannot wrap.
/// A width rule would refuse those, and a decision record is added rather than
/// edited once it is accepted, so it would refuse them permanently.
#[must_use]
pub fn shape_findings(path: &str, text: &str) -> Vec<Finding> {
    let kinds = classify(text);
    let mut findings = Vec::new();
    let mut previous_level = 0usize;
    let finding = |line: usize, what: String, quoted: &str| Finding {
        path: path.to_owned(),
        line,
        rule: SHAPE,
        what,
        quoted: quoted.trim().to_owned(),
    };
    for (at, raw) in text.lines().enumerate() {
        let prose = kinds.get(at).copied().unwrap_or(Kind::Prose) == Kind::Prose;
        if raw.len() != raw.trim_end().len() {
            findings.push(finding(
                at + 1,
                String::from("this line ends in whitespace nobody can see."),
                raw,
            ));
        }
        if prose && raw.contains('\t') {
            findings.push(finding(
                at + 1,
                String::from("a tab outside a code block renders at a width nobody chose."),
                raw,
            ));
        }
        if prose {
            if raw.starts_with("* ") || raw.starts_with("+ ") {
                findings.push(finding(
                    at + 1,
                    String::from("the unordered list marker in this tree is `-`."),
                    raw,
                ));
            }
            if let Some(level) = heading_level(raw) {
                let body = raw.trim_start_matches('#');
                if body.starts_with("  ") {
                    findings.push(finding(
                        at + 1,
                        String::from("a heading carries one space after its hashes."),
                        raw,
                    ));
                }
                if raw.trim_end().ends_with('#') {
                    findings.push(finding(
                        at + 1,
                        String::from("a heading in this tree carries no closing hashes."),
                        raw,
                    ));
                }
                if previous_level > 0 && level > previous_level + 1 {
                    findings.push(finding(
                        at + 1,
                        format!(
                            "this heading is level {level} under a level {previous_level}, so a \
                             level is skipped and the outline has a hole in it."
                        ),
                        raw,
                    ));
                }
                previous_level = level;
            }
        }
    }
    if !text.ends_with('\n') {
        findings.push(finding(
            text.lines().count(),
            String::from("the file does not end in a newline."),
            "",
        ));
    } else if text.ends_with("\n\n") {
        findings.push(finding(
            text.lines().count(),
            String::from("the file ends in a blank line."),
            "",
        ));
    }
    findings
}

fn heading_level(line: &str) -> Option<usize> {
    let hashes = line.chars().take_while(|one| *one == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    if line.chars().nth(hashes) == Some(' ') {
        Some(hashes)
    } else {
        None
    }
}

/// What a decision record owes. The reversal condition is the field this rule
/// exists for: a record that does not say what would overturn it is a decision
/// nobody can reopen with evidence.
///
/// Which of a record's structural rules are enforced and which are registered
/// is a separate obligation, held by the architecture suite rather than here.
#[must_use]
pub fn record_findings(path: &str, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(name) = path.strip_prefix("docs/decisions/") else {
        return findings;
    };
    let finding = |what: String| Finding {
        path: path.to_owned(),
        line: 1,
        rule: RECORD,
        what,
        quoted: name.to_owned(),
    };
    let Some(number) = name.split('-').next().filter(|one| one.len() == 4) else {
        findings.push(finding(String::from(
            "a decision record is named NNNN-slug.md, four digits and a slug.",
        )));
        return findings;
    };
    if !number.chars().all(|one| one.is_ascii_digit()) {
        findings.push(finding(String::from(
            "a decision record is named NNNN-slug.md, four digits and a slug.",
        )));
        return findings;
    }
    let title = text.lines().next().unwrap_or_default();
    if !title.starts_with(&format!("# {number} ")) {
        findings.push(finding(format!(
            "the first line is not `# {number} ` followed by the title, so the file name and the \
             record disagree about which decision this is."
        )));
    }
    for owed in ["Status: ", "Date: ", "Issue: #"] {
        if !text.lines().any(|line| line.starts_with(owed)) {
            findings.push(finding(format!("no line begins `{owed}`.")));
        }
    }
    for heading in ["## The question", "## What would reverse it"] {
        if !text.lines().any(|line| line.trim_end() == heading) {
            findings.push(finding(format!(
                "the record carries no `{heading}` section."
            )));
        }
    }
    if let Some(date) = text
        .lines()
        .find_map(|line| line.strip_prefix("Date: "))
        .map(str::trim)
        && !is_a_date(date)
    {
        findings.push(finding(format!(
            "`Date: {date}` is not a date written YYYY-MM-DD."
        )));
    }
    findings
}

fn is_a_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts
            .iter()
            .all(|one| one.chars().all(|c| c.is_ascii_digit()))
}

/// The other direction. A declaration nothing uses is a hole that was closed
/// somewhere else and left standing here, and the next reader takes it for a
/// live exemption.
#[must_use]
pub fn judge_declarations(
    path: &str,
    declared: &Declarations,
    programs_used: &BTreeSet<String>,
    references_used: &BTreeSet<String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for one in &declared.programs {
        if !programs_used.contains(&one.value) {
            findings.push(Finding {
                path: path.to_owned(),
                line: one.line,
                rule: DECLARATIONS,
                what: format!(
                    "no document invokes `{}`, so this entry allows nothing.",
                    one.value
                ),
                quoted: format!("program {} {}", one.value, one.reason),
            });
        }
    }
    for one in &declared.references {
        if !references_used.contains(&one.value) {
            findings.push(Finding {
                path: path.to_owned(),
                line: one.line,
                rule: DECLARATIONS,
                what: format!(
                    "no document names `{}`, so this entry exempts nothing.",
                    one.value
                ),
                quoted: format!("reference {} {}", one.value, one.reason),
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::{
        Declarations, Finding, block_openings, classify, invoked_program, judge_declarations,
        judge_document, read_declarations, record_findings, references, resolves, shape_findings,
    };
    use std::collections::{BTreeSet, HashSet};

    fn tracked(of: &[&str]) -> HashSet<String> {
        of.iter().map(|one| (*one).to_owned()).collect()
    }

    fn declarations(text: &str) -> Declarations {
        match read_declarations(text) {
            Ok(read) => read,
            Err(why) => panic!("the fixture declarations parse: {why}"),
        }
    }

    fn rules(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|one| one.rule).collect()
    }

    const ROSTER: &str = "program git   in every clone\nprogram cargo the pinned toolchain\n";

    // --- the block model -------------------------------------------------

    #[test]
    fn an_indented_block_needs_a_blank_line_above_it() {
        // The second document is the same text with the blank line removed,
        // which is a wrapped list item rather than a code block.
        let code = "text\n\n    git status\n";
        let list = "- text\n    git status\n";
        assert_eq!(block_openings(code).len(), 1);
        assert_eq!(block_openings(list).len(), 0);
    }

    #[test]
    fn a_paragraph_continuing_a_list_item_is_not_a_block() {
        // Four spaces under a list item is a continuation paragraph. The same
        // four spaces after a paragraph is a code block, and the two are one
        // line apart here.
        let list = "- an item\n\n    a continuation paragraph under the item\n";
        let prose = "a paragraph\n\n    a continuation paragraph under the item\n";
        assert_eq!(block_openings(list).len(), 0);
        assert_eq!(block_openings(prose).len(), 1);
    }

    #[test]
    fn a_block_under_a_list_item_starts_at_eight_spaces() {
        let text = "- an item\n\n        git status\n";
        let openings = block_openings(text);
        assert_eq!(openings.len(), 1);
        assert_eq!(openings[0].1, "git status");
    }

    #[test]
    fn a_list_ends_at_the_next_paragraph_and_the_threshold_goes_back() {
        let text = "- an item\n\nback to prose\n\n    git status\n";
        assert_eq!(block_openings(text).len(), 1);
    }

    #[test]
    fn a_numbered_list_counts_as_a_list() {
        assert_eq!(
            block_openings("1. an item\n\n    a continuation\n").len(),
            0
        );
    }

    #[test]
    fn a_fence_hides_what_is_inside_it() {
        let text = "text\n\n```\nnot judged here\n\n    are public and that a record\n```\n";
        let openings = block_openings(text);
        assert_eq!(openings.len(), 1);
        assert_eq!(openings[0].1, "not judged here");
    }

    #[test]
    fn a_block_ends_at_the_first_line_that_is_not_indented() {
        let text = "one\n\n    git status\n    output\n\ntwo\n\n    cargo test\n";
        let openings = block_openings(text);
        assert_eq!(openings.len(), 2);
        assert_eq!(openings[1].1, "cargo test");
    }

    #[test]
    fn prose_and_code_are_told_apart() {
        let kinds = classify("prose\n\n    code\n\nprose\n");
        assert_eq!(kinds.len(), 5);
        assert!(kinds[0] == super::Kind::Prose);
        assert!(kinds[2] == super::Kind::Code);
        assert!(kinds[4] == super::Kind::Prose);
    }

    // --- what is read as an invocation -----------------------------------

    #[test]
    fn an_invocation_is_a_lower_case_word_and_at_least_one_more() {
        assert_eq!(invoked_program("git status"), Some("git"));
        assert_eq!(invoked_program("cargo test --locked"), Some("cargo"));
        assert_eq!(invoked_program("conditions"), None);
        assert_eq!(invoked_program("FAIL note.md carries"), None);
        assert_eq!(invoked_program("tracked=6 binary=2"), None);
        assert_eq!(invoked_program("--samples <n>  how many"), None);
        assert_eq!(invoked_program("#[ignore = \"hardware\"]"), None);
        assert_eq!(invoked_program("* text=auto eol=lf"), None);
    }

    // --- the command rule and its near-miss ------------------------------

    #[test]
    fn a_program_the_tree_does_not_declare_is_refused() {
        let text = "text\n\n    mdformat --check docs/\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert_eq!(rules(&judged.findings), vec![super::COMMANDS]);
    }

    #[test]
    fn the_same_block_with_a_declared_program_is_refused_nothing() {
        // One word apart from the fixture above.
        let text = "text\n\n    cargo --check docs/\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(judged.findings.is_empty());
        assert!(judged.programs_used.contains("cargo"));
    }

    #[test]
    fn a_marker_takes_a_block_out_of_the_rule() {
        let text = "text\n\n<!-- docs-lint: illustrative, output of the guard -->\n\n    exempt  legacy.md (declared)\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(judged.findings.is_empty());
    }

    #[test]
    fn a_marker_with_no_reason_is_refused() {
        let text =
            "text\n\n<!-- docs-lint: illustrative, -->\n\n    exempt  legacy.md (declared)\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(!judged.findings.is_empty());
    }

    #[test]
    fn a_marker_over_a_declared_program_is_refused_as_decoration() {
        let text = "text\n\n<!-- docs-lint: illustrative, not needed -->\n\n    git status\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert_eq!(rules(&judged.findings), vec![super::COMMANDS]);
    }

    #[test]
    fn a_marker_above_no_block_is_refused() {
        let text = "text\n\n<!-- docs-lint: illustrative, nothing follows -->\n\nmore prose\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert_eq!(rules(&judged.findings), vec![super::COMMANDS]);
    }

    // --- the reference rule and its near-miss ----------------------------

    #[test]
    fn a_backticked_path_that_resolves_is_left_alone() {
        let text = "See `docs/a.md` for it.\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(judged.findings.is_empty());
    }

    #[test]
    fn a_backticked_path_one_character_wrong_is_refused() {
        let text = "See `docs/b.md` for it.\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert_eq!(rules(&judged.findings), vec![super::REFERENCES]);
    }

    #[test]
    fn a_link_target_that_does_not_resolve_is_refused() {
        let text = "See [the note](docs/gone.md).\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert_eq!(rules(&judged.findings), vec![super::REFERENCES]);
    }

    #[test]
    fn a_url_and_an_anchor_are_not_paths() {
        let text = "See [there](https://example.invalid/x) and [here](#a-section).\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(judged.findings.is_empty());
    }

    #[test]
    fn a_directory_resolves_through_the_files_under_it() {
        assert!(resolves("docs/", &tracked(&["docs/a.md"])));
        assert!(resolves("docs", &tracked(&["docs/a.md"])));
        assert!(!resolves("doc", &tracked(&["docs/a.md"])));
    }

    #[test]
    fn a_declared_reference_is_exempt_and_counted_as_used() {
        let text = "The repository `iderex/retusche` is elsewhere.\n";
        let declared = declarations("reference iderex/retusche a repository, not a path\n");
        let judged = judge_document("docs/a.md", text, &tracked(&["docs/a.md"]), &declared);
        assert!(judged.findings.is_empty());
        assert!(judged.references_used.contains("iderex/retusche"));
    }

    #[test]
    fn a_token_that_is_not_shaped_like_a_path_is_not_read_as_one() {
        let text = "The version is `1.1` and the licence is `AGPL-3.0-only`.\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(judged.findings.is_empty());
    }

    #[test]
    fn a_path_inside_a_code_block_is_not_read() {
        let text = "text\n\n    git add tests/fixtures/new.txt\n";
        let judged = judge_document(
            "docs/a.md",
            text,
            &tracked(&["docs/a.md"]),
            &declarations(ROSTER),
        );
        assert!(judged.findings.is_empty());
    }

    // --- the shape rules -------------------------------------------------

    #[test]
    fn trailing_whitespace_is_refused_and_its_absence_is_not() {
        assert_eq!(shape_findings("docs/a.md", "text \n").len(), 1);
        assert_eq!(shape_findings("docs/a.md", "text\n").len(), 0);
    }

    #[test]
    fn a_tab_is_refused_in_prose_and_allowed_in_a_block() {
        assert_eq!(shape_findings("docs/a.md", "a\tb\n").len(), 1);
        assert_eq!(
            shape_findings("docs/a.md", "text\n\n    v1\t2026-07-28\n").len(),
            0
        );
    }

    #[test]
    fn a_missing_final_newline_and_a_trailing_blank_line_are_both_refused() {
        assert_eq!(shape_findings("docs/a.md", "text").len(), 1);
        assert_eq!(shape_findings("docs/a.md", "text\n\n").len(), 1);
        assert_eq!(shape_findings("docs/a.md", "text\n").len(), 0);
    }

    #[test]
    fn a_heading_carries_one_space_and_no_closing_hashes() {
        assert_eq!(shape_findings("docs/a.md", "#  Title\n").len(), 1);
        assert_eq!(shape_findings("docs/a.md", "# Title #\n").len(), 1);
        assert_eq!(shape_findings("docs/a.md", "# Title\n").len(), 0);
    }

    #[test]
    fn a_line_beginning_with_an_issue_reference_is_not_a_heading() {
        assert_eq!(
            shape_findings("docs/a.md", "#5 for the measurement and #6 for it\n").len(),
            0
        );
    }

    #[test]
    fn a_skipped_heading_level_is_refused() {
        assert_eq!(shape_findings("docs/a.md", "# One\n\n### Three\n").len(), 1);
        assert_eq!(
            shape_findings("docs/a.md", "# One\n\n## Two\n\n### Three\n").len(),
            0
        );
    }

    #[test]
    fn a_foreign_list_marker_is_refused() {
        assert_eq!(shape_findings("docs/a.md", "* one\n").len(), 1);
        assert_eq!(shape_findings("docs/a.md", "- one\n").len(), 0);
    }

    // --- the decision record rules ---------------------------------------

    const RECORD: &str = "# 0001 A title\n\nStatus: accepted\nDate: 2026-08-06\nIssue: #2\n\n## The question\n\nWhat.\n\n## What would reverse it\n\nThat.\n";

    #[test]
    fn a_complete_record_is_refused_nothing() {
        assert!(record_findings("docs/decisions/0001-a.md", RECORD).is_empty());
    }

    #[test]
    fn a_record_without_its_reversal_condition_is_refused() {
        let cut = RECORD.replace("## What would reverse it", "## Something else");
        assert_eq!(record_findings("docs/decisions/0001-a.md", &cut).len(), 1);
    }

    #[test]
    fn a_record_whose_title_disagrees_with_its_name_is_refused() {
        let wrong = RECORD.replace("# 0001 ", "# 0002 ");
        assert_eq!(record_findings("docs/decisions/0001-a.md", &wrong).len(), 1);
    }

    #[test]
    fn a_record_with_a_date_that_is_not_one_is_refused() {
        let wrong = RECORD.replace("2026-08-06", "last Tuesday");
        assert_eq!(record_findings("docs/decisions/0001-a.md", &wrong).len(), 1);
    }

    #[test]
    fn a_document_outside_the_decision_directory_owes_none_of_it() {
        assert!(record_findings("docs/layout.md", "# A note\n").is_empty());
    }

    // --- the declarations ------------------------------------------------

    #[test]
    fn a_declaration_without_a_reason_fails_the_parse() {
        assert!(read_declarations("program git\n").is_err());
    }

    #[test]
    fn an_unknown_declaration_kind_fails_the_parse() {
        assert!(read_declarations("allow git because\n").is_err());
    }

    #[test]
    fn a_comment_and_a_blank_line_are_skipped() {
        let read = declarations("# a comment\n\nprogram git in every clone\n");
        assert_eq!(read.programs.len(), 1);
    }

    #[test]
    fn a_program_no_document_invokes_is_refused() {
        let declared = declarations(ROSTER);
        let used: BTreeSet<String> = ["git".to_owned()].into_iter().collect();
        let findings = judge_declarations("d.txt", &declared, &used, &BTreeSet::new());
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn a_reference_no_document_names_is_refused() {
        let declared = declarations("reference a/b a repository\n");
        let findings = judge_declarations("d.txt", &declared, &BTreeSet::new(), &BTreeSet::new());
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn every_declaration_that_is_used_is_refused_nothing() {
        let declared = declarations(ROSTER);
        let used: BTreeSet<String> = ["git".to_owned(), "cargo".to_owned()].into_iter().collect();
        let findings = judge_declarations("d.txt", &declared, &used, &BTreeSet::new());
        assert!(findings.is_empty());
    }

    #[test]
    fn references_reads_a_link_and_a_backtick_and_nothing_else() {
        let found =
            references("See [x](docs/a.md) and `docs/b.md`, not http://example.invalid/c\n");
        assert_eq!(found.len(), 2);
    }
}
