//! The documentation lint (#103).
//!
//! This plan puts a great deal in documents: decision records, the gate
//! mapping, the measurements and later the published quality figures. Nothing
//! read any of them, and the drift that arrives first is a path or a command
//! that stopped being true. This is the thing that refuses it.
//!
//! It is a program in the workspace rather than a pattern in a `run:` block
//! because every rule it holds carries a near-miss, and a near-miss cannot be
//! written for a pattern buried in YAML. `src/lint.rs` holds the judgement as
//! pure functions over text; this file does the reading, and the workflow
//! hands it the tracked list and nothing else.
//!
//! ## What this check cannot judge
//!
//! Whether a document is true. A path that resolves is a path that exists, not
//! a path that is the right one, and a command whose program is declared is a
//! command that can start rather than one that produces what the paragraph
//! above it says it produces. A green result here is not a review of the
//! documentation and nobody should read it as one.
//!
//! Whether a rule stated in a decision record is enforced or registered. That
//! obligation is held by `crates/lichttisch/tests/architecture_rules.rs`, which
//! reads the records for it, and repeating it here would put one rule in two
//! places.
//!
//! ## Where its reading stops
//!
//! References are read out of prose and never out of a code block. A path
//! inside a command is an argument to that command, and nothing here can tell
//! one the command expects to find from one it is about to create. The
//! carriage-return demonstration in `docs/text-and-line-endings.md` is the
//! second kind, and a rule reading inside blocks would refuse it for existing.
//!
//! A command is read from the first line of a block and no further. What
//! follows the first line is the output of it, and telling a second command
//! from output that begins with a lower-case word is not something a reading of
//! the text does. A command written under output is not judged.
//!
//! ## The hole it leaves open
//!
//! A program is judged by name against a declared roster, so `cargo` is
//! accepted whatever follows it. A command naming a declared program with
//! arguments that cannot work is a command this check calls runnable. What is
//! refused is the program this tree never says it has, which is the case that
//! reaches a contributor as an instruction they cannot follow.

mod lint;

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Args {
    tracked: PathBuf,
    declarations: PathBuf,
}

fn usage() -> String {
    String::from(
        "usage: docs-lint --tracked <path> --declarations <path>\n\
         \n\
         --tracked       one tracked path per line, as `git ls-files` prints them\n\
         --declarations  the programs a document may invoke and the references that are not paths\n",
    )
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut tracked = None;
    let mut declarations = None;
    let mut at = 0;
    while at < raw.len() {
        let flag = raw[at].as_str();
        let value = || {
            raw.get(at + 1)
                .map(PathBuf::from)
                .ok_or_else(|| format!("{flag} takes a path"))
        };
        match flag {
            "--tracked" => {
                tracked = Some(value()?);
                at += 2;
            }
            "--declarations" => {
                declarations = Some(value()?);
                at += 2;
            }
            other => return Err(format!("unknown argument {other}\n\n{}", usage())),
        }
    }
    let (Some(tracked), Some(declarations)) = (tracked, declarations) else {
        return Err(format!("a path is missing\n\n{}", usage()));
    };
    Ok(Args {
        tracked,
        declarations,
    })
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|why| format!("could not read {}: {why}", path.display()))
}

/// Fail closed and say which half could not be read. A lint that cannot read
/// the tree reports that rather than reporting a clean tree.
fn stop(why: &str) -> ExitCode {
    println!("::error::docs-lint could not judge this tree: {why}");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&raw) {
        Ok(args) => args,
        Err(why) => return stop(&why),
    };

    let tracked_text = match read(&args.tracked) {
        Ok(text) => text,
        Err(why) => return stop(&why),
    };
    let tracked: HashSet<String> = tracked_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    if tracked.is_empty() {
        // An empty list is a listing that failed, not a repository with no
        // files in it. Passing here would examine nothing and report nothing.
        return stop("the tracked list is empty, which is a listing that did not happen");
    }

    let declarations_path = args.declarations.to_string_lossy().replace('\\', "/");
    if !tracked.contains(&declarations_path) {
        return stop(&format!(
            "{declarations_path} is not a tracked path, so the declarations being read are not \
             the ones the tree carries"
        ));
    }
    let declarations_text = match read(&args.declarations) {
        Ok(text) => text,
        Err(why) => return stop(&why),
    };
    let declared = match lint::read_declarations(&declarations_text) {
        Ok(read) => read,
        Err(why) => return stop(&format!("{declarations_path}: {why}")),
    };

    let mut documents: Vec<&String> = tracked
        .iter()
        .filter(|path| {
            Path::new(path)
                .extension()
                .is_some_and(|kind| kind.eq_ignore_ascii_case("md"))
        })
        .collect();
    documents.sort();
    if documents.is_empty() {
        return stop("no tracked path ends in .md, so there is no documentation to judge");
    }

    let mut findings = Vec::new();
    let mut programs_used = BTreeSet::new();
    let mut references_used = BTreeSet::new();
    for document in &documents {
        let text = match read(Path::new(document)) {
            Ok(text) => text,
            Err(why) => return stop(&why),
        };
        let judged = lint::judge_document(document, &text, &tracked, &declared);
        findings.extend(judged.findings);
        programs_used.extend(judged.programs_used);
        references_used.extend(judged.references_used);
    }
    findings.extend(lint::judge_declarations(
        &declarations_path,
        &declared,
        &programs_used,
        &references_used,
    ));

    println!(
        "examined={} document(s) tracked={} path(s) programs={} declared / {} used references={} declared / {} used",
        documents.len(),
        tracked.len(),
        declared.programs.len(),
        programs_used.len(),
        declared.references.len(),
        references_used.len()
    );

    if findings.is_empty() {
        println!(
            "Every reference resolves, every command names a declared program, every decision \
             record carries its fields, and the shape rules hold. Whether any of it is true is \
             not judged here."
        );
        return ExitCode::SUCCESS;
    }

    for finding in &findings {
        println!(
            "FAIL    {}:{} [{}] {}",
            finding.path, finding.line, finding.rule, finding.what
        );
        println!("read    {}", finding.quoted);
    }
    println!(
        "::error::{} thing(s) in the documentation do not hold. Each line above names the file, \
         the line and the rule. The declarations are in {declarations_path}.",
        findings.len()
    );
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    fn args(of: &[&str]) -> Vec<String> {
        of.iter().map(|one| (*one).to_owned()).collect()
    }

    #[test]
    fn the_two_paths_are_required() {
        assert!(parse_args(&args(&["--tracked", "t", "--declarations", "d"])).is_ok());
    }

    #[test]
    fn a_missing_path_is_an_error_rather_than_a_default() {
        assert!(parse_args(&args(&["--tracked", "t"])).is_err());
    }

    #[test]
    fn a_flag_with_no_value_is_an_error() {
        assert!(parse_args(&args(&["--tracked"])).is_err());
    }

    #[test]
    fn an_unknown_flag_is_an_error_rather_than_being_ignored() {
        assert!(
            parse_args(&args(&[
                "--tracked",
                "t",
                "--declarations",
                "d",
                "--skip-records",
            ]))
            .is_err()
        );
    }
}
