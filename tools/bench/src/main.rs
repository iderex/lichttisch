// SPDX-License-Identifier: AGPL-3.0-only
//! The benchmark harness (#25).
//!
//! The performance bar is a set of numbers, and numbers produced by different
//! methods are not comparable. This is what makes a measurement in an issue, a
//! measurement in a decision record and a measurement in an automated check
//! the same measurement.
//!
//! It exists before there is much to measure, so the first real case has
//! somewhere to go rather than a reason to be timed by hand once and quoted
//! forever.
//!
//! Three rules it holds to.
//!
//! Every number carries its conditions. The machine, the build profile and the
//! corpus seed are printed above the table and written into the result file,
//! because a number pasted into an issue without them is not evidence of
//! anything.
//!
//! A case that could not run is reported as skipped, never omitted. A run that
//! measured less than the whole set must not be readable as one that measured
//! all of it and found nothing, which is the same rule this project applies to
//! its gates.
//!
//! The comparison does the subtraction. A result file is written every run and
//! a later run reads one back and prints what moved, because a difference a
//! person has to work out by eye is a difference nobody works out.
//!
//! What it is not: a statistical instrument. It reports a median and one high
//! percentile by nearest rank, over a sample count the caller sets, on whatever
//! machine it was run on, with no attempt to quiet that machine down.
//!
//! `--gate` is where a number stops being a report and starts refusing (#107).
//! It answers the question the paragraph above leaves open, by measuring how
//! stable the numbers were in the same run that judges them rather than by
//! assuming an answer. `gate` is the module, and the reasoning is at the top of
//! it.

mod cases;
mod gate;
mod report;
mod stats;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use gate::Verdict;
use report::{Change, Run, compare};
use stats::Outcome;

/// The seed every generated corpus comes from unless the caller says otherwise.
///
/// Fixed rather than taken from the clock, so two runs measure the same bytes
/// and a difference between them is a difference in the code.
const DEFAULT_SEED: u64 = 0x6c69_6368_7474_6973;

/// How many times each case is timed unless the caller says otherwise.
///
/// Thirty is enough for a median to mean something and not enough for a p95 to
/// mean much, which is stated in the output rather than left to be assumed.
const DEFAULT_SAMPLES: usize = 30;

#[derive(Debug)]
struct Options {
    samples: usize,
    seed: u64,
    write: Option<PathBuf>,
    compare_with: Option<PathBuf>,
    gate_against: Option<PathBuf>,
    /// Tenths of a percent. `None` where the caller asked for no gate.
    ///
    /// There is no default. A margin is the whole of what this gate means, and
    /// a default would put the number in two places the day somebody passed
    /// one: the workflow that owns the gate is where it is stated, which is the
    /// same rule every other gate in this tree follows.
    margin_tenths: Option<i64>,
}

