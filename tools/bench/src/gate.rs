//! Judging a run against a recorded baseline (#107).
//!
//! A number in a document is a claim. A number a check refuses to fall below is
//! a rule, and this module is the distance between the two.
//!
//! The comparison is the easy half. The hard half is that a runner is noisy,
//! and a threshold set below that noise produces a red check for a reason
//! nobody can act on, which is how a performance gate turns into a check people
//! learn to wave through. So this does not compare one run against a baseline
//! and stop there. It takes two runs of the same code, back to back, and uses
//! the distance between them as what this run says its own quiet was worth. A
//! case whose two passes disagree by more than the margin is reported as not
//! judged, which is the same thing the harness already does with a case it
//! could not run.
//!
//! Three consequences, stated here rather than left to be met later.
//!
//! Noise can make this gate quieter and never redder. The faster of the two
//! passes is what is judged, so a slow pass costs a case its verdict and never
//! its colour.
//!
//! Two passes back to back measure how quiet one machine was over one minute.
//! They say nothing about the difference between two machines on two days, and
//! the margin is set above the widest gap that has been observed rather than at
//! it.
//!
//! A baseline recorded somewhere else is refused rather than compared against.
//! The conditions travel with the numbers, and a number produced under
//! different conditions is a different number wearing the same case name.

use std::collections::{BTreeMap, BTreeSet};

use crate::report::{Run, relative_change};
use crate::stats::Outcome;

/// The conditions a baseline has to have been recorded under for this run to be
/// judged against it.
///
/// Not every condition, and the omission is the point. `host_identity` is
/// always `not recorded`, so requiring it would compare two constants. The six
/// below are the ones that change a duration: the machine, the build, and what
/// the harness was asked to measure.
pub const CONDITIONS_THAT_MUST_MATCH: [&str; 6] =
    ["arch", "cpus", "os", "profile", "samples_asked_for", "seed"];

/// What the gate decided about one case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Judged, and no slower than the baseline by more than the margin.
    Held {
        baseline_ns: u64,
        judged_ns: u64,
        tenths_of_a_percent: i64,
    },
    /// Judged, and slower than the baseline by more than the margin.
    Regressed {
        baseline_ns: u64,
        judged_ns: u64,
        tenths_of_a_percent: i64,
    },
    /// The two passes of this run disagreed by more than the margin, so this
    /// run has nothing to say about the case.
    NotJudged { spread_tenths_of_a_percent: i64 },
    /// This run measures the case and the baseline does not carry it.
    NotInTheBaseline,
    /// The baseline carries the case and this run does not measure it.
    GoneFromTheRun,
    /// One of the three numbers is not a number: a skip on either side, or two
    /// skips for different reasons.
    NotComparable { what: String },
}

impl Verdict {
    /// Whether this verdict makes the run refuse.
    ///
    /// Two do. A case slower than its baseline by more than the margin is the
    /// regression this gate exists for. A case the baseline carries and the run
    /// no longer measures is the other one: a case removed or renamed takes its
    /// baseline with it, and a gate that passed over that silently would be a
    /// gate anybody could turn off by renaming what it judges.
    ///
    /// The rest do not. A case with no baseline, a case one side skipped and a
    /// case too noisy to judge are all states in which this run knows nothing,
    /// and a check that reddens where it knows nothing is a check that gets
    /// ignored. Each of them is counted and printed instead.
    #[must_use]
    pub const fn refuses(&self) -> bool {
        matches!(self, Self::Regressed { .. } | Self::GoneFromTheRun)
    }
}

/// Why a baseline could not be used at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionMismatch {
    pub condition: String,
    pub baseline: String,
    pub run: String,
}

/// Which of the conditions that have to match do not.
///
/// A condition the baseline states and this run does not, or the other way
/// round, counts as a difference rather than as an absence to be overlooked: a
/// result file from a version of the harness that recorded one fewer condition
/// is exactly the file a reader would otherwise compare against without
/// noticing.
#[must_use]
pub fn conditions_that_differ(baseline: &Run, run: &Run) -> Vec<ConditionMismatch> {
    let mut differences = Vec::new();
    for condition in CONDITIONS_THAT_MUST_MATCH {
        let recorded = baseline.conditions.get(condition);
        let now = run.conditions.get(condition);
        if recorded != now {
            differences.push(ConditionMismatch {
                condition: condition.to_owned(),
                baseline: recorded.map_or_else(|| "absent".to_owned(), Clone::clone),
                run: now.map_or_else(|| "absent".to_owned(), Clone::clone),
            });
        }
    }
    differences
}

