// SPDX-License-Identifier: AGPL-3.0-only
//! The scripted signal, used from a module above the signal layer (#51).
//!
//! One of that issue's conditions is that the signal is available to tests in
//! every module above the signal layer without those modules depending on an
//! inference runtime. A declaration in a manifest is not that condition met: it
//! is met when a test up here builds one, calls it through the runner and gets
//! an answer, which is what this file is.
//!
//! `session` is the module directly above the signals, and
//! `crates/module-boundaries.txt` gives it `catalogue culling foreign`. This
//! file adds no dependency to reach a signal, which is the half of the
//! condition about the inference runtime: what is reached is `culling` alone,
//! by the edge that was already declared, and nothing about a model or a device
//! arrives with it.
//!
//! The two modules above this one are not covered here and neither is the
//! binary. `surface` names only `session` in its manifest today, and neither it
//! nor `lichttisch` has a test that uses a signal, so nothing here should be
//! read as the condition having been met for them. The condition's last half,
//! that the scheduler, the ranking and the surface all have tests using only
//! this signal, is the work of #58, #63 and the issue that builds the
//! scheduler, and it is not met by this file.

use culling::runner::{Signal, over_one_photograph};
use culling::scripted::{Scripted, declaring};
use culling::signal::{Name, Needs, Opinion, Photograph, Produces, Scope, Silence, Value};

#[expect(
    clippy::expect_used,
    reason = "a fixture that cannot be built is a broken test, and stopping is correct"
)]
fn named(name: &'static str) -> Name {
    Name::new(name).expect("the fixture name is a name")
}

/// A photograph with nothing in it but an identity and what it offers.
struct Frame {
    identity: &'static str,
}

impl Photograph for Frame {
    fn identity(&self) -> &str {
        self.identity
    }
    fn available(&self) -> Needs {
        Needs::MetadataOnly
    }
}

#[test]
fn a_module_above_the_signals_can_script_one_and_call_it() {
    let signal = Scripted::new(
        declaring(
            named("scripted"),
            Scope::OnePhotograph,
            Needs::MetadataOnly,
            Produces::Category(&["keeper", "unsure"]),
        ),
        Opinion::NoOpinion(Silence::NothingToRemarkOn),
    )
    .saying(
        "0f0f0f",
        Opinion::Says(culling::signal::Measurement {
            value: Value::Category("keeper"),
            #[expect(
                clippy::expect_used,
                reason = "a fixture that cannot be built is a broken test"
            )]
            confidence: culling::signal::Confidence::new(0.8)
                .expect("nought point eight is a confidence"),
            evidence: culling::signal::Evidence {
                region: culling::signal::Region::WholeFrame,
                because: "the script said so",
            },
        }),
    );

    let frame = Frame { identity: "0f0f0f" };
    let reading = over_one_photograph(&Signal::OnePhotograph(&signal), &frame);

    #[expect(
        clippy::expect_used,
        reason = "a refusal is this test failing, and the refusal is what the reader needs"
    )]
    let reading = reading.expect("the runner accepted the scripted answer");
    assert_eq!(
        reading.origin.signal,
        named("scripted"),
        "the value carries the origin of the signal that made it, up here as well as inside \
         the culling module, so a session test can tell a scripted value from any other"
    );
    assert!(
        matches!(reading.opinion, Opinion::Says(_)),
        "a test in the session gets the answer its own script wrote, with no weights, no \
         accelerator and nothing fetched. got {:?}",
        reading.opinion
    );
}
