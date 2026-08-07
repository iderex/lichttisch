//! What the corpus is, worked out before a byte is written.
//!
//! Everything here is a function of the parameters and nothing else. No clock
//! is read, no directory is listed, no random number arrives from the
//! operating system. That is what lets a second machine produce the same tree,
//! and it is why the plan is built in one place rather than decided while
//! writing.
//!
//! What is realistic here is the shape of the load, not the picture. A file in
//! this corpus holds no photograph, so nothing measured on it says anything
//! about culling quality. It is built to have the file sizes, the timestamp
//! clustering, the name collisions, the directory depth and the damage that a
//! real card and a real library have, because those are what a catalogue, a
//! metadata reader and an import path actually meet.

/// A camera this corpus pretends to have been shot on.
///
/// The byte ranges are stated constants chosen to cover the span raw files
/// occupy. No camera was read to produce them, and this sentence is here so
/// the numbers are never quoted as a measurement of anything.
struct Model {
    /// What the plan calls it. Not a manufacturer's name for anything.
    label: &'static str,
    /// The filename stem a camera of this kind writes.
    prefix: &'static str,
    /// The extension, uppercase, the way a camera writes it.
    extension: &'static str,
    /// The tag a camera of this kind puts in its own directory names.
    tag: &'static str,
    smallest: u64,
    largest: u64,
}

const MODELS: [Model; 3] = [
    Model {
        label: "raw-24mp",
        prefix: "IMG_",
        extension: "CR3",
        tag: "CANON",
        smallest: 24_000_000,
        largest: 33_000_000,
    },
    Model {
        label: "raw-45mp",
        prefix: "DSC_",
        extension: "NEF",
        tag: "NIKON",
        smallest: 45_000_000,
        largest: 62_000_000,
    },
    Model {
        label: "raw-33mp",
        prefix: "DSC0",
        extension: "ARW",
        tag: "MSDCF",
        smallest: 33_000_000,
        largest: 42_000_000,
    },
];

/// The counter a camera writes into a filename, and the value it wraps at.
///
/// Four digits and back to one. This is where duplicate filenames across
/// directories come from in a real archive, and a card fresh out of the box
/// starts at the same place, so two cards collide from the first frame.
const COUNTER_WRAP: u32 = 9999;

/// Where the corpus starts in time. Stated rather than derived from a clock,
/// because a corpus whose timestamps depend on the day it was generated is not
/// reproducible.
const WINDOW_START_UNIX: u64 = 1_640_995_200; // 2022-01-01T00:00:00Z

/// How many bytes the header occupies, so a divided file is never shorter than
/// the thing that says what it is.
pub(crate) const HEADER_BUDGET: usize = 128;

/// How many frames a session holds, and how they are broken into bursts.
const FRAMES_PER_SESSION: (u64, u64) = (40, 400);
const FRAMES_PER_BURST: (u64, u64) = (2, 9);
/// Milliseconds between two frames of one burst.
const BURST_GAP_MS: (u64, u64) = (180, 2_400);
/// Seconds between two bursts of one session.
const SESSION_GAP_S: (u64, u64) = (20, 2_400);
/// The hours of the day a session starts in.
const SESSION_START_HOUR: (u64, u64) = (7, 20);

/// What one run was asked for.
#[derive(Clone, Copy)]
pub(crate) struct Params {
    pub files: usize,
    pub seed: u64,
    pub cards: u32,
    pub years: u32,
    /// Files per thousand that arrive damaged, the way a failing card damages
    /// them.
    pub malformed_permille: u32,
    /// What every planned length is divided by before anything is written.
    ///
    /// One writes the lengths the plan states. Anything above one writes a
    /// tree with the same shape and smaller files, which is the only way a
    /// hundred thousand file corpus fits on an ordinary disk. The value is
    /// printed with every run, so a set generated with a divisor cannot be
    /// mistaken for one generated without it.
    pub byte_divisor: u64,
}

/// What is wrong with a file, where something is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Defect {
    /// Nothing. The file is as long as it says it is.
    None,
    /// The write stopped part way, the way a card pulled mid-write stops it.
    Truncated,
    /// The file is its full length and its opening bytes are not what a reader
    /// of this format expects.
    BadHeader,
}

