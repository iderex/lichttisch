// SPDX-License-Identifier: AGPL-3.0-only
//! The signal interface, as things that fail (#50).
//!
//! A signal declares its scope, what it needs, and what it produces. Those
//! declarations only mean something if something refuses a call or a result
//! that disagrees with them, so every declaration in
//! `crates/culling/src/signal.rs` has a test here that trips it and a neighbour
//! that must stay green. A guard proven only against a signal that already
//! passes it is a guard proven against nothing.
//!
//! Each pair is written against the mistake somebody building a signal will
//! actually make rather than an obvious one. The size boundary is one pixel,
//! the value boundary is the declared end of the scale itself, and the region
//! boundary is a box that ends exactly on the edge of the frame.
//!
//! One property in this interface is not tested here and cannot be. That no
//! reader can mistake having no opinion for a low measurement is held by the
//! shape of `Opinion`: the `NoOpinion` arm carries no value, so there is nothing
//! to read out of it, and code that tried would not compile. Proving that would
//! take a harness that compiles a program and expects it to fail, which is a
//! dependency this tree does not carry. It is stated in
//! `docs/decisions/0012-signal-interface.md` as a structural property rather
//! than left to look like a tested one.

use std::time::Duration;

use culling::runner::{Refused, Signal, over_a_group, over_one_photograph};
use culling::signal::{
    AGroupSignal, Accelerator, Confidence, Cost, Describes, Evidence, Group, INTERFACE_VERSION,
    Measurement, Name, Needs, OnePhotographSignal, Opinion, Photograph, Produces, Region,
    Requirements, Scope, Silence, Value,
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
fn sure() -> Confidence {
    Confidence::new(1.0).expect("one is a confidence")
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

/// A signal that says whatever the test tells it to say.
struct Fixture {
    describes: Describes,
    says: Opinion,
}

impl Fixture {
    fn new(scope: Scope, needs: Needs, produces: Produces, says: Opinion) -> Self {
        Self {
            describes: Describes {
                name: named("fixture-signal"),
                version: 3,
                scope,
                needs,
                cost: Cost {
                    per_photograph: Duration::from_millis(4),
                },
                requires: Requirements {
                    accelerator: Accelerator::Unused,
                    model: None,
                },
                produces,
            },
            says,
        }
    }
}

impl OnePhotographSignal for Fixture {
    fn describes(&self) -> Describes {
        self.describes
    }
    fn look(&self, _at: &dyn Photograph) -> Opinion {
        self.says
    }
}

impl AGroupSignal for Fixture {
    fn describes(&self) -> Describes {
        self.describes
    }
    fn look(&self, _at: &dyn Group) -> Opinion {
        self.says
    }
}

/// A measurement about the whole frame, at the value given.
fn saying(value: Value) -> Opinion {
    Opinion::Says(Measurement {
        value,
        confidence: sure(),
        evidence: Evidence {
            region: Region::WholeFrame,
            because: "the fixture said so",
        },
    })
}

/// A measurement placed at the region given.
fn saying_at(region: Region) -> Opinion {
    Opinion::Says(Measurement {
        value: Value::Number(0.5),
        confidence: sure(),
        evidence: Evidence {
            region,
            because: "the fixture said so",
        },
    })
}

const ZERO_TO_ONE: Produces = Produces::Number {
    low: 0.0,
    high: 1.0,
};

/// A frame signal that wants metadata and returns a number in range.
fn ordinary() -> Fixture {
    Fixture::new(
        Scope::OnePhotograph,
        Needs::MetadataOnly,
        ZERO_TO_ONE,
        saying(Value::Number(0.5)),
    )
}

fn metadata_only() -> Offering {
    Offering {
        identity: "0f0f0f",
        available: Needs::MetadataOnly,
    }
}

mod scope {
    use super::{
        Fixture, Needs, Opinion, Produces, Refused, Scope, Signal, Value, Whichever, metadata_only,
        ordinary, over_a_group, over_one_photograph, saying,
    };

    #[test]
    fn a_group_signal_asked_about_one_photograph_is_refused() {
        let signal = Fixture::new(
            Scope::AGroup,
            Needs::MetadataOnly,
            super::ZERO_TO_ONE,
            saying(Value::Number(0.5)),
        );
        let refusal = over_one_photograph(&Signal::AGroup(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::WrongScope { .. })),
            "a signal that declares a group and is handed one photograph has to be refused, \
             because it would answer about the frame as though it had seen the group. \
             got {refusal:?}"
        );
    }

    #[test]
    fn the_same_group_signal_asked_about_a_group_is_not_refused() {
        let signal = Fixture::new(
            Scope::AGroup,
            Needs::MetadataOnly,
            super::ZERO_TO_ONE,
            saying(Value::Number(0.5)),
        );
        let one = metadata_only();
        let group = Whichever {
            photographs: vec![&one],
        };
        assert!(
            over_a_group(&Signal::AGroup(&signal), &group).is_ok(),
            "the neighbour of the refusal above is the same signal called correctly, and it \
             has to pass, or the guard is refusing the scope rather than the mismatch"
        );
    }

    #[test]
    fn a_signal_declaring_a_group_and_placed_as_one_photograph_is_refused() {
        // The one-character version of this mistake: the declaration is edited
        // and the place the signal is registered is not, or the other way
        // round. Either way the two disagree and only one of them is read by
        // anything downstream.
        let signal = Fixture::new(
            Scope::AGroup,
            Needs::MetadataOnly,
            super::ZERO_TO_ONE,
            saying(Value::Number(0.5)),
        );
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::WrongScope { .. })),
            "a signal placed at a scope it did not declare has to be refused. got {refusal:?}"
        );
    }

    #[test]
    fn an_ordinary_frame_signal_passes() {
        let signal = ordinary();
        assert!(
            over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only()).is_ok(),
            "a signal declaring one photograph and asked about one photograph passes"
        );
    }

    #[test]
    fn an_empty_group_is_refused() {
        let signal = Fixture::new(
            Scope::AGroup,
            Needs::MetadataOnly,
            super::ZERO_TO_ONE,
            saying(Value::Number(0.5)),
        );
        let group = Whichever {
            photographs: Vec::new(),
        };
        let refusal = over_a_group(&Signal::AGroup(&signal), &group);
        assert!(
            matches!(refusal, Err(Refused::EmptyGroup { .. })),
            "a group signal handed nothing would answer about nothing and store the answer \
             as though it had looked. got {refusal:?}"
        );
        let _ = Opinion::NoOpinion(super::Silence::NothingToRemarkOn);
        let _: Produces = super::ZERO_TO_ONE;
    }
}

