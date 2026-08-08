// SPDX-License-Identifier: AGPL-3.0-only
//! Re-running the commands a document pastes an output under (#148).
//!
//! A number pasted under the command that produced it is the shape this tree
//! trusts most, and until this leg existed nothing checked that it was still
//! the number that command produces. Two of them had drifted, in the same
//! direction, from the same cause: a workspace member landed and the lock file
//! counts moved with it.
//!
//! The judgement here is pure, the same as `lint.rs`. This module parses a
//! declared block into commands and the output pasted under each one, and it
//! counts matches in a string it is handed. `main.rs` does every read, so a
//! near-miss for each rule is two strings rather than two trees.
//!
//! ## What is re-run, and what is not
//!
//! A block is re-run when it carries the marker directly above it, and never
//! otherwise. An undeclared block is not run and is not trusted either: this
//! leg says nothing at all about it, and the summary line says how many blocks
//! it did read so a run that covered two cannot be read as one that covered
//! every pasted output in the tree.
//!
//! The marker is read out of any tracked file rather than out of `.md` alone,
//! because one of the two instances that forced this leg is in `deny.toml`,
//! which no Markdown route reaches. The comment prefix a file needs is taken
//! from the marker line itself, so a `#` in front of the marker is a `#` in
//! front of every line of the block, and nothing here holds a list of comment
//! syntaxes.
//!
//! ## The subset of `grep` this implements, and why that is a subset
//!
//! The command is not handed to a shell. Nothing here starts a process: a
//! check that executed text taken out of a document would be a check that runs
//! whatever a document says, and it would also need `grep` to be installed and
//! to behave identically on every machine a contributor uses.
//!
//! So `grep -c` is reimplemented over the file's bytes, for patterns made of
//! literal characters, backslash-escaped metacharacters, and the two anchors.
//! A pattern using anything else is refused by name rather than approximated,
//! which is the part that keeps the reimplementation honest: the residual is
//! that this leg agrees with `grep` on the patterns it accepts and refuses to
//! judge the rest, not that it is `grep`.
//!
//! One consequence of that is worth stating rather than leaving to be found. A
//! line is what lies between newlines and a carriage return before one is part
//! of the line, which is what `grep` does with the same bytes and is therefore
//! the right behaviour here. It means a pattern anchored at the end answers
//! differently on a working tree holding carriage returns, and it means that
//! answer is the one `grep` gives on that tree. What keeps the two the same in
//! practice is the line-endings guard: the stored bytes are LF, and
//! `docs/text-and-line-endings.md` is where that is argued.
//!
//! ## The marker this file cannot write
//!
//! The scan reads every tracked file, so a file discussing the marker would
//! declare one by discussing it. The condition that stops that is the closing
//! `-->` at the end of the line: a marker inside a Rust string literal ends in
//! a quote and a comma, and the fixtures below are built from joined lines for
//! that reason. A marker that does get read and governs nothing a command can
//! be parsed out of fails loudly rather than passing, which is the direction
//! this ought to fail in.

use std::collections::HashSet;

use crate::lint::Finding;

/// The rule name, printed with every finding this leg produces.
pub const RERUN: &str = "rerun";

/// What declares a block re-runnable. It sits on its own line above the block,
/// invisible when Markdown is rendered and unmissable when the file is read as
/// text.
const MARKER: &str = "<!-- docs-lint: rerun,";

/// What closes it. The scan requires this at the end of the line, which is what
/// keeps a marker quoted inside source from being read as a live one.
const CLOSE: &str = "-->";

/// The only program this leg knows how to re-run, and the only flag of it.
const PROGRAM: &str = "grep";
const COUNT_FLAG: &str = "-c";

/// One command out of a declared block, with the output the document pastes
/// under it.
pub struct Pasted {
    /// One-based line of the command itself.
    pub line: usize,
    /// The command as the document writes it, quoted back in a finding.
    pub command: String,
    /// The tracked path the command reads.
    pub reads: String,
    /// What the pattern matches.
    pub matcher: Matcher,
    /// The lines pasted under the command, with the comment prefix removed.
    pub output: Vec<String>,
}

/// A `grep` pattern reduced to a literal and its anchors.
pub struct Matcher {
    at_start: bool,
    at_end: bool,
    literal: String,
}

impl Matcher {
    /// How many lines of `text` the pattern matches, which is what `grep -c`
    /// prints.
    ///
    /// A line is what lies between newlines. A final line with no newline after
    /// it still counts, and a trailing newline does not add an empty one, which
    /// is what `grep` does with the same bytes.
    #[must_use]
    pub fn count(&self, text: &str) -> usize {
        let mut lines: Vec<&str> = text.split('\n').collect();
        if lines.last() == Some(&"") {
            lines.pop();
        }
        lines.iter().filter(|line| self.matches(line)).count()
    }