/// Judge two passes of one run against a baseline, case by case.
///
/// `margin_tenths` is read twice and means one thing both times: how far a
/// duration may move before the difference is a difference rather than the
/// machine. It bounds the spread between the two passes above which a case is
/// not judged, and it bounds the move against the baseline above which a case
/// is refused.
#[must_use]
pub fn judge(
    baseline: &Run,
    first: &Run,
    second: &Run,
    margin_tenths: i64,
) -> BTreeMap<String, Verdict> {
    let mut verdicts = BTreeMap::new();

    for (name, first_outcome) in &first.cases {
        let Some(recorded) = baseline.cases.get(name) else {
            verdicts.insert(name.clone(), Verdict::NotInTheBaseline);
            continue;
        };
        let Some(second_outcome) = second.cases.get(name) else {
            verdicts.insert(
                name.clone(),
                Verdict::NotComparable {
                    what: "the second pass of this run does not carry it".to_owned(),
                },
            );
            continue;
        };

        let (Outcome::Measured(recorded), Outcome::Measured(one), Outcome::Measured(two)) =
            (recorded, first_outcome, second_outcome)
        else {
            verdicts.insert(
                name.clone(),
                Verdict::NotComparable {
                    what: not_comparable(recorded, first_outcome, second_outcome),
                },
            );
            continue;
        };

        let quicker = one.median_ns.min(two.median_ns);
        let slower = one.median_ns.max(two.median_ns);
        let spread = relative_change(quicker, slower);
        if spread > margin_tenths {
            verdicts.insert(
                name.clone(),
                Verdict::NotJudged {
                    spread_tenths_of_a_percent: spread,
                },
            );
            continue;
        }

        let moved = relative_change(recorded.median_ns, quicker);
        let verdict = if moved > margin_tenths {
            Verdict::Regressed {
                baseline_ns: recorded.median_ns,
                judged_ns: quicker,
                tenths_of_a_percent: moved,
            }
        } else {
            Verdict::Held {
                baseline_ns: recorded.median_ns,
                judged_ns: quicker,
                tenths_of_a_percent: moved,
            }
        };
        verdicts.insert(name.clone(), verdict);
    }

    for name in baseline.cases.keys() {
        if !first.cases.contains_key(name) {
            verdicts.insert(name.clone(), Verdict::GoneFromTheRun);
        }
    }

    verdicts
}