/// One file, and everything about it that is decided before it is written.
pub(crate) struct PlannedFile {
    /// Relative to the output directory, with forward slashes.
    pub path: String,
    pub model: &'static str,
    pub captured_unix: u64,
    /// The length a file of this kind would have on a real card.
    pub declared_len: u64,
    /// The length this run writes, after the divisor and after any damage.
    pub written_len: u64,
    pub defect: Defect,
    pub session: u32,
    pub burst: u32,
    /// The seed the filler bytes come from.
    pub content_seed: u64,
}

/// The whole corpus, in the order it will be written.
pub(crate) struct Plan {
    pub files: Vec<PlannedFile>,
    pub sessions: u32,
    pub bursts: u32,
}

/// The generator, written out rather than depended on.
///
/// A linear congruential generator. It is not a good source of randomness and
/// it does not have to be: what is wanted is a stream that is the same on
/// every machine given the same seed. Anything stronger would be a dependency,
/// and a dependency is a thing that can change what this produces without this
/// tree saying so.
struct Stream(u64);

impl Stream {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // The low bits of this generator are the weak ones, so what is handed
        // out is the state rotated rather than the state.
        self.0.rotate_right(17)
    }

    /// A value in `0..span`, or zero where the span is empty.
    fn below(&mut self, span: u64) -> u64 {
        if span == 0 { 0 } else { self.next() % span }
    }

    /// A value in `low..=high`.
    fn between(&mut self, low: u64, high: u64) -> u64 {
        low + self.below(high.saturating_sub(low).saturating_add(1))
    }
}

/// Work out the whole corpus from the parameters and nothing else.
pub(crate) fn build(params: Params) -> Plan {
    let mut stream = Stream::new(params.seed);

    // How many frames each session holds is drawn before any session is
    // placed, because the placement needs to know how many there will be. A
    // session placed against a guess would pack the last ones together and the
    // corpus would not span the years it was asked for.
    let mut session_sizes = Vec::new();
    let mut planned = 0_usize;
    while planned < params.files {
        let wanted = usize::try_from(stream.between(FRAMES_PER_SESSION.0, FRAMES_PER_SESSION.1))
            .unwrap_or(usize::MAX);
        session_sizes.push(wanted);
        planned = planned.saturating_add(wanted);
    }
    if session_sizes.is_empty() {
        session_sizes.push(0);
    }
    let session_count = u64::try_from(session_sizes.len()).unwrap_or(1).max(1);

    let cards = params.cards.max(1);
    let mut counters = vec![1_u32; usize::try_from(cards).unwrap_or(1)];
    let window = u64::from(params.years.max(1)) * 365 * 24 * 3600;

    let mut files = Vec::with_capacity(params.files);
    let mut bursts = 0_u32;
    let mut sessions = 0_u32;

    for (index, wanted) in session_sizes.iter().copied().enumerate() {
        if files.len() >= params.files {
            break;
        }
        let session = u32::try_from(index).unwrap_or(u32::MAX);
        sessions = session.saturating_add(1);
        let card = usize::try_from(session % cards).unwrap_or(0);
        let model = &MODELS[card % MODELS.len()];

        // Evenly across the window, then moved by a few days and started at an
        // hour somebody would actually shoot at.
        let place = u64::try_from(index).unwrap_or(0) * window / session_count;
        let hour = stream.between(SESSION_START_HOUR.0, SESSION_START_HOUR.1);
        let drift = stream.below(3 * 86_400);
        let start = WINDOW_START_UNIX + place + drift + hour * 3600;

        let mut moment_ms = start * 1000;
        let mut in_session = 0_usize;

        while in_session < wanted && files.len() < params.files {
            let burst_len = stream.between(FRAMES_PER_BURST.0, FRAMES_PER_BURST.1);
            let burst = bursts;
            bursts = bursts.saturating_add(1);

            for _ in 0..burst_len {
                if in_session >= wanted || files.len() >= params.files {
                    break;
                }
                let counter = counters[card];
                counters[card] = if counter >= COUNTER_WRAP {
                    1
                } else {
                    counter + 1
                };

                let captured_unix = moment_ms / 1000;
                let prefix = model.prefix;
                let extension = model.extension;
                let name = format!("{prefix}{counter:04}.{extension}");
                let directory = if session % 2 == 0 {
                    // The tree a camera writes.
                    let tag = model.tag;
                    let folder = 100 + (counter / 100) % 900;
                    format!("card-{card:02}/DCIM/{folder:03}{tag}")
                } else {
                    // The tree an import produces.
                    let (year, month, day) = civil_from_unix(captured_unix);
                    format!("library/{year:04}/{year:04}-{month:02}-{day:02}/session-{session:04}")
                };

                files.push(PlannedFile {
                    path: format!("{directory}/{name}"),
                    model: model.label,
                    captured_unix,
                    declared_len: stream.between(model.smallest, model.largest),
                    written_len: 0,
                    defect: Defect::None,
                    session,
                    burst,
                    content_seed: stream.next(),
                });

                moment_ms += stream.between(BURST_GAP_MS.0, BURST_GAP_MS.1);
                in_session += 1;
            }

            moment_ms += stream.between(SESSION_GAP_S.0, SESSION_GAP_S.1) * 1000;
        }
    }

    apply_damage(&mut files, params);
    // Sorted by path so the digest is a fact about the set rather than about
    // the order the plan happened to build it in.
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Plan {
        files,
        sessions,
        bursts,
    }
}