    fn matches(&self, line: &str) -> bool {
        match (self.at_start, self.at_end) {
            (true, true) => line == self.literal,
            (true, false) => line.starts_with(&self.literal),
            (false, true) => line.ends_with(&self.literal),
            (false, false) => line.contains(&self.literal),
        }
    }
}

/// What one file's markers produced: the commands to re-run, how many blocks
/// were declared, and what was wrong with the declarations themselves.
#[derive(Default)]
pub struct Read {
    pub blocks: usize,
    pub pasted: Vec<Pasted>,
    pub findings: Vec<Finding>,
}

/// Read every declared block out of one file.
///
/// `programs` is the roster the declarations file holds. It is what separates a
/// line that is an attempt at a command from a line that is output: a first
/// word this tree says it has is the first kind, and anything else is the
/// second. Without it, a command outside the supported subset would be filed as
/// output and silently believed.
#[must_use]
pub fn read(path: &str, text: &str, programs: &HashSet<&str>) -> Read {
    let lines: Vec<&str> = text.lines().collect();
    let mut read = Read::default();
    for (at, prefix, reason) in markers(&lines) {
        let finding = |line: usize, what: String, quoted: &str| Finding {
            path: path.to_owned(),
            line,
            rule: RERUN,
            what,
            quoted: quoted.trim().to_owned(),
        };
        if reason.is_empty() {
            read.findings.push(finding(
                at + 1,
                String::from("this marker carries no reason after the comma."),
                lines.get(at).copied().unwrap_or_default(),
            ));
            continue;
        }
        let block = block_after(&lines, at, &prefix);
        if block.is_empty() {
            read.findings.push(finding(
                at + 1,
                String::from(
                    "this marker is followed by no block, so nothing is re-run and the marker \
                     reads as though something is. A block under a marker with no comment prefix \
                     is an indented block.",
                ),
                lines.get(at).copied().unwrap_or_default(),
            ));
            continue;
        }
        read.blocks += 1;
        let mut current: Option<Pasted> = None;
        // Whether any line of this block was an attempt at a command. It is
        // what stops a command this leg refused from producing a second finding
        // for every line under it: the block did open with a command, and the
        // refusal above already says which one and why.
        let mut attempted = false;
        for (line_at, content) in block {
            let quoted = lines.get(line_at).copied().unwrap_or_default();
            if !is_an_invocation(&content, programs) {
                if let Some(open) = current.as_mut() {
                    open.output.push(content);
                } else if !attempted {
                    read.findings.push(finding(
                        line_at + 1,
                        String::from(
                            "this block opens with output rather than with a command, so there is \
                             nothing to re-run it against.",
                        ),
                        quoted,
                    ));
                    attempted = true;
                }
                continue;
            }
            attempted = true;
            if let Some(done) = current.take() {
                read.pasted.push(done);
            }
            match parse(&content) {
                Ok((reads, matcher)) => {
                    current = Some(Pasted {
                        line: line_at + 1,
                        command: content,
                        reads,
                        matcher,
                        output: Vec::new(),
                    });
                }
                Err(why) => {
                    read.findings.push(finding(line_at + 1, why, quoted));
                }
            }
        }
        if let Some(done) = current.take() {
            read.pasted.push(done);
        }
    }
    for one in &read.pasted {
        if one.output.is_empty() {
            read.findings.push(Finding {
                path: path.to_owned(),
                line: one.line,
                rule: RERUN,
                what: String::from(
                    "nothing is pasted under this command, so there is nothing for the re-run to \
                     disagree with. Paste what it returns, or take the marker off the block.",
                ),
                quoted: one.command.clone(),
            });
        }
    }
    read
}

/// Compare what a document pastes against what the command returns.
///
/// `contents` is the file the command reads. The finding names the file, the
/// line, what is pasted and what came back, because a failure that says only
/// that two things differ sends the reader to run the command themselves.
#[must_use]
pub fn compare(path: &str, one: &Pasted, contents: &str) -> Option<Finding> {
    let produced = one.matcher.count(contents).to_string();
    if one.output.len() == 1 && one.output[0] == produced {
        return None;
    }
    let pasted = one.output.join(" / ");
    Some(Finding {
        path: path.to_owned(),
        line: one.line,
        rule: RERUN,
        what: format!(
            "this document pastes `{pasted}` under the command, and the command returns \
             `{produced}` against {} at this commit.",
            one.reads
        ),
        quoted: one.command.clone(),
    })
}

