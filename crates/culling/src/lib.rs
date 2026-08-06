//! The culling signals.
//!
//! The only module allowed to depend on an inference runtime, and it reaches
//! one only through `foreign`. What a signal produces and what it may not
//! produce is `docs/decisions/0005-verdicts-and-error-cost.md` once issue #8
//! lands it: a signal makes a rank and a flag with a reason, never a verdict.