/// Mark exactly the stated fraction as damaged, and set what each file writes.
///
/// The selection is a counted step rather than a coin flip per file, so the
/// count is exactly the fraction that was asked for and not a sample from it.
/// A fraction that came out near the asked-for one would be a number nobody
/// could quote.
fn apply_damage(files: &mut [PlannedFile], params: Params) {
    let total = u64::try_from(files.len()).unwrap_or(u64::MAX).max(1);
    let wanted = total * u64::from(params.malformed_permille) / 1000;
    let divisor = params.byte_divisor.max(1);
    let mut damaged = 0_u64;

    for (index, file) in files.iter_mut().enumerate() {
        let position = u64::try_from(index).unwrap_or(0);
        let floor = u64::try_from(HEADER_BUDGET).unwrap_or(128);
        let full = (file.declared_len / divisor).max(floor);

        if position * wanted / total == (position + 1) * wanted / total {
            file.defect = Defect::None;
            file.written_len = full;
            continue;
        }

        // Two kinds, alternating, because they fail a reader in different
        // places: one runs out of bytes, the other has bytes that do not mean
        // what the extension says they mean.
        if damaged.is_multiple_of(2) {
            file.defect = Defect::Truncated;
            file.written_len = (full * 37 / 100).max(1);
        } else {
            file.defect = Defect::BadHeader;
            file.written_len = full;
        }
        damaged += 1;
    }
}

