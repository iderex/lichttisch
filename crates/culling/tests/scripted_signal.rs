// SPDX-License-Identifier: AGPL-3.0-only
//! The signal that needs no model, as things that fail (#51).
//!
//! Three of this issue's conditions are properties of the module rather than of
//! any one call, and each has its tests here.
//!
//! That it can be told to produce every answer the interface allows is the one
//! most easily half-met. A fixture that can produce a number and a silence
//! looks complete until the first test above the signal layer needs a signal
//! that returns something the runner refuses, and there is no way to ask for
//! one. So every arm of `Opinion`, every arm of `Silence`, both arms of `Value`
//! and both of the runner's result-side refusals are asked for here by name.
//!
//! That it is deterministic is asserted rather than assumed, and what the
//! assertion covers is narrower than the condition. What runs here is one
//! process on one machine, so what these tests prove is that two calls, and two
//! scripts built in different orders, agree. Across machines is held by the
//! module using no clock, no environment, no random source and no arithmetic,
//! which is a property of the source rather than a result the suite produced,
//! and it is written that way in `crates/culling/src/scripted.rs`. Nothing here
//! should be read as having run this on a second machine.
//!
//! That it needs no model and no accelerator is a property of `declaring` and
//! not of the type. `Scripted::new` takes any `Describes` a caller builds, on
//! purpose: a scheduler test needs a signal that declares an accelerator it
//! cannot have. The guarantee is that a signal declared through `declaring`
//! never does, and the test below is what holds it there.

use std::time::Duration;

use culling::runner::{Reading, Refused, Signal, over_a_group, over_one_photograph};
use culling::scripted::{COST, Scripted, declaring};
use culling::signal::{
    Accelerator, Confidence, Cost, Describes, Evidence, Group, INTERFACE_VERSION, Measurement,
    Name, Needs, Opinion, Photograph, Produces, Region, Requirements, Scope, Silence, Value,
};

#[expect(
    clippy::expect_used,
    reason = "a fixture that cannot be built is a broken test, and stopping is correct"
)]
fn named(name: &'static str) -> Name {
    Name::new(name).expect("the fixture name is a name")
}

#[expect(
    clippy::expect_used,
    reason = "a fixture that cannot be built is a broken test, and stopping is correct"
)]
fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("the fixture confidence is between nought and one")
}

/// A photograph offering exactly what the test hands it.
struct Offering {
    identity: &'static str,
    available: Needs,
}

impl Photograph for Offering {
    fn identity(&self) -> &str {
        self.identity
    }
    fn available(&self) -> Needs {
        self.available
    }
}

/// A group holding whatever the test puts in it.
struct Whichever<'a> {
    photographs: Vec<&'a dyn Photograph>,
}

impl Group for Whichever<'_> {
    fn photographs(&self) -> &[&dyn Photograph] {
        &self.photographs
    }
}

const ZERO_TO_ONE: Produces = Produces::Number {
    low: 0.0,
    high: 1.0,
};

const THREE_NAMES: Produces = Produces::Category(&["keeper", "unsure", "failure"]);

fn frame(identity: &'static str) -> Offering {
    Offering {
        identity,
        available: Needs::MetadataOnly,
    }
}

/// A measurement about the whole frame, held with certainty.
fn saying(value: Value) -> Opinion {
    Opinion::Says(Measurement {
        value,
        confidence: confidence(1.0),
        evidence: Evidence {
            region: Region::WholeFrame,
            because: "the script said so",
        },
    })
}

/// A one-photograph scripted signal on the nought-to-one scale, silent by
/// default.
fn over_frames() -> Scripted {
    Scripted::new(
        declaring(
            named("scripted"),
            Scope::OnePhotograph,
            Needs::MetadataOnly,
            ZERO_TO_ONE,
        ),
        Opinion::NoOpinion(Silence::NothingToRemarkOn),
    )
}

fn asked(signal: &Scripted, about: &Offering) -> Result<Reading, Refused> {
    over_one_photograph(&Signal::OnePhotograph(signal), about)
}

mod every_answer_the_interface_allows {
    use super::{
        Confidence, Evidence, Measurement, Needs, Offering, Opinion, Produces, Reading, Refused,
        Region, Scope, Scripted, Signal, Silence, THREE_NAMES, Value, Whichever, asked, confidence,
        declaring, frame, named, over_a_group, over_frames, saying,
    };

