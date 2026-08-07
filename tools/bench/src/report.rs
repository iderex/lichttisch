//! The result file, and the difference between two of them.
//!
//! A number is only comparable to another number produced the same way, so the
//! conditions travel with the numbers rather than in whoever pasted them. The
//! format is line-based `key=value` because it is the smallest thing that a
//! later run can read back without this tree gaining a parser dependency, and
//! because a person can read a result file without a tool.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::stats::{Outcome, Summary};

/// One run: the conditions it was under, and what each case produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub conditions: BTreeMap<String, String>,
    pub cases: BTreeMap<String, Outcome>,
}

impl Run {
    /// The run as the text a later run reads back.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for (key, value) in &self.conditions {
            let _ = writeln!(text, "run.{key}={value}");
        }
        for (name, outcome) in &self.cases {
            match outcome {
                Outcome::Measured(summary) => {
                    let _ = writeln!(text, "case.{name}.samples={}", summary.samples);
                    let _ = writeln!(text, "case.{name}.median_ns={}", summary.median_ns);
                    let _ = writeln!(text, "case.{name}.p95_ns={}", summary.p95_ns);
                }
                Outcome::Skipped(reason) => {
                    let _ = writeln!(text, "case.{name}.skipped={reason}");
                }
            }
        }
        text
    }

    /// Read back what `to_text` wrote.
    ///
    /// A line it cannot place is an error rather than a line it ignores. A
    /// reader that skips what it does not understand turns a result file from
    /// a later version into a comparison against half the cases, silently.
    ///
    /// # Errors
    ///
    /// Returns the offending line, with its number, for anything that is not a
    /// comment, a blank, or one of the keys `to_text` writes.
    pub fn from_text(text: &str) -> Result<Self, String> {
        let mut conditions = BTreeMap::new();
        let mut samples: BTreeMap<String, usize> = BTreeMap::new();
        let mut medians: BTreeMap<String, u64> = BTreeMap::new();
        let mut tails: BTreeMap<String, u64> = BTreeMap::new();
        let mut skipped: BTreeMap<String, String> = BTreeMap::new();

        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {number} carries no `=`: {line}"));
            };
            if let Some(name) = key.strip_prefix("run.") {
                conditions.insert(name.to_owned(), value.to_owned());
                continue;
            }
            let Some(rest) = key.strip_prefix("case.") else {
                return Err(format!(
                    "line {number} names neither a run nor a case: {line}"
                ));
            };
            let Some((name, field)) = rest.rsplit_once('.') else {
                return Err(format!("line {number} names a case with no field: {line}"));
            };
            let name = name.to_owned();
            match field {
                "samples" => {
                    let parsed = value.parse().map_err(|_| {
                        format!("line {number} has a sample count that is not a number: {line}")
                    })?;
                    samples.insert(name, parsed);
                }
                "median_ns" => {
                    let parsed = value.parse().map_err(|_| {
                        format!("line {number} has a median that is not a number: {line}")
                    })?;
                    medians.insert(name, parsed);
                }
                "p95_ns" => {
                    let parsed = value.parse().map_err(|_| {
                        format!("line {number} has a p95 that is not a number: {line}")
                    })?;
                    tails.insert(name, parsed);
                }
                "skipped" => {
                    skipped.insert(name, value.to_owned());
                }
                other => {
                    return Err(format!(
                        "line {number} names an unknown field `{other}`: {line}"
                    ));
                }
            }
        }

        let mut cases: BTreeMap<String, Outcome> = BTreeMap::new();
        for (name, reason) in skipped {
            cases.insert(name, Outcome::Skipped(reason));
        }
        for (name, median_ns) in medians {
            let Some(&count) = samples.get(&name) else {
                return Err(format!("case `{name}` has a median and no sample count"));
            };
            let Some(&p95_ns) = tails.get(&name) else {
                return Err(format!("case `{name}` has a median and no p95"));
            };
            cases.insert(
                name,
                Outcome::Measured(Summary {
                    samples: count,
                    median_ns,
                    p95_ns,
                }),
            );
        }

        Ok(Self { conditions, cases })
    }
}

/// What happened to one case between two runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Both runs measured it.
    Moved {
        before_ns: u64,
        after_ns: u64,
        /// Tenths of a percent, so the difference is printed rather than left
        /// for a reader to work out, and without a floating point number in a
        /// comparison anybody might rely on.
        tenths_of_a_percent: i64,
    },
    /// Both runs skipped it, for the same reason.
    StillSkipped(String),
    /// One run measured it and the other skipped it, either way round, or both
    /// skipped it for different reasons.
    NotComparable { before: Outcome, after: Outcome },
    /// Only the later run has it.
    Added,
    /// Only the earlier run has it.
    Gone,
}

/// The difference between two runs, case by case.
///
/// The conditions are not compared here and are printed by the caller. Two
/// runs under different conditions are still worth diffing; what they are not
/// worth is diffing silently, which is why both sets of conditions are printed
/// above the table.
#[must_use]
pub fn compare(before: &Run, after: &Run) -> BTreeMap<String, Change> {
    let mut changes = BTreeMap::new();
    for (name, after_outcome) in &after.cases {
        let Some(before_outcome) = before.cases.get(name) else {
            changes.insert(name.clone(), Change::Added);
            continue;
        };
        let change = match (before_outcome, after_outcome) {
            (Outcome::Measured(before_summary), Outcome::Measured(after_summary)) => {
                Change::Moved {
                    before_ns: before_summary.median_ns,
                    after_ns: after_summary.median_ns,
                    tenths_of_a_percent: relative_change(
                        before_summary.median_ns,
                        after_summary.median_ns,
                    ),
                }
            }
            (Outcome::Skipped(before_reason), Outcome::Skipped(after_reason))
                if before_reason == after_reason =>
            {
                Change::StillSkipped(after_reason.clone())
            }
            (before_outcome, after_outcome) => Change::NotComparable {
                before: before_outcome.clone(),
                after: after_outcome.clone(),
            },
        };
        changes.insert(name.clone(), change);
    }
    for name in before.cases.keys() {
        if !after.cases.contains_key(name) {
            changes.insert(name.clone(), Change::Gone);
        }
    }
    changes
}

