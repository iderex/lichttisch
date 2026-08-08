// SPDX-License-Identifier: AGPL-3.0-only
//! The culling signals.
//!
//! The only module allowed to depend on an inference runtime, and it reaches
//! one only through `foreign`. What a signal produces and what it may not
//! produce is `docs/decisions/0005-verdicts-and-error-cost.md`: a signal makes
//! a measurement and a reason, never a verdict.
//!
//! `signal` holds the shape every signal has. `runner` holds the one route a
//! signal is called through, and the refusals that route makes. The shape and
//! the reasoning behind it are `docs/decisions/0012-signal-interface.md`.
//!
//! `scripted` holds the one signal that measures nothing. It answers from what
//! a caller wrote down, so that everything above the signal layer has something
//! to be tested against that needs no weights, no accelerator and no network.
//! It is a real implementation of the shape rather than a stand-in beside it,
//! and it is built by issue #51.
//!
//! No signal that looks at a photograph is implemented here. Which of those
//! exist at first release is `docs/decisions/0006-signal-sources.md`, and each
//! is built by its own issue. This module is the shape they are built against,
//! so that the ranking, the surface, the explanation and the evaluation work
//! against one thing rather than against each signal separately.

pub mod runner;
pub mod scripted;
pub mod signal;