mod what_it_was_handed {
    use super::{
        Fixture, Needs, Offering, Produces, Refused, Scope, Signal, Value, Whichever, over_a_group,
        over_one_photograph, saying,
    };

    fn wants(longest_edge: u32) -> Fixture {
        Fixture::new(
            Scope::OnePhotograph,
            Needs::ReducedFrame { longest_edge },
            Produces::Number {
                low: 0.0,
                high: 1.0,
            },
            saying(Value::Number(0.5)),
        )
    }

    fn offers(longest_edge: u32) -> Offering {
        Offering {
            identity: "0f0f0f",
            available: Needs::ReducedFrame { longest_edge },
        }
    }

    #[test]
    fn one_pixel_under_what_it_asked_for_is_refused() {
        let signal = wants(1024);
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &offers(1023));
        assert!(
            matches!(refusal, Err(Refused::NotEnoughOfThePhotograph { .. })),
            "a signal answering below the size it declared it is honest at is the failure \
             this guard exists for, and one pixel under is the version of it somebody \
             actually writes. got {refusal:?}"
        );
    }

    #[test]
    fn exactly_what_it_asked_for_is_not_refused() {
        let signal = wants(1024);
        assert!(
            over_one_photograph(&Signal::OnePhotograph(&signal), &offers(1024)).is_ok(),
            "the declared size is the smallest it is honest at, so being handed exactly that \
             passes. A guard refusing it would push every signal to declare one pixel less"
        );
    }

    #[test]
    fn more_than_it_asked_for_is_not_refused() {
        let signal = wants(1024);
        let full = Offering {
            identity: "0f0f0f",
            available: Needs::FullFrame,
        };
        assert!(
            over_one_photograph(&Signal::OnePhotograph(&signal), &full).is_ok(),
            "an offer larger than the need satisfies it"
        );
    }

    #[test]
    fn metadata_alone_does_not_satisfy_a_signal_that_needs_pixels() {
        let signal = wants(64);
        let metadata = Offering {
            identity: "0f0f0f",
            available: Needs::MetadataOnly,
        };
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata);
        assert!(
            matches!(refusal, Err(Refused::NotEnoughOfThePhotograph { .. })),
            "a signal needing pixels and handed only what the camera wrote would answer from \
             the wrong thing entirely. got {refusal:?}"
        );
    }

    #[test]
    fn one_photograph_in_a_group_offering_too_little_refuses_the_whole_call() {
        let signal = Fixture::new(
            Scope::AGroup,
            Needs::ReducedFrame { longest_edge: 512 },
            Produces::Number {
                low: 0.0,
                high: 1.0,
            },
            saying(Value::Number(0.5)),
        );
        let enough = offers(512);
        let short = offers(511);
        let group = Whichever {
            photographs: vec![&enough, &short],
        };
        let refusal = over_a_group(&Signal::AGroup(&signal), &group);
        assert!(
            matches!(refusal, Err(Refused::NotEnoughOfThePhotograph { .. })),
            "a group is judged as a whole, so one frame short of what the signal needs makes \
             the answer about the group wrong rather than about that frame. got {refusal:?}"
        );
    }

    #[test]
    fn a_group_where_every_photograph_is_enough_passes() {
        let signal = Fixture::new(
            Scope::AGroup,
            Needs::ReducedFrame { longest_edge: 512 },
            Produces::Number {
                low: 0.0,
                high: 1.0,
            },
            saying(Value::Number(0.5)),
        );
        let first = offers(512);
        let second = offers(2048);
        let group = Whichever {
            photographs: vec![&first, &second],
        };
        assert!(
            over_a_group(&Signal::AGroup(&signal), &group).is_ok(),
            "the neighbour of the refusal above"
        );
    }
}

