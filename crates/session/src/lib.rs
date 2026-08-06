//! The culling session, as a headless state machine.
//!
//! The session holds what the operator is doing and answers questions about it.
//! It draws nothing. Issue #63 is where that separation is argued and issue #69
//! is where undoing a whole session rather than a keystroke lands.
