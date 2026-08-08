// SPDX-License-Identifier: AGPL-3.0-only
//! The one route a signal is called through, and what it refuses.
//!
//! A signal declares things about itself that only mean something if something
//! holds it to them. This module is that something. Everything above the
//! signals calls through here, so the declarations in
//! `crates/culling/src/signal.rs` are conditions rather than comments.
//!
//! Four refusals, and each one is a way a signal could otherwise put a value
//! into the catalogue that nothing downstream can use:
//!
//! - called at the wrong scope, so a signal that reads a group is handed one
//!   frame and answers about it anyway
//! - handed less of the photograph than it said it needs, so it answers from
//!   metadata a measurement it declared it needs pixels for
//! - returning a value outside what it declared it produces, which is what makes
//!   a stored number impossible to compare against the scale it was read under
//! - returning evidence that cannot be placed on the frame, which
//!   `docs/decisions/0005-verdicts-and-error-cost.md` says is a flag that is not
//!   raised
//!
//! What this module does not do is decide anything about the photographs. It
//! calls one signal and judges what came back against what that signal said
//! about itself.

use std::fmt;

use crate::signal::{
    AGroupSignal, Describes, Group, Name, Needs, OnePhotographSignal, Opinion, Origin, Photograph,
    Region, Scope, Value,
};

/// A signal, at whichever scope it declared.
///
/// Holding the two in one enum is what makes calling one wrongly a thing the
/// caller cannot write: there is no way to hand a group to the arm carrying a
/// one-photograph signal, because that arm's trait takes a photograph.
pub enum Signal<'a> {
    /// A signal that looks at one photograph.
    OnePhotograph(&'a dyn OnePhotographSignal),
    /// A signal that looks at a group together.
    AGroup(&'a dyn AGroupSignal),
}

impl Signal<'_> {
    /// What the signal says about itself, whichever scope it is.
    #[must_use]
    pub fn describes(&self) -> Describes {
        match self {
            Self::OnePhotograph(one) => one.describes(),
            Self::AGroup(many) => many.describes(),
        }
    }
}

/// What a signal said, with the origin that lets a stored copy of it be read
/// back years later.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    /// Which version of which signal, under which version of the interface.
    pub origin: Origin,
    /// What it said.
    pub opinion: Opinion,
}

/// Why a call was refused.
///
/// Every one names the signal, because the reader of a refusal is somebody
/// looking at a shoot that did not finish and needing to know which of a dozen
/// signals stopped it.
#[derive(Clone, Debug, PartialEq)]
pub enum Refused {
    /// Called at a scope the signal did not declare.
    WrongScope {
        /// Which signal.
        signal: Name,
        /// What it declared.
        declared: Scope,
        /// What it was asked for.
        asked: Scope,
    },
    /// Handed less of the photograph than it said it needs.
    NotEnoughOfThePhotograph {
        /// Which signal.
        signal: Name,
        /// What it asked for.
        needs: Needs,
        /// What it was offered.
        offered: Needs,
        /// The photograph that was offered.
        photograph: String,
    },
    /// Handed a group with nothing in it.
    ///
    /// A group signal that answers about an empty group has judged nothing and
    /// said something, which is the shape of a run that covered less than it
    /// appears to have covered.
    EmptyGroup {
        /// Which signal.
        signal: Name,
    },
    /// Returned a value outside what it declared it produces.
    ValueItDidNotDeclare {
        /// Which signal.
        signal: Name,
        /// What came back.
        value: Value,
    },
    /// Returned evidence that cannot be placed on the frame.
    EvidenceCannotBeShown {
        /// Which signal.
        signal: Name,
        /// The region that cannot be placed.
        region: Region,
    },
}

