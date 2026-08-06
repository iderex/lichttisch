//! Samples, and the numbers taken out of them.
//!
//! Kept apart from the running and the printing because this is the part that
//! can be wrong without anybody noticing. A harness that reports the wrong
//! median is worse than no harness: it produces a number with the authority of
//! a measurement and the content of a mistake, and every issue that quotes it
//! inherits that.

/// What one case produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The case ran, and these are its sample durations in nanoseconds.
    Measured(Summary),
    /// The case did not run, and this is what was missing.
    Skipped(String),
}

/// The numbers taken out of one case's samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub samples: usize,
    pub median_ns: u64,
    pub p95_ns: u64,
}

impl Summary {
    /// Summarise the samples of one case.
    ///
    /// Returns `None` for an empty set rather than inventing a zero, because a
    /// case that produced no sample and a case that ran instantly are opposite
    /// statements and printing both as `0` collapses them.
    #[must_use]
    pub fn of(samples: &[u64]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        Some(Self {
            samples: sorted.len(),
            median_ns: percentile(&sorted, 50),
            p95_ns: percentile(&sorted, 95),
        })
    }
}

/// The nearest-rank percentile of an already sorted, non-empty slice.
///
/// Nearest rank rather than interpolation, so every number this harness prints
/// is a duration that was actually observed. An interpolated p95 is a number
/// no run produced, which is the wrong thing to paste into an issue as
/// evidence.
///
/// The rank is `ceil(p/100 * n)`, clamped into the slice, which is the
/// definition that makes `percentile(xs, 100)` the maximum and
/// `percentile(xs, 0)` the minimum.
fn percentile(sorted: &[u64], p: u64) -> u64 {
    debug_assert!(!sorted.is_empty(), "the caller checked this");
    debug_assert!(p <= 100, "a percentile above a hundred is a mistake");
    let n = sorted.len() as u64;
    let rank = (p * n).div_ceil(100);
    let index = rank.saturating_sub(1).min(n - 1);
    // The index came from n, so it fits wherever n came from.
    let index = usize::try_from(index).unwrap_or(0);
    sorted[index]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "stopping is right in a test")]

    use super::{Summary, percentile};

    #[test]
    fn one_sample_is_its_own_median_and_tail() {
        let summary = Summary::of(&[42]).expect("one sample is not none");
        assert_eq!(summary.samples, 1);
        assert_eq!(summary.median_ns, 42);
        assert_eq!(summary.p95_ns, 42);
    }

    #[test]
    fn no_samples_is_none_rather_than_zero() {
        assert_eq!(Summary::of(&[]), None);
    }

    #[test]
    fn the_summary_does_not_need_its_input_sorted() {
        let ascending = Summary::of(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let shuffled = Summary::of(&[7, 2, 9, 4, 1, 10, 3, 8, 5, 6]);
        assert_eq!(ascending, shuffled);
        assert_eq!(
            ascending.map(|summary| summary.median_ns),
            Some(5),
            "nearest rank puts the median of ten at the fifth value"
        );
    }

    #[test]
    fn the_tail_is_a_value_that_was_actually_observed() {
        // The case this leg is against: an interpolating p95 over these
        // samples lands between 95 and 1000 and reports a duration no run
        // produced. Nearest rank reports the slow one, which is the point of
        // measuring a tail at all.
        let mut samples: Vec<u64> = (1..=99).collect();
        samples.push(1000);
        let summary = Summary::of(&samples).expect("a hundred samples is not none");
        assert!(
            samples.contains(&summary.p95_ns),
            "the p95 is {} and no sample has that value",
            summary.p95_ns
        );
        assert_eq!(summary.p95_ns, 95);
    }

    #[test]
    fn the_ends_are_the_ends() {
        let sorted: Vec<u64> = (10..=20).collect();
        assert_eq!(percentile(&sorted, 0), 10);
        assert_eq!(percentile(&sorted, 100), 20);
    }
}
