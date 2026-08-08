// SPDX-License-Identifier: AGPL-3.0-only
//! What a signal is, what it declares about itself, and what it may return.
//!
//! Every type here exists because something downstream needs it: the scheduler
//! needs the cost and the requirements before it decides what runs during an
//! import, the surface needs the region before it can show the operator why a
//! frame was flagged, the catalogue needs the origin before it can store a
//! value that outlives the code that made it, and the evaluation needs all of
//! them at once. `docs/decisions/0012-signal-interface.md` is where each of
//! those is argued.
//!
//! One rule shapes the whole file and comes from
//! `docs/decisions/0005-verdicts-and-error-cost.md`: a signal never produces a
//! verdict. It produces a measurement with the evidence for it, or it says it
//! has no opinion, which is the expected answer on most frames rather than a
//! failure to produce one.

use std::time::Duration;

/// The name of a signal, as it is stored beside every value it produced.
///
/// Deliberately not an enum. The set of signals is open, it is declared in
/// `docs/decisions/0006-signal-sources.md` rather than in this file, and a
/// closed set here would make adding a signal a change to the interface every
/// other signal is written against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(&'static str);

impl Name {
    /// The name, or `None` where it is not one a stored value can be read back
    /// by: lower case, digits and single hyphens, starting and ending with a
    /// letter.
    ///
    /// Refused rather than normalised. A name that arrives in two spellings is
    /// two signals to anything reading the catalogue afterwards, and correcting
    /// it silently is how the two spellings both end up stored.
    #[must_use]
    pub fn new(name: &'static str) -> Option<Self> {
        let ok = !name.is_empty()
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && name.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !name.contains("--");
        ok.then_some(Self(name))
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

/// Which version of which signal, under which version of this interface,
/// produced a value.
///
/// Stored with every value rather than derived later. A value in a catalogue
/// outlives the code that made it, and a number with no origin cannot be
/// compared with a number made by a different version, or dropped when that
/// version is found to be wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    /// Which signal.
    pub signal: Name,
    /// Which version of that signal. Its own issue decides when this moves.
    pub version: u16,
    /// Which version of the shape in this file. Moved only by a change that a
    /// stored value cannot be read under.
    pub interface: u16,
}

/// The version of the shape in this file.
///
/// It starts at 1 and moves when a stored value made under the previous number
/// can no longer be read as this shape. A field added that every reader may
/// ignore is not that; a field removed, retyped or given a new meaning is.
pub const INTERFACE_VERSION: u16 = 1;

/// Whether a signal looks at one photograph or at a group of them.
///
/// Declared by the signal rather than inferred from how it is called. The
/// runner is the only route a signal is called through and it refuses a call
/// that disagrees with this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// One photograph at a time, with no knowledge of its neighbours.
    OnePhotograph,
    /// A group of photographs, judged together. Grouping itself is #56.
    AGroup,
}

/// What a signal needs to be handed before it can answer.
///
/// Ordered from cheapest to most expensive, and the ordering is what the runner
/// compares: an offer of more than a signal asked for satisfies it, an offer of
/// less does not. How pixels reach this module is the decoding work in M3 and is
/// deliberately not decided here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    /// Only what the camera wrote. No decode, so it can run during an import.
    MetadataOnly,
    /// The frame decoded no smaller than this many pixels on its longest edge.
    ///
    /// A signal states the smallest size it is honest at rather than the size it
    /// would prefer, because the scheduler pays for the difference on every
    /// frame of a shoot.
    ReducedFrame {
        /// The smallest longest edge this signal will answer at.
        longest_edge: u32,
    },
    /// The frame at the size the camera wrote it.
    FullFrame,
}

impl Needs {
    /// Whether being handed `offered` satisfies a signal that asked for `self`.
    #[must_use]
    pub fn met_by(self, offered: Self) -> bool {
        match (self, offered) {
            (Self::MetadataOnly, _) | (_, Self::FullFrame) => true,
            (
                Self::ReducedFrame {
                    longest_edge: asked,
                },
                Self::ReducedFrame { longest_edge: got },
            ) => got >= asked,
            (Self::ReducedFrame { .. } | Self::FullFrame, Self::MetadataOnly)
            | (Self::FullFrame, Self::ReducedFrame { .. }) => false,
        }
    }
}

