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
//! machine it was run on, with no attempt to quiet that machine down. Issue
//! #107 is where a number becomes a red check, and that is where the question
//! of how stable these are has to be answered rather than here.

mod cases;
mod report;
mod stats;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

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

    ExitCode::SUCCESS
}

const USAGE: &str = "\
usage: cargo run --locked --release -p bench -- [options]

    --samples <n>     how many times each case is timed
    --seed <n>        the seed every generated corpus comes from
    --write <path>    write the result file a later run can compare against
    --compare <path>  read a result file and print what moved
";

fn parse_arguments(arguments: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        samples: DEFAULT_SAMPLES,
        seed: DEFAULT_SEED,
        write: None,
        compare_with: None,
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
            other => return Err(format!("unknown argument {other}")),
        }
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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{USAGE, nanoseconds, parse_arguments, percent};

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
        for option in ["--samples", "--seed", "--write", "--compare"] {
            assert!(
                USAGE.contains(option),
                "the usage does not mention {option}"
            );
        }
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