mod what_it_returned {
    use super::{
        Fixture, Needs, Produces, Refused, Region, Scope, Signal, Value, metadata_only,
        over_one_photograph, saying, saying_at,
    };

    fn returning(produces: Produces, says: super::Opinion) -> Fixture {
        Fixture::new(Scope::OnePhotograph, Needs::MetadataOnly, produces, says)
    }

    #[test]
    fn a_number_just_past_the_declared_end_of_the_scale_is_refused() {
        let signal = returning(super::ZERO_TO_ONE, saying(Value::Number(1.000_001)));
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::ValueItDidNotDeclare { .. })),
            "a number outside the scale it was read under cannot be compared with any other \
             number from the same signal, and a hair over the top is how it happens. \
             got {refusal:?}"
        );
    }

    #[test]
    fn a_number_exactly_at_the_declared_end_of_the_scale_is_not_refused() {
        let signal = returning(super::ZERO_TO_ONE, saying(Value::Number(1.0)));
        assert!(
            over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only()).is_ok(),
            "both ends are included, so the top of the scale is a value and not an error"
        );
    }

    #[test]
    fn a_category_the_signal_never_declared_is_refused() {
        let signal = returning(
            Produces::Category(&["sharp", "soft"]),
            saying(Value::Category("blurred")),
        );
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::ValueItDidNotDeclare { .. })),
            "a category nobody declared is a value no reader of the catalogue has a meaning \
             for. got {refusal:?}"
        );
    }

    #[test]
    fn a_category_the_signal_declared_is_not_refused() {
        let signal = returning(
            Produces::Category(&["sharp", "soft"]),
            saying(Value::Category("soft")),
        );
        assert!(
            over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only()).is_ok(),
            "the neighbour of the refusal above"
        );
    }

    #[test]
    fn a_number_from_a_signal_that_declared_categories_is_refused() {
        let signal = returning(
            Produces::Category(&["sharp", "soft"]),
            saying(Value::Number(0.5)),
        );
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::ValueItDidNotDeclare { .. })),
            "the kind of the value has to match the declaration as well as its contents. \
             got {refusal:?}"
        );
    }

    #[test]
    fn a_region_reaching_one_step_past_the_edge_is_refused() {
        let signal = returning(
            super::ZERO_TO_ONE,
            saying_at(Region::Part {
                left: 0.800_001,
                top: 0.0,
                width: 0.2,
                height: 1.0,
            }),
        );
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::EvidenceCannotBeShown { .. })),
            "evidence the surface cannot place is a flag whose reason cannot be shown, and \
             the arithmetic that produces one lands just outside rather than far outside. \
             got {refusal:?}"
        );
    }

    #[test]
    fn a_region_ending_exactly_on_the_edge_is_not_refused() {
        let signal = returning(
            super::ZERO_TO_ONE,
            saying_at(Region::Part {
                left: 0.8,
                top: 0.0,
                width: 0.2,
                height: 1.0,
            }),
        );
        assert!(
            over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only()).is_ok(),
            "a box against the right edge is an ordinary answer, not an error"
        );
    }

    #[test]
    fn a_region_with_no_area_is_refused() {
        let signal = returning(
            super::ZERO_TO_ONE,
            saying_at(Region::Part {
                left: 0.5,
                top: 0.5,
                width: 0.0,
                height: 0.0,
            }),
        );
        let refusal = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(refusal, Err(Refused::EvidenceCannotBeShown { .. })),
            "a region with no area is inside the frame and still shows the operator nothing. \
             got {refusal:?}"
        );
    }
}

