//! Cluster B — closed FFmpeg-coded enums w/ lossless `from_u32`, colour /
//! pixel-format / frame geometry / disposition structs.
//!
//! Coded enums: `T::from_u32(u32::arbitrary(g))` — covers `Unknown(u32)` /
//! `Reserved(_)` arms.
//!
//! Structs: build via public `new(...)` with each field via
//! `<FieldT>::arbitrary(g)`. Watch `NonZeroU32` for `Rational` denom.
//!
//! Owned types: 13 coded enums + 11 structs (colour×6, frame×5).
