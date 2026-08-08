// SPDX-License-Identifier: AGPL-3.0-only
//! The signal that needs no model (#51).
//!
//! Everything above the signal layer needs a signal to test against, and the
//! usual answer is a mock. A mock is the wrong answer here, because it stands
//! beside the interface rather than implementing it: the interface can then be
//! changed in a way every real signal has to follow and the mock does not, and
//! the tests above go on passing against a shape nothing produces any more.
//! What is here implements `OnePhotographSignal` and `AGroupSignal`, is called
//! through `crate::runner` like any other signal, and is held to its own
//! declarations by the same four refusals.
//!
//! What it is not is a signal about photographs. It reads no pixels and no
//! metadata. It looks up an answer by the identity of the frame it was handed
//! and returns it, and that is the whole rule. That is also what makes it the
//! fixed point the rest of the plan can be tested against: a ranking, a
//! scheduler or a surface can be exercised over answers a test wrote down,
//! rather than over answers a model produced on whichever machine ran the
//! suite.
//!
//! Three properties, each of them a condition of #51.
//!
//! No model, no accelerator and no network. [`declaring`] states the first two
//! to the scheduler rather than leaving them to be discovered, and this module
//! reaches nothing outside the standard library.
//!
//! It can be told to produce every answer the interface allows. All three
//! silences, a measurement at any confidence including nought, a category as
//! well as a number, and answers the runner refuses. A fixture that can only
//! produce answers that pass cannot be used to test what happens to one that
//! does not, and every guard above the signal layer will need exactly that.
//!
//! It is deterministic. No clock, no environment, no random source, and no
//! arithmetic: every answer is a copy of one the caller wrote down, so there is
//! nothing in the path for a machine to differ on. The answers are held in a
//! [`BTreeMap`], so the order they were written in cannot reach the answer
//! either. What the suite proves of that and what it cannot is stated in
//! `crates/culling/tests/scripted_signal.rs` rather than claimed here.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::signal::{
    AGroupSignal, Accelerator, Cost, Describes, Group, Name, Needs, OnePhotographSignal, Opinion,
    Photograph, Produces, Requirements, Scope,
};

/// What one call to a scripted signal is declared to cost.
///
/// A lookup in a map, so the honest declaration is the smallest one that is not
/// nought. It is stated rather than measured for the same reason every other
/// signal's is: the scheduler reads it before anything has run.
pub const COST: Cost = Cost {
    per_photograph: Duration::from_micros(1),
};

/// What a scripted signal declares about itself.
///
/// The name, the scope, what it needs and what it produces are the caller's,
/// because a test above the signal layer is usually testing how the thing above
/// handles a signal of a particular shape. What is not the caller's is the pair
/// that makes this the signal that runs anywhere: the accelerator is
/// [`Accelerator::Unused`] and the model is `None`, and neither can be set from
/// outside this module. A scripted signal that could declare it needs a device
/// would be a fixture the headless suite has to skip, which is the thing #51
/// exists to prevent.
#[must_use]
pub fn declaring(name: Name, scope: Scope, needs: Needs, produces: Produces) -> Describes {
    Describes {
        name,
        version: 1,
        scope,
        needs,
        cost: COST,
        requires: Requirements {
            accelerator: Accelerator::Unused,
            model: None,
        },
        produces,
    }
}

/// A signal that answers from a script rather than from a photograph.
///
/// Built from what it declares and the answer it gives to a frame the script
/// says nothing about, and then told what to say about the frames the test
/// cares about. Nothing else reaches an answer.
pub struct Scripted {
    describes: Describes,
    answers: BTreeMap<String, Opinion>,
    otherwise: Opinion,
}

impl Scripted {
    /// A scripted signal declaring `describes` and answering `otherwise` to
    /// every frame.
    ///
    /// The default answer is required rather than defaulted to a silence. A
    /// test that forgot to script an identity and a test that meant the
    /// ordinary silence would otherwise be indistinguishable, and the first of
    /// those is a test measuring nothing while looking like one that measured.
    #[must_use]
    pub fn new(describes: Describes, otherwise: Opinion) -> Self {
        Self {
            describes,
            answers: BTreeMap::new(),
            otherwise,
        }
    }

    /// The same signal, saying `opinion` about the frame whose identity is
    /// `about`.
    ///
    /// A later entry for one identity replaces the earlier one, so the script
    /// holds one answer per frame and there is no order in which two entries
    /// both apply.
    #[must_use]
    pub fn saying(mut self, about: impl Into<String>, opinion: Opinion) -> Self {
        self.answers.insert(about.into(), opinion);
        self
    }

    /// What this signal says about the frame with this identity.
    ///
    /// The one place the rule lives. Both traits below call it, so a
    /// one-photograph call and the group call cannot drift into two rules.
    #[must_use]
    pub fn answer_for(&self, identity: &str) -> Opinion {
        self.answers
            .get(identity)
            .copied()
            .unwrap_or(self.otherwise)
    }

    /// How many frames the script names.
    ///
    /// Read by a test that wants to know the script it built is the script it
    /// meant, rather than one where two entries collapsed into one.
    #[must_use]
    pub fn scripted(&self) -> usize {
        self.answers.len()
    }
}

impl OnePhotographSignal for Scripted {
    fn describes(&self) -> Describes {
        self.describes
    }

    fn look(&self, at: &dyn Photograph) -> Opinion {
        self.answer_for(at.identity())
    }
}

impl AGroupSignal for Scripted {
    fn describes(&self) -> Describes {
        self.describes
    }

    /// The group is answered by the identity of its first photograph.
    ///
    /// Capture order is the order `Group` declares, so the first frame is a
    /// stable name for the group without this module inventing one. The runner
    /// has already refused an empty group before this is reached, and the
    /// silence below is what a group with no photographs would get if that ever
    /// stopped being true, rather than a panic on an empty slice.
    fn look(&self, at: &dyn Group) -> Opinion {
        at.photographs()
            .first()
            .map_or(self.otherwise, |first| self.answer_for(first.identity()))
    }
}