    /// What came back, or the test's own failure naming what was refused.
    #[expect(
        clippy::expect_used,
        reason = "a refusal here is the test failing, and stopping with the refusal printed is \
                  the whole of what the reader needs"
    )]
    fn opinion(reading: Result<Reading, Refused>) -> Opinion {
        reading
            .expect("the runner accepted the scripted answer")
            .opinion
    }

    #[test]
    fn a_number_is_carried_through_unchanged() {
        let signal = over_frames().saying("aaa", saying(Value::Number(0.25)));
        assert_eq!(
            opinion(asked(&signal, &frame("aaa"))),
            saying(Value::Number(0.25)),
            "the value a test wrote into the script is the value everything above the signal \
             layer will be tested against, so anything this module does to it on the way \
             through makes those tests measure this module instead"
        );
    }

    #[test]
    fn a_category_is_carried_through_unchanged() {
        let signal = Scripted::new(
            declaring(
                named("scripted"),
                Scope::OnePhotograph,
                Needs::MetadataOnly,
                THREE_NAMES,
            ),
            Opinion::NoOpinion(Silence::NothingToRemarkOn),
        )
        .saying("aaa", saying(Value::Category("keeper")));
        assert_eq!(
            opinion(asked(&signal, &frame("aaa"))),
            saying(Value::Category("keeper")),
            "both arms of Value are answers the interface allows, and a fixture that can only \
             produce numbers cannot test a ranking over a signal that produces names"
        );
    }

    #[test]
    fn a_confidence_of_nought_is_a_measurement_rather_than_a_silence() {
        let unsure = Opinion::Says(Measurement {
            value: Value::Number(0.9),
            confidence: confidence(0.0),
            evidence: Evidence {
                region: Region::WholeFrame,
                because: "the script said so",
            },
        });
        let signal = over_frames().saying("aaa", unsure);
        let came_back = opinion(asked(&signal, &frame("aaa")));
        assert_eq!(
            came_back, unsure,
            "a high value held with no confidence at all is the case that separates the two \
             statements the interface refuses to collapse, so the fixture has to be able to \
             produce it"
        );
        assert!(
            matches!(came_back, Opinion::Says(_)),
            "and it has to come back as a measurement rather than as a silence, or the thing \
             above is being handed the answer the interface exists to keep apart from it"
        );
    }

    #[test]
    fn each_of_the_three_silences_can_be_asked_for_by_name() {
        let reasons = [
            Silence::NothingToRemarkOn,
            Silence::SubjectAbsent,
            Silence::CouldNotAnswer("the script was told to fail here"),
        ];
        for reason in reasons {
            let signal = over_frames().saying("aaa", Opinion::NoOpinion(reason));
            assert_eq!(
                opinion(asked(&signal, &frame("aaa"))),
                Opinion::NoOpinion(reason),
                "the three silences are three different statements about a frame, and a \
                 fixture that can produce only the ordinary one cannot test what happens \
                 when a signal could not answer. {reason:?} did not come back"
            );
        }
    }

    #[test]
    fn it_can_be_told_to_return_a_value_the_runner_refuses() {
        // The failure this issue asks for. It is not a silence: it is a signal
        // behaving wrongly, which is what every guard above the signal layer
        // has to be tested against and what no well-behaved fixture can supply.
        let signal = over_frames().saying("aaa", saying(Value::Number(1.5)));
        let refusal = asked(&signal, &frame("aaa"));
        assert!(
            matches!(refusal, Err(Refused::ValueItDidNotDeclare { .. })),
            "a scripted signal has to be able to return a value outside its own declared \
             scale, or nothing above it can be tested against a signal that does. \
             got {refusal:?}"
        );
    }

    #[test]
    fn the_same_script_one_step_inside_the_scale_is_accepted() {
        let signal = over_frames().saying("aaa", saying(Value::Number(1.0)));
        assert!(
            asked(&signal, &frame("aaa")).is_ok(),
            "the neighbour of the refusal above, at the declared end of the scale itself: \
             the fixture is producing a bad answer only when it was told to"
        );
    }

    #[test]
    fn it_can_be_told_to_return_evidence_that_cannot_be_shown() {
        let off_the_frame = Opinion::Says(Measurement {
            value: Value::Number(0.5),
            confidence: confidence(1.0),
            evidence: Evidence {
                region: Region::Part {
                    left: 0.5,
                    top: 0.0,
                    width: 0.6,
                    height: 0.5,
                },
                because: "the script said so",
            },
        });
        let signal = over_frames().saying("aaa", off_the_frame);
        let refusal = asked(&signal, &frame("aaa"));
        assert!(
            matches!(refusal, Err(Refused::EvidenceCannotBeShown { .. })),
            "the second way a signal can misbehave, and the surface work in #67 needs a \
             signal that does it. got {refusal:?}"
        );
    }

    #[test]
    fn it_answers_a_group_when_it_declares_one() {
        let signal = Scripted::new(
            declaring(
                named("scripted"),
                Scope::AGroup,
                Needs::MetadataOnly,
                super::ZERO_TO_ONE,
            ),
            Opinion::NoOpinion(Silence::NothingToRemarkOn),
        )
        .saying("first", saying(Value::Number(0.75)));
        let first = frame("first");
        let second = frame("second");
        let group = Whichever {
            photographs: vec![&first, &second],
        };
        let reading = over_a_group(&Signal::AGroup(&signal), &group);
        assert_eq!(
            opinion(reading),
            saying(Value::Number(0.75)),
            "the group work in #56 and the ranking in #58 are both over groups, so a fixture \
             that only answers about one frame would leave both of them without one"
        );
    }

    #[test]
    fn a_frame_the_script_says_nothing_about_gets_the_default() {
        let signal = over_frames().saying("aaa", saying(Value::Number(0.25)));
        assert_eq!(
            opinion(asked(&signal, &frame("bbb"))),
            Opinion::NoOpinion(Silence::NothingToRemarkOn),
            "a script covers the frames a test cares about and a shoot has thousands it does \
             not, so the answer to the rest is stated once when the signal is built rather \
             than left to whatever the last entry happened to be"
        );
    }

    #[test]
    fn a_signal_offering_less_than_it_declared_is_still_refused() {
        // The scripted signal is held to its own declarations like any other.
        // A fixture the runner waves through would let a test above pass on a
        // call the real route refuses.
        let signal = Scripted::new(
            declaring(
                named("scripted"),
                Scope::OnePhotograph,
                Needs::ReducedFrame { longest_edge: 512 },
                super::ZERO_TO_ONE,
            ),
            Opinion::NoOpinion(Silence::NothingToRemarkOn),
        );
        let too_little = Offering {
            identity: "aaa",
            available: Needs::MetadataOnly,
        };
        let refusal = super::over_one_photograph(&Signal::OnePhotograph(&signal), &too_little);
        assert!(
            matches!(refusal, Err(Refused::NotEnoughOfThePhotograph { .. })),
            "got {refusal:?}"
        );
        let _: Confidence = confidence(0.5);
        let _: Produces = THREE_NAMES;
    }
}