fn main() -> ExitCode {
    let options = match parse_arguments(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let run = execute(&options);
    // The second pass exists only to say how quiet this machine was while the
    // first one ran, so it is paid for only where something is being judged.
    let second_pass = options.gate_against.as_ref().map(|_| execute(&options));

    println!("conditions");
    for (key, value) in &run.conditions {
        println!("    {key}={value}");
    }
    println!();
    print_table(&run);

    if let Some(path) = &options.write {
        if let Some(parent) = path.parent()
            && let Err(err) = std::fs::create_dir_all(parent)
        {
            eprintln!("could not create {}: {err}", parent.display());
            return ExitCode::FAILURE;
        }
        if let Err(err) = std::fs::write(path, run.to_text()) {
            eprintln!("could not write {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!();
        println!("written to {}", path.display());
    }

    if let Some(path) = &options.compare_with {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("could not read {}: {err}", path.display());
                return ExitCode::FAILURE;
            }
        };
        let before = match Run::from_text(&text) {
            Ok(before) => before,
            Err(message) => {
                eprintln!("{} is not a result file: {message}", path.display());
                return ExitCode::FAILURE;
            }
        };
        println!();
        print_comparison(&before, &run, path);
    }

    if let (Some(path), Some(margin), Some(second)) = (
        &options.gate_against,
        options.margin_tenths,
        second_pass.as_ref(),
    ) {
        println!();
        return match run_the_gate(path, margin, &run, second) {
            Ok(held) => {
                if held {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        };
    }

    ExitCode::SUCCESS
}

const USAGE: &str = "\
usage: cargo run --locked --release -p bench -- [options]

    --samples <n>     how many times each case is timed
    --seed <n>        the seed every generated corpus comes from
    --write <path>    write the result file a later run can compare against
    --compare <path>  read a result file and print what moved
    --gate <path>     judge this run against a recorded baseline and refuse a
                      regression, which runs every case a second time
    --margin <tenths> how far a case may move before the gate calls it a
                      regression, in tenths of a percent, required with --gate
";

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        samples: DEFAULT_SAMPLES,
        seed: DEFAULT_SEED,
        write: None,
        compare_with: None,
        gate_against: None,
        margin_tenths: None,
    };
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        let mut value = || {
            arguments
                .next()
                .ok_or_else(|| format!("{argument} wants a value after it"))
        };
        match argument.as_str() {
            "--samples" => {
                let raw = value()?;
                options.samples = raw
                    .parse()
                    .map_err(|_| format!("--samples wants a number, not {raw}"))?;
                if options.samples == 0 {
                    return Err("--samples 0 would measure nothing".to_owned());
                }
            }
            "--seed" => {
                let raw = value()?;
                options.seed = raw
                    .parse()
                    .map_err(|_| format!("--seed wants a number, not {raw}"))?;
            }
            "--write" => options.write = Some(PathBuf::from(value()?)),
            "--compare" => options.compare_with = Some(PathBuf::from(value()?)),
            "--gate" => options.gate_against = Some(PathBuf::from(value()?)),
            "--margin" => {
                let raw = value()?;
                let margin: i64 = raw
                    .parse()
                    .map_err(|_| format!("--margin wants a number, not {raw}"))?;
                if margin < 0 {
                    return Err(
                        "--margin is tenths of a percent and a negative one refuses a case \
                         for getting faster"
                            .to_owned(),
                    );
                }
                options.margin_tenths = Some(margin);
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }

    // Refused rather than defaulted, in both directions. A gate with no margin
    // would have to invent the number that is the whole of what it means, and a
    // margin with no gate is a caller who believes something is being judged.
    match (&options.gate_against, options.margin_tenths) {
        (Some(_), None) => return Err("--gate wants a --margin to judge against".to_owned()),
        (None, Some(_)) => {
            return Err("--margin judges nothing without a --gate to read a baseline".to_owned());
        }
        _ => {}
    }

    Ok(options)
}

/// Run every case and collect what each produced.
fn execute(options: &Options) -> Run {
    let mut conditions = BTreeMap::new();
    conditions.insert("os".to_owned(), std::env::consts::OS.to_owned());
    conditions.insert("arch".to_owned(), std::env::consts::ARCH.to_owned());
    conditions.insert(
        "cpus".to_owned(),
        std::thread::available_parallelism()
            .map_or_else(|_| "unknown".to_owned(), |count| count.get().to_string()),
    );
    // Read from the build rather than from an argument, so a debug run cannot
    // be filed as a release one by whoever typed the command.
    conditions.insert(
        "profile".to_owned(),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_owned(),
    );
    conditions.insert("seed".to_owned(), options.seed.to_string());
    conditions.insert("samples_asked_for".to_owned(), options.samples.to_string());
    conditions.insert("cpu_model".to_owned(), cpu_model());
    // Named rather than left out. A reader comparing two files wants to know
    // whether the machine was the same one, and this harness cannot tell them.
    conditions.insert("host_identity".to_owned(), "not recorded".to_owned());

    let mut cases = BTreeMap::new();
    for case in cases::all() {
        cases.insert(
            case.name.to_owned(),
            case.run(options.samples, options.seed),
        );
    }
    Run { conditions, cases }
}

/// Which processor produced these durations, where the machine says so.
///
/// This is not decoration. Two runs of one tree on `ubuntu-latest` were
/// measured 13.9 and 17.4 percent apart while two runs on one processor were
/// measured 0.03 percent apart, so the processor is the whole of the difference
/// between two runs of this harness on that fleet, and a baseline that does not
/// name it is a number from another machine.
///
/// Read from the one file that states it, and reported as unread where there is
/// no such file rather than guessed at. Two machines that both report it as
/// unread compare as though they were one machine, which is the same hole
/// `host_identity` already declares and is stated here for the same reason.
fn cpu_model() -> String {
    let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") else {
        return "not readable on this platform".to_owned();
    };
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.trim() == "model name")
        .map_or_else(
            || "not named in /proc/cpuinfo".to_owned(),
            |(_, value)| value.trim().to_owned(),
        )
}

