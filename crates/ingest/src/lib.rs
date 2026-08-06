//! Ingest and decoding.
//!
//! The only module allowed to depend on an image library, and it reaches one
//! only through `foreign`. Reading metadata without decoding the image is issue
//! #29, and importing a folder without waiting for the slowest file is issue
//! #28.