/// Every marker in a file: where it is, the comment prefix in front of it, and
/// the reason it carries. A marker with no reason is kept rather than dropped,
/// so it is refused rather than ignored.
fn markers(lines: &[&str]) -> Vec<(usize, String, String)> {
    let mut found = Vec::new();
    for (at, raw) in lines.iter().enumerate() {
        let line = raw.trim_end();
        let Some(start) = line.find(MARKER) else {
            continue;
        };
        let Some(rest) = line[start..].strip_prefix(MARKER) else {
            continue;
        };
        let Some(reason) = rest.strip_suffix(CLOSE) else {
            continue;
        };
        let prefix = line[..start].trim().to_owned();
        found.push((at, prefix, reason.trim().to_owned()));
    }
    found
}

/// The block a marker governs, as line index and content with the prefix off.
///
/// It begins at the first line under the marker that carries something, and it
/// ends at the first line that carries nothing or that has dropped the prefix.
/// Where the prefix is empty the block is an indented one, so a line at column
/// zero ends it: that is what stops the prose under a Markdown block from being
/// read as part of it.
fn block_after(lines: &[&str], marker: usize, prefix: &str) -> Vec<(usize, String)> {
    let mut block = Vec::new();
    for (at, raw) in lines.iter().enumerate().skip(marker + 1) {
        if !prefix.is_empty() && !raw.trim_start().starts_with(prefix) {
            break;
        }
        if prefix.is_empty() && !raw.trim().is_empty() && !raw.starts_with(char::is_whitespace) {
            break;
        }
        let content = strip(raw, prefix);
        if content.is_empty() {
            if block.is_empty() {
                continue;
            }
            break;
        }
        block.push((at, content));
    }
    block
}

fn strip(raw: &str, prefix: &str) -> String {
    let body = raw.trim_start();
    let body = if prefix.is_empty() {
        body
    } else {
        body.strip_prefix(prefix).unwrap_or(body)
    };
    body.trim().to_owned()
}

/// Whether a line is an attempt at a command rather than output, judged by its
/// first word against the roster this tree declares it has.
fn is_an_invocation(content: &str, programs: &HashSet<&str>) -> bool {
    content
        .split_whitespace()
        .next()
        .is_some_and(|first| programs.contains(first))
}

/// Parse one command down to the path it reads and the pattern it matches.
///
/// Everything outside the supported subset is refused by name. The message is
/// what a contributor meets when they mark a block this leg cannot judge, so it
/// says what is supported rather than only what was wrong.
fn parse(content: &str) -> Result<(String, Matcher), String> {
    let Some(words) = split(content) else {
        return Err(String::from(
            "this command carries an unclosed quote, so what it would run cannot be read.",
        ));
    };
    let unsupported = |what: &str| {
        Err(format!(
            "{what} A re-runnable block is `grep -c '<pattern>' <tracked path>`, and the pattern \
             is literal characters, backslash escapes and the two anchors."
        ))
    };
    // The program first, then the shape. A contributor who marked a block
    // running something else entirely is told that before being told the word
    // count is wrong, because the word count is not what they got wrong.
    if words.first().map(|one| one.value.as_str()) != Some(PROGRAM) {
        return unsupported("this check re-runs no program but `grep -c`.");
    }
    if words.len() != 4 {
        return unsupported("this command is not the shape this check re-runs.");
    }
    if words[1].value != COUNT_FLAG {
        return unsupported("this check re-runs no flag of `grep` but `-c`.");
    }
    if !words[2].quoted {
        return unsupported(
            "the pattern is not in single quotes, so what a shell would do to it before `grep` saw it is a second question.",
        );
    }
    let matcher = compile(&words[2].value)?;
    Ok((words[3].value.clone(), matcher))
}

struct Word {
    value: String,
    quoted: bool,
}

/// Split a command into words, keeping single quotes together and recording
/// which words carried them. Returns `None` on a quote that never closes.
fn split(content: &str) -> Option<Vec<Word>> {
    let mut words = Vec::new();
    let mut value = String::new();
    let mut quoted = false;
    let mut open = false;
    let mut started = false;
    for one in content.chars() {
        if one == '\'' {
            open = !open;
            quoted = true;
            started = true;
            continue;
        }
        if one.is_whitespace() && !open {
            if started {
                words.push(Word { value, quoted });
                value = String::new();
                quoted = false;
                started = false;
            }
            continue;
        }
        started = true;
        value.push(one);
    }
    if open {
        return None;
    }
    if started {
        words.push(Word { value, quoted });
    }
    Some(words)
}