fn print_table(run: &Run) {
    let width = run.cases.keys().map(String::len).max().unwrap_or(4).max(4);
    println!(
        "{:<width$}  {:>7}  {:>14}  {:>14}",
        "case", "samples", "median", "p95"
    );
    for (name, outcome) in &run.cases {
        match outcome {
            Outcome::Measured(summary) => println!(
                "{:<width$}  {:>7}  {:>14}  {:>14}",
                name,
                summary.samples,
                nanoseconds(summary.median_ns),
                nanoseconds(summary.p95_ns),
            ),
            Outcome::Skipped(reason) => {
                println!("{name:<width$}  {:>7}  skipped: {reason}", "-");
            }
        }
    }

    let skipped = run
        .cases
        .values()
        .filter(|outcome| matches!(outcome, Outcome::Skipped(_)))
        .count();
    println!();
    if skipped == 0 {
        println!("{} case(s), none skipped", run.cases.len());
    } else {
        println!(
            "{} case(s), {skipped} skipped. This run measured less than the whole \
             set and must not be read as one that measured all of it.",
            run.cases.len()
        );
    }
}

fn print_comparison(before: &Run, after: &Run, path: &std::path::Path) {
    println!("against {}", path.display());
    for (key, value) in &before.conditions {
        let now = after.conditions.get(key);
        if now != Some(value) {
            println!(
                "    {key} was {value} and is now {}",
                now.map_or("absent", String::as_str)
            );
        }
    }
    println!();

    let changes = compare(before, after);
    let width = changes.keys().map(String::len).max().unwrap_or(4).max(4);
    for (name, change) in &changes {
        match change {
            Change::Moved {
                before_ns,
                after_ns,
                tenths_of_a_percent,
            } => println!(
                "{name:<width$}  {} -> {}  {}",
                nanoseconds(*before_ns),
                nanoseconds(*after_ns),
                percent(*tenths_of_a_percent),
            ),
            Change::NotComparable { before, after } => println!(
                "{name:<width$}  not comparable: {} then {}",
                describe(before),
                describe(after),
            ),
            Change::StillSkipped(reason) => {
                println!("{name:<width$}  skipped by both runs: {reason}");
            }
            Change::Added => println!("{name:<width$}  only in this run"),
            Change::Gone => println!("{name:<width$}  only in the earlier run"),
        }
    }
}

