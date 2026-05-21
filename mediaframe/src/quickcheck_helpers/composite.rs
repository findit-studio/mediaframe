//! Cluster C — audio composite metadata + capture + language.
//!
//! Validated `try_new` types build valid inputs FIRST (in-range floats via
//! `i32::arbitrary` clamped / scaled, non-empty strings via fallback like `"x"`
//! / `"application/octet-stream"`), THEN `try_new(...).expect(...)`. Never
//! pattern `try_new(arbitrary_float).unwrap()` — that would panic on input.
//!
//! Owned types:
//!   - audio::{Loudness, Fingerprint, CoverArt, Tags}
//!   - capture::{Device, GeoLocation}
//!   - lang::Language