mod determinism {
    use super::{Opinion, Silence, Value, asked, frame, over_frames, saying};

    #[test]
    fn the_same_frame_asked_twice_gives_the_same_answer() {
        let signal = over_frames().saying("aaa", saying(Value::Number(0.25)));
        let once = asked(&signal, &frame("aaa"));
        let twice = asked(&signal, &frame("aaa"));
        assert_eq!(
            once, twice,
            "a signal whose second answer differs from its first makes every test above it \
             a test that sometimes passes, and the failure would be read as a defect in the \
             thing being tested"
        );
    }

    #[test]
    fn two_scripts_built_in_opposite_orders_answer_identically() {
        // The order entries were written in is the state most easily leaked
        // into an answer, and it leaks as a test that passes on one machine's
        // hash seed and not on another's. The answers are held in a map ordered
        // by identity, so writing them backwards cannot reach the answer.
        let forwards = over_frames()
            .saying("aaa", saying(Value::Number(0.1)))
            .saying("bbb", saying(Value::Number(0.2)))
            .saying("ccc", saying(Value::Number(0.3)));
        let backwards = over_frames()
            .saying("ccc", saying(Value::Number(0.3)))
            .saying("bbb", saying(Value::Number(0.2)))
            .saying("aaa", saying(Value::Number(0.1)));
        for identity in ["aaa", "bbb", "ccc", "not-in-either"] {
            assert_eq!(
                forwards.answer_for(identity),
                backwards.answer_for(identity),
                "the two scripts hold the same entries in the opposite order and disagreed \
                 about {identity}"
            );
        }
    }

    #[test]
    fn two_signals_built_from_one_recipe_agree_on_every_frame() {
        let recipe = || {
            over_frames()
                .saying("aaa", saying(Value::Number(0.1)))
                .saying("bbb", Opinion::NoOpinion(Silence::SubjectAbsent))
        };
        let one = recipe();
        let other = recipe();
        for identity in ["aaa", "bbb", "ccc"] {
            assert_eq!(
                one.answer_for(identity),
                other.answer_for(identity),
                "two signals built the same way disagreed about {identity}, so the answer \
                 depends on something outside the configuration"
            );
        }
    }

