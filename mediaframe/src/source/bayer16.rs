//! 10 / 12 / 14 / 16-bit Bayer — single-plane mosaic source marker.
//!
//! `Bayer16<const BITS: u32, const BE: bool = false>` covers the full
//! family of high-bit-depth Bayer formats (10, 12, 14, and 16-bit; each
//! sample's *logical* value is low-packed in its `u16` — the stored
//! element is the raw wire byte order, see the endian contract below) in
//! both byte orders.
//!
//! # Endian contract — `<const BE: bool = false>`
//!
//! The marker carries `<const BE: bool = false>`, mirroring the Y2xx
//! (`Y210`/`Y212`/`Y216`) family. `BE` records the **byte order of the
//! plane `u16` samples**, matching the FFmpeg `bayer_*16LE` / `bayer_*16BE`
//! pixel-format suffix:
//!
//! - `BE = false` (default; e.g. `Bayer16<16>` = `Bayer16<16, false>`) —
//!   plane bytes are LE-encoded (at 16-bit, `AV_PIX_FMT_BAYER_*16LE`).
//! - `BE = true` (e.g. `Bayer16<16, true>`) — plane bytes are BE-encoded
//!   (at 16-bit, `AV_PIX_FMT_BAYER_*16BE`).
//!
//! FFmpeg defines a Bayer LE/BE split only at 16-bit; the 10 / 12 / 14-bit
//! forms are mediaframe extensions with no FFmpeg pixel format.
//!
//! Downstream row kernels handle the byte-swap (or no-op) under the hood;
//! callers do **not** pre-swap. The `BE` parameter rides on the
//! [`crate::frame::BayerFrame16`] / [`crate::frame::BayerSink16`] and is
//! propagated by the walker
//! ([`bayer16_to_endian`](crate::frame::bayer16_to_endian)) into the
//! kernel dispatch. The `BE = false` default keeps the single-generic
//! `Bayer16<BITS>` spelling source-compatible (it resolves to
//! `Bayer16<BITS, false>`).
//!
//! # Design note
//!
//! The walker functions ([`bayer16_to`](crate::frame::bayer16_to) /
//! [`bayer16_to_endian`](crate::frame::bayer16_to_endian)) and the row
//! borrow type ([`BayerRow16`](crate::frame::BayerRow16)) live in
//! [`crate::frame`] because they reference demosaic parameters
//! (`BayerDemosaic`, `WhiteBalance`, `ColorCorrectionMatrix`). Only the
//! zero-sized [`Bayer16`] marker lives here so that:
//!
//! - `Bayer16<BITS, BE>` can implement `mediaframe::SourceFormat`, and
//! - downstream crates that need to name the source format type do not
//!   need to depend on `colconv`.

marker! {
  /// Zero-sized marker for the high-bit-depth Bayer source family.
  ///
  /// Parameterized on the active bit depth `BITS` ∈ {10, 12, 14, 16}
  /// and the plane byte order `BE` (`false` = LE-encoded, the default;
  /// `true` = BE-encoded). Each sample's **logical** value (after
  /// byte-order normalization) is low-packed — active bits in
  /// `[BITS-1 : 0]`, high bits zero; the stored `u16` is the raw wire
  /// element. `BE` records that wire byte order and is honored by the
  /// downstream kernel; see the module-level endian contract.
  #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
  struct Bayer16<const BITS: u32, const BE: bool = false>;
}

/// Type alias for the 10-bit LE Bayer source marker.
pub type Bayer10 = Bayer16<10, false>;
/// Type alias for the 12-bit LE Bayer source marker.
pub type Bayer12 = Bayer16<12, false>;
/// Type alias for the 14-bit LE Bayer source marker.
pub type Bayer14 = Bayer16<14, false>;
/// Type alias for the 16-bit LE Bayer source marker (full-range `u16`).
pub type Bayer16Bit = Bayer16<16, false>;

/// Type alias for the 10-bit BE Bayer source marker (mediaframe extension).
pub type Bayer10Be = Bayer16<10, true>;
/// Type alias for the 12-bit BE Bayer source marker (mediaframe extension).
pub type Bayer12Be = Bayer16<12, true>;
/// Type alias for the 14-bit BE Bayer source marker (mediaframe extension).
pub type Bayer14Be = Bayer16<14, true>;
/// Type alias for the 16-bit BE Bayer source marker
/// (`AV_PIX_FMT_BAYER_*16BE`, full-range `u16`).
pub type Bayer16BitBe = Bayer16<16, true>;
