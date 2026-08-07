//! The pull-request hygiene check (#137).
//!
//! `CONTRIBUTING.md` asks a pull request body to name the issue it closes and
//! the failure the change prevents. Nothing in this tree read a body, so a
//! body naming neither merged exactly like one naming both. This is the thing
//! that refuses it.
//!
//! It is a program in the workspace rather than a block of shell inside a
//! workflow because every rule it holds carries a near-miss, and a near-miss
//! cannot be written for a pattern buried in YAML. `src/shape.rs` holds the
//! judgement as pure functions over text; this file does the reading, and the
//! workflow does the fetching and nothing else.
//!
//! ## What this check cannot judge
//!
//! Whether the failure a body claims to prevent is the real one. That is a
//! question about meaning, and no reading of a body answers it. What is
//! checkable is that the sentence exists, that it points at an issue that
//! exists, and that the issue's state is consistent with the body. A green
//! result here is not a review, and nobody should read it as one.
//!
//! Whether the issue named is the issue the change is about. A body naming any
//! open issue in this repository passes.
//!
//! ## Where its determinism stops
//!
//! The same body and the same index produce the same verdict, always. The
//! index is not part of the pull request: an issue closed after the check ran
//! turns a green verdict red on the next run without the pull request having
//! changed. That is the direct cost of refusing a stale reference, which the
//! issue asks for in the same breath as determinism, and it is written here
//! rather than left for somebody to discover on a re-run. The ordinary shape
//! of it is benign, because the issue a pull request closes is closed by the
//! merge itself, after the last run of this check.
//!
//! ## The hole it leaves open
//!
//! A pull request opened by a bot is not judged, and the run says so. A
//! dependency bump's body is written by the tool that opened it, and neither
//! the template nor the issue reference is something that tool can be asked
//! for. So the rule this check enforces is a rule about changes a person
//! wrote, and a change a bot wrote passes without being read. That is a hole
//! rather than an exemption with a reason that fully covers it.

mod shape;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Args {
    template: PathBuf,
    body: PathBuf,
    index: PathBuf,
    index_cap: usize,
    author_is_bot: bool,
}

/// The number of rows the fetch was allowed to return.
///
/// An index that came back exactly full is an index that may have been cut
/// off, and a reference beyond the cut reads as an issue that does not exist.
/// The default is far above this repository's issue count and the workflow
/// passes the same number it asked the fetch for.
const DEFAULT_INDEX_CAP: usize = 1000;

fn usage() -> String {
    String::from(
        "usage: pr-hygiene --template <path> --body <path> --index <path> [--author-is-bot]\n\
         \n\
         --template  the pull request template, which declares the shape a body owes\n\
         --body      the body under judgement\n\
         --index     one `<number> <OPEN|CLOSED>` line per issue in this repository\n",
    )
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut template = None;
    let mut body = None;
    let mut index = None;
    let mut index_cap = DEFAULT_INDEX_CAP;
    let mut author_is_bot = false;
    let mut at = 0;
    while at < raw.len() {
        let flag = raw[at].as_str();
        let value = || {
            raw.get(at + 1)
                .map(PathBuf::from)
                .ok_or_else(|| format!("{flag} takes a path"))
        };
        match flag {
            "--template" => {
                template = Some(value()?);
                at += 2;
            }
            "--body" => {
                body = Some(value()?);
                at += 2;
            }
            "--index" => {
                index = Some(value()?);
                at += 2;
            }
            "--index-cap" => {
                let Some(raw) = raw.get(at + 1) else {
                    return Err(String::from("--index-cap takes a number"));
                };
                index_cap = raw
                    .parse()
                    .map_err(|_| format!("--index-cap is not a number: {raw}"))?;
                at += 2;
            }
            "--author-is-bot" => {
                author_is_bot = true;
                at += 1;
            }
            other => return Err(format!("unknown argument {other}\n\n{}", usage())),
        }
    }
    let (Some(template), Some(body), Some(index)) = (template, body, index) else {
        return Err(format!("a path is missing\n\n{}", usage()));
    };
    Ok(Args {
        template,
        body,
        index,
        index_cap,
        author_is_bot,
    })
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|why| format!("could not read {}: {why}", path.display()))
}

