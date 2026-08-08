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
//! No signal is implemented here. Which signals exist at first release is
//! `docs/decisions/0006-signal-sources.md`, and each is built by its own issue.
//! This module is the shape they are built against, so that the ranking, the
//! surface, the explanation and the evaluation work against one thing rather
//! than against each signal separately.

pub mod runner;
pub mod signal;