/// Judge this run against a recorded baseline, and say what was decided.
///
/// `Ok(true)` is a run that refused nothing. `Ok(false)` is a run that refused
/// something and has already said what. `Err` is a run that could not read the
/// baseline at all, which is a failure of the gate rather than a verdict about
/// the code.
fn run_the_gate(path: &Path, margin: i64, first: &Run, second: &Run) -> Result<bool, String> {
    let recorded = read_baselines(path)?;
    let chosen = gate::choose(&recorded, first);

    let name = match &chosen {
        gate::Chosen::TheOne(name) => (*name).to_owned(),
        gate::Chosen::MoreThanOne(names) => {
            return Err(format!(
                "{} holds more than one baseline recorded under the conditions this run is \
                 under, so which one judges would be whichever sorted first: {}",
                path.display(),
                names.join(", ")
            ));
        }
        gate::Chosen::NoneForTheseConditions => {
            // Reported, not refused. This fleet hands out more than one
            // machine, so a machine nobody has recorded a baseline for is an
            // ordinary event rather than a defect in the change under test, and
            // a red gate here would block a queue on something the change did
            // not touch. What it costs is that a green tick from this gate
            // means either "judged and held" or "could not judge", and only the
            // run itself says which. docs/required-checks.md carries that.
            println!(
                "gate against {}, margin {}",
                path.display(),
                magnitude(margin)
            );
            println!();
            for condition in gate::CONDITIONS_THAT_MUST_MATCH {
                println!(
                    "    {condition}={}",
                    first
                        .conditions
                        .get(condition)
                        .map_or("absent", String::as_str)
                );
            }
            println!();
            println!(
                "No baseline in {} was recorded under those conditions, so this run judged \
                 nothing and must not be read as one that judged and held. Record one by \
                 putting a result file from this machine there, in a change that says why.",
                path.display()
            );
            return Ok(true);
        }
    };

    let baseline = recorded
        .iter()
        .find(|(recorded_name, _)| *recorded_name == name)
        .map(|(_, baseline)| baseline)
        .ok_or_else(|| format!("the chosen baseline {name} is not among the ones read"))?;

    println!("gate against {name}, margin {}", magnitude(margin));

    let verdicts = gate::judge(baseline, first, second, margin);
    println!();
    print_verdicts(&verdicts);

    let judged = verdicts
        .values()
        .filter(|verdict| matches!(verdict, Verdict::Held { .. } | Verdict::Regressed { .. }))
        .count();
    println!();
    println!(
        "{} case(s), {judged} judged against the baseline.",
        verdicts.len()
    );
    if judged < verdicts.len() {
        println!(
            "This run judged less than the whole set and must not be read as one that judged \
             all of it."
        );
    }

    Ok(print_refusals(&verdicts, &name, margin))
}

/// Every baseline the gate was pointed at, named by its path.
///
/// One file is one baseline. A directory is every result file in it, which is
/// how one gate covers a fleet that hands out more than one machine. A file in
/// there that will not parse stops the run rather than being skipped: a
/// baseline nobody can read and a baseline that does not apply are opposite
/// states, and a gate that treated them alike would go quiet the day one was
/// mistyped.
fn read_baselines(path: &Path) -> Result<Vec<(String, Run)>, String> {
    let mut paths = Vec::new();
    if path.is_dir() {
        let entries = std::fs::read_dir(path)
            .map_err(|err| format!("could not read {}: {err}", path.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("could not read {}: {err}", path.display()))?;
            let found = entry.path();
            if found.extension().is_some_and(|kind| kind == "txt") {
                paths.push(found);
            }
        }
        paths.sort();
        if paths.is_empty() {
            return Err(format!("{} holds no result file", path.display()));
        }
    } else {
        paths.push(path.to_path_buf());
    }

    let mut baselines = Vec::with_capacity(paths.len());
    for found in paths {
        let text = std::fs::read_to_string(&found)
            .map_err(|err| format!("could not read the baseline {}: {err}", found.display()))?;
        let run = Run::from_text(&text)
            .map_err(|message| format!("{} is not a result file: {message}", found.display()))?;
        baselines.push((found.display().to_string(), run));
    }
    Ok(baselines)
}