/// The move from `before` to `after`, in tenths of a percent of `before`.
///
/// Integer arithmetic throughout. A baseline of zero has no percentage against
/// it and reports no move, which is the honest answer rather than an infinity
/// dressed up as a number.
pub fn relative_change(before_ns: u64, after_ns: u64) -> i64 {
    if before_ns == 0 {
        return 0;
    }
    let before = i128::from(before_ns);
    let after = i128::from(after_ns);
    let tenths = (after - before) * 1000 / before;
    i64::try_from(tenths).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use std::collections::BTreeMap;

    use super::{Change, Run, compare, relative_change};
    use crate::stats::{Outcome, Summary};

    fn a_run() -> Run {
        let mut conditions = BTreeMap::new();
        conditions.insert("profile".to_owned(), "release".to_owned());
        conditions.insert("seed".to_owned(), "7".to_owned());
        let mut cases = BTreeMap::new();
        cases.insert(
            "sort".to_owned(),
            Outcome::Measured(Summary {
                samples: 30,
                median_ns: 1_000,
                p95_ns: 1_400,
            }),
        );
        cases.insert(
            "capture".to_owned(),
            Outcome::Skipped("no camera device present".to_owned()),
        );
        Run { conditions, cases }
    }

    #[test]
    fn a_run_survives_being_written_and_read_back() {
        let run = a_run();
        let read = Run::from_text(&run.to_text()).expect("what to_text wrote is readable");
        assert_eq!(read, run);
    }

    #[test]
    fn a_skipped_case_is_still_there_after_the_round_trip() {
        // The failure this is against: a skipped case dropped on the way
        // through the file, so a later comparison reads the run as one that
        // measured everything it lists.
        let read = Run::from_text(&a_run().to_text()).expect("readable");
        assert_eq!(
            read.cases.get("capture"),
            Some(&Outcome::Skipped("no camera device present".to_owned()))
        );
    }

    #[test]
    fn a_line_the_reader_cannot_place_is_an_error() {
        let error =
            Run::from_text("case.sort.mean_ns=5\n").expect_err("an unknown field is refused");
        assert!(
            error.contains("mean_ns"),
            "the error names the field: {error}"
        );
    }

    #[test]
    fn a_case_with_a_median_and_no_sample_count_is_an_error() {
        let error =
            Run::from_text("case.sort.median_ns=5\ncase.sort.p95_ns=9\n").expect_err("refused");
        assert!(error.contains("sample count"), "{error}");
    }

    #[test]
    fn the_comparison_prints_the_difference() {
        let before = a_run();
        let mut after = a_run();
        after.cases.insert(
            "sort".to_owned(),
            Outcome::Measured(Summary {
                samples: 30,
                median_ns: 1_250,
                p95_ns: 1_600,
            }),
        );
        let changes = compare(&before, &after);
        assert_eq!(
            changes.get("sort"),
            Some(&Change::Moved {
                before_ns: 1_000,
                after_ns: 1_250,
                tenths_of_a_percent: 250,
            }),
            "a quarter slower is 250 tenths of a percent"
        );
    }

    #[test]
    fn a_case_skipped_by_both_runs_says_so_rather_than_reading_as_a_puzzle() {
        let changes = compare(&a_run(), &a_run());
        assert_eq!(
            changes.get("capture"),
            Some(&Change::StillSkipped("no camera device present".to_owned())),
            "two runs that both could not measure it have not disagreed about anything"
        );
    }

    #[test]
    fn a_case_that_ran_and_then_was_skipped_is_not_a_speed_up() {
        // The failure this is against: a case skipped for a missing resource
        // being read as a case that got faster, or worse, quietly omitted.
        let before = a_run();
        let mut after = a_run();
        after.cases.insert(
            "sort".to_owned(),
            Outcome::Skipped("no corpus present".to_owned()),
        );
        let changes = compare(&before, &after);
        assert!(
            matches!(changes.get("sort"), Some(Change::NotComparable { .. })),
            "got {:?}",
            changes.get("sort")
        );
    }

    #[test]
    fn a_case_only_one_run_has_is_named_rather_than_dropped() {
        let before = a_run();
        let mut after = a_run();
        after.cases.remove("capture");
        after.cases.insert(
            "decode".to_owned(),
            Outcome::Measured(Summary {
                samples: 5,
                median_ns: 3,
                p95_ns: 4,
            }),
        );
        let changes = compare(&before, &after);
        assert_eq!(changes.get("capture"), Some(&Change::Gone));
        assert_eq!(changes.get("decode"), Some(&Change::Added));
    }

    #[test]
    fn a_zero_baseline_reports_no_move_rather_than_an_infinity() {
        assert_eq!(relative_change(0, 500), 0);
        assert_eq!(relative_change(500, 0), -1000);
    }
}