/// Which of the three sides could not produce a number, in the words the
/// harness already uses for a case it did not run.
///
/// The whole-set case is written short on purpose. A case nothing could measure
/// is the ordinary state of the one that wants a camera, it will be printed on
/// every run for as long as the checks have no camera, and three clauses saying
/// the same sentence is how a line stops being read.
fn not_comparable(recorded: &Outcome, first: &Outcome, second: &Outcome) -> String {
    let sides = [
        ("the baseline", recorded),
        ("the first pass", first),
        ("the second pass", second),
    ];

    let mut reasons = BTreeSet::new();
    for (_, outcome) in sides {
        if let Outcome::Skipped(reason) = outcome {
            reasons.insert(reason.as_str());
        }
    }
    match (reasons.len(), reasons.iter().next()) {
        (0, _) => return "one side carries no duration".to_owned(),
        (1, Some(reason)) if sides.iter().all(|(_, outcome)| outcome.is_skipped()) => {
            return format!("skipped by the baseline and by both passes: {reason}");
        }
        _ => {}
    }

    sides
        .iter()
        .filter_map(|(side, outcome)| match outcome {
            Outcome::Skipped(reason) => Some(format!("{side} skipped it: {reason}")),
            Outcome::Measured(_) => None,
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use std::collections::BTreeMap;

    use super::{Verdict, conditions_that_differ, judge};
    use crate::report::Run;
    use crate::stats::{Outcome, Summary};

    /// A margin of two percent, which is the shape of the one the workflow
    /// uses. Every fixture below is built around it so a reader can see which
    /// side of it each case falls on.
    const MARGIN: i64 = 20;

    fn measured(median_ns: u64) -> Outcome {
        Outcome::Measured(Summary {
            samples: 30,
            median_ns,
            p95_ns: median_ns * 2,
        })
    }

    fn run(cases: &[(&str, Outcome)]) -> Run {
        let mut conditions = BTreeMap::new();
        for (key, value) in [
            ("arch", "x86_64"),
            ("cpus", "4"),
            ("os", "linux"),
            ("profile", "release"),
            ("samples_asked_for", "30"),
            ("seed", "7"),
            ("host_identity", "not recorded"),
        ] {
            conditions.insert(key.to_owned(), value.to_owned());
        }
        Run {
            conditions,
            cases: cases
                .iter()
                .map(|(name, outcome)| ((*name).to_owned(), outcome.clone()))
                .collect(),
        }
    }

    #[test]
    fn a_case_inside_the_margin_holds() {
        let baseline = run(&[("sort", measured(1_000))]);
        let first = run(&[("sort", measured(1_015))]);
        let second = run(&[("sort", measured(1_010))]);
        assert_eq!(
            judge(&baseline, &first, &second, MARGIN).get("sort"),
            Some(&Verdict::Held {
                baseline_ns: 1_000,
                judged_ns: 1_010,
                tenths_of_a_percent: 10,
            })
        );
    }

    #[test]
    fn a_case_past_the_margin_is_refused_and_says_by_how_much() {
        let baseline = run(&[("sort", measured(1_000))]);
        let first = run(&[("sort", measured(1_250))]);
        let second = run(&[("sort", measured(1_255))]);
        let verdict = judge(&baseline, &first, &second, MARGIN);
        assert_eq!(
            verdict.get("sort"),
            Some(&Verdict::Regressed {
                baseline_ns: 1_000,
                judged_ns: 1_250,
                tenths_of_a_percent: 250,
            }),
            "a quarter slower is 250 tenths of a percent"
        );
        assert!(verdict["sort"].refuses());
    }

    #[test]
    fn the_margin_is_the_last_value_that_holds_rather_than_the_first_that_fails() {
        // The one-character mistake this is against: `>=` where the operator
        // says `>`, which would refuse a case that moved by exactly the margin
        // the tree declares acceptable, and would make every stated margin
        // one tenth of a percent tighter than it reads.
        let baseline = run(&[("sort", measured(1_000))]);
        let at = run(&[("sort", measured(1_020))]);
        let past = run(&[("sort", measured(1_021))]);
        assert!(
            !judge(&baseline, &at, &at, MARGIN)["sort"].refuses(),
            "a move of exactly the margin is inside it"
        );
        assert!(
            judge(&baseline, &past, &past, MARGIN)["sort"].refuses(),
            "one tenth of a percent past the margin is outside it"
        );
    }

    #[test]
    fn a_case_that_got_faster_is_not_a_regression_however_far_it_moved() {
        let baseline = run(&[("sort", measured(1_000))]);
        let quick = run(&[("sort", measured(100))]);
        assert!(!judge(&baseline, &quick, &quick, MARGIN)["sort"].refuses());
    }

    #[test]
    fn two_passes_that_disagree_by_more_than_the_margin_leave_the_case_unjudged() {
        // The failure this is against: a runner that went quiet for one pass
        // and not the other, reported as a regression in the code. The run
        // knows nothing about this case and says so.
        let baseline = run(&[("sort", measured(1_000))]);
        let first = run(&[("sort", measured(1_010))]);
        let second = run(&[("sort", measured(1_400))]);
        let verdict = judge(&baseline, &first, &second, MARGIN);
        assert_eq!(
            verdict.get("sort"),
            Some(&Verdict::NotJudged {
                spread_tenths_of_a_percent: 386,
            })
        );
        assert!(!verdict["sort"].refuses());
    }

    #[test]
    fn the_quicker_pass_is_the_one_judged() {
        // Noise makes this gate quieter and never redder. With the slower pass
        // judged, this case would be refused; with the quicker one it holds,
        // and the pair is a tenth of a percent inside the margin so the case is
        // judged rather than dropped.
        let baseline = run(&[("sort", measured(1_000))]);
        let first = run(&[("sort", measured(1_010))]);
        let second = run(&[("sort", measured(1_029))]);
        let verdict = judge(&baseline, &first, &second, MARGIN);
        assert!(
            matches!(
                verdict["sort"],
                Verdict::Held {
                    judged_ns: 1_010,
                    ..
                }
            ),
            "got {:?}",
            verdict["sort"]
        );
    }

    #[test]
    fn a_case_the_baseline_has_and_the_run_lost_is_refused() {
        // The failure this is against: a case renamed or removed, taking its
        // baseline with it, leaving a gate that judges one case fewer and says
        // nothing about the one it stopped judging.
        let baseline = run(&[("sort", measured(1_000)), ("digest", measured(500))]);
        let first = run(&[("sort", measured(1_000))]);
        let verdict = judge(&baseline, &first, &first, MARGIN);
        assert_eq!(verdict.get("digest"), Some(&Verdict::GoneFromTheRun));
        assert!(verdict["digest"].refuses());
    }

    #[test]
    fn a_case_the_baseline_never_carried_is_reported_rather_than_refused() {
        let baseline = run(&[("sort", measured(1_000))]);
        let first = run(&[("sort", measured(1_000)), ("decode", measured(9))]);
        let verdict = judge(&baseline, &first, &first, MARGIN);
        assert_eq!(verdict.get("decode"), Some(&Verdict::NotInTheBaseline));
        assert!(!verdict["decode"].refuses());
    }

    #[test]
    fn a_case_skipped_on_one_side_is_not_comparable_and_names_which_side() {
        let baseline = run(&[(
            "capture",
            Outcome::Skipped("no camera device present".to_owned()),
        )]);
        let first = run(&[("capture", measured(1_000))]);
        let verdict = judge(&baseline, &first, &first, MARGIN);
        let Some(Verdict::NotComparable { what }) = verdict.get("capture") else {
            panic!("got {:?}", verdict.get("capture"))
        };
        assert!(what.contains("the baseline skipped it"), "{what}");
        assert!(!verdict["capture"].refuses());
    }

    #[test]
    fn a_case_both_sides_skipped_is_not_comparable_rather_than_a_pass() {
        // A skipped case has no duration on either side. Reading that as a
        // case that held is the failure the harness already refuses to make in
        // its own table, and the gate makes the same distinction.
        let skipped = Outcome::Skipped("no camera device present".to_owned());
        let baseline = run(&[("capture", skipped.clone())]);
        let first = run(&[("capture", skipped)]);
        let verdict = judge(&baseline, &first, &first, MARGIN);
        let Some(Verdict::NotComparable { what }) = verdict.get("capture") else {
            panic!("got {:?}", verdict.get("capture"))
        };
        assert_eq!(
            what, "skipped by the baseline and by both passes: no camera device present",
            "the case nothing could measure says so once rather than three times"
        );
    }

    #[test]
    fn a_baseline_from_another_machine_is_a_difference_rather_than_a_comparison() {
        let baseline = run(&[("sort", measured(1_000))]);
        let mut elsewhere = run(&[("sort", measured(1_000))]);
        elsewhere
            .conditions
            .insert("cpus".to_owned(), "32".to_owned());
        let differences = conditions_that_differ(&baseline, &elsewhere);
        assert_eq!(differences.len(), 1, "{differences:?}");
        assert_eq!(differences[0].condition, "cpus");
        assert_eq!(differences[0].baseline, "4");
        assert_eq!(differences[0].run, "32");
    }

    #[test]
    fn a_condition_one_side_does_not_carry_is_a_difference_rather_than_an_absence() {
        // The failure this is against: a result file written by a version of
        // the harness that recorded one condition fewer, compared against
        // silently because the missing one matched nothing.
        let baseline = run(&[("sort", measured(1_000))]);
        let mut older = run(&[("sort", measured(1_000))]);
        older.conditions.remove("profile");
        let differences = conditions_that_differ(&baseline, &older);
        assert_eq!(differences.len(), 1, "{differences:?}");
        assert_eq!(differences[0].run, "absent");
    }

    #[test]
    fn a_host_identity_nobody_records_is_not_one_of_the_conditions_that_must_match() {
        let baseline = run(&[("sort", measured(1_000))]);
        let mut named = run(&[("sort", measured(1_000))]);
        named
            .conditions
            .insert("host_identity".to_owned(), "runner-4".to_owned());
        assert!(conditions_that_differ(&baseline, &named).is_empty());
    }
}