/// The verdict table, one row per case.
fn print_verdicts(verdicts: &BTreeMap<String, Verdict>) {
    let width = verdicts.keys().map(String::len).max().unwrap_or(4).max(4);
    println!(
        "{:<width$}  {:>14}  {:>14}  verdict",
        "case", "baseline", "this run"
    );
    for (name, verdict) in verdicts {
        let (baseline, now, said) = match verdict {
            Verdict::Held {
                baseline_ns,
                judged_ns,
                tenths_of_a_percent,
            } => (
                nanoseconds(*baseline_ns),
                nanoseconds(*judged_ns),
                format!("held {}", percent(*tenths_of_a_percent)),
            ),
            Verdict::Regressed {
                baseline_ns,
                judged_ns,
                tenths_of_a_percent,
            } => (
                nanoseconds(*baseline_ns),
                nanoseconds(*judged_ns),
                format!("REGRESSED {}", percent(*tenths_of_a_percent)),
            ),
            Verdict::NotJudged {
                spread_tenths_of_a_percent,
            } => (
                "-".to_owned(),
                "-".to_owned(),
                format!(
                    "not judged: the two passes of this run are {} apart",
                    magnitude(*spread_tenths_of_a_percent)
                ),
            ),
            Verdict::NotInTheBaseline => (
                "-".to_owned(),
                "-".to_owned(),
                "not in the baseline".to_owned(),
            ),
            Verdict::GoneFromTheRun => (
                "-".to_owned(),
                "-".to_owned(),
                "GONE: the baseline carries it and this run does not measure it".to_owned(),
            ),
            Verdict::NotComparable { what } => (
                "-".to_owned(),
                "-".to_owned(),
                format!("not comparable: {what}"),
            ),
        };
        println!("{name:<width$}  {baseline:>14}  {now:>14}  {said}");
    }
}

/// Say what this run refused, and whether it refused anything.
///
/// Returns whether the run held. The sentences are separate from the table
/// above so that a refusal is legible in a log somebody is scrolling rather
/// than one row among however many the harness has grown to.
fn print_refusals(verdicts: &BTreeMap<String, Verdict>, baseline: &str, margin: i64) -> bool {
    let refused: Vec<(&String, &Verdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| verdict.refuses())
        .collect();
    if refused.is_empty() {
        return true;
    }

    println!();
    for (name, verdict) in refused {
        match verdict {
            Verdict::Regressed {
                baseline_ns,
                judged_ns,
                tenths_of_a_percent,
            } => println!(
                "refused: {name} is {} against {baseline}, which is {} to {} and is past the {} \
                 margin",
                percent(*tenths_of_a_percent),
                nanoseconds(*baseline_ns),
                nanoseconds(*judged_ns),
                magnitude(margin),
            ),
            Verdict::GoneFromTheRun => println!(
                "refused: {baseline} carries {name} and this run does not measure it, so its \
                 baseline judges nothing. Remove it from the baseline in a change that says why."
            ),
            _ => {}
        }
    }
    false
}

fn describe(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Measured(summary) => nanoseconds(summary.median_ns),
        Outcome::Skipped(reason) => format!("skipped ({reason})"),
    }
}

/// A duration, in the unit that keeps three or four digits in front of a
/// reader rather than a wall of zeroes.
fn nanoseconds(value: u64) -> String {
    if value >= 1_000_000_000 {
        format!(
            "{}.{:03} s",
            value / 1_000_000_000,
            value % 1_000_000_000 / 1_000_000
        )
    } else if value >= 1_000_000 {
        format!("{}.{:03} ms", value / 1_000_000, value % 1_000_000 / 1_000)
    } else if value >= 1_000 {
        format!("{}.{:03} us", value / 1_000, value % 1_000)
    } else {
        format!("{value} ns")
    }
}

/// Tenths of a percent, printed with a sign so a reader does not have to work
/// out which direction the run went.
fn percent(tenths: i64) -> String {
    let sign = match tenths.signum() {
        1 => "+",
        -1 => "-",
        _ => " ",
    };
    let magnitude = tenths.unsigned_abs();
    format!("{sign}{}.{} percent", magnitude / 10, magnitude % 10)
}