/// Whether a signal can run without an accelerator, and what one buys.
///
/// Declared rather than discovered at run time, because a signal that finds out
/// mid-import that it needs a device it does not have has already spent the
/// import. Issue #59 measures what an accelerator actually buys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accelerator {
    /// Never uses one. The answer is the same on every machine.
    Unused,
    /// Uses one where there is one, and answers the same either way, slower.
    Optional,
    /// Cannot answer without one, so a machine without one does not run it.
    Required,
}

/// What a signal needs from the machine before it is scheduled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirements {
    /// Whether an accelerator is needed, useful, or nothing to it.
    pub accelerator: Accelerator,
    /// The model this signal loads, where it loads one.
    ///
    /// `None` means it computes rather than infers, which is what
    /// `docs/decisions/0006-signal-sources.md` calls a measured signal. Issue
    /// #60 pins the artefact and records its terms before anything ships.
    pub model: Option<Name>,
}

/// What a signal declares it will cost, per photograph it is handed.
///
/// A declared budget rather than a measurement. The scheduler reads it before
/// anything has run, so it cannot be the time the last run took. Issue #59 is
/// where the declared figure and the measured one are compared, and a signal
/// whose declaration is wrong is a defect in that signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cost {
    /// What one call is expected to take, on the machine described by the
    /// performance record, without an accelerator.
    pub per_photograph: Duration,
}

/// What the values of a signal look like, so a reader can tell a number outside
/// its scale from a number at the edge of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Produces {
    /// A number between these two, both ends included, on a stated scale.
    Number {
        /// The lowest value this signal may return.
        low: f64,
        /// The highest value this signal may return.
        high: f64,
    },
    /// One of a fixed set of names, all of them listed here.
    Category(&'static [&'static str]),
}

/// A value a signal produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    /// A number, which has to sit inside what the signal declared.
    Number(f64),
    /// A name, which has to be one the signal declared.
    Category(&'static str),
}

/// How sure the signal is of the value beside it.
///
/// A number between zero and one, and it is not the value. A sharpness of 0.1
/// held with certainty and a sharpness of 0.9 held with none are different
/// statements, and collapsing them is how a tool starts sounding confident
/// about frames it has no opinion on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(f64);

impl Confidence {
    /// A confidence, or `None` where it is not between zero and one.
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && (0.0..=1.0).contains(&value)).then_some(Self(value))
    }

    /// The confidence as a number.
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

/// Where in the photograph a value came from.
///
/// In fractions of the frame rather than in pixels, so the surface can place it
/// on a preview of any size without knowing which preview the signal saw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Region {
    /// The value is about the frame as a whole.
    WholeFrame,
    /// The value is about this part of it.
    Part {
        /// Distance from the left edge, as a fraction of the width.
        left: f64,
        /// Distance from the top edge, as a fraction of the height.
        top: f64,
        /// Width, as a fraction of the frame's width.
        width: f64,
        /// Height, as a fraction of the frame's height.
        height: f64,
    },
}

impl Region {
    /// Whether this region lies inside the frame.
    ///
    /// A region that does not cannot be shown, and
    /// `docs/decisions/0005-verdicts-and-error-cost.md` says a flag whose
    /// evidence cannot be shown is a flag that is not raised.
    #[must_use]
    pub fn is_showable(self) -> bool {
        match self {
            Self::WholeFrame => true,
            Self::Part {
                left,
                top,
                width,
                height,
            } => {
                [left, top, width, height].iter().all(|one| one.is_finite())
                    && left >= 0.0
                    && top >= 0.0
                    && width > 0.0
                    && height > 0.0
                    && left + width <= 1.0
                    && top + height <= 1.0
            }
        }
    }
}