impl fmt::Display for Refused {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongScope {
                signal,
                declared,
                asked,
            } => write!(
                out,
                "{} declares it looks at {}, and it was asked to look at {}.",
                signal.as_str(),
                said(*declared),
                said(*asked)
            ),
            Self::NotEnoughOfThePhotograph {
                signal,
                needs,
                offered,
                photograph,
            } => write!(
                out,
                "{} needs {}, and {photograph} offered {}.",
                signal.as_str(),
                offering(*needs),
                offering(*offered)
            ),
            Self::EmptyGroup { signal } => write!(
                out,
                "{} was handed a group with no photographs in it, so it would have \
                 answered about nothing.",
                signal.as_str()
            ),
            Self::ValueItDidNotDeclare { signal, value } => write!(
                out,
                "{} returned {value:?}, which is outside what it declares it produces.",
                signal.as_str()
            ),
            Self::EvidenceCannotBeShown { signal, region } => write!(
                out,
                "{} returned evidence at {region:?}, which cannot be placed on the frame, \
                 so the operator could not be shown what it is about.",
                signal.as_str()
            ),
        }
    }
}

fn said(scope: Scope) -> &'static str {
    match scope {
        Scope::OnePhotograph => "one photograph",
        Scope::AGroup => "a group",
    }
}

fn offering(needs: Needs) -> String {
    match needs {
        Needs::MetadataOnly => String::from("metadata only"),
        Needs::ReducedFrame { longest_edge } => {
            format!("a frame of at least {longest_edge} on its longest edge")
        }
        Needs::FullFrame => String::from("the full frame"),
    }
}

/// Ask a signal about one photograph.
///
/// # Errors
///
/// Refuses a signal that declared a group, a photograph offering less than the
/// signal needs, a value outside what the signal declared, and evidence that
/// cannot be placed on the frame.
pub fn over_one_photograph(
    signal: &Signal<'_>,
    photograph: &dyn Photograph,
) -> Result<Reading, Refused> {
    let describes = signal.describes();
    let Signal::OnePhotograph(one) = signal else {
        return Err(Refused::WrongScope {
            signal: describes.name,
            declared: describes.scope,
            asked: Scope::OnePhotograph,
        });
    };
    if describes.scope != Scope::OnePhotograph {
        return Err(Refused::WrongScope {
            signal: describes.name,
            declared: describes.scope,
            asked: Scope::OnePhotograph,
        });
    }
    enough(&describes, photograph)?;
    judge(&describes, one.look(photograph))
}

/// Ask a signal about a group of photographs.
///
/// # Errors
///
/// Refuses a signal that declared one photograph, an empty group, a photograph
/// in the group offering less than the signal needs, a value outside what the
/// signal declared, and evidence that cannot be placed on the frame.
pub fn over_a_group(signal: &Signal<'_>, group: &dyn Group) -> Result<Reading, Refused> {
    let describes = signal.describes();
    let Signal::AGroup(many) = signal else {
        return Err(Refused::WrongScope {
            signal: describes.name,
            declared: describes.scope,
            asked: Scope::AGroup,
        });
    };
    if describes.scope != Scope::AGroup {
        return Err(Refused::WrongScope {
            signal: describes.name,
            declared: describes.scope,
            asked: Scope::AGroup,
        });
    }
    let photographs = group.photographs();
    if photographs.is_empty() {
        return Err(Refused::EmptyGroup {
            signal: describes.name,
        });
    }
    for one in photographs {
        enough(&describes, *one)?;
    }
    judge(&describes, many.look(group))
}

/// Whether the photograph offers at least what the signal asked for.
fn enough(describes: &Describes, photograph: &dyn Photograph) -> Result<(), Refused> {
    let offered = photograph.available();
    if describes.needs.met_by(offered) {
        return Ok(());
    }
    Err(Refused::NotEnoughOfThePhotograph {
        signal: describes.name,
        needs: describes.needs,
        offered,
        photograph: photograph.identity().to_owned(),
    })
}

/// Whether what came back is what the signal said it would produce.
fn judge(describes: &Describes, opinion: Opinion) -> Result<Reading, Refused> {
    if let Opinion::Says(measurement) = opinion {
        if !describes.admits(measurement.value) {
            return Err(Refused::ValueItDidNotDeclare {
                signal: describes.name,
                value: measurement.value,
            });
        }
        if !measurement.evidence.region.is_showable() {
            return Err(Refused::EvidenceCannotBeShown {
                signal: describes.name,
                region: measurement.evidence.region,
            });
        }
    }
    Ok(Reading {
        origin: describes.origin(),
        opinion,
    })
}