mod having_no_opinion {
    use super::{
        Fixture, Needs, Opinion, Produces, Scope, Signal, Silence, Value, metadata_only,
        over_one_photograph,
    };

    fn silent(because: Silence) -> Fixture {
        Fixture::new(
            Scope::OnePhotograph,
            Needs::MetadataOnly,
            Produces::Number {
                low: 0.0,
                high: 1.0,
            },
            Opinion::NoOpinion(because),
        )
    }

    #[test]
    fn having_no_opinion_is_not_a_refusal() {
        let signal = silent(Silence::NothingToRemarkOn);
        let reading = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        assert!(
            matches!(
                reading,
                Ok(culling::runner::Reading {
                    opinion: Opinion::NoOpinion(Silence::NothingToRemarkOn),
                    ..
                })
            ),
            "most frames in most shoots are this answer, so it passes the runner and is \
             stored like any other. got {reading:?}"
        );
    }

    #[test]
    fn why_it_was_silent_survives_the_runner() {
        let signal = silent(Silence::SubjectAbsent);
        let reading = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only());
        let Ok(reading) = reading else {
            panic!("a silence is not a refusal");
        };
        assert_eq!(
            reading.opinion,
            Opinion::NoOpinion(Silence::SubjectAbsent),
            "a signal that found no face and a signal that found nothing to remark on are \
             opposite statements about a frame, and reading one as the other is how a shoot \
             appears to have been judged when it was not"
        );
    }

    #[test]
    fn a_silence_carries_no_value_to_be_read_as_a_low_one() {
        // The compiler holds this rather than this assertion: `NoOpinion` has
        // no value in it, so nothing downstream can reach for one. What is
        // asserted here is only that the two arms are not equal, which is the
        // part a runtime test can reach.
        assert_ne!(
            Opinion::NoOpinion(Silence::NothingToRemarkOn),
            super::saying(Value::Number(0.0)),
            "having no opinion and measuring zero are different answers"
        );
    }
}

mod what_is_stamped_on_a_value {
    use super::{INTERFACE_VERSION, Signal, metadata_only, named, ordinary, over_one_photograph};

    #[test]
    fn every_reading_carries_which_version_of_which_signal_made_it() {
        let signal = ordinary();
        let Ok(reading) = over_one_photograph(&Signal::OnePhotograph(&signal), &metadata_only())
        else {
            panic!("the ordinary fixture passes");
        };
        assert_eq!(reading.origin.signal, named("fixture-signal"));
        assert_eq!(
            reading.origin.version, 3,
            "the signal's own version is stored, or a value cannot be dropped when that \
             version is found to be wrong"
        );
        assert_eq!(
            reading.origin.interface, INTERFACE_VERSION,
            "the interface version is stored too, because a stored value has to be readable \
             under the shape it was written for"
        );
    }
}

mod names_and_confidences {
    use super::{Confidence, Name};

    #[test]
    fn a_name_that_would_be_stored_two_ways_is_refused() {
        for spelling in ["", "Sharpness", "sharp--ness", "-sharpness", "sharpness-"] {
            assert!(
                Name::new(spelling).is_none(),
                "{spelling:?} would sit in a catalogue beside another spelling of the same \
                 signal, and nothing afterwards could tell they were one signal"
            );
        }
    }

    #[test]
    fn an_ordinary_name_is_accepted() {
        for spelling in ["sharpness", "focus-on-the-subject", "eyes-open2"] {
            assert!(
                Name::new(spelling).is_some(),
                "{spelling:?} is a name a signal would actually be given"
            );
        }
    }

    #[test]
    fn a_confidence_outside_zero_to_one_is_refused() {
        for value in [-0.000_001, 1.000_001, f64::NAN, f64::INFINITY] {
            assert!(
                Confidence::new(value).is_none(),
                "{value} is not a confidence, and a stored one that is not between zero and \
                 one cannot be compared with any other"
            );
        }
    }

    #[test]
    fn both_ends_of_the_range_are_confidences() {
        assert!(Confidence::new(0.0).is_some());
        assert!(Confidence::new(1.0).is_some());
    }
}