/// What the operator is shown when they ask why.
///
/// The region is not optional and neither is the sentence. Both are here
/// because a photographer disagreeing with this software has to be able to
/// disagree with a specific claim about a specific frame rather than with the
/// tool in general.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Evidence {
    /// Where the value came from.
    pub region: Region,
    /// One sentence, in the operator's terms, naming what was measured.
    pub because: &'static str,
}

/// A measurement, with everything a reader needs in order to disagree with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    /// What was measured.
    pub value: Value,
    /// How sure the signal is of it.
    pub confidence: Confidence,
    /// What the operator is shown when they ask why.
    pub evidence: Evidence,
}

/// What a signal returns.
///
/// Having no opinion is a variant carrying no value at all rather than a low
/// number or a low confidence, which is what makes the two impossible to
/// confuse: there is nothing to read out of `NoOpinion`, so no reader can treat
/// it as a weak measurement by accident. Most frames in most shoots are this
/// answer, and `docs/decisions/0005-verdicts-and-error-cost.md` says why that is
/// the expected case rather than a gap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opinion {
    /// Nothing to say about this photograph, and why not.
    NoOpinion(Silence),
    /// A measurement.
    Says(Measurement),
}

/// Why a signal had nothing to say.
///
/// Stored with the silence, because "this signal could not run here" and "this
/// signal ran and found nothing to remark on" are opposite statements about a
/// frame and reading one as the other is how a shoot appears to have been
/// judged when it was not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Silence {
    /// It ran and found nothing worth saying. The ordinary answer.
    NothingToRemarkOn,
    /// What this signal looks for is not in this photograph at all, such as a
    /// face signal on a frame with no face in it.
    SubjectAbsent,
    /// It could not answer here, and this is the reason.
    CouldNotAnswer(&'static str),
}

/// What a signal says about itself, before it is ever called.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Describes {
    /// Which signal this is.
    pub name: Name,
    /// Which version of it. Moved by the issue that owns the signal.
    pub version: u16,
    /// One photograph or a group.
    pub scope: Scope,
    /// What it has to be handed.
    pub needs: Needs,
    /// What it expects to cost per photograph.
    pub cost: Cost,
    /// What it needs from the machine.
    pub requires: Requirements,
    /// What its values look like.
    pub produces: Produces,
}

impl Describes {
    /// Whether `value` is one this signal declared it produces.
    #[must_use]
    pub fn admits(&self, value: Value) -> bool {
        match (self.produces, value) {
            (Produces::Number { low, high }, Value::Number(one)) => {
                one.is_finite() && one >= low && one <= high
            }
            (Produces::Category(names), Value::Category(one)) => names.contains(&one),
            (Produces::Number { .. }, Value::Category(_))
            | (Produces::Category(_), Value::Number(_)) => false,
        }
    }

    /// The origin stamped onto every value this signal produces.
    #[must_use]
    pub fn origin(&self) -> Origin {
        Origin {
            signal: self.name,
            version: self.version,
            interface: INTERFACE_VERSION,
        }
    }
}

/// One photograph, as a signal sees it.
///
/// A trait rather than a type, because what a photograph is to the catalogue is
/// `docs/decisions/0010-catalogue-schema.md` and how its pixels are decoded is
/// M3, and this module is written before either. What a signal needs from a
/// photograph in order to be scheduled and to place its evidence is here and
/// nothing else is.
pub trait Photograph {
    /// The digest of the file's bytes, which is what identifies it.
    fn identity(&self) -> &str;
    /// What of this photograph is available to a call right now.
    fn available(&self) -> Needs;
}

/// A group of photographs, as a group signal sees it.
pub trait Group {
    /// The photographs in the group, in capture order.
    fn photographs(&self) -> &[&dyn Photograph];
}

/// A signal that looks at one photograph.
pub trait OnePhotographSignal {
    /// What this signal says about itself.
    fn describes(&self) -> Describes;
    /// What it says about this photograph.
    fn look(&self, at: &dyn Photograph) -> Opinion;
}

/// A signal that looks at a group of photographs together.
pub trait AGroupSignal {
    /// What this signal says about itself.
    fn describes(&self) -> Describes;
    /// What it says about this group.
    fn look(&self, at: &dyn Group) -> Opinion;
}
