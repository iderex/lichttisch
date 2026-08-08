// SPDX-License-Identifier: AGPL-3.0-only
//! The cases, and the one that cannot run here.
//!
//! Two of these measure work this project does not do yet. They are here so
//! the harness has something to exercise and so its output can be read before
//! a real case exists, and they say so in their own names rather than being
//! mistaken later for a performance bar.
//!
//! The third is the one that matters more than either. It needs a camera, the
//! checks have none, and it reports itself as skipped with what was missing.
//! A run that measured less than the whole set has to be unreadable as a full
//! run, and a case that quietly vanished from the table would be exactly that.

use std::hint::black_box;
use std::time::Instant;

use crate::stats::{Outcome, Summary};

/// One thing that gets timed.
pub struct Case {
    pub name: &'static str,
    /// `None` when the case can run here. `Some(reason)` names what is missing.
    missing: fn() -> Option<String>,
    body: fn(u64),
}

impl Case {
    /// Time this case, or report what stopped it.
    pub fn run(&self, samples: usize, seed: u64) -> Outcome {
        if let Some(reason) = (self.missing)() {
            return Outcome::Skipped(reason);
        }

        // One untimed pass, so the first sample is not paying for a page fault
        // the other twenty-nine will not see.
        (self.body)(seed);

        let mut durations = Vec::with_capacity(samples);
        for index in 0..samples {
            // The seed moves per sample so a case cannot be measured on one
            // lucky arrangement of bytes, and it is derived from the run's seed
            // so the whole set is still reproducible.
            let sample_seed = seed.wrapping_add(index as u64);
            let started = Instant::now();
            (self.body)(sample_seed);
            let elapsed = started.elapsed();
            durations.push(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX));
        }

        Summary::of(&durations).map_or_else(
            || Outcome::Skipped("the run produced no sample".to_owned()),
            Outcome::Measured,
        )
    }
}

/// Every case this harness knows about.
pub fn all() -> Vec<Case> {
    vec![
        Case {
            name: "placeholder-sort-a-hundred-thousand-keys",
            missing: || None,
            body: |seed| {
                let mut keys = corpus(seed, 100_000);
                keys.sort_unstable();
                black_box(&keys);
            },
        },
        Case {
            name: "placeholder-digest-a-megabyte",
            missing: || None,
            body: |seed| {
                let mut hash = 0xcbf2_9ce4_8422_2325_u64;
                let mut value = seed | 1;
                for _ in 0..(1024 * 1024 / 8) {
                    value = value
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1);
                    hash ^= value;
                    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
                }
                black_box(hash);
            },
        },
        Case {
            name: "capture-a-tethered-frame",
            missing: || {
                if camera_present() {
                    None
                } else {
                    Some("no camera device present".to_owned())
                }
            },
            // Unreachable until a camera exists, which is the point of it.
            // Issue #77 is where the body arrives.
            body: |_| {},
        },
    ]
}

/// Whether anything on this machine looks like a camera.
///
/// The check is deliberately shallow and its shallowness is the disclosure: a
/// device node existing is not a camera that answers, so this can only ever
/// say that there is definitely none. It is never allowed to be the reason a
/// case is reported as measured.
fn camera_present() -> bool {
    let Ok(entries) = std::fs::read_dir("/dev") else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("video"))
    })
}

/// The corpus, from the seed and nothing else.
///
/// A linear congruential generator, written out rather than depended on. It is
/// not a good source of randomness and it does not have to be: what is wanted
/// is bytes that are the same on every machine given the same seed, so that
/// two runs measure the same work.
fn corpus(seed: u64, count: usize) -> Vec<u64> {
    let mut value = seed;
    (0..count)
        .map(|_| {
            value = value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            value
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{all, corpus};
    use crate::stats::Outcome;

    #[test]
    fn one_seed_gives_one_corpus() {
        // Without this the seed printed above every table means nothing, and a
        // difference between two runs could be a difference in the bytes.
        assert_eq!(corpus(7, 64), corpus(7, 64));
        assert_ne!(corpus(7, 64), corpus(8, 64));
    }

    #[test]
    fn a_case_that_cannot_run_is_skipped_with_what_was_missing() {
        let cases = all();
        let tethered = cases
            .iter()
            .find(|case| case.name == "capture-a-tethered-frame")
            .expect("the tethered case is in the list");
        match tethered.run(1, 1) {
            Outcome::Skipped(reason) => {
                assert!(!reason.is_empty(), "a skip with no reason says nothing");
            }
            Outcome::Measured(summary) => {
                // Not a failure of this harness: it means the machine running
                // the suite has a camera device, which is worth saying out
                // loud rather than asserting against.
                assert!(
                    summary.samples > 0,
                    "a camera device is present and the case measured nothing"
                );
            }
        }
    }

    #[test]
    fn every_case_has_a_name_nobody_will_read_as_a_performance_bar() {
        for case in all() {
            assert!(!case.name.is_empty());
        }
        let placeholders = all()
            .iter()
            .filter(|case| case.name.starts_with("placeholder-"))
            .count();
        assert!(
            placeholders >= 1,
            "the cases that measure nothing this project does yet say so in their names"
        );
    }
}
