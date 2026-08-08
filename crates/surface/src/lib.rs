// SPDX-License-Identifier: AGPL-3.0-only
//! The operator surface, as a thin renderer over `session`.
//!
//! Nothing decides anything here. Whether this project ships a surface of its
//! own at all is entry 5 of issue #13 and it is open; this module exists so
//! that the session above it is written against a boundary rather than against
//! a window, and it holds no toolkit choice.