    #[test]
    fn a_second_entry_for_one_frame_replaces_the_first() {
        let signal = over_frames()
            .saying("aaa", saying(Value::Number(0.1)))
            .saying("aaa", saying(Value::Number(0.9)));
        assert_eq!(
            signal.scripted(),
            1,
            "one identity holds one answer, or which of two entries applies is a question \
             about the order they were written in"
        );
        assert_eq!(
            signal.answer_for("aaa"),
            saying(Value::Number(0.9)),
            "and it is the later one, so a test correcting a script does not have to know \
             whether the first entry is still in there"
        );
    }
}

mod what_it_declares {
    use super::{
        Accelerator, COST, Cost, Describes, Duration, INTERFACE_VERSION, Name, Needs, Opinion,
        Produces, Requirements, Scope, Scripted, Silence, ZERO_TO_ONE, declaring, named,
    };

    #[test]
    fn nothing_declared_through_declaring_needs_a_model_or_an_accelerator() {
        // The condition of #51 that has to hold for every shape a caller can
        // ask for rather than for one, because a fixture that declares a device
        // is a fixture the headless suite skips, and a skipped fixture is how
        // the thing above it stops being tested at all.
        let shapes = [
            (Scope::OnePhotograph, Needs::MetadataOnly),
            (
                Scope::OnePhotograph,
                Needs::ReducedFrame { longest_edge: 1 },
            ),
            (Scope::OnePhotograph, Needs::FullFrame),
            (Scope::AGroup, Needs::MetadataOnly),
            (Scope::AGroup, Needs::FullFrame),
        ];
        for (scope, needs) in shapes {
            let describes = declaring(named("scripted"), scope, needs, ZERO_TO_ONE);
            assert_eq!(
                describes.requires,
                Requirements {
                    accelerator: Accelerator::Unused,
                    model: None,
                },
                "declaring({scope:?}, {needs:?}) has to state no accelerator and no model"
            );
        }
    }

    #[test]
    fn what_the_caller_asked_for_is_what_it_declares() {
        let describes = declaring(
            named("scripted"),
            Scope::AGroup,
            Needs::ReducedFrame { longest_edge: 640 },
            ZERO_TO_ONE,
        );
        assert_eq!(describes.name, named("scripted"));
        assert_eq!(describes.scope, Scope::AGroup);
        assert_eq!(describes.needs, Needs::ReducedFrame { longest_edge: 640 });
        assert_eq!(describes.produces, ZERO_TO_ONE);
        assert_eq!(describes.cost, COST);
        assert!(
            COST.per_photograph > Duration::ZERO,
            "a declared cost of nought is a signal the scheduler believes is free, and a \
             lookup is cheap rather than free"
        );
    }

    #[test]
    fn a_value_carries_the_origin_of_the_signal_that_made_it() {
        let describes = declaring(
            named("scripted"),
            Scope::OnePhotograph,
            Needs::MetadataOnly,
            ZERO_TO_ONE,
        );
        let origin = describes.origin();
        assert_eq!(origin.signal, named("scripted"));
        assert_eq!(origin.version, 1);
        assert_eq!(
            origin.interface, INTERFACE_VERSION,
            "a value a test wrote is stored with an origin like any other, so a catalogue \
             test can tell a scripted value from one a real signal made"
        );
    }

    #[test]
    fn a_caller_may_still_declare_a_shape_declaring_refuses_to() {
        // Stated as a test rather than left implicit, because it looks like a
        // hole in the rule above and is not one. A scheduler has to be tested
        // against a signal that says it needs a device, and this is the only
        // signal in the tree to build one from.
        let needs_a_device = Describes {
            name: named("scripted"),
            version: 1,
            scope: Scope::OnePhotograph,
            needs: Needs::MetadataOnly,
            cost: Cost {
                per_photograph: Duration::from_millis(1),
            },
            requires: Requirements {
                accelerator: Accelerator::Required,
                model: Some(named("something")),
            },
            produces: ZERO_TO_ONE,
        };
        let signal = Scripted::new(
            needs_a_device,
            Opinion::NoOpinion(Silence::NothingToRemarkOn),
        );
        assert_eq!(
            <Scripted as culling::signal::OnePhotographSignal>::describes(&signal).requires,
            needs_a_device.requires,
            "Scripted::new declares what it was handed, and only declaring() is the promise"
        );
        let _: Produces = ZERO_TO_ONE;
        let _: Name = named("scripted");
    }
}
