// SPDX-License-Identifier: AGPL-3.0-only
//! The whole foreign-function surface.
//!
//! Every declaration of a foreign function, and every conversion of a foreign
//! value into a Rust one, lives here and nowhere else. No other crate in this
//! workspace declares an `extern` block, and that is the rule
//! `docs/decisions/0001-means.md` states rather than a description of what
//! happens to be true today.
//!
//! What crosses this boundary is bytes in one direction and pixels, dimensions
//! and capture metadata in the other. No catalogue type, no session type and no
//! query crosses it.
//!
//! It is empty because the decoder is not chosen yet, which is issue #39, and
//! the inference runtime is issue #50. Issue #40 is where the size of this
//! surface stops being a sentence and becomes a number a check refuses to
//! exceed.