/// Fail closed and say which half could not be read.
fn stop(why: &str) -> ExitCode {
    println!("::error::pr-hygiene could not judge this pull request: {why}");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&raw) {
        Ok(args) => args,
        Err(why) => return stop(&why),
    };

    let template = match read(&args.template) {
        Ok(text) => text,
        Err(why) => return stop(&why),
    };
    let declared = match shape::shape_of(&template) {
        Ok(shape) => shape,
        Err(why) => return stop(&format!("{}: {why}", args.template.display())),
    };

    let index_text = match read(&args.index) {
        Ok(text) => text,
        Err(why) => return stop(&why),
    };
    let index = match shape::read_index(&index_text) {
        Ok(rows) => rows,
        Err(why) => return stop(&why),
    };
    if index.is_empty() {
        // An empty index is a fetch that failed, not a repository with no
        // issues in it. Passing here would call every reference unknown, or
        // call none of them anything, depending on which way the code fell.
        return stop("the issue index is empty, which is a fetch that did not happen");
    }
    if index.len() >= args.index_cap {
        return stop(&format!(
            "the issue index came back with {} rows against a cap of {}, so it may have been \
             cut off and a reference beyond the cut would read as an issue that does not exist",
            index.len(),
            args.index_cap
        ));
    }

    let body = match read(&args.body) {
        Ok(text) => text,
        Err(why) => return stop(&why),
    };

    println!(
        "template={} headings={} keyword={} index={} rows under a cap of {}",
        args.template.display(),
        declared.headings.len(),
        declared.keyword,
        index.len(),
        args.index_cap
    );

    if args.author_is_bot {
        println!(
            "not judged: this pull request was opened by a bot, whose body is written by the \
             tool that opened it. See the hole named in tools/pr-hygiene/src/main.rs."
        );
        return ExitCode::SUCCESS;
    }

    let refusals = shape::judge(&body, &declared, &index);
    if refusals.is_empty() {
        println!(
            "The body names an open issue in this repository and carries every section the \
             template declares. What it says is not judged here."
        );
        return ExitCode::SUCCESS;
    }

    for refusal in &refusals {
        println!("FAIL    {}", refusal.what);
        println!("read    {}", refusal.quoted);
    }
    println!(
        "::error::{} thing(s) the pull request body owes are missing. The shape is declared in \
         .github/pull_request_template.md; fill it in rather than working around this check.",
        refusals.len()
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
    fn the_three_paths_are_required() {
        let parsed = parse_args(&args(&["--template", "t", "--body", "b", "--index", "i"]));
        let Ok(parsed) = parsed else {
            panic!("the three paths parse");
        };
        assert!(!parsed.author_is_bot);
    }

    #[test]
    fn a_missing_path_is_an_error_rather_than_a_default() {
        assert!(parse_args(&args(&["--template", "t", "--body", "b"])).is_err());
    }

    #[test]
    fn a_flag_with_no_value_is_an_error() {
        assert!(parse_args(&args(&["--template"])).is_err());
    }

    #[test]
    fn an_unknown_flag_is_an_error_rather_than_being_ignored() {
        let parsed = parse_args(&args(&[
            "--template",
            "t",
            "--body",
            "b",
            "--index",
            "i",
            "--skip",
        ]));
        assert!(parsed.is_err());
    }

    #[test]
    fn the_index_cap_defaults_rather_than_being_absent() {
        let parsed = parse_args(&args(&["--template", "t", "--body", "b", "--index", "i"]));
        let Ok(parsed) = parsed else {
            panic!("the three paths parse");
        };
        assert_eq!(parsed.index_cap, super::DEFAULT_INDEX_CAP);
    }

    #[test]
    fn an_index_cap_that_is_not_a_number_is_an_error() {
        assert!(
            parse_args(&args(&[
                "--template",
                "t",
                "--body",
                "b",
                "--index",
                "i",
                "--index-cap",
                "many",
            ]))
            .is_err()
        );
    }

    #[test]
    fn the_bot_flag_is_read() {
        let parsed = parse_args(&args(&[
            "--template",
            "t",
            "--body",
            "b",
            "--index",
            "i",
            "--author-is-bot",
        ]));
        let Ok(parsed) = parsed else {
            panic!("the flag parses");
        };
        assert!(parsed.author_is_bot);
    }
}