/// Reduce a pattern to a literal and its anchors, or refuse it.
///
/// The refusal is the point. A pattern this cannot reduce is one where the
/// reimplementation would have to guess at `grep`, and a guess here would put a
/// wrong number in front of a reader under a command that returns a right one.
fn compile(pattern: &str) -> Result<Matcher, String> {
    let mut rest = pattern;
    let at_start = rest.starts_with('^');
    if at_start {
        rest = &rest[1..];
    }
    let mut at_end = false;
    let mut literal = String::new();
    let mut characters = rest.chars().peekable();
    while let Some(one) = characters.next() {
        match one {
            '\\' => {
                let Some(escaped) = characters.next() else {
                    return Err(String::from(
                        "this pattern ends in a backslash, which escapes nothing.",
                    ));
                };
                if "\\.[]*^$".contains(escaped) {
                    literal.push(escaped);
                } else {
                    return Err(format!(
                        "`\\{escaped}` is outside the subset this check re-runs. It reduces a \
                         pattern to a literal and its anchors, and refuses what it would have to \
                         guess at."
                    ));
                }
            }
            '$' if characters.peek().is_none() => at_end = true,
            '.' | '*' | '[' | ']' | '^' | '$' => {
                return Err(format!(
                    "`{one}` is a regular expression operator, and this check re-runs literal \
                     patterns and the two anchors only. Escape it if the file holds it literally."
                ));
            }
            other => literal.push(other),
        }
    }
    Ok(Matcher {
        at_start,
        at_end,
        literal,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{Read, compare, read};
    use std::collections::HashSet;

    /// Build a file out of lines. The fixtures carry markers, and a marker
    /// written as one line of a raw string in this file would be a live marker
    /// in a tracked file, because the scan reads every tracked file. Joined
    /// lines end in a quote and a comma instead.
    fn file(of: &[&str]) -> String {
        of.join("\n")
    }

    fn roster() -> HashSet<&'static str> {
        ["grep", "cargo", "git"].into_iter().collect()
    }

    fn judged(text: &str) -> Read {
        read("docs/example.md", text, &roster())
    }

    const LOCK: &str = "[[package]]\nname = \"one\"\n\n[[package]]\nname = \"two\"\n";

    #[test]
    fn a_declared_block_is_read_and_matches_what_the_file_holds() {
        let text = file(&[
            "Two entries:",
            "",
            "<!-- docs-lint: rerun, the count moves when a member lands -->",
            "",
            "    grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "    2",
            "",
        ]);
        let judged = judged(&text);
        assert!(judged.findings.is_empty(), "no finding was owed");
        assert_eq!(judged.blocks, 1);
        assert_eq!(judged.pasted.len(), 1);
        assert_eq!(judged.pasted[0].reads, "Cargo.lock");
        assert!(compare("docs/example.md", &judged.pasted[0], LOCK).is_none());
    }

    /// The near-miss the issue asked for: one character of the pasted number.
    #[test]
    fn a_stale_number_is_refused_and_the_finding_carries_both() {
        let text = file(&[
            "<!-- docs-lint: rerun, the count moves when a member lands -->",
            "",
            "    grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "    3",
            "",
        ]);
        let judged = judged(&text);
        let found = compare("docs/example.md", &judged.pasted[0], LOCK)
            .expect("a stale number is a finding");
        assert!(found.what.contains('3'), "what the document pastes");
        assert!(found.what.contains('2'), "what the command returns");
        assert_eq!(found.line, 3);
    }

    #[test]
    fn two_commands_in_one_block_are_two_re_runs() {
        let text = file(&[
            "<!-- docs-lint: rerun, both counts move together -->",
            "",
            "    grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "    2",
            "    grep -c 'source = ' Cargo.lock",
            "    0",
            "",
        ]);
        let judged = judged(&text);
        assert!(judged.findings.is_empty());
        assert_eq!(judged.pasted.len(), 2);
        for one in &judged.pasted {
            assert!(compare("docs/example.md", one, LOCK).is_none());
        }
    }

    /// The `deny.toml` case. The prefix is taken off the marker line, so
    /// nothing here holds a list of comment syntaxes.
    #[test]
    fn a_block_inside_comments_is_read_through_its_prefix() {
        let text = file(&[
            "# because there is no registry crate at all:",
            "#",
            "#     <!-- docs-lint: rerun, the count moves when a member lands -->",
            "#     grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "#     2",
            "wildcards = \"warn\"",
        ]);
        let judged = read("deny.toml", &text, &roster());
        assert!(judged.findings.is_empty(), "no finding was owed");
        assert_eq!(judged.pasted.len(), 1);
        assert_eq!(judged.pasted[0].output, vec![String::from("2")]);
        assert!(compare("deny.toml", &judged.pasted[0], LOCK).is_none());
    }

    #[test]
    fn a_line_that_dropped_the_prefix_is_not_part_of_the_block() {
        let text = file(&[
            "#     <!-- docs-lint: rerun, the count moves when a member lands -->",
            "#     grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "#     2",
            "wildcards = \"warn\"",
        ]);
        let judged = read("deny.toml", &text, &roster());
        assert_eq!(judged.pasted[0].output, vec![String::from("2")]);
    }

    #[test]
    fn prose_under_a_markdown_block_is_not_read_as_output() {
        let text = file(&[
            "<!-- docs-lint: rerun, the count moves when a member lands -->",
            "",
            "    grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "    2",
            "So a green run today says almost nothing.",
        ]);
        let judged = judged(&text);
        assert_eq!(judged.pasted[0].output, vec![String::from("2")]);
    }

    #[test]
    fn an_undeclared_block_is_neither_run_nor_counted() {
        let text = file(&[
            "Two entries:",
            "",
            "    grep -c '^\\[\\[package\\]\\]' Cargo.lock",
            "    99",
            "",
        ]);
        let judged = judged(&text);
        assert!(judged.findings.is_empty());
        assert_eq!(judged.blocks, 0);
        assert!(judged.pasted.is_empty());
    }

    #[test]
    fn a_marker_with_no_reason_is_refused() {
        let text = file(&[
            "<!-- docs-lint: rerun, -->",
            "",
            "    grep -c 'source = ' Cargo.lock",
            "    0",
            "",
        ]);
        let judged = judged(&text);
        assert_eq!(judged.findings.len(), 1);
        assert!(judged.findings[0].what.contains("no reason"));
    }

    #[test]
    fn a_marker_governing_nothing_is_refused() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "Prose at column zero, which is not a block.",
        ]);
        let judged = judged(&text);
        assert_eq!(judged.findings.len(), 1);
        assert!(judged.findings[0].what.contains("no block"));
    }

    #[test]
    fn a_command_outside_the_subset_is_refused_rather_than_filed_as_output() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "    cargo deny check",
            "    advisories ok, bans ok, licenses ok, sources ok",
            "",
        ]);
        let judged = judged(&text);
        assert_eq!(judged.findings.len(), 1);
        assert!(judged.findings[0].what.contains("no program but"));
        assert!(judged.pasted.is_empty());
    }

    #[test]
    fn a_pattern_operator_is_refused_rather_than_approximated() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "    grep -c '^name = .*' Cargo.lock",
            "    2",
            "",
        ]);
        let judged = judged(&text);
        assert_eq!(judged.findings.len(), 1);
        assert!(
            judged.findings[0]
                .what
                .contains("regular expression operator")
        );
    }

    #[test]
    fn a_command_with_nothing_pasted_under_it_is_refused() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "    grep -c 'source = ' Cargo.lock",
            "",
        ]);
        let judged = judged(&text);
        assert_eq!(judged.findings.len(), 1);
        assert!(judged.findings[0].what.contains("nothing is pasted"));
    }

    #[test]
    fn a_block_opening_with_output_is_refused() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "    2",
            "    grep -c 'source = ' Cargo.lock",
            "    0",
            "",
        ]);
        let judged = judged(&text);
        assert!(
            judged
                .findings
                .iter()
                .any(|one| one.what.contains("opens with output"))
        );
    }

    #[test]
    fn the_anchors_count_what_grep_counts() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "    grep -c 'name' Cargo.lock",
            "    2",
            "    grep -c '^name = \"one\"$' Cargo.lock",
            "    1",
            "",
        ]);
        let judged = judged(&text);
        assert!(judged.findings.is_empty());
        for one in &judged.pasted {
            assert!(compare("docs/example.md", one, LOCK).is_none());
        }
    }

    /// A file with no final newline still has a final line, and a file with one
    /// does not gain an empty line from it.
    #[test]
    fn the_last_line_is_counted_the_way_grep_counts_it() {
        let text = file(&[
            "<!-- docs-lint: rerun, a reason -->",
            "",
            "    grep -c 'x' f",
            "    2",
            "",
        ]);
        let judged = judged(&text);
        assert!(compare("docs/example.md", &judged.pasted[0], "x\nx").is_none());
        assert!(compare("docs/example.md", &judged.pasted[0], "x\nx\n").is_none());
        assert!(compare("docs/example.md", &judged.pasted[0], "x\nx\n\n").is_none());
    }
}