/// Civil date from a Unix second.
///
/// Hinnant's days-to-civil algorithm, written out for the same reason the
/// generator is: a date in a path has to come out the same on every machine,
/// and this is a few lines of arithmetic rather than a dependency.
pub(crate) fn civil_from_unix(unix: u64) -> (u64, u64, u64) {
    let days = i64::try_from(unix / 86_400).unwrap_or(0) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (
        u64::try_from(year).unwrap_or(0),
        u64::try_from(month).unwrap_or(1),
        u64::try_from(day).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use super::{Defect, Params, Plan, build, civil_from_unix};
    use std::collections::{HashMap, HashSet};

    fn params(files: usize, seed: u64) -> Params {
        Params {
            files,
            seed,
            cards: 6,
            years: 4,
            malformed_permille: 10,
            byte_divisor: 1,
        }
    }

    fn shape(plan: &Plan) -> Vec<(String, u64, u64, u64)> {
        plan.files
            .iter()
            .map(|file| {
                (
                    file.path.clone(),
                    file.declared_len,
                    file.content_seed,
                    file.captured_unix,
                )
            })
            .collect()
    }

    #[test]
    fn one_seed_gives_one_plan() {
        // Without this the seed printed above every run means nothing, and a
        // difference between two corpora could be a difference in the plan
        // rather than in what was asked for.
        let left = build(params(2_000, 7));
        let right = build(params(2_000, 7));
        let other = build(params(2_000, 8));

        assert_eq!(shape(&left), shape(&right));
        assert_ne!(shape(&left), shape(&other));
    }

    #[test]
    fn the_file_count_is_the_count_that_was_asked_for() {
        for files in [1_usize, 41, 500, 2_000] {
            assert_eq!(build(params(files, 3)).files.len(), files);
        }
    }

    #[test]
    fn the_damaged_fraction_is_the_fraction_that_was_asked_for() {
        // A fraction that came out near the asked-for one would be a number
        // nobody could quote, which is the whole reason the selection is a
        // counted step rather than a coin flip.
        let plan = build(params(2_000, 11));
        let damaged = plan
            .files
            .iter()
            .filter(|file| file.defect != Defect::None)
            .count();
        assert_eq!(damaged, 2_000 * 10 / 1000);

        let truncated = plan
            .files
            .iter()
            .filter(|file| file.defect == Defect::Truncated)
            .count();
        let bad_header = plan
            .files
            .iter()
            .filter(|file| file.defect == Defect::BadHeader)
            .count();
        assert!(truncated > 0 && bad_header > 0, "both kinds are present");
        assert_eq!(truncated + bad_header, damaged);
    }

    #[test]
    fn a_truncated_file_is_shorter_than_the_length_it_declares() {
        let plan = build(params(2_000, 29));
        let mut seen = 0;
        for file in &plan.files {
            if file.defect == Defect::Truncated {
                assert!(
                    file.written_len < file.declared_len,
                    "{} declares {} and writes {}",
                    file.path,
                    file.declared_len,
                    file.written_len
                );
                seen += 1;
            }
        }
        assert!(seen > 0, "no truncated file to check");
    }

    #[test]
    fn the_same_filename_appears_in_more_than_one_directory() {
        // The case an import path gets wrong. Two cards fresh out of the box
        // both start their counter at one, so the collision is not an exotic
        // arrangement, it is the ordinary one.
        let plan = build(params(2_000, 13));
        let mut homes: HashMap<&str, HashSet<&str>> = HashMap::new();
        for file in &plan.files {
            let (directory, name) = file
                .path
                .rsplit_once('/')
                .unwrap_or(("", file.path.as_str()));
            homes.entry(name).or_default().insert(directory);
        }
        let colliding = homes.values().filter(|homes| homes.len() > 1).count();
        assert!(
            colliding > 0,
            "no filename appeared in two directories, so the corpus does not carry the case"
        );
    }

    #[test]
    fn the_timestamps_span_the_years_that_were_asked_for() {
        let plan = build(params(4_000, 17));
        let earliest = plan
            .files
            .iter()
            .map(|file| file.captured_unix)
            .min()
            .unwrap_or_default();
        let latest = plan
            .files
            .iter()
            .map(|file| file.captured_unix)
            .max()
            .unwrap_or_default();
        let span_years = (latest - earliest) / (365 * 24 * 3600);
        assert!(
            span_years >= 2,
            "the corpus spans {span_years} year(s), which is not several"
        );
    }

    #[test]
    fn a_session_holds_bursts_and_a_burst_holds_frames_seconds_apart() {
        let plan = build(params(2_000, 19));
        assert!(plan.sessions > 1, "one session is not a shoot structure");
        assert!(
            plan.bursts > plan.sessions,
            "a session holds several bursts"
        );

        let mut by_burst: HashMap<u32, Vec<u64>> = HashMap::new();
        for file in &plan.files {
            by_burst
                .entry(file.burst)
                .or_default()
                .push(file.captured_unix);
        }
        let mut tight = 0;
        for moments in by_burst.values_mut() {
            moments.sort_unstable();
            if moments.len() > 1 && moments[moments.len() - 1] - moments[0] < 30 {
                tight += 1;
            }
        }
        assert!(tight > 0, "no burst held frames seconds apart");
    }

    #[test]
    fn the_tree_has_the_depth_a_photographer_produces() {
        let plan = build(params(2_000, 23));
        let deepest = plan
            .files
            .iter()
            .map(|file| file.path.matches('/').count())
            .max()
            .unwrap_or_default();
        assert!(deepest >= 3, "the deepest path is {deepest} levels down");
    }

    #[test]
    fn the_divisor_shrinks_the_bytes_and_leaves_the_shape_alone() {
        let full = build(params(2_000, 31));
        let mut divided = params(2_000, 31);
        divided.byte_divisor = 1_000;
        let divided = build(divided);

        let paths = |plan: &Plan| {
            plan.files
                .iter()
                .map(|file| file.path.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(paths(&full), paths(&divided), "the tree is the same tree");

        let full_bytes: u64 = full.files.iter().map(|file| file.written_len).sum();
        let divided_bytes: u64 = divided.files.iter().map(|file| file.written_len).sum();
        assert!(
            divided_bytes * 10 < full_bytes,
            "a divisor of a thousand wrote {divided_bytes} against {full_bytes}"
        );
    }

    #[test]
    fn the_date_arithmetic_agrees_with_dates_worked_out_by_hand() {
        assert_eq!(civil_from_unix(0), (1970, 1, 1));
        assert_eq!(civil_from_unix(1_640_995_200), (2022, 1, 1));
        // A leap day, which is where a hand-rolled calendar goes wrong.
        assert_eq!(civil_from_unix(1_709_164_800), (2024, 2, 29));
    }
}