/// Tenths of a percent with no direction, for a distance rather than a move. A
/// margin and a spread are both widths, and a sign in front of one reads as a
/// claim about which way something went.
fn magnitude(tenths: i64) -> String {
    let magnitude = tenths.unsigned_abs();
    format!("{}.{} percent", magnitude / 10, magnitude % 10)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{USAGE, magnitude, nanoseconds, parse_arguments, percent};

    fn arguments(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn the_defaults_measure_something() {
        let options = parse_arguments(arguments(&[]).into_iter()).expect("no arguments is valid");
        assert!(options.samples > 0);
        assert!(options.write.is_none());
        assert!(options.compare_with.is_none());
    }

    #[test]
    fn asking_for_no_samples_is_refused_rather_than_producing_an_empty_table() {
        let error = parse_arguments(arguments(&["--samples", "0"]).into_iter())
            .expect_err("zero samples is refused");
        assert!(error.contains("measure nothing"), "{error}");
    }

    #[test]
    fn an_option_with_nothing_after_it_is_refused() {
        let error =
            parse_arguments(arguments(&["--seed"]).into_iter()).expect_err("a value is wanted");
        assert!(error.contains("--seed"), "{error}");
    }

    #[test]
    fn an_unknown_argument_is_refused_rather_than_ignored() {
        // The failure this is against: a mistyped option silently doing
        // nothing, so a run believed to be at one seed was at another.
        let error = parse_arguments(arguments(&["--sed", "7"]).into_iter())
            .expect_err("an unknown argument is refused");
        assert!(error.contains("--sed"), "{error}");
    }

    #[test]
    fn the_usage_names_every_option_the_parser_accepts() {
        for option in [
            "--samples",
            "--seed",
            "--write",
            "--compare",
            "--gate",
            "--margin",
        ] {
            assert!(
                USAGE.contains(option),
                "the usage does not mention {option}"
            );
        }
    }

    #[test]
    fn a_gate_with_no_margin_is_refused_rather_than_given_one() {
        // The failure this is against: a gate falling back to a number nobody
        // passed, so a run believed to be judging at one margin was judging at
        // another, and the workflow that owns the gate is no longer the
        // authority for what it refuses.
        let error = parse_arguments(arguments(&["--gate", "b.txt"]).into_iter())
            .expect_err("a margin is wanted");
        assert!(error.contains("--margin"), "{error}");
    }

    #[test]
    fn a_margin_with_no_gate_is_refused_rather_than_ignored() {
        let error = parse_arguments(arguments(&["--margin", "20"]).into_iter())
            .expect_err("a gate is wanted");
        assert!(error.contains("--gate"), "{error}");
    }

    #[test]
    fn a_negative_margin_is_refused() {
        let error = parse_arguments(arguments(&["--gate", "b.txt", "--margin", "-20"]).into_iter())
            .expect_err("a negative margin is refused");
        assert!(error.contains("faster"), "{error}");
    }

    #[test]
    fn a_gate_and_a_margin_together_are_accepted() {
        let options =
            parse_arguments(arguments(&["--gate", "b.txt", "--margin", "20"]).into_iter())
                .expect("a gate with a margin is valid");
        assert!(options.gate_against.is_some());
        assert_eq!(options.margin_tenths, Some(20));
    }

    #[test]
    fn a_width_is_printed_without_a_direction() {
        assert_eq!(magnitude(20), "2.0 percent");
        assert_eq!(magnitude(0), "0.0 percent");
    }

    #[test]
    fn a_duration_is_printed_in_a_unit_a_reader_can_hold() {
        assert_eq!(nanoseconds(999), "999 ns");
        assert_eq!(nanoseconds(1_500), "1.500 us");
        assert_eq!(nanoseconds(2_250_000), "2.250 ms");
        assert_eq!(nanoseconds(3_400_000_000), "3.400 s");
    }

    #[test]
    fn a_difference_is_printed_with_its_direction() {
        assert_eq!(percent(250), "+25.0 percent");
        assert_eq!(percent(-125), "-12.5 percent");
        assert_eq!(percent(0), " 0.0 percent");
    }
}
