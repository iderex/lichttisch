//! The catalogue and its storage.
//!
//! What a photograph is to this module is `docs/decisions/0010-catalogue-schema.md`,
//! and which values this project owns rather than caches is the authority rule
//! in `docs/decisions/0002-integration-shape.md`. Nothing above this module
//! composes a query in the storage engine's own language; that rule is the
//! reason a later engine change is a change in one directory.
//!
//! Which engine sits underneath is not decided yet. Issue #5 measures the
//! candidates and issue #6 chooses one.
