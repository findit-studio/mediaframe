//! 8-bit Bayer (`AV_PIX_FMT_BAYER_BGGR8` / `RGGB8` / `GRBG8` /
//! `GBRG8`) — single-plane mosaic source marker.
//!
//! # Design note
//!
//! The walker function (`colconv::raw::bayer_to`) and the row borrow
//! type (`colconv::raw::BayerRow`) live in `colconv::raw` because they
//! require demosaic parameters (`BayerDemosaic`, `WhiteBalance`,
//! `ColorCorrectionMatrix`) that are colconv processing-layer types.
//! Only the zero-sized [`Bayer`] marker lives here so that:
//!
//! - `Bayer` can implement `mediaframe::SourceFormat` (which uses a
//!   `pub(crate)` seal), and
//! - downstream crates that need to name the source format type do not
//!   need to depend on `colconv`.

marker! {
  /// Zero-sized marker for the 8-bit Bayer mosaic source format
  /// (`AV_PIX_FMT_BAYER_BGGR8`, `RGGB8`, `GRBG8`, `GBRG8`).
  ///
  /// Used as the `F` type parameter on `colconv::sinker::MixedSinker`
  /// and as a [`crate::SourceFormat`] bound on Bayer-specific sinks.
  #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
  struct Bayer;
}
