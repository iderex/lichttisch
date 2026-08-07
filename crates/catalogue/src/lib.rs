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
//!
//! One thing here does not wait for that choice. Who is allowed to write a
//! catalogue is a question about processes rather than about storage, and
//! `lock` answers it: one writer, refused by name rather than serialised.
//! `docs/catalogue-locking.md` is where the two restrictions that follow from
//! how it is done are stated.

pub mod lock;
