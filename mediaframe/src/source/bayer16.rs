//! 10 / 12 / 14 / 16-bit Bayer — single-plane mosaic source marker.
//!
//! `Bayer16<const BITS: u32>` covers the full family of high-bit-depth
//! Bayer formats (10, 12, 14, and 16-bit low-packed `u16` planes).
//!
//! # Design note
//!
//! The walker function (`colconv::raw::bayer16_to`) and the row borrow
//! type (`colconv::raw::BayerRow16`) live in `colconv::raw` because they
//! require demosaic parameters (`BayerDemosaic`, `WhiteBalance`,
//! `ColorCorrectionMatrix`) that are colconv processing-layer types.
//! Only the zero-sized [`Bayer16`] marker lives here so that:
//!
//! - `Bayer16<BITS>` can implement `mediaframe::SourceFormat`, and
//! - downstream crates that need to name the source format type do not
//!   need to depend on `colconv`.

marker! {
  /// Zero-sized marker for the high-bit-depth Bayer source family.
  ///
  /// Parameterized on the active bit depth `BITS` ∈ {10, 12, 14, 16}.
  /// Samples are low-packed `u16` — active bits in positions
  /// `[BITS-1 : 0]`, remaining high bits are zero.
  #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
  struct Bayer16<const BITS: u32>;
}

/// Type alias for the 10-bit Bayer source marker.
pub type Bayer10 = Bayer16<10>;
/// Type alias for the 12-bit Bayer source marker.
pub type Bayer12 = Bayer16<12>;
/// Type alias for the 14-bit Bayer source marker.
pub type Bayer14 = Bayer16<14>;
/// Type alias for the 16-bit Bayer source marker (full-range `u16`).
pub type Bayer16Bit = Bayer16<16>;
