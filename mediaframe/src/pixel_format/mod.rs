//! Pixel format identifier — comprehensive coverage of FFmpeg's
//! `AVPixelFormat` enum plus Bayer mosaic and cinema-RAW formats.
//!
//! Naming convention: each variant's [`Display`] form is the
//! lowercase FFmpeg name where one exists (`yuv420p`, `nv12`, `p010le`,
//! …) so logs / wire formats line up with FFmpeg / `colconv`. The
//! variant identifier is the FFmpeg name in PascalCase
//! (`Yuv420p`, `Nv12`, `P010Le`, …).
//!
//! The enum covers:
//! - **Planar YUV** at 4:2:0 / 4:2:2 / 4:4:0 / 4:4:4, 8-bit and
//!   high-bit-depth (9 / 10 / 12 / 14 / 16-bit).
//! - **Planar YUVA** (with alpha) at the same subsampling × bit-depth.
//! - **Semi-planar YUV** (NV-family) at 4:2:0 / 4:2:2 / 4:4:4, 8-bit
//!   and 10 / 12 / 16-bit (P0xx / P2xx / P4xx).
//! - **Packed YUV** (yuyv / uyvy / yvyu / v210 / v410 / xv36 / Y2xx /
//!   ayuv64 / vuya / vuyx).
//! - **Packed RGB** at 8-bit (rgb24 / bgr24 / rgba / bgra / argb /
//!   abgr / rgbx / bgrx / xrgb / xbgr), low-bit (rgb444 / 555 / 565,
//!   bgr444 / 555 / 565), and high-bit (rgb48 / bgr48 / rgba64 / bgra64
//!   / x2rgb10 / x2bgr10), plus float (rgbf16 / rgbf32).
//! - **Planar GBR / GBRA** at 8-bit + high-bit + float.
//! - **Greyscale** (gray8 / 9 / 10 / 12 / 14 / 16 / f32) and
//!   greyscale-with-alpha (ya8 / ya16) and monochrome 1-bit
//!   (monowhite / monoblack).
//! - **Bayer** (BGGR / RGGB / GBRG / GRBG) at 8 / 10 / 12 / 14 / 16-bit.
//! - **Paletted** (pal8).
//!
//! Hardware-frame markers (FFmpeg's `AV_PIX_FMT_VIDEOTOOLBOX` /
//! `_VAAPI` / `_CUDA` / `_D3D11` / `_DRM_PRIME` / `_MEDIACODEC` /
//! `_VULKAN`) are intentionally **not** in this enum: the unified
//! vocabulary describes CPU-side decoded pixel data, and a frame
//! carrying GPU-resident buffers must be transferred to a CPU format
//! before reaching a `mediadecode::VideoFrame` consumer. Backend
//! crates handle the HW path internally.
//!
//! The **text** form is the wire form: [`PixelFormat::as_str`] renders the
//! FFmpeg slug and [`FromStr`](core::str::FromStr) reads it back, with
//! [`PixelFormat::Other`] carrying any name this build does not enumerate.
//! The two sides are deliberately not mirror images — **emission is one
//! canonical slug per variant, parse additionally accepts the documented
//! FFmpeg synonyms** (`gray`, `monob`, `monow`, the three names FFmpeg's
//! descriptor table spells differently from its `AV_PIX_FMT_<NAME>`
//! enumerator). Round-tripping a value is unaffected; reading a name off
//! `ffprobe` now lands on the named variant.
//! [`PixelFormat::to_u32`] / [`PixelFormat::from_u32`] remain as FFmpeg
//! interop helpers over a number space that has no room for a name, and
//! return [`None`] outside it.

use derive_more::{Display, IsVariant};
#[cfg(any(feature = "std", feature = "alloc"))]
use smol_bytes::Utf8Bytes;

/// Pixel format identifier covering FFmpeg + Bayer + cinema-RAW.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::pixel_format")
)]
pub enum PixelFormat {
  /// No format — FFmpeg's own `AV_PIX_FMT_NONE`, and the [`Default`].
  ///
  /// A **named** member of the vocabulary, not an escape arm: it carries
  /// no payload, owns the slug `"none"`, and round-trips exactly. It is
  /// the state a freshly-defaulted descriptor is in, before a decoder has
  /// said what it produces.
  None,

  // ===================================================================
  // Planar YUV 8-bit
  // ===================================================================
  /// Planar 4:2:0 YUV, 8-bit (`AV_PIX_FMT_YUV420P`).
  Yuv420p,
  /// Planar 4:2:2 YUV, 8-bit.
  Yuv422p,
  /// Planar 4:4:0 YUV, 8-bit (vertically subsampled chroma).
  Yuv440p,
  /// Planar 4:4:4 YUV, 8-bit.
  Yuv444p,
  /// Planar 4:1:1 YUV, 8-bit.
  Yuv411p,
  /// Planar 4:1:0 YUV, 8-bit.
  Yuv410p,

  // ===================================================================
  // Deprecated full-range YUV aliases (yuvj-family)
  // ===================================================================
  /// Deprecated full-range alias of [`Self::Yuv411p`] (FFmpeg keeps for
  /// backward compat; downstream should prefer [`Self::Yuv411p`] +
  /// `DynamicRange::Full`).
  Yuvj411p,
  /// Deprecated full-range alias of [`Self::Yuv420p`] (FFmpeg keeps for
  /// backward compat; downstream should prefer [`Self::Yuv420p`] +
  /// `DynamicRange::Full`).
  Yuvj420p,
  /// Deprecated full-range alias of [`Self::Yuv422p`] (FFmpeg keeps for
  /// backward compat; downstream should prefer [`Self::Yuv422p`] +
  /// `DynamicRange::Full`).
  Yuvj422p,
  /// Deprecated full-range alias of [`Self::Yuv440p`] (FFmpeg keeps for
  /// backward compat; downstream should prefer [`Self::Yuv440p`] +
  /// `DynamicRange::Full`).
  Yuvj440p,
  /// Deprecated full-range alias of [`Self::Yuv444p`] (FFmpeg keeps for
  /// backward compat; downstream should prefer [`Self::Yuv444p`] +
  /// `DynamicRange::Full`).
  Yuvj444p,

  // ===================================================================
  // Planar YUV high-bit-depth (4:2:0)
  // ===================================================================
  /// Planar 4:2:0 YUV, 9-bit little-endian.
  Yuv420p9Le,
  /// Planar 4:2:0 YUV, 9-bit big-endian.
  Yuv420p9Be,
  /// Planar 4:2:0 YUV, 10-bit little-endian.
  Yuv420p10Le,
  /// Planar 4:2:0 YUV, 10-bit big-endian.
  Yuv420p10Be,
  /// Planar 4:2:0 YUV, 12-bit little-endian.
  Yuv420p12Le,
  /// Planar 4:2:0 YUV, 12-bit big-endian.
  Yuv420p12Be,
  /// Planar 4:2:0 YUV, 14-bit little-endian.
  Yuv420p14Le,
  /// Planar 4:2:0 YUV, 14-bit big-endian.
  Yuv420p14Be,
  /// Planar 4:2:0 YUV, 16-bit little-endian.
  Yuv420p16Le,
  /// Planar 4:2:0 YUV, 16-bit big-endian.
  Yuv420p16Be,

  // ===================================================================
  // Planar YUV high-bit-depth (4:2:2)
  // ===================================================================
  /// Planar 4:2:2 YUV, 9-bit little-endian.
  Yuv422p9Le,
  /// Planar 4:2:2 YUV, 9-bit big-endian.
  Yuv422p9Be,
  /// Planar 4:2:2 YUV, 10-bit little-endian.
  Yuv422p10Le,
  /// Planar 4:2:2 YUV, 10-bit big-endian.
  Yuv422p10Be,
  /// Planar 4:2:2 YUV, 12-bit little-endian.
  Yuv422p12Le,
  /// Planar 4:2:2 YUV, 12-bit big-endian.
  Yuv422p12Be,
  /// Planar 4:2:2 YUV, 14-bit little-endian.
  Yuv422p14Le,
  /// Planar 4:2:2 YUV, 14-bit big-endian.
  Yuv422p14Be,
  /// Planar 4:2:2 YUV, 16-bit little-endian.
  Yuv422p16Le,
  /// Planar 4:2:2 YUV, 16-bit big-endian.
  Yuv422p16Be,

  // ===================================================================
  // Planar YUV high-bit-depth (4:4:0)
  // ===================================================================
  /// Planar 4:4:0 YUV, 10-bit little-endian.
  Yuv440p10Le,
  /// Planar 4:4:0 YUV, 10-bit big-endian.
  Yuv440p10Be,
  /// Planar 4:4:0 YUV, 12-bit little-endian.
  Yuv440p12Le,
  /// Planar 4:4:0 YUV, 12-bit big-endian.
  Yuv440p12Be,

  // ===================================================================
  // Planar YUV high-bit-depth (4:4:4)
  // ===================================================================
  /// Planar 4:4:4 YUV, 9-bit little-endian.
  Yuv444p9Le,
  /// Planar 4:4:4 YUV, 9-bit big-endian.
  Yuv444p9Be,
  /// Planar 4:4:4 YUV, 10-bit little-endian.
  Yuv444p10Le,
  /// Planar 4:4:4 YUV, 10-bit big-endian.
  Yuv444p10Be,
  /// Planar 4:4:4 YUV, 12-bit little-endian.
  Yuv444p12Le,
  /// Planar 4:4:4 YUV, 12-bit big-endian.
  Yuv444p12Be,
  /// Planar 4:4:4 YUV, 14-bit little-endian.
  Yuv444p14Le,
  /// Planar 4:4:4 YUV, 14-bit big-endian.
  Yuv444p14Be,
  /// Planar 4:4:4 YUV, 16-bit little-endian.
  Yuv444p16Le,
  /// Planar 4:4:4 YUV, 16-bit big-endian.
  Yuv444p16Be,

  // ===================================================================
  // MSB-packed YUV (4:4:4)
  // ===================================================================
  /// Planar 4:4:4 YUV, 10-bit MSB-packed, little-endian.
  Yuv444p10MsbLe,
  /// Planar 4:4:4 YUV, 10-bit MSB-packed, big-endian.
  Yuv444p10MsbBe,
  /// Planar 4:4:4 YUV, 12-bit MSB-packed, little-endian.
  Yuv444p12MsbLe,
  /// Planar 4:4:4 YUV, 12-bit MSB-packed, big-endian.
  Yuv444p12MsbBe,

  // ===================================================================
  // Planar YUVA (with alpha)
  // ===================================================================
  /// Planar 4:2:0 YUVA, 8-bit.
  Yuva420p,
  /// Planar 4:2:2 YUVA, 8-bit.
  Yuva422p,
  /// Planar 4:4:4 YUVA, 8-bit.
  Yuva444p,
  /// Planar 4:2:0 YUVA, 9-bit little-endian.
  Yuva420p9Le,
  /// Planar 4:2:0 YUVA, 9-bit big-endian.
  Yuva420p9Be,
  /// Planar 4:2:2 YUVA, 9-bit little-endian.
  Yuva422p9Le,
  /// Planar 4:2:2 YUVA, 9-bit big-endian.
  Yuva422p9Be,
  /// Planar 4:4:4 YUVA, 9-bit little-endian.
  Yuva444p9Le,
  /// Planar 4:4:4 YUVA, 9-bit big-endian.
  Yuva444p9Be,
  /// Planar 4:2:0 YUVA, 10-bit little-endian.
  Yuva420p10Le,
  /// Planar 4:2:0 YUVA, 10-bit big-endian.
  Yuva420p10Be,
  /// Planar 4:2:2 YUVA, 10-bit little-endian.
  Yuva422p10Le,
  /// Planar 4:2:2 YUVA, 10-bit big-endian.
  Yuva422p10Be,
  /// Planar 4:4:4 YUVA, 10-bit little-endian.
  Yuva444p10Le,
  /// Planar 4:4:4 YUVA, 10-bit big-endian.
  Yuva444p10Be,
  /// Planar 4:2:0 YUVA, 12-bit little-endian
  /// (`AV_PIX_FMT_YUVA420P12LE`). Discriminant placed after
  /// the 16-bit block because the 12-bit slot in the original
  /// 200-series numbering (between 10Le at 206 and the 4:2:2
  /// 12Le at 209) was already taken by the 4:2:2 / 4:4:4
  /// 12Le forms; adding a new tail slot keeps existing
  /// discriminants stable. Surfaced by WebCodecs as the
  /// `I420AP12` `VideoPixelFormat`.
  Yuva420p12Le,
  /// Planar 4:2:0 YUVA, 12-bit big-endian (`yuva420p12be` — no FFmpeg
  /// `AV_PIX_FMT` enum; the byte-order sibling of [`Self::Yuva420p12Le`],
  /// supported for symmetry with the 9 / 10 / 16-bit YUVA 4:2:0 BE
  /// forms). Discriminant placed at the YUVA tail (after the 16Be
  /// block) for the same reason the 12Le form was: the in-order 200-
  /// series slots were already consumed, so a new tail slot keeps every
  /// existing discriminant stable.
  Yuva420p12Be,
  /// Planar 4:2:2 YUVA, 12-bit little-endian.
  Yuva422p12Le,
  /// Planar 4:2:2 YUVA, 12-bit big-endian.
  Yuva422p12Be,
  /// Planar 4:4:4 YUVA, 12-bit little-endian.
  Yuva444p12Le,
  /// Planar 4:4:4 YUVA, 12-bit big-endian.
  Yuva444p12Be,
  /// Planar 4:4:4 YUVA, 14-bit little-endian.
  Yuva444p14Le,
  /// Planar 4:2:0 YUVA, 16-bit little-endian.
  Yuva420p16Le,
  /// Planar 4:2:0 YUVA, 16-bit big-endian.
  Yuva420p16Be,
  /// Planar 4:2:2 YUVA, 16-bit little-endian.
  Yuva422p16Le,
  /// Planar 4:2:2 YUVA, 16-bit big-endian.
  Yuva422p16Be,
  /// Planar 4:4:4 YUVA, 16-bit little-endian.
  Yuva444p16Le,
  /// Planar 4:4:4 YUVA, 16-bit big-endian.
  Yuva444p16Be,

  // ===================================================================
  // Semi-planar YUV (NV-family) — 8-bit
  // ===================================================================
  /// 4:2:0 semi-planar Y plane + interleaved Cb/Cr (`AV_PIX_FMT_NV12`).
  Nv12,
  /// 4:2:0 semi-planar Y + interleaved Cr/Cb (`AV_PIX_FMT_NV21`).
  Nv21,
  /// 4:2:2 semi-planar Y + interleaved Cb/Cr.
  Nv16,
  /// 4:4:4 semi-planar Y + interleaved Cb/Cr.
  Nv24,
  /// 4:4:4 semi-planar Y + interleaved Cr/Cb.
  Nv42,
  /// 10-bit semi-planar 4:2:2 YUV — a Y plane plus an interleaved UV plane,
  /// one `u16` per sample with the 10 data bits in the **low** bits (P210's
  /// low-bit-packed twin), little-endian.
  Nv20Le,
  /// 10-bit semi-planar 4:2:2 YUV — a Y plane plus an interleaved UV plane,
  /// one `u16` per sample with the 10 data bits in the **low** bits (P210's
  /// low-bit-packed twin), big-endian.
  Nv20Be,

  // ===================================================================
  // Semi-planar YUV high-bit-depth (P0xx / P2xx / P4xx)
  // ===================================================================
  /// 4:2:0 semi-planar 10-bit, little-endian (`AV_PIX_FMT_P010LE`).
  P010Le,
  /// 4:2:0 semi-planar 10-bit, big-endian.
  P010Be,
  /// 4:2:0 semi-planar 12-bit, little-endian.
  P012Le,
  /// 4:2:0 semi-planar 12-bit, big-endian.
  P012Be,
  /// 4:2:0 semi-planar 16-bit, little-endian.
  P016Le,
  /// 4:2:0 semi-planar 16-bit, big-endian.
  P016Be,
  /// 4:2:2 semi-planar 10-bit, little-endian.
  P210Le,
  /// 4:2:2 semi-planar 10-bit, big-endian.
  P210Be,
  /// 4:2:2 semi-planar 12-bit, little-endian (FFmpeg 5.1+).
  P212Le,
  /// 4:2:2 semi-planar 12-bit, big-endian (FFmpeg 5.1+).
  P212Be,
  /// 4:2:2 semi-planar 16-bit, little-endian.
  P216Le,
  /// 4:2:2 semi-planar 16-bit, big-endian.
  P216Be,
  /// 4:4:4 semi-planar 10-bit, little-endian.
  P410Le,
  /// 4:4:4 semi-planar 10-bit, big-endian.
  P410Be,
  /// 4:4:4 semi-planar 12-bit, little-endian (FFmpeg 5.1+).
  P412Le,
  /// 4:4:4 semi-planar 12-bit, big-endian (FFmpeg 5.1+).
  P412Be,
  /// 4:4:4 semi-planar 16-bit, little-endian.
  P416Le,
  /// 4:4:4 semi-planar 16-bit, big-endian.
  P416Be,

  // ===================================================================
  // Packed YUV 8-bit
  // ===================================================================
  /// 4:2:2 packed YUV: Y0 U Y1 V (`AV_PIX_FMT_YUYV422`).
  Yuyv422,
  /// 4:2:2 packed YUV: U Y0 V Y1 (`AV_PIX_FMT_UYVY422`).
  Uyvy422,
  /// 4:2:2 packed YUV: Y0 V Y1 U (`AV_PIX_FMT_YVYU422`).
  Yvyu422,
  /// Packed YUV 4:1:1, 12bpp (`AV_PIX_FMT_UYYVYY411`).
  Uyyvyy411,

  // ===================================================================
  // Packed YUV high-bit-depth
  // ===================================================================
  /// 4:2:2 packed YUV 10-bit, little-endian (`AV_PIX_FMT_Y210LE`).
  Y210Le,
  /// 4:2:2 packed YUV 10-bit, big-endian.
  Y210Be,
  /// 4:2:2 packed YUV 12-bit, little-endian (`AV_PIX_FMT_Y212LE`).
  Y212Le,
  /// 4:2:2 packed YUV 12-bit, big-endian.
  Y212Be,
  /// 4:2:2 packed YUV 16-bit, little-endian (`AV_PIX_FMT_Y216LE`).
  Y216Le,
  /// 4:2:2 packed YUV 16-bit, big-endian.
  Y216Be,
  /// 4:2:2 packed 10-bit, 3 samples per 32-bit word (`AV_PIX_FMT_V210`).
  V210,
  /// 4:4:4 packed 10-bit, one 32-bit word per sample (`AV_PIX_FMT_V410LE`).
  V410Le,
  /// 4:4:4 packed 10-bit, one 32-bit word per sample (`AV_PIX_FMT_V410BE`),
  /// big-endian.
  V410Be,
  /// 4:4:4 packed 10-bit, alternative layout (`AV_PIX_FMT_XV30LE`),
  /// little-endian.
  Xv30Le,
  /// 4:4:4 packed 10-bit, alternative layout (`AV_PIX_FMT_XV30BE`),
  /// big-endian.
  Xv30Be,
  /// 4:4:4 packed 10-bit, alternative layout (`AV_PIX_FMT_V30XLE`),
  /// little-endian (distinct slug from `xv30le`).
  V30xLe,
  /// 4:4:4 packed 10-bit, alternative layout (`AV_PIX_FMT_V30XBE`),
  /// big-endian (distinct slug from `xv30be`).
  V30xBe,
  /// 4:4:4 packed 12-bit, one 16-bit word per channel (`AV_PIX_FMT_XV36LE`),
  /// little-endian.
  Xv36Le,
  /// 4:4:4 packed 12-bit, one 16-bit word per channel (`AV_PIX_FMT_XV36BE`),
  /// big-endian.
  Xv36Be,
  /// 4:4:4 packed 16-bit, little-endian (`AV_PIX_FMT_XV48LE`).
  Xv48Le,
  /// 4:4:4 packed 16-bit, big-endian (`AV_PIX_FMT_XV48BE`).
  Xv48Be,
  /// 4:4:4 packed 8-bit byte quadruple V, U, Y, A (`AV_PIX_FMT_VUYA`).
  Vuya,
  /// 4:4:4 packed 8-bit V, U, Y, X (alpha-as-padding).
  Vuyx,
  /// Packed AYUV 4:4:4, 32bpp (8-bit; distinct from [`Self::Ayuv64Le`]/[`Self::Ayuv64Be`]).
  Ayuv,
  /// 4:4:4 packed 16-bit word quadruple A, Y, U, V (`AV_PIX_FMT_AYUV64LE`),
  /// little-endian.
  Ayuv64Le,
  /// 4:4:4 packed 16-bit word quadruple A, Y, U, V (`AV_PIX_FMT_AYUV64BE`),
  /// big-endian.
  Ayuv64Be,
  /// Packed UYVA 4:4:4, 32bpp.
  Uyva,
  /// Packed VYU 4:4:4 8-bit (3 bytes per pixel).
  Vyu444,

  // ===================================================================
  // XYZ color space
  // ===================================================================
  /// Packed XYZ 4:4:4, 36bpp (12 bits each), little-endian.
  Xyz12Le,
  /// Packed XYZ 4:4:4, 36bpp (12 bits each), big-endian.
  Xyz12Be,

  // ===================================================================
  // Packed RGB 8-bit
  // ===================================================================
  /// 24-bit packed RGB (`AV_PIX_FMT_RGB24`).
  Rgb24,
  /// 24-bit packed BGR.
  Bgr24,
  /// 32-bit packed RGBA.
  Rgba,
  /// 32-bit packed BGRA.
  Bgra,
  /// 32-bit packed ARGB.
  Argb,
  /// 32-bit packed ABGR.
  Abgr,
  /// 32-bit packed RGB with X (unused) byte.
  /// FFmpeg slug uses `rgb0`-suffix; Rust variant uses `X` because
  /// identifiers can't start with a digit.
  Rgbx,
  /// 32-bit packed BGR with X (unused) byte.
  /// FFmpeg slug uses `bgr0`-suffix; Rust variant uses `X` because
  /// identifiers can't start with a digit.
  Bgrx,
  /// 32-bit packed XRGB (X unused, then RGB).
  /// FFmpeg slug uses `0rgb`-prefix; Rust variant uses `X` because
  /// identifiers can't start with a digit.
  Xrgb,
  /// 32-bit packed XBGR.
  /// FFmpeg slug uses `0bgr`-prefix; Rust variant uses `X` because
  /// identifiers can't start with a digit.
  Xbgr,
  /// 32-bit RGB10 in low bits, 2 bits unused (`AV_PIX_FMT_X2RGB10LE`),
  /// little-endian.
  X2Rgb10Le,
  /// 32-bit RGB10 in low bits, 2 bits unused, big-endian.
  X2Rgb10Be,
  /// 32-bit BGR10 in low bits, 2 bits unused, little-endian.
  X2Bgr10Le,
  /// 32-bit BGR10 in low bits, 2 bits unused, big-endian.
  X2Bgr10Be,
  /// Packed GBR 24bpp (distinct from planar [`Self::Gbrp`]).
  Gbr24p,

  // ===================================================================
  // Packed RGB low-bit (4-bit and 8-bit)
  // ===================================================================
  /// 1+1+1+1-bit packed RGB.
  Rgb4,
  /// Same data as [`Self::Rgb4`], 1 byte per pixel.
  Rgb4Byte,
  /// 3+3+2-bit packed RGB.
  Rgb8,
  /// 1+1+1+1-bit packed BGR.
  Bgr4,
  /// Same data as [`Self::Bgr4`], 1 byte per pixel.
  Bgr4Byte,
  /// 3+3+2-bit packed BGR.
  Bgr8,

  // ===================================================================
  // Packed RGB low-bit (16-bit)
  // ===================================================================
  /// 16-bit packed RGB, 4 bits per channel + 4 unused, little-endian.
  Rgb444Le,
  /// 16-bit packed RGB, 4 bits per channel + 4 unused, big-endian.
  Rgb444Be,
  /// 16-bit packed BGR, 4 bits per channel + 4 unused, little-endian.
  Bgr444Le,
  /// 16-bit packed BGR, 4 bits per channel + 4 unused, big-endian.
  Bgr444Be,
  /// 16-bit packed RGB, 5/5/5 layout, little-endian.
  Rgb555Le,
  /// 16-bit packed RGB, 5/5/5 layout, big-endian.
  Rgb555Be,
  /// 16-bit packed BGR, 5/5/5 layout, little-endian.
  Bgr555Le,
  /// 16-bit packed BGR, 5/5/5 layout, big-endian.
  Bgr555Be,
  /// 16-bit packed RGB, 5/6/5 layout, little-endian.
  Rgb565Le,
  /// 16-bit packed RGB, 5/6/5 layout, big-endian.
  Rgb565Be,
  /// 16-bit packed BGR, 5/6/5 layout, little-endian.
  Bgr565Le,
  /// 16-bit packed BGR, 5/6/5 layout, big-endian.
  Bgr565Be,

  // ===================================================================
  // Packed RGB high-bit-depth
  // ===================================================================
  /// 48-bit packed RGB, 16 bits per channel, little-endian.
  Rgb48Le,
  /// 48-bit packed RGB, 16 bits per channel, big-endian.
  Rgb48Be,
  /// 48-bit packed BGR, 16 bits per channel, little-endian.
  Bgr48Le,
  /// 48-bit packed BGR, 16 bits per channel, big-endian.
  Bgr48Be,
  /// 64-bit packed RGBA, 16 bits per channel, little-endian.
  Rgba64Le,
  /// 64-bit packed RGBA, 16 bits per channel, big-endian.
  Rgba64Be,
  /// 64-bit packed BGRA, 16 bits per channel, little-endian.
  Bgra64Le,
  /// 64-bit packed BGRA, 16 bits per channel, big-endian.
  Bgra64Be,

  // ===================================================================
  // Packed RGB 96-bit / 128-bit (new in n8.1)
  // ===================================================================
  /// 96-bit packed RGB, 32 bits per channel, little-endian.
  Rgb96Le,
  /// 96-bit packed RGB, 32 bits per channel, big-endian.
  Rgb96Be,
  /// 128-bit packed RGBA, 32 bits per channel, little-endian.
  Rgba128Le,
  /// 128-bit packed RGBA, 32 bits per channel, big-endian.
  Rgba128Be,

  // ===================================================================
  // Packed RGB float / half-float
  // ===================================================================
  /// 48-bit packed RGB, 16-bit half-float per channel, little-endian.
  Rgbf16Le,
  /// 48-bit packed RGB, 16-bit half-float per channel, big-endian.
  Rgbf16Be,
  /// 96-bit packed RGB, 32-bit float per channel, little-endian.
  Rgbf32Le,
  /// 96-bit packed RGB, 32-bit float per channel, big-endian.
  Rgbf32Be,
  /// 64-bit packed RGBA, 16-bit half-float per channel, little-endian.
  Rgbaf16Le,
  /// 64-bit packed RGBA, 16-bit half-float per channel, big-endian.
  Rgbaf16Be,
  /// 128-bit packed RGBA, 32-bit float per channel, little-endian.
  Rgbaf32Le,
  /// 128-bit packed RGBA, 32-bit float per channel, big-endian.
  Rgbaf32Be,

  // ===================================================================
  // Planar GBR 8-bit
  // ===================================================================
  /// Planar 4:4:4 G/B/R, 8-bit.
  Gbrp,
  /// Planar 4:4:4 G/B/R, 9-bit little-endian.
  Gbrp9Le,
  /// Planar 4:4:4 G/B/R, 9-bit big-endian.
  Gbrp9Be,
  /// Planar 4:4:4 G/B/R, 10-bit little-endian.
  Gbrp10Le,
  /// Planar 4:4:4 G/B/R, 10-bit big-endian.
  Gbrp10Be,
  /// Planar 4:4:4 G/B/R, 10-bit MSB-packed, little-endian.
  Gbrp10MsbLe,
  /// Planar 4:4:4 G/B/R, 10-bit MSB-packed, big-endian.
  Gbrp10MsbBe,
  /// Planar 4:4:4 G/B/R, 12-bit little-endian.
  Gbrp12Le,
  /// Planar 4:4:4 G/B/R, 12-bit big-endian.
  Gbrp12Be,
  /// Planar 4:4:4 G/B/R, 12-bit MSB-packed, little-endian.
  Gbrp12MsbLe,
  /// Planar 4:4:4 G/B/R, 12-bit MSB-packed, big-endian.
  Gbrp12MsbBe,
  /// Planar 4:4:4 G/B/R, 14-bit little-endian.
  Gbrp14Le,
  /// Planar 4:4:4 G/B/R, 14-bit big-endian.
  Gbrp14Be,
  /// Planar 4:4:4 G/B/R, 16-bit little-endian.
  Gbrp16Le,
  /// Planar 4:4:4 G/B/R, 16-bit big-endian.
  Gbrp16Be,
  /// Planar 4:4:4 G/B/R, 16-bit half-float, little-endian.
  Gbrpf16Le,
  /// Planar 4:4:4 G/B/R, 16-bit half-float, big-endian.
  Gbrpf16Be,
  /// Planar 4:4:4 G/B/R, 32-bit float, little-endian.
  Gbrpf32Le,
  /// Planar 4:4:4 G/B/R, 32-bit float, big-endian.
  Gbrpf32Be,

  // ===================================================================
  // Planar GBRA (with alpha)
  // ===================================================================
  /// Planar 4:4:4 G/B/R/A, 8-bit.
  Gbrap,
  /// Planar 4:4:4 G/B/R/A, 10-bit little-endian.
  Gbrap10Le,
  /// Planar 4:4:4 G/B/R/A, 10-bit big-endian.
  Gbrap10Be,
  /// Planar 4:4:4 G/B/R/A, 12-bit little-endian.
  Gbrap12Le,
  /// Planar 4:4:4 G/B/R/A, 12-bit big-endian.
  Gbrap12Be,
  /// Planar 4:4:4 G/B/R/A, 14-bit little-endian.
  Gbrap14Le,
  /// Planar 4:4:4 G/B/R/A, 14-bit big-endian.
  Gbrap14Be,
  /// Planar 4:4:4 G/B/R/A, 16-bit little-endian.
  Gbrap16Le,
  /// Planar 4:4:4 G/B/R/A, 16-bit big-endian.
  Gbrap16Be,
  /// Planar 4:4:4 G/B/R/A, 32-bit integer, little-endian.
  Gbrap32Le,
  /// Planar 4:4:4 G/B/R/A, 32-bit integer, big-endian.
  Gbrap32Be,
  /// Planar 4:4:4 G/B/R/A, 16-bit half-float, little-endian.
  Gbrapf16Le,
  /// Planar 4:4:4 G/B/R/A, 16-bit half-float, big-endian.
  Gbrapf16Be,
  /// Planar 4:4:4 G/B/R/A, 32-bit float, little-endian.
  Gbrapf32Le,
  /// Planar 4:4:4 G/B/R/A, 32-bit float, big-endian.
  Gbrapf32Be,

  // ===================================================================
  // Greyscale
  // ===================================================================
  /// 8-bit greyscale (`AV_PIX_FMT_GRAY8`).
  ///
  /// Renders `"gray8"` — the header identifier. FFmpeg's own descriptor
  /// name for this format is `"gray"`, which [`FromStr`](core::str::FromStr)
  /// accepts as a synonym.
  Gray8,
  /// 8-bit greyscale — FFmpeg `AV_PIX_FMT_GRAY8A` alias of [`Self::Ya8`];
  /// preserved as a separate variant since mediaframe's wire format is
  /// discriminant-independent.
  Gray8a,
  /// 9-bit greyscale, little-endian.
  Gray9Le,
  /// 9-bit greyscale, big-endian.
  Gray9Be,
  /// 10-bit greyscale, little-endian.
  Gray10Le,
  /// 10-bit greyscale, big-endian.
  Gray10Be,
  /// 12-bit greyscale, little-endian.
  Gray12Le,
  /// 12-bit greyscale, big-endian.
  Gray12Be,
  /// 14-bit greyscale, little-endian.
  Gray14Le,
  /// 14-bit greyscale, big-endian.
  Gray14Be,
  /// 16-bit greyscale, little-endian.
  Gray16Le,
  /// 16-bit greyscale, big-endian.
  Gray16Be,
  /// 32-bit integer greyscale, little-endian.
  Gray32Le,
  /// 32-bit integer greyscale, big-endian.
  Gray32Be,
  /// 32-bit float greyscale, little-endian.
  Grayf32Le,
  /// 32-bit float greyscale, big-endian.
  Grayf32Be,
  /// 16-bit half-float greyscale, little-endian.
  Grayf16Le,
  /// 16-bit half-float greyscale, big-endian.
  Grayf16Be,
  /// 16-bit greyscale-with-alpha.
  Ya8,
  /// FFmpeg `AV_PIX_FMT_Y400A` alias of [`Self::Ya8`]; preserved as a separate
  /// variant since mediaframe's wire format is discriminant-independent.
  Y400a,
  /// 32-bit greyscale-with-alpha, little-endian.
  Ya16Le,
  /// 32-bit greyscale-with-alpha, big-endian.
  Ya16Be,
  /// 16-bit half-float greyscale-with-alpha, little-endian.
  Yaf16Le,
  /// 16-bit half-float greyscale-with-alpha, big-endian.
  Yaf16Be,
  /// 64-bit float greyscale-with-alpha, little-endian.
  Yaf32Le,
  /// 64-bit float greyscale-with-alpha, big-endian.
  Yaf32Be,

  // ===================================================================
  // Monochrome 1-bit
  // ===================================================================
  /// 1-bit monochrome, white = 0 (`AV_PIX_FMT_MONOWHITE`).
  ///
  /// Renders `"monowhite"` — the header identifier. FFmpeg's own
  /// descriptor name is `"monow"`, accepted as a parse synonym.
  Monowhite,
  /// 1-bit monochrome, black = 0 (`AV_PIX_FMT_MONOBLACK`).
  ///
  /// Renders `"monoblack"` — the header identifier. FFmpeg's own
  /// descriptor name is `"monob"`, accepted as a parse synonym.
  Monoblack,

  // ===================================================================
  // Paletted
  // ===================================================================
  /// Paletted 8-bit (`AV_PIX_FMT_PAL8`).
  Pal8,

  // ===================================================================
  // Bayer
  // ===================================================================
  /// Bayer BGGR pattern, 8-bit.
  BayerBggr8,
  /// Bayer RGGB pattern, 8-bit.
  BayerRggb8,
  /// Bayer GBRG pattern, 8-bit.
  BayerGbrg8,
  /// Bayer GRBG pattern, 8-bit.
  BayerGrbg8,
  /// Bayer BGGR pattern, 10-bit little-endian (low-packed in u16).
  BayerBggr10Le,
  /// Bayer BGGR pattern, 10-bit big-endian (low-packed in u16).
  BayerBggr10Be,
  /// Bayer RGGB pattern, 10-bit little-endian.
  BayerRggb10Le,
  /// Bayer RGGB pattern, 10-bit big-endian.
  BayerRggb10Be,
  /// Bayer GBRG pattern, 10-bit little-endian.
  BayerGbrg10Le,
  /// Bayer GBRG pattern, 10-bit big-endian.
  BayerGbrg10Be,
  /// Bayer GRBG pattern, 10-bit little-endian.
  BayerGrbg10Le,
  /// Bayer GRBG pattern, 10-bit big-endian.
  BayerGrbg10Be,
  /// Bayer BGGR pattern, 12-bit little-endian.
  BayerBggr12Le,
  /// Bayer BGGR pattern, 12-bit big-endian.
  BayerBggr12Be,
  /// Bayer RGGB pattern, 12-bit little-endian.
  BayerRggb12Le,
  /// Bayer RGGB pattern, 12-bit big-endian.
  BayerRggb12Be,
  /// Bayer GBRG pattern, 12-bit little-endian.
  BayerGbrg12Le,
  /// Bayer GBRG pattern, 12-bit big-endian.
  BayerGbrg12Be,
  /// Bayer GRBG pattern, 12-bit little-endian.
  BayerGrbg12Le,
  /// Bayer GRBG pattern, 12-bit big-endian.
  BayerGrbg12Be,
  /// Bayer BGGR pattern, 14-bit little-endian.
  BayerBggr14Le,
  /// Bayer BGGR pattern, 14-bit big-endian.
  BayerBggr14Be,
  /// Bayer RGGB pattern, 14-bit little-endian.
  BayerRggb14Le,
  /// Bayer RGGB pattern, 14-bit big-endian.
  BayerRggb14Be,
  /// Bayer GBRG pattern, 14-bit little-endian.
  BayerGbrg14Le,
  /// Bayer GBRG pattern, 14-bit big-endian.
  BayerGbrg14Be,
  /// Bayer GRBG pattern, 14-bit little-endian.
  BayerGrbg14Le,
  /// Bayer GRBG pattern, 14-bit big-endian.
  BayerGrbg14Be,
  /// Bayer BGGR pattern, 16-bit little-endian.
  BayerBggr16Le,
  /// Bayer BGGR pattern, 16-bit big-endian.
  BayerBggr16Be,
  /// Bayer RGGB pattern, 16-bit little-endian.
  BayerRggb16Le,
  /// Bayer RGGB pattern, 16-bit big-endian.
  BayerRggb16Be,
  /// Bayer GBRG pattern, 16-bit little-endian.
  BayerGbrg16Le,
  /// Bayer GBRG pattern, 16-bit big-endian.
  BayerGbrg16Be,
  /// Bayer GRBG pattern, 16-bit little-endian.
  BayerGrbg16Le,
  /// Bayer GRBG pattern, 16-bit big-endian.
  BayerGrbg16Be,
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for PixelFormat {
  /// [`Self::None`] — FFmpeg's `AV_PIX_FMT_NONE`, "no format yet".
  #[inline]
  fn default() -> Self {
    Self::None
  }
}

impl PixelFormat {
  /// Stable wire representation. Known variants return their
  /// assigned id.
  ///
  /// [`None`] for [`Self::Other`]: it names a format this build does not
  /// enumerate, and there is no id to invent for it.
  #[inline]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::None => 0,
      Self::Yuv420p => 100,
      Self::Yuv422p => 101,
      Self::Yuv440p => 102,
      Self::Yuv444p => 103,
      Self::Yuv411p => 104,
      Self::Yuv410p => 105,
      Self::Yuvj411p => 106,
      Self::Yuvj420p => 107,
      Self::Yuvj422p => 108,
      Self::Yuvj440p => 109,
      Self::Yuvj444p => 110,
      Self::Yuv420p9Le => 111,
      Self::Yuv420p9Be => 112,
      Self::Yuv420p10Le => 113,
      Self::Yuv420p10Be => 114,
      Self::Yuv420p12Le => 115,
      Self::Yuv420p12Be => 116,
      Self::Yuv420p14Le => 117,
      Self::Yuv420p14Be => 118,
      Self::Yuv420p16Le => 119,
      Self::Yuv420p16Be => 120,
      Self::Yuv422p9Le => 121,
      Self::Yuv422p9Be => 122,
      Self::Yuv422p10Le => 123,
      Self::Yuv422p10Be => 124,
      Self::Yuv422p12Le => 125,
      Self::Yuv422p12Be => 126,
      Self::Yuv422p14Le => 127,
      Self::Yuv422p14Be => 128,
      Self::Yuv422p16Le => 129,
      Self::Yuv422p16Be => 130,
      Self::Yuv440p10Le => 131,
      Self::Yuv440p10Be => 132,
      Self::Yuv440p12Le => 133,
      Self::Yuv440p12Be => 134,
      Self::Yuv444p9Le => 140,
      Self::Yuv444p9Be => 141,
      Self::Yuv444p10Le => 142,
      Self::Yuv444p10Be => 143,
      Self::Yuv444p12Le => 144,
      Self::Yuv444p12Be => 145,
      Self::Yuv444p14Le => 146,
      Self::Yuv444p14Be => 147,
      Self::Yuv444p16Le => 148,
      Self::Yuv444p16Be => 149,
      Self::Yuv444p10MsbLe => 150,
      Self::Yuv444p10MsbBe => 151,
      Self::Yuv444p12MsbLe => 152,
      Self::Yuv444p12MsbBe => 153,
      Self::Yuva420p => 200,
      Self::Yuva422p => 201,
      Self::Yuva444p => 202,
      Self::Yuva420p9Le => 203,
      Self::Yuva420p9Be => 216,
      Self::Yuva422p9Le => 204,
      Self::Yuva422p9Be => 217,
      Self::Yuva444p9Le => 205,
      Self::Yuva444p9Be => 218,
      Self::Yuva420p10Le => 206,
      Self::Yuva420p10Be => 219,
      Self::Yuva422p10Le => 207,
      Self::Yuva422p10Be => 220,
      Self::Yuva444p10Le => 208,
      Self::Yuva444p10Be => 221,
      Self::Yuva420p12Le => 215,
      Self::Yuva420p12Be => 227,
      Self::Yuva422p12Le => 209,
      Self::Yuva422p12Be => 222,
      Self::Yuva444p12Le => 210,
      Self::Yuva444p12Be => 223,
      Self::Yuva444p14Le => 211,
      Self::Yuva420p16Le => 212,
      Self::Yuva420p16Be => 224,
      Self::Yuva422p16Le => 213,
      Self::Yuva422p16Be => 225,
      Self::Yuva444p16Le => 214,
      Self::Yuva444p16Be => 226,
      Self::Nv12 => 300,
      Self::Nv21 => 301,
      Self::Nv16 => 302,
      Self::Nv24 => 303,
      Self::Nv42 => 304,
      Self::Nv20Le => 305,
      Self::Nv20Be => 306,
      Self::P010Le => 310,
      Self::P010Be => 311,
      Self::P012Le => 312,
      Self::P012Be => 320,
      Self::P016Le => 313,
      Self::P016Be => 321,
      Self::P210Le => 314,
      Self::P210Be => 322,
      Self::P212Le => 315,
      Self::P212Be => 323,
      Self::P216Le => 316,
      Self::P216Be => 324,
      Self::P410Le => 317,
      Self::P410Be => 325,
      Self::P412Le => 318,
      Self::P412Be => 326,
      Self::P416Le => 319,
      Self::P416Be => 327,
      Self::Yuyv422 => 400,
      Self::Uyvy422 => 401,
      Self::Yvyu422 => 402,
      Self::Uyyvyy411 => 403,
      Self::Y210Le => 410,
      Self::Y210Be => 420,
      Self::Y212Le => 411,
      Self::Y212Be => 421,
      Self::Y216Le => 412,
      Self::Y216Be => 422,
      Self::V210 => 413,
      Self::V410Le => 414,
      Self::V410Be => 435,
      Self::Xv30Le => 415,
      Self::Xv30Be => 423,
      Self::V30xLe => 433,
      Self::V30xBe => 434,
      Self::Xv36Le => 416,
      Self::Xv36Be => 424,
      Self::Xv48Le => 425,
      Self::Xv48Be => 426,
      Self::Vuya => 417,
      Self::Vuyx => 418,
      Self::Ayuv => 427,
      Self::Ayuv64Le => 419,
      Self::Ayuv64Be => 428,
      Self::Uyva => 429,
      Self::Vyu444 => 430,
      Self::Xyz12Le => 431,
      Self::Xyz12Be => 432,
      Self::Rgb24 => 500,
      Self::Bgr24 => 501,
      Self::Rgba => 502,
      Self::Bgra => 503,
      Self::Argb => 504,
      Self::Abgr => 505,
      Self::Rgbx => 506,
      Self::Bgrx => 507,
      Self::Xrgb => 508,
      Self::Xbgr => 509,
      Self::X2Rgb10Le => 510,
      Self::X2Rgb10Be => 512,
      Self::X2Bgr10Le => 511,
      Self::X2Bgr10Be => 513,
      Self::Gbr24p => 514,
      Self::Rgb4 => 515,
      Self::Rgb4Byte => 516,
      Self::Rgb8 => 517,
      Self::Bgr4 => 518,
      Self::Bgr4Byte => 519,
      Self::Bgr8 => 560,
      Self::Rgb444Le => 520,
      Self::Rgb444Be => 561,
      Self::Bgr444Le => 521,
      Self::Bgr444Be => 562,
      Self::Rgb555Le => 522,
      Self::Rgb555Be => 563,
      Self::Bgr555Le => 523,
      Self::Bgr555Be => 564,
      Self::Rgb565Le => 524,
      Self::Rgb565Be => 565,
      Self::Bgr565Le => 525,
      Self::Bgr565Be => 566,
      Self::Rgb48Le => 530,
      Self::Rgb48Be => 567,
      Self::Bgr48Le => 531,
      Self::Bgr48Be => 568,
      Self::Rgba64Le => 532,
      Self::Rgba64Be => 569,
      Self::Bgra64Le => 533,
      Self::Bgra64Be => 570,
      Self::Rgb96Le => 571,
      Self::Rgb96Be => 572,
      Self::Rgba128Le => 573,
      Self::Rgba128Be => 574,
      Self::Rgbf16Le => 540,
      Self::Rgbf16Be => 541,
      Self::Rgbf32Le => 542,
      Self::Rgbf32Be => 543,
      Self::Rgbaf16Le => 544,
      Self::Rgbaf16Be => 545,
      Self::Rgbaf32Le => 546,
      Self::Rgbaf32Be => 547,
      Self::Gbrp => 600,
      Self::Gbrp9Le => 601,
      Self::Gbrp9Be => 608,
      Self::Gbrp10Le => 602,
      Self::Gbrp10Be => 609,
      Self::Gbrp10MsbLe => 630,
      Self::Gbrp10MsbBe => 631,
      Self::Gbrp12Le => 603,
      Self::Gbrp12Be => 610,
      Self::Gbrp12MsbLe => 632,
      Self::Gbrp12MsbBe => 633,
      Self::Gbrp14Le => 604,
      Self::Gbrp14Be => 611,
      Self::Gbrp16Le => 605,
      Self::Gbrp16Be => 612,
      Self::Gbrpf16Le => 606,
      Self::Gbrpf16Be => 613,
      Self::Gbrpf32Le => 607,
      Self::Gbrpf32Be => 614,
      Self::Gbrap => 620,
      Self::Gbrap10Le => 621,
      Self::Gbrap10Be => 634,
      Self::Gbrap12Le => 622,
      Self::Gbrap12Be => 635,
      Self::Gbrap14Le => 623,
      Self::Gbrap14Be => 636,
      Self::Gbrap16Le => 624,
      Self::Gbrap16Be => 637,
      Self::Gbrap32Le => 638,
      Self::Gbrap32Be => 639,
      Self::Gbrapf16Le => 625,
      Self::Gbrapf16Be => 640,
      Self::Gbrapf32Le => 626,
      Self::Gbrapf32Be => 641,
      Self::Gray8 => 700,
      Self::Gray8a => 701,
      Self::Gray9Le => 702,
      Self::Gray9Be => 712,
      Self::Gray10Le => 703,
      Self::Gray10Be => 713,
      Self::Gray12Le => 704,
      Self::Gray12Be => 714,
      Self::Gray14Le => 705,
      Self::Gray14Be => 715,
      Self::Gray16Le => 706,
      Self::Gray16Be => 716,
      Self::Gray32Le => 717,
      Self::Gray32Be => 718,
      Self::Grayf32Le => 707,
      Self::Grayf32Be => 719,
      Self::Grayf16Le => 720,
      Self::Grayf16Be => 721,
      Self::Ya8 => 730,
      Self::Y400a => 731,
      Self::Ya16Le => 732,
      Self::Ya16Be => 733,
      Self::Yaf16Le => 734,
      Self::Yaf16Be => 735,
      Self::Yaf32Le => 736,
      Self::Yaf32Be => 737,
      Self::Monowhite => 740,
      Self::Monoblack => 741,
      Self::Pal8 => 800,
      Self::BayerBggr8 => 900,
      Self::BayerRggb8 => 901,
      Self::BayerGbrg8 => 902,
      Self::BayerGrbg8 => 903,
      Self::BayerBggr10Le => 910,
      Self::BayerRggb10Le => 911,
      Self::BayerGbrg10Le => 912,
      Self::BayerGrbg10Le => 913,
      Self::BayerBggr10Be => 914,
      Self::BayerRggb10Be => 915,
      Self::BayerGbrg10Be => 916,
      Self::BayerGrbg10Be => 917,
      Self::BayerBggr12Le => 920,
      Self::BayerRggb12Le => 921,
      Self::BayerGbrg12Le => 922,
      Self::BayerGrbg12Le => 923,
      Self::BayerBggr12Be => 924,
      Self::BayerRggb12Be => 925,
      Self::BayerGbrg12Be => 926,
      Self::BayerGrbg12Be => 927,
      Self::BayerBggr14Le => 930,
      Self::BayerRggb14Le => 931,
      Self::BayerGbrg14Le => 932,
      Self::BayerGrbg14Le => 933,
      Self::BayerBggr14Be => 934,
      Self::BayerRggb14Be => 935,
      Self::BayerGbrg14Be => 936,
      Self::BayerGrbg14Be => 937,
      Self::BayerBggr16Le => 940,
      Self::BayerBggr16Be => 944,
      Self::BayerRggb16Le => 941,
      Self::BayerRggb16Be => 945,
      Self::BayerGbrg16Le => 942,
      Self::BayerGbrg16Be => 946,
      Self::BayerGrbg16Le => 943,
      Self::BayerGrbg16Be => 947,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the stable `u32` representation produced by
  /// [`Self::to_u32`]. [`None`] for an id this build names nothing for.
  #[inline]
  pub const fn from_u32(value: u32) -> Option<Self> {
    Some(match value {
      0 => Self::None,
      // Planar YUV 8-bit.
      100 => Self::Yuv420p,
      101 => Self::Yuv422p,
      102 => Self::Yuv440p,
      103 => Self::Yuv444p,
      104 => Self::Yuv411p,
      105 => Self::Yuv410p,
      106 => Self::Yuvj411p,
      107 => Self::Yuvj420p,
      108 => Self::Yuvj422p,
      109 => Self::Yuvj440p,
      110 => Self::Yuvj444p,
      // Planar YUV high-bit-depth (4:2:0).
      111 => Self::Yuv420p9Le,
      112 => Self::Yuv420p9Be,
      113 => Self::Yuv420p10Le,
      114 => Self::Yuv420p10Be,
      115 => Self::Yuv420p12Le,
      116 => Self::Yuv420p12Be,
      117 => Self::Yuv420p14Le,
      118 => Self::Yuv420p14Be,
      119 => Self::Yuv420p16Le,
      120 => Self::Yuv420p16Be,
      // Planar YUV high-bit-depth (4:2:2).
      121 => Self::Yuv422p9Le,
      122 => Self::Yuv422p9Be,
      123 => Self::Yuv422p10Le,
      124 => Self::Yuv422p10Be,
      125 => Self::Yuv422p12Le,
      126 => Self::Yuv422p12Be,
      127 => Self::Yuv422p14Le,
      128 => Self::Yuv422p14Be,
      129 => Self::Yuv422p16Le,
      130 => Self::Yuv422p16Be,
      // Planar YUV (4:4:0).
      131 => Self::Yuv440p10Le,
      132 => Self::Yuv440p10Be,
      133 => Self::Yuv440p12Le,
      134 => Self::Yuv440p12Be,
      // Planar YUV high-bit-depth (4:4:4).
      140 => Self::Yuv444p9Le,
      141 => Self::Yuv444p9Be,
      142 => Self::Yuv444p10Le,
      143 => Self::Yuv444p10Be,
      144 => Self::Yuv444p12Le,
      145 => Self::Yuv444p12Be,
      146 => Self::Yuv444p14Le,
      147 => Self::Yuv444p14Be,
      148 => Self::Yuv444p16Le,
      149 => Self::Yuv444p16Be,
      150 => Self::Yuv444p10MsbLe,
      151 => Self::Yuv444p10MsbBe,
      152 => Self::Yuv444p12MsbLe,
      153 => Self::Yuv444p12MsbBe,
      // Planar YUVA.
      200 => Self::Yuva420p,
      201 => Self::Yuva422p,
      202 => Self::Yuva444p,
      203 => Self::Yuva420p9Le,
      204 => Self::Yuva422p9Le,
      205 => Self::Yuva444p9Le,
      206 => Self::Yuva420p10Le,
      207 => Self::Yuva422p10Le,
      208 => Self::Yuva444p10Le,
      209 => Self::Yuva422p12Le,
      210 => Self::Yuva444p12Le,
      211 => Self::Yuva444p14Le,
      212 => Self::Yuva420p16Le,
      213 => Self::Yuva422p16Le,
      214 => Self::Yuva444p16Le,
      215 => Self::Yuva420p12Le,
      216 => Self::Yuva420p9Be,
      217 => Self::Yuva422p9Be,
      218 => Self::Yuva444p9Be,
      219 => Self::Yuva420p10Be,
      220 => Self::Yuva422p10Be,
      221 => Self::Yuva444p10Be,
      222 => Self::Yuva422p12Be,
      223 => Self::Yuva444p12Be,
      224 => Self::Yuva420p16Be,
      225 => Self::Yuva422p16Be,
      226 => Self::Yuva444p16Be,
      227 => Self::Yuva420p12Be,
      // Semi-planar YUV.
      300 => Self::Nv12,
      301 => Self::Nv21,
      302 => Self::Nv16,
      303 => Self::Nv24,
      304 => Self::Nv42,
      305 => Self::Nv20Le,
      306 => Self::Nv20Be,
      // Semi-planar YUV high-bit-depth.
      310 => Self::P010Le,
      311 => Self::P010Be,
      312 => Self::P012Le,
      313 => Self::P016Le,
      314 => Self::P210Le,
      315 => Self::P212Le,
      316 => Self::P216Le,
      317 => Self::P410Le,
      318 => Self::P412Le,
      319 => Self::P416Le,
      320 => Self::P012Be,
      321 => Self::P016Be,
      322 => Self::P210Be,
      323 => Self::P212Be,
      324 => Self::P216Be,
      325 => Self::P410Be,
      326 => Self::P412Be,
      327 => Self::P416Be,
      // Packed YUV 8-bit.
      400 => Self::Yuyv422,
      401 => Self::Uyvy422,
      402 => Self::Yvyu422,
      403 => Self::Uyyvyy411,
      // Packed YUV high-bit-depth.
      410 => Self::Y210Le,
      411 => Self::Y212Le,
      412 => Self::Y216Le,
      413 => Self::V210,
      414 => Self::V410Le,
      415 => Self::Xv30Le,
      416 => Self::Xv36Le,
      417 => Self::Vuya,
      418 => Self::Vuyx,
      419 => Self::Ayuv64Le,
      420 => Self::Y210Be,
      421 => Self::Y212Be,
      422 => Self::Y216Be,
      423 => Self::Xv30Be,
      433 => Self::V30xLe,
      434 => Self::V30xBe,
      424 => Self::Xv36Be,
      425 => Self::Xv48Le,
      426 => Self::Xv48Be,
      427 => Self::Ayuv,
      428 => Self::Ayuv64Be,
      429 => Self::Uyva,
      430 => Self::Vyu444,
      431 => Self::Xyz12Le,
      432 => Self::Xyz12Be,
      435 => Self::V410Be,
      // Packed RGB 8-bit.
      500 => Self::Rgb24,
      501 => Self::Bgr24,
      502 => Self::Rgba,
      503 => Self::Bgra,
      504 => Self::Argb,
      505 => Self::Abgr,
      506 => Self::Rgbx,
      507 => Self::Bgrx,
      508 => Self::Xrgb,
      509 => Self::Xbgr,
      510 => Self::X2Rgb10Le,
      511 => Self::X2Bgr10Le,
      512 => Self::X2Rgb10Be,
      513 => Self::X2Bgr10Be,
      514 => Self::Gbr24p,
      515 => Self::Rgb4,
      516 => Self::Rgb4Byte,
      517 => Self::Rgb8,
      518 => Self::Bgr4,
      519 => Self::Bgr4Byte,
      560 => Self::Bgr8,
      // Packed RGB low-bit.
      520 => Self::Rgb444Le,
      521 => Self::Bgr444Le,
      522 => Self::Rgb555Le,
      523 => Self::Bgr555Le,
      524 => Self::Rgb565Le,
      525 => Self::Bgr565Le,
      561 => Self::Rgb444Be,
      562 => Self::Bgr444Be,
      563 => Self::Rgb555Be,
      564 => Self::Bgr555Be,
      565 => Self::Rgb565Be,
      566 => Self::Bgr565Be,
      // Packed RGB high-bit.
      530 => Self::Rgb48Le,
      531 => Self::Bgr48Le,
      532 => Self::Rgba64Le,
      533 => Self::Bgra64Le,
      567 => Self::Rgb48Be,
      568 => Self::Bgr48Be,
      569 => Self::Rgba64Be,
      570 => Self::Bgra64Be,
      571 => Self::Rgb96Le,
      572 => Self::Rgb96Be,
      573 => Self::Rgba128Le,
      574 => Self::Rgba128Be,
      // Packed RGB float.
      540 => Self::Rgbf16Le,
      541 => Self::Rgbf16Be,
      542 => Self::Rgbf32Le,
      543 => Self::Rgbf32Be,
      544 => Self::Rgbaf16Le,
      545 => Self::Rgbaf16Be,
      546 => Self::Rgbaf32Le,
      547 => Self::Rgbaf32Be,
      // Planar GBR.
      600 => Self::Gbrp,
      601 => Self::Gbrp9Le,
      602 => Self::Gbrp10Le,
      603 => Self::Gbrp12Le,
      604 => Self::Gbrp14Le,
      605 => Self::Gbrp16Le,
      606 => Self::Gbrpf16Le,
      607 => Self::Gbrpf32Le,
      608 => Self::Gbrp9Be,
      609 => Self::Gbrp10Be,
      610 => Self::Gbrp12Be,
      611 => Self::Gbrp14Be,
      612 => Self::Gbrp16Be,
      613 => Self::Gbrpf16Be,
      614 => Self::Gbrpf32Be,
      630 => Self::Gbrp10MsbLe,
      631 => Self::Gbrp10MsbBe,
      632 => Self::Gbrp12MsbLe,
      633 => Self::Gbrp12MsbBe,
      // Planar GBRA.
      620 => Self::Gbrap,
      621 => Self::Gbrap10Le,
      622 => Self::Gbrap12Le,
      623 => Self::Gbrap14Le,
      624 => Self::Gbrap16Le,
      625 => Self::Gbrapf16Le,
      626 => Self::Gbrapf32Le,
      634 => Self::Gbrap10Be,
      635 => Self::Gbrap12Be,
      636 => Self::Gbrap14Be,
      637 => Self::Gbrap16Be,
      638 => Self::Gbrap32Le,
      639 => Self::Gbrap32Be,
      640 => Self::Gbrapf16Be,
      641 => Self::Gbrapf32Be,
      // Greyscale.
      700 => Self::Gray8,
      701 => Self::Gray8a,
      702 => Self::Gray9Le,
      703 => Self::Gray10Le,
      704 => Self::Gray12Le,
      705 => Self::Gray14Le,
      706 => Self::Gray16Le,
      707 => Self::Grayf32Le,
      712 => Self::Gray9Be,
      713 => Self::Gray10Be,
      714 => Self::Gray12Be,
      715 => Self::Gray14Be,
      716 => Self::Gray16Be,
      717 => Self::Gray32Le,
      718 => Self::Gray32Be,
      719 => Self::Grayf32Be,
      720 => Self::Grayf16Le,
      721 => Self::Grayf16Be,
      730 => Self::Ya8,
      731 => Self::Y400a,
      732 => Self::Ya16Le,
      733 => Self::Ya16Be,
      734 => Self::Yaf16Le,
      735 => Self::Yaf16Be,
      736 => Self::Yaf32Le,
      737 => Self::Yaf32Be,
      // Monochrome.
      740 => Self::Monowhite,
      741 => Self::Monoblack,
      // Paletted.
      800 => Self::Pal8,
      // Bayer.
      900 => Self::BayerBggr8,
      901 => Self::BayerRggb8,
      902 => Self::BayerGbrg8,
      903 => Self::BayerGrbg8,
      910 => Self::BayerBggr10Le,
      911 => Self::BayerRggb10Le,
      912 => Self::BayerGbrg10Le,
      913 => Self::BayerGrbg10Le,
      914 => Self::BayerBggr10Be,
      915 => Self::BayerRggb10Be,
      916 => Self::BayerGbrg10Be,
      917 => Self::BayerGrbg10Be,
      920 => Self::BayerBggr12Le,
      921 => Self::BayerRggb12Le,
      922 => Self::BayerGbrg12Le,
      923 => Self::BayerGrbg12Le,
      924 => Self::BayerBggr12Be,
      925 => Self::BayerRggb12Be,
      926 => Self::BayerGbrg12Be,
      927 => Self::BayerGrbg12Be,
      930 => Self::BayerBggr14Le,
      931 => Self::BayerRggb14Le,
      932 => Self::BayerGbrg14Le,
      933 => Self::BayerGrbg14Le,
      934 => Self::BayerBggr14Be,
      935 => Self::BayerRggb14Be,
      936 => Self::BayerGbrg14Be,
      937 => Self::BayerGrbg14Be,
      940 => Self::BayerBggr16Le,
      941 => Self::BayerRggb16Le,
      942 => Self::BayerGbrg16Le,
      943 => Self::BayerGrbg16Le,
      944 => Self::BayerBggr16Be,
      945 => Self::BayerRggb16Be,
      946 => Self::BayerGbrg16Be,
      947 => Self::BayerGrbg16Be,
      _ => return None,
    })
  }

  /// Returns `true` for Bayer-mosaic formats (any pattern, any bit
  /// depth). Bayer frames carry undebayered sensor data; downstream
  /// consumers (e.g. `colconv::raw`) demosaic + white-balance + colour-
  /// correct to produce RGB.
  #[inline]
  pub fn is_bayer(&self) -> bool {
    matches!(
      self,
      Self::BayerBggr8
        | Self::BayerRggb8
        | Self::BayerGbrg8
        | Self::BayerGrbg8
        | Self::BayerBggr10Le
        | Self::BayerRggb10Le
        | Self::BayerGbrg10Le
        | Self::BayerGrbg10Le
        | Self::BayerBggr10Be
        | Self::BayerRggb10Be
        | Self::BayerGbrg10Be
        | Self::BayerGrbg10Be
        | Self::BayerBggr12Le
        | Self::BayerRggb12Le
        | Self::BayerGbrg12Le
        | Self::BayerGrbg12Le
        | Self::BayerBggr12Be
        | Self::BayerRggb12Be
        | Self::BayerGbrg12Be
        | Self::BayerGrbg12Be
        | Self::BayerBggr14Le
        | Self::BayerRggb14Le
        | Self::BayerGbrg14Le
        | Self::BayerGrbg14Le
        | Self::BayerBggr14Be
        | Self::BayerRggb14Be
        | Self::BayerGbrg14Be
        | Self::BayerGrbg14Be
        | Self::BayerBggr16Le
        | Self::BayerBggr16Be
        | Self::BayerRggb16Le
        | Self::BayerRggb16Be
        | Self::BayerGbrg16Le
        | Self::BayerGbrg16Be
        | Self::BayerGrbg16Le
        | Self::BayerGrbg16Be,
    )
  }

  /// Resolves a deprecated / aliased pixel format to its **canonical
  /// decode format** plus the [`DynamicRange`](crate::color::DynamicRange)
  /// the alias *pins*, if any.
  ///
  /// mediaframe keeps FFmpeg's deprecated alias formats as distinct
  /// variants so the wire round-trip ([`from_u32`](Self::from_u32) /
  /// [`to_u32`](Self::to_u32) / [`as_str`](Self::as_str)) stays lossless.
  /// Decoders and converters, however, want a *single* representative
  /// format per pixel layout. `canonical()` is the format authority's
  /// mapping from an alias to the non-deprecated format describing the
  /// same bytes, so downstream crates (e.g. `colconv`) consume one table
  /// here instead of each re-deriving the alias set.
  ///
  /// The second element is `Some(_)` only when the alias *carries* range
  /// information the bare format does not — the `yuvj*` aliases, which
  /// decode as their `yuv*p` base with the range pinned to
  /// [`DynamicRange::Full`](crate::color::DynamicRange::Full). For every
  /// other format the dynamic range is stream-driven (carried in
  /// [`color::Info`](crate::color::Info), not implied by the format
  /// identity), so the second element is `None`.
  ///
  /// # Mappings
  ///
  /// - `Yuvj{411,420,422,440,444}p` → the matching `Yuv*p` +
  ///   `Some(DynamicRange::Full)` — FFmpeg's deprecated full-range
  ///   ("JPEG") YUV aliases decode as the base planar format with the
  ///   range pinned full.
  /// - [`Gray8a`](Self::Gray8a) / [`Y400a`](Self::Y400a) →
  ///   [`Ya8`](Self::Ya8) — both are FFmpeg aliases (`AV_PIX_FMT_GRAY8A`
  ///   / `AV_PIX_FMT_Y400A`) of the same 8-bit grey-plus-alpha layout.
  /// - [`Xv30Le`](Self::Xv30Le) → [`V410Le`](Self::V410Le) and
  ///   [`Xv30Be`](Self::Xv30Be) → [`V410Be`](Self::V410Be) — `XV30` is
  ///   the modern FFmpeg name for the identical-bit-pattern `V410` 4:4:4
  ///   10-bit packed layout (the `AV_PIX_FMT_V410` symbol was renamed to
  ///   `XV30`). Both endians resolve onto their matching `V410` variant,
  ///   preserving byte order: the [`V410Frame<'a, BE>`](crate::frame::V410Frame)
  ///   borrow view and the `v410_to::<BE>` walker decode either endian.
  /// - Every other variant — including [`Other`](Self::Other) — is
  ///   already canonical and maps to `(self, None)`.
  ///
  /// The match is intentionally **exhaustive without a wildcard**:
  /// `#[non_exhaustive]` does not force a catch-all arm *inside* the
  /// defining crate, so every future variant must be classified here
  /// explicitly. The compiler then flags any newly added format that is
  /// not yet routed, ensuring a new alias can never silently fall through
  /// to the "already canonical" arm.
  #[inline]
  pub fn canonical(self) -> (PixelFormat, Option<crate::color::DynamicRange>) {
    use crate::color::DynamicRange;
    match self {
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => (self, None),
      Self::Yuvj411p => (Self::Yuv411p, Some(DynamicRange::Full)),
      Self::Yuvj420p => (Self::Yuv420p, Some(DynamicRange::Full)),
      Self::Yuvj422p => (Self::Yuv422p, Some(DynamicRange::Full)),
      Self::Yuvj440p => (Self::Yuv440p, Some(DynamicRange::Full)),
      Self::Yuvj444p => (Self::Yuv444p, Some(DynamicRange::Full)),
      Self::Gray8a | Self::Y400a => (Self::Ya8, None),
      Self::Xv30Le => (Self::V410Le, None),
      Self::Xv30Be => (Self::V410Be, None),
      Self::None
      | Self::Yuv420p
      | Self::Yuv422p
      | Self::Yuv440p
      | Self::Yuv444p
      | Self::Yuv411p
      | Self::Yuv410p
      | Self::Yuv420p9Le
      | Self::Yuv420p9Be
      | Self::Yuv420p10Le
      | Self::Yuv420p10Be
      | Self::Yuv420p12Le
      | Self::Yuv420p12Be
      | Self::Yuv420p14Le
      | Self::Yuv420p14Be
      | Self::Yuv420p16Le
      | Self::Yuv420p16Be
      | Self::Yuv422p9Le
      | Self::Yuv422p9Be
      | Self::Yuv422p10Le
      | Self::Yuv422p10Be
      | Self::Yuv422p12Le
      | Self::Yuv422p12Be
      | Self::Yuv422p14Le
      | Self::Yuv422p14Be
      | Self::Yuv422p16Le
      | Self::Yuv422p16Be
      | Self::Yuv440p10Le
      | Self::Yuv440p10Be
      | Self::Yuv440p12Le
      | Self::Yuv440p12Be
      | Self::Yuv444p9Le
      | Self::Yuv444p9Be
      | Self::Yuv444p10Le
      | Self::Yuv444p10Be
      | Self::Yuv444p12Le
      | Self::Yuv444p12Be
      | Self::Yuv444p14Le
      | Self::Yuv444p14Be
      | Self::Yuv444p16Le
      | Self::Yuv444p16Be
      | Self::Yuv444p10MsbLe
      | Self::Yuv444p10MsbBe
      | Self::Yuv444p12MsbLe
      | Self::Yuv444p12MsbBe
      | Self::Yuva420p
      | Self::Yuva422p
      | Self::Yuva444p
      | Self::Yuva420p9Le
      | Self::Yuva420p9Be
      | Self::Yuva422p9Le
      | Self::Yuva422p9Be
      | Self::Yuva444p9Le
      | Self::Yuva444p9Be
      | Self::Yuva420p10Le
      | Self::Yuva420p10Be
      | Self::Yuva422p10Le
      | Self::Yuva422p10Be
      | Self::Yuva444p10Le
      | Self::Yuva444p10Be
      | Self::Yuva420p12Le
      | Self::Yuva420p12Be
      | Self::Yuva422p12Le
      | Self::Yuva422p12Be
      | Self::Yuva444p12Le
      | Self::Yuva444p12Be
      | Self::Yuva444p14Le
      | Self::Yuva420p16Le
      | Self::Yuva420p16Be
      | Self::Yuva422p16Le
      | Self::Yuva422p16Be
      | Self::Yuva444p16Le
      | Self::Yuva444p16Be
      | Self::Nv12
      | Self::Nv21
      | Self::Nv16
      | Self::Nv24
      | Self::Nv42
      | Self::Nv20Le
      | Self::Nv20Be
      | Self::P010Le
      | Self::P010Be
      | Self::P012Le
      | Self::P012Be
      | Self::P016Le
      | Self::P016Be
      | Self::P210Le
      | Self::P210Be
      | Self::P212Le
      | Self::P212Be
      | Self::P216Le
      | Self::P216Be
      | Self::P410Le
      | Self::P410Be
      | Self::P412Le
      | Self::P412Be
      | Self::P416Le
      | Self::P416Be
      | Self::Yuyv422
      | Self::Uyvy422
      | Self::Yvyu422
      | Self::Uyyvyy411
      | Self::Y210Le
      | Self::Y210Be
      | Self::Y212Le
      | Self::Y212Be
      | Self::Y216Le
      | Self::Y216Be
      | Self::V210
      | Self::V410Le
      | Self::V410Be
      | Self::V30xLe
      | Self::V30xBe
      | Self::Xv36Le
      | Self::Xv36Be
      | Self::Xv48Le
      | Self::Xv48Be
      | Self::Vuya
      | Self::Vuyx
      | Self::Ayuv
      | Self::Ayuv64Le
      | Self::Ayuv64Be
      | Self::Uyva
      | Self::Vyu444
      | Self::Xyz12Le
      | Self::Xyz12Be
      | Self::Rgb24
      | Self::Bgr24
      | Self::Rgba
      | Self::Bgra
      | Self::Argb
      | Self::Abgr
      | Self::Rgbx
      | Self::Bgrx
      | Self::Xrgb
      | Self::Xbgr
      | Self::X2Rgb10Le
      | Self::X2Rgb10Be
      | Self::X2Bgr10Le
      | Self::X2Bgr10Be
      | Self::Gbr24p
      | Self::Rgb4
      | Self::Rgb4Byte
      | Self::Rgb8
      | Self::Bgr4
      | Self::Bgr4Byte
      | Self::Bgr8
      | Self::Rgb444Le
      | Self::Rgb444Be
      | Self::Bgr444Le
      | Self::Bgr444Be
      | Self::Rgb555Le
      | Self::Rgb555Be
      | Self::Bgr555Le
      | Self::Bgr555Be
      | Self::Rgb565Le
      | Self::Rgb565Be
      | Self::Bgr565Le
      | Self::Bgr565Be
      | Self::Rgb48Le
      | Self::Rgb48Be
      | Self::Bgr48Le
      | Self::Bgr48Be
      | Self::Rgba64Le
      | Self::Rgba64Be
      | Self::Bgra64Le
      | Self::Bgra64Be
      | Self::Rgb96Le
      | Self::Rgb96Be
      | Self::Rgba128Le
      | Self::Rgba128Be
      | Self::Rgbf16Le
      | Self::Rgbf16Be
      | Self::Rgbf32Le
      | Self::Rgbf32Be
      | Self::Rgbaf16Le
      | Self::Rgbaf16Be
      | Self::Rgbaf32Le
      | Self::Rgbaf32Be
      | Self::Gbrp
      | Self::Gbrp9Le
      | Self::Gbrp9Be
      | Self::Gbrp10Le
      | Self::Gbrp10Be
      | Self::Gbrp10MsbLe
      | Self::Gbrp10MsbBe
      | Self::Gbrp12Le
      | Self::Gbrp12Be
      | Self::Gbrp12MsbLe
      | Self::Gbrp12MsbBe
      | Self::Gbrp14Le
      | Self::Gbrp14Be
      | Self::Gbrp16Le
      | Self::Gbrp16Be
      | Self::Gbrpf16Le
      | Self::Gbrpf16Be
      | Self::Gbrpf32Le
      | Self::Gbrpf32Be
      | Self::Gbrap
      | Self::Gbrap10Le
      | Self::Gbrap10Be
      | Self::Gbrap12Le
      | Self::Gbrap12Be
      | Self::Gbrap14Le
      | Self::Gbrap14Be
      | Self::Gbrap16Le
      | Self::Gbrap16Be
      | Self::Gbrap32Le
      | Self::Gbrap32Be
      | Self::Gbrapf16Le
      | Self::Gbrapf16Be
      | Self::Gbrapf32Le
      | Self::Gbrapf32Be
      | Self::Gray8
      | Self::Gray9Le
      | Self::Gray9Be
      | Self::Gray10Le
      | Self::Gray10Be
      | Self::Gray12Le
      | Self::Gray12Be
      | Self::Gray14Le
      | Self::Gray14Be
      | Self::Gray16Le
      | Self::Gray16Be
      | Self::Gray32Le
      | Self::Gray32Be
      | Self::Grayf32Le
      | Self::Grayf32Be
      | Self::Grayf16Le
      | Self::Grayf16Be
      | Self::Ya8
      | Self::Ya16Le
      | Self::Ya16Be
      | Self::Yaf16Le
      | Self::Yaf16Be
      | Self::Yaf32Le
      | Self::Yaf32Be
      | Self::Monowhite
      | Self::Monoblack
      | Self::Pal8
      | Self::BayerBggr8
      | Self::BayerRggb8
      | Self::BayerGbrg8
      | Self::BayerGrbg8
      | Self::BayerBggr10Le
      | Self::BayerBggr10Be
      | Self::BayerRggb10Le
      | Self::BayerRggb10Be
      | Self::BayerGbrg10Le
      | Self::BayerGbrg10Be
      | Self::BayerGrbg10Le
      | Self::BayerGrbg10Be
      | Self::BayerBggr12Le
      | Self::BayerBggr12Be
      | Self::BayerRggb12Le
      | Self::BayerRggb12Be
      | Self::BayerGbrg12Le
      | Self::BayerGbrg12Be
      | Self::BayerGrbg12Le
      | Self::BayerGrbg12Be
      | Self::BayerBggr14Le
      | Self::BayerBggr14Be
      | Self::BayerRggb14Le
      | Self::BayerRggb14Be
      | Self::BayerGbrg14Le
      | Self::BayerGbrg14Be
      | Self::BayerGrbg14Le
      | Self::BayerGrbg14Be
      | Self::BayerBggr16Le
      | Self::BayerBggr16Be
      | Self::BayerRggb16Le
      | Self::BayerRggb16Be
      | Self::BayerGbrg16Le
      | Self::BayerGbrg16Be
      | Self::BayerGrbg16Le
      | Self::BayerGrbg16Be => (self, None),
    }
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

/// The error [`PixelFormat`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a pixel-format name")]
#[non_exhaustive]
pub struct ParsePixelFormatError;

impl core::str::FromStr for PixelFormat {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParsePixelFormatError;

  /// Reads a pixel-format name: the canonical slug [`Self::as_str`]
  /// renders, **or** the FFmpeg spelling of the same format where the
  /// two differ.
  ///
  /// Emission is injective and canonical — [`Self::as_str`],
  /// [`Display`](core::fmt::Display) and serde render one slug per
  /// variant and never a synonym — so `parse(display(x)) == x` holds
  /// for every named variant and `display(parse(s))` is idempotent.
  /// Parse is the wider side: it also takes the three names FFmpeg's
  /// `av_get_pix_fmt_name` renders differently from the
  /// `AV_PIX_FMT_<NAME>` header identifier this vocabulary is spelled
  /// after — `gray` ([`Self::Gray8`]), `monob` ([`Self::Monoblack`])
  /// and `monow` ([`Self::Monowhite`]) — so a name copied off
  /// `ffprobe` lands on the named variant instead of the escape.
  ///
  /// # Errors
  ///
  /// Returns [`ParsePixelFormatError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"none" => Self::None,
      b"yuv420p" => Self::Yuv420p,
      b"yuv422p" => Self::Yuv422p,
      b"yuv440p" => Self::Yuv440p,
      b"yuv444p" => Self::Yuv444p,
      b"yuv411p" => Self::Yuv411p,
      b"yuv410p" => Self::Yuv410p,
      b"yuvj411p" => Self::Yuvj411p,
      b"yuvj420p" => Self::Yuvj420p,
      b"yuvj422p" => Self::Yuvj422p,
      b"yuvj440p" => Self::Yuvj440p,
      b"yuvj444p" => Self::Yuvj444p,
      b"yuv420p9le" => Self::Yuv420p9Le,
      b"yuv420p9be" => Self::Yuv420p9Be,
      b"yuv420p10le" => Self::Yuv420p10Le,
      b"yuv420p10be" => Self::Yuv420p10Be,
      b"yuv420p12le" => Self::Yuv420p12Le,
      b"yuv420p12be" => Self::Yuv420p12Be,
      b"yuv420p14le" => Self::Yuv420p14Le,
      b"yuv420p14be" => Self::Yuv420p14Be,
      b"yuv420p16le" => Self::Yuv420p16Le,
      b"yuv420p16be" => Self::Yuv420p16Be,
      b"yuv422p9le" => Self::Yuv422p9Le,
      b"yuv422p9be" => Self::Yuv422p9Be,
      b"yuv422p10le" => Self::Yuv422p10Le,
      b"yuv422p10be" => Self::Yuv422p10Be,
      b"yuv422p12le" => Self::Yuv422p12Le,
      b"yuv422p12be" => Self::Yuv422p12Be,
      b"yuv422p14le" => Self::Yuv422p14Le,
      b"yuv422p14be" => Self::Yuv422p14Be,
      b"yuv422p16le" => Self::Yuv422p16Le,
      b"yuv422p16be" => Self::Yuv422p16Be,
      b"yuv440p10le" => Self::Yuv440p10Le,
      b"yuv440p10be" => Self::Yuv440p10Be,
      b"yuv440p12le" => Self::Yuv440p12Le,
      b"yuv440p12be" => Self::Yuv440p12Be,
      b"yuv444p9le" => Self::Yuv444p9Le,
      b"yuv444p9be" => Self::Yuv444p9Be,
      b"yuv444p10le" => Self::Yuv444p10Le,
      b"yuv444p10be" => Self::Yuv444p10Be,
      b"yuv444p12le" => Self::Yuv444p12Le,
      b"yuv444p12be" => Self::Yuv444p12Be,
      b"yuv444p14le" => Self::Yuv444p14Le,
      b"yuv444p14be" => Self::Yuv444p14Be,
      b"yuv444p16le" => Self::Yuv444p16Le,
      b"yuv444p16be" => Self::Yuv444p16Be,
      b"yuv444p10msble" => Self::Yuv444p10MsbLe,
      b"yuv444p10msbbe" => Self::Yuv444p10MsbBe,
      b"yuv444p12msble" => Self::Yuv444p12MsbLe,
      b"yuv444p12msbbe" => Self::Yuv444p12MsbBe,
      b"yuva420p" => Self::Yuva420p,
      b"yuva422p" => Self::Yuva422p,
      b"yuva444p" => Self::Yuva444p,
      b"yuva420p9le" => Self::Yuva420p9Le,
      b"yuva420p9be" => Self::Yuva420p9Be,
      b"yuva422p9le" => Self::Yuva422p9Le,
      b"yuva422p9be" => Self::Yuva422p9Be,
      b"yuva444p9le" => Self::Yuva444p9Le,
      b"yuva444p9be" => Self::Yuva444p9Be,
      b"yuva420p10le" => Self::Yuva420p10Le,
      b"yuva420p10be" => Self::Yuva420p10Be,
      b"yuva422p10le" => Self::Yuva422p10Le,
      b"yuva422p10be" => Self::Yuva422p10Be,
      b"yuva444p10le" => Self::Yuva444p10Le,
      b"yuva444p10be" => Self::Yuva444p10Be,
      b"yuva420p12le" => Self::Yuva420p12Le,
      b"yuva420p12be" => Self::Yuva420p12Be,
      b"yuva422p12le" => Self::Yuva422p12Le,
      b"yuva422p12be" => Self::Yuva422p12Be,
      b"yuva444p12le" => Self::Yuva444p12Le,
      b"yuva444p12be" => Self::Yuva444p12Be,
      b"yuva444p14le" => Self::Yuva444p14Le,
      b"yuva420p16le" => Self::Yuva420p16Le,
      b"yuva420p16be" => Self::Yuva420p16Be,
      b"yuva422p16le" => Self::Yuva422p16Le,
      b"yuva422p16be" => Self::Yuva422p16Be,
      b"yuva444p16le" => Self::Yuva444p16Le,
      b"yuva444p16be" => Self::Yuva444p16Be,
      b"nv12" => Self::Nv12,
      b"nv21" => Self::Nv21,
      b"nv16" => Self::Nv16,
      b"nv24" => Self::Nv24,
      b"nv42" => Self::Nv42,
      b"nv20le" => Self::Nv20Le,
      b"nv20be" => Self::Nv20Be,
      b"p010le" => Self::P010Le,
      b"p010be" => Self::P010Be,
      b"p012le" => Self::P012Le,
      b"p012be" => Self::P012Be,
      b"p016le" => Self::P016Le,
      b"p016be" => Self::P016Be,
      b"p210le" => Self::P210Le,
      b"p210be" => Self::P210Be,
      b"p212le" => Self::P212Le,
      b"p212be" => Self::P212Be,
      b"p216le" => Self::P216Le,
      b"p216be" => Self::P216Be,
      b"p410le" => Self::P410Le,
      b"p410be" => Self::P410Be,
      b"p412le" => Self::P412Le,
      b"p412be" => Self::P412Be,
      b"p416le" => Self::P416Le,
      b"p416be" => Self::P416Be,
      b"yuyv422" => Self::Yuyv422,
      b"uyvy422" => Self::Uyvy422,
      b"yvyu422" => Self::Yvyu422,
      b"uyyvyy411" => Self::Uyyvyy411,
      b"y210le" => Self::Y210Le,
      b"y210be" => Self::Y210Be,
      b"y212le" => Self::Y212Le,
      b"y212be" => Self::Y212Be,
      b"y216le" => Self::Y216Le,
      b"y216be" => Self::Y216Be,
      b"v210" => Self::V210,
      b"v410le" => Self::V410Le,
      b"v410be" => Self::V410Be,
      b"xv30le" => Self::Xv30Le,
      b"xv30be" => Self::Xv30Be,
      b"v30xle" => Self::V30xLe,
      b"v30xbe" => Self::V30xBe,
      b"xv36le" => Self::Xv36Le,
      b"xv36be" => Self::Xv36Be,
      b"xv48le" => Self::Xv48Le,
      b"xv48be" => Self::Xv48Be,
      b"vuya" => Self::Vuya,
      b"vuyx" => Self::Vuyx,
      b"ayuv" => Self::Ayuv,
      b"ayuv64le" => Self::Ayuv64Le,
      b"ayuv64be" => Self::Ayuv64Be,
      b"uyva" => Self::Uyva,
      b"vyu444" => Self::Vyu444,
      b"xyz12le" => Self::Xyz12Le,
      b"xyz12be" => Self::Xyz12Be,
      b"rgb24" => Self::Rgb24,
      b"bgr24" => Self::Bgr24,
      b"rgba" => Self::Rgba,
      b"bgra" => Self::Bgra,
      b"argb" => Self::Argb,
      b"abgr" => Self::Abgr,
      b"rgb0" => Self::Rgbx,
      b"bgr0" => Self::Bgrx,
      b"0rgb" => Self::Xrgb,
      b"0bgr" => Self::Xbgr,
      b"x2rgb10le" => Self::X2Rgb10Le,
      b"x2rgb10be" => Self::X2Rgb10Be,
      b"x2bgr10le" => Self::X2Bgr10Le,
      b"x2bgr10be" => Self::X2Bgr10Be,
      b"gbr24p" => Self::Gbr24p,
      b"rgb4" => Self::Rgb4,
      b"rgb4_byte" => Self::Rgb4Byte,
      b"rgb8" => Self::Rgb8,
      b"bgr4" => Self::Bgr4,
      b"bgr4_byte" => Self::Bgr4Byte,
      b"bgr8" => Self::Bgr8,
      b"rgb444le" => Self::Rgb444Le,
      b"rgb444be" => Self::Rgb444Be,
      b"bgr444le" => Self::Bgr444Le,
      b"bgr444be" => Self::Bgr444Be,
      b"rgb555le" => Self::Rgb555Le,
      b"rgb555be" => Self::Rgb555Be,
      b"bgr555le" => Self::Bgr555Le,
      b"bgr555be" => Self::Bgr555Be,
      b"rgb565le" => Self::Rgb565Le,
      b"rgb565be" => Self::Rgb565Be,
      b"bgr565le" => Self::Bgr565Le,
      b"bgr565be" => Self::Bgr565Be,
      b"rgb48le" => Self::Rgb48Le,
      b"rgb48be" => Self::Rgb48Be,
      b"bgr48le" => Self::Bgr48Le,
      b"bgr48be" => Self::Bgr48Be,
      b"rgba64le" => Self::Rgba64Le,
      b"rgba64be" => Self::Rgba64Be,
      b"bgra64le" => Self::Bgra64Le,
      b"bgra64be" => Self::Bgra64Be,
      b"rgb96le" => Self::Rgb96Le,
      b"rgb96be" => Self::Rgb96Be,
      b"rgba128le" => Self::Rgba128Le,
      b"rgba128be" => Self::Rgba128Be,
      b"rgbf16le" => Self::Rgbf16Le,
      b"rgbf16be" => Self::Rgbf16Be,
      b"rgbf32le" => Self::Rgbf32Le,
      b"rgbf32be" => Self::Rgbf32Be,
      b"rgbaf16le" => Self::Rgbaf16Le,
      b"rgbaf16be" => Self::Rgbaf16Be,
      b"rgbaf32le" => Self::Rgbaf32Le,
      b"rgbaf32be" => Self::Rgbaf32Be,
      b"gbrp" => Self::Gbrp,
      b"gbrp9le" => Self::Gbrp9Le,
      b"gbrp9be" => Self::Gbrp9Be,
      b"gbrp10le" => Self::Gbrp10Le,
      b"gbrp10be" => Self::Gbrp10Be,
      b"gbrp10msble" => Self::Gbrp10MsbLe,
      b"gbrp10msbbe" => Self::Gbrp10MsbBe,
      b"gbrp12le" => Self::Gbrp12Le,
      b"gbrp12be" => Self::Gbrp12Be,
      b"gbrp12msble" => Self::Gbrp12MsbLe,
      b"gbrp12msbbe" => Self::Gbrp12MsbBe,
      b"gbrp14le" => Self::Gbrp14Le,
      b"gbrp14be" => Self::Gbrp14Be,
      b"gbrp16le" => Self::Gbrp16Le,
      b"gbrp16be" => Self::Gbrp16Be,
      b"gbrpf16le" => Self::Gbrpf16Le,
      b"gbrpf16be" => Self::Gbrpf16Be,
      b"gbrpf32le" => Self::Gbrpf32Le,
      b"gbrpf32be" => Self::Gbrpf32Be,
      b"gbrap" => Self::Gbrap,
      b"gbrap10le" => Self::Gbrap10Le,
      b"gbrap10be" => Self::Gbrap10Be,
      b"gbrap12le" => Self::Gbrap12Le,
      b"gbrap12be" => Self::Gbrap12Be,
      b"gbrap14le" => Self::Gbrap14Le,
      b"gbrap14be" => Self::Gbrap14Be,
      b"gbrap16le" => Self::Gbrap16Le,
      b"gbrap16be" => Self::Gbrap16Be,
      b"gbrap32le" => Self::Gbrap32Le,
      b"gbrap32be" => Self::Gbrap32Be,
      b"gbrapf16le" => Self::Gbrapf16Le,
      b"gbrapf16be" => Self::Gbrapf16Be,
      b"gbrapf32le" => Self::Gbrapf32Le,
      b"gbrapf32be" => Self::Gbrapf32Be,
      b"gray8" => Self::Gray8,
      b"gray8a" => Self::Gray8a,
      b"gray9le" => Self::Gray9Le,
      b"gray9be" => Self::Gray9Be,
      b"gray10le" => Self::Gray10Le,
      b"gray10be" => Self::Gray10Be,
      b"gray12le" => Self::Gray12Le,
      b"gray12be" => Self::Gray12Be,
      b"gray14le" => Self::Gray14Le,
      b"gray14be" => Self::Gray14Be,
      b"gray16le" => Self::Gray16Le,
      b"gray16be" => Self::Gray16Be,
      b"gray32le" => Self::Gray32Le,
      b"gray32be" => Self::Gray32Be,
      b"grayf32le" => Self::Grayf32Le,
      b"grayf32be" => Self::Grayf32Be,
      b"grayf16le" => Self::Grayf16Le,
      b"grayf16be" => Self::Grayf16Be,
      b"ya8" => Self::Ya8,
      b"y400a" => Self::Y400a,
      b"ya16le" => Self::Ya16Le,
      b"ya16be" => Self::Ya16Be,
      b"yaf16le" => Self::Yaf16Le,
      b"yaf16be" => Self::Yaf16Be,
      b"yaf32le" => Self::Yaf32Le,
      b"yaf32be" => Self::Yaf32Be,
      b"monowhite" => Self::Monowhite,
      b"monoblack" => Self::Monoblack,
      b"pal8" => Self::Pal8,
      b"bayer_bggr8" => Self::BayerBggr8,
      b"bayer_rggb8" => Self::BayerRggb8,
      b"bayer_gbrg8" => Self::BayerGbrg8,
      b"bayer_grbg8" => Self::BayerGrbg8,
      b"bayer_bggr10le" => Self::BayerBggr10Le,
      b"bayer_bggr10be" => Self::BayerBggr10Be,
      b"bayer_rggb10le" => Self::BayerRggb10Le,
      b"bayer_rggb10be" => Self::BayerRggb10Be,
      b"bayer_gbrg10le" => Self::BayerGbrg10Le,
      b"bayer_gbrg10be" => Self::BayerGbrg10Be,
      b"bayer_grbg10le" => Self::BayerGrbg10Le,
      b"bayer_grbg10be" => Self::BayerGrbg10Be,
      b"bayer_bggr12le" => Self::BayerBggr12Le,
      b"bayer_bggr12be" => Self::BayerBggr12Be,
      b"bayer_rggb12le" => Self::BayerRggb12Le,
      b"bayer_rggb12be" => Self::BayerRggb12Be,
      b"bayer_gbrg12le" => Self::BayerGbrg12Le,
      b"bayer_gbrg12be" => Self::BayerGbrg12Be,
      b"bayer_grbg12le" => Self::BayerGrbg12Le,
      b"bayer_grbg12be" => Self::BayerGrbg12Be,
      b"bayer_bggr14le" => Self::BayerBggr14Le,
      b"bayer_bggr14be" => Self::BayerBggr14Be,
      b"bayer_rggb14le" => Self::BayerRggb14Le,
      b"bayer_rggb14be" => Self::BayerRggb14Be,
      b"bayer_gbrg14le" => Self::BayerGbrg14Le,
      b"bayer_gbrg14be" => Self::BayerGbrg14Be,
      b"bayer_grbg14le" => Self::BayerGrbg14Le,
      b"bayer_grbg14be" => Self::BayerGrbg14Be,
      b"bayer_bggr16le" => Self::BayerBggr16Le,
      b"bayer_bggr16be" => Self::BayerBggr16Be,
      b"bayer_rggb16le" => Self::BayerRggb16Le,
      b"bayer_rggb16be" => Self::BayerRggb16Be,
      b"bayer_gbrg16le" => Self::BayerGbrg16Le,
      b"bayer_gbrg16be" => Self::BayerGbrg16Be,
      b"bayer_grbg16le" => Self::BayerGrbg16Le,
      b"bayer_grbg16be" => Self::BayerGrbg16Be,

      // FFmpeg's own spelling for the three formats where
      // `av_get_pix_fmt_name` disagrees with the `AV_PIX_FMT_<NAME>`
      // header identifier. Accepted, never emitted — keep these in step
      // with `FFMPEG_SYNONYMS`, which is what the collision nail test
      // sweeps.
      b"gray" => Self::Gray8,
      b"monob" => Self::Monoblack,
      b"monow" => Self::Monowhite,

      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParsePixelFormatError),
    })
  }
}

/// The FFmpeg spellings [`FromStr`](core::str::FromStr) accepts on top of
/// the canonical slugs, as `(ffmpeg_name, canonical_slug)`.
///
/// FFmpeg names a pixel format twice: the `AV_PIX_FMT_<NAME>` enumerator
/// (what this vocabulary is spelled after) and the descriptor name
/// `av_get_pix_fmt_name` returns (what `ffprobe` prints). The two agree
/// for every format but these three. Emission stays canonical, so this
/// table is parse-side only; it is `cfg(test)` because its one consumer
/// is the nail test that proves no synonym shadows a canonical slug.
#[cfg(test)]
const FFMPEG_SYNONYMS: &[(&str, &str)] = &[
  ("gray", "gray8"),
  ("monob", "monoblack"),
  ("monow", "monowhite"),
];

impl PixelFormat {
  /// Lowercase FFmpeg-style identifier for this variant — the FFmpeg
  /// `AV_PIX_FMT_*` lowercase slug where one exists, or the same
  /// lowercase convention applied to mediaframe-extension formats that
  /// have no FFmpeg pixel format (e.g. the sub-16-bit Bayer variants).
  /// Matches the enum's [`Display`] output exactly — single source of
  /// truth.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::None => "none",
      Self::Yuv420p => "yuv420p",
      Self::Yuv422p => "yuv422p",
      Self::Yuv440p => "yuv440p",
      Self::Yuv444p => "yuv444p",
      Self::Yuv411p => "yuv411p",
      Self::Yuv410p => "yuv410p",
      Self::Yuvj411p => "yuvj411p",
      Self::Yuvj420p => "yuvj420p",
      Self::Yuvj422p => "yuvj422p",
      Self::Yuvj440p => "yuvj440p",
      Self::Yuvj444p => "yuvj444p",
      Self::Yuv420p9Le => "yuv420p9le",
      Self::Yuv420p9Be => "yuv420p9be",
      Self::Yuv420p10Le => "yuv420p10le",
      Self::Yuv420p10Be => "yuv420p10be",
      Self::Yuv420p12Le => "yuv420p12le",
      Self::Yuv420p12Be => "yuv420p12be",
      Self::Yuv420p14Le => "yuv420p14le",
      Self::Yuv420p14Be => "yuv420p14be",
      Self::Yuv420p16Le => "yuv420p16le",
      Self::Yuv420p16Be => "yuv420p16be",
      Self::Yuv422p9Le => "yuv422p9le",
      Self::Yuv422p9Be => "yuv422p9be",
      Self::Yuv422p10Le => "yuv422p10le",
      Self::Yuv422p10Be => "yuv422p10be",
      Self::Yuv422p12Le => "yuv422p12le",
      Self::Yuv422p12Be => "yuv422p12be",
      Self::Yuv422p14Le => "yuv422p14le",
      Self::Yuv422p14Be => "yuv422p14be",
      Self::Yuv422p16Le => "yuv422p16le",
      Self::Yuv422p16Be => "yuv422p16be",
      Self::Yuv440p10Le => "yuv440p10le",
      Self::Yuv440p10Be => "yuv440p10be",
      Self::Yuv440p12Le => "yuv440p12le",
      Self::Yuv440p12Be => "yuv440p12be",
      Self::Yuv444p9Le => "yuv444p9le",
      Self::Yuv444p9Be => "yuv444p9be",
      Self::Yuv444p10Le => "yuv444p10le",
      Self::Yuv444p10Be => "yuv444p10be",
      Self::Yuv444p12Le => "yuv444p12le",
      Self::Yuv444p12Be => "yuv444p12be",
      Self::Yuv444p14Le => "yuv444p14le",
      Self::Yuv444p14Be => "yuv444p14be",
      Self::Yuv444p16Le => "yuv444p16le",
      Self::Yuv444p16Be => "yuv444p16be",
      Self::Yuv444p10MsbLe => "yuv444p10msble",
      Self::Yuv444p10MsbBe => "yuv444p10msbbe",
      Self::Yuv444p12MsbLe => "yuv444p12msble",
      Self::Yuv444p12MsbBe => "yuv444p12msbbe",
      Self::Yuva420p => "yuva420p",
      Self::Yuva422p => "yuva422p",
      Self::Yuva444p => "yuva444p",
      Self::Yuva420p9Le => "yuva420p9le",
      Self::Yuva420p9Be => "yuva420p9be",
      Self::Yuva422p9Le => "yuva422p9le",
      Self::Yuva422p9Be => "yuva422p9be",
      Self::Yuva444p9Le => "yuva444p9le",
      Self::Yuva444p9Be => "yuva444p9be",
      Self::Yuva420p10Le => "yuva420p10le",
      Self::Yuva420p10Be => "yuva420p10be",
      Self::Yuva422p10Le => "yuva422p10le",
      Self::Yuva422p10Be => "yuva422p10be",
      Self::Yuva444p10Le => "yuva444p10le",
      Self::Yuva444p10Be => "yuva444p10be",
      Self::Yuva420p12Le => "yuva420p12le",
      Self::Yuva420p12Be => "yuva420p12be",
      Self::Yuva422p12Le => "yuva422p12le",
      Self::Yuva422p12Be => "yuva422p12be",
      Self::Yuva444p12Le => "yuva444p12le",
      Self::Yuva444p12Be => "yuva444p12be",
      Self::Yuva444p14Le => "yuva444p14le",
      Self::Yuva420p16Le => "yuva420p16le",
      Self::Yuva420p16Be => "yuva420p16be",
      Self::Yuva422p16Le => "yuva422p16le",
      Self::Yuva422p16Be => "yuva422p16be",
      Self::Yuva444p16Le => "yuva444p16le",
      Self::Yuva444p16Be => "yuva444p16be",
      Self::Nv12 => "nv12",
      Self::Nv21 => "nv21",
      Self::Nv16 => "nv16",
      Self::Nv24 => "nv24",
      Self::Nv42 => "nv42",
      Self::Nv20Le => "nv20le",
      Self::Nv20Be => "nv20be",
      Self::P010Le => "p010le",
      Self::P010Be => "p010be",
      Self::P012Le => "p012le",
      Self::P012Be => "p012be",
      Self::P016Le => "p016le",
      Self::P016Be => "p016be",
      Self::P210Le => "p210le",
      Self::P210Be => "p210be",
      Self::P212Le => "p212le",
      Self::P212Be => "p212be",
      Self::P216Le => "p216le",
      Self::P216Be => "p216be",
      Self::P410Le => "p410le",
      Self::P410Be => "p410be",
      Self::P412Le => "p412le",
      Self::P412Be => "p412be",
      Self::P416Le => "p416le",
      Self::P416Be => "p416be",
      Self::Yuyv422 => "yuyv422",
      Self::Uyvy422 => "uyvy422",
      Self::Yvyu422 => "yvyu422",
      Self::Uyyvyy411 => "uyyvyy411",
      Self::Y210Le => "y210le",
      Self::Y210Be => "y210be",
      Self::Y212Le => "y212le",
      Self::Y212Be => "y212be",
      Self::Y216Le => "y216le",
      Self::Y216Be => "y216be",
      Self::V210 => "v210",
      Self::V410Le => "v410le",
      Self::V410Be => "v410be",
      Self::Xv30Le => "xv30le",
      Self::Xv30Be => "xv30be",
      Self::V30xLe => "v30xle",
      Self::V30xBe => "v30xbe",
      Self::Xv36Le => "xv36le",
      Self::Xv36Be => "xv36be",
      Self::Xv48Le => "xv48le",
      Self::Xv48Be => "xv48be",
      Self::Vuya => "vuya",
      Self::Vuyx => "vuyx",
      Self::Ayuv => "ayuv",
      Self::Ayuv64Le => "ayuv64le",
      Self::Ayuv64Be => "ayuv64be",
      Self::Uyva => "uyva",
      Self::Vyu444 => "vyu444",
      Self::Xyz12Le => "xyz12le",
      Self::Xyz12Be => "xyz12be",
      Self::Rgb24 => "rgb24",
      Self::Bgr24 => "bgr24",
      Self::Rgba => "rgba",
      Self::Bgra => "bgra",
      Self::Argb => "argb",
      Self::Abgr => "abgr",
      Self::Rgbx => "rgb0",
      Self::Bgrx => "bgr0",
      Self::Xrgb => "0rgb",
      Self::Xbgr => "0bgr",
      Self::X2Rgb10Le => "x2rgb10le",
      Self::X2Rgb10Be => "x2rgb10be",
      Self::X2Bgr10Le => "x2bgr10le",
      Self::X2Bgr10Be => "x2bgr10be",
      Self::Gbr24p => "gbr24p",
      Self::Rgb4 => "rgb4",
      Self::Rgb4Byte => "rgb4_byte",
      Self::Rgb8 => "rgb8",
      Self::Bgr4 => "bgr4",
      Self::Bgr4Byte => "bgr4_byte",
      Self::Bgr8 => "bgr8",
      Self::Rgb444Le => "rgb444le",
      Self::Rgb444Be => "rgb444be",
      Self::Bgr444Le => "bgr444le",
      Self::Bgr444Be => "bgr444be",
      Self::Rgb555Le => "rgb555le",
      Self::Rgb555Be => "rgb555be",
      Self::Bgr555Le => "bgr555le",
      Self::Bgr555Be => "bgr555be",
      Self::Rgb565Le => "rgb565le",
      Self::Rgb565Be => "rgb565be",
      Self::Bgr565Le => "bgr565le",
      Self::Bgr565Be => "bgr565be",
      Self::Rgb48Le => "rgb48le",
      Self::Rgb48Be => "rgb48be",
      Self::Bgr48Le => "bgr48le",
      Self::Bgr48Be => "bgr48be",
      Self::Rgba64Le => "rgba64le",
      Self::Rgba64Be => "rgba64be",
      Self::Bgra64Le => "bgra64le",
      Self::Bgra64Be => "bgra64be",
      Self::Rgb96Le => "rgb96le",
      Self::Rgb96Be => "rgb96be",
      Self::Rgba128Le => "rgba128le",
      Self::Rgba128Be => "rgba128be",
      Self::Rgbf16Le => "rgbf16le",
      Self::Rgbf16Be => "rgbf16be",
      Self::Rgbf32Le => "rgbf32le",
      Self::Rgbf32Be => "rgbf32be",
      Self::Rgbaf16Le => "rgbaf16le",
      Self::Rgbaf16Be => "rgbaf16be",
      Self::Rgbaf32Le => "rgbaf32le",
      Self::Rgbaf32Be => "rgbaf32be",
      Self::Gbrp => "gbrp",
      Self::Gbrp9Le => "gbrp9le",
      Self::Gbrp9Be => "gbrp9be",
      Self::Gbrp10Le => "gbrp10le",
      Self::Gbrp10Be => "gbrp10be",
      Self::Gbrp10MsbLe => "gbrp10msble",
      Self::Gbrp10MsbBe => "gbrp10msbbe",
      Self::Gbrp12Le => "gbrp12le",
      Self::Gbrp12Be => "gbrp12be",
      Self::Gbrp12MsbLe => "gbrp12msble",
      Self::Gbrp12MsbBe => "gbrp12msbbe",
      Self::Gbrp14Le => "gbrp14le",
      Self::Gbrp14Be => "gbrp14be",
      Self::Gbrp16Le => "gbrp16le",
      Self::Gbrp16Be => "gbrp16be",
      Self::Gbrpf16Le => "gbrpf16le",
      Self::Gbrpf16Be => "gbrpf16be",
      Self::Gbrpf32Le => "gbrpf32le",
      Self::Gbrpf32Be => "gbrpf32be",
      Self::Gbrap => "gbrap",
      Self::Gbrap10Le => "gbrap10le",
      Self::Gbrap10Be => "gbrap10be",
      Self::Gbrap12Le => "gbrap12le",
      Self::Gbrap12Be => "gbrap12be",
      Self::Gbrap14Le => "gbrap14le",
      Self::Gbrap14Be => "gbrap14be",
      Self::Gbrap16Le => "gbrap16le",
      Self::Gbrap16Be => "gbrap16be",
      Self::Gbrap32Le => "gbrap32le",
      Self::Gbrap32Be => "gbrap32be",
      Self::Gbrapf16Le => "gbrapf16le",
      Self::Gbrapf16Be => "gbrapf16be",
      Self::Gbrapf32Le => "gbrapf32le",
      Self::Gbrapf32Be => "gbrapf32be",
      Self::Gray8 => "gray8",
      Self::Gray8a => "gray8a",
      Self::Gray9Le => "gray9le",
      Self::Gray9Be => "gray9be",
      Self::Gray10Le => "gray10le",
      Self::Gray10Be => "gray10be",
      Self::Gray12Le => "gray12le",
      Self::Gray12Be => "gray12be",
      Self::Gray14Le => "gray14le",
      Self::Gray14Be => "gray14be",
      Self::Gray16Le => "gray16le",
      Self::Gray16Be => "gray16be",
      Self::Gray32Le => "gray32le",
      Self::Gray32Be => "gray32be",
      Self::Grayf32Le => "grayf32le",
      Self::Grayf32Be => "grayf32be",
      Self::Grayf16Le => "grayf16le",
      Self::Grayf16Be => "grayf16be",
      Self::Ya8 => "ya8",
      Self::Y400a => "y400a",
      Self::Ya16Le => "ya16le",
      Self::Ya16Be => "ya16be",
      Self::Yaf16Le => "yaf16le",
      Self::Yaf16Be => "yaf16be",
      Self::Yaf32Le => "yaf32le",
      Self::Yaf32Be => "yaf32be",
      Self::Monowhite => "monowhite",
      Self::Monoblack => "monoblack",
      Self::Pal8 => "pal8",
      Self::BayerBggr8 => "bayer_bggr8",
      Self::BayerRggb8 => "bayer_rggb8",
      Self::BayerGbrg8 => "bayer_gbrg8",
      Self::BayerGrbg8 => "bayer_grbg8",
      Self::BayerBggr10Le => "bayer_bggr10le",
      Self::BayerBggr10Be => "bayer_bggr10be",
      Self::BayerRggb10Le => "bayer_rggb10le",
      Self::BayerRggb10Be => "bayer_rggb10be",
      Self::BayerGbrg10Le => "bayer_gbrg10le",
      Self::BayerGbrg10Be => "bayer_gbrg10be",
      Self::BayerGrbg10Le => "bayer_grbg10le",
      Self::BayerGrbg10Be => "bayer_grbg10be",
      Self::BayerBggr12Le => "bayer_bggr12le",
      Self::BayerBggr12Be => "bayer_bggr12be",
      Self::BayerRggb12Le => "bayer_rggb12le",
      Self::BayerRggb12Be => "bayer_rggb12be",
      Self::BayerGbrg12Le => "bayer_gbrg12le",
      Self::BayerGbrg12Be => "bayer_gbrg12be",
      Self::BayerGrbg12Le => "bayer_grbg12le",
      Self::BayerGrbg12Be => "bayer_grbg12be",
      Self::BayerBggr14Le => "bayer_bggr14le",
      Self::BayerBggr14Be => "bayer_bggr14be",
      Self::BayerRggb14Le => "bayer_rggb14le",
      Self::BayerRggb14Be => "bayer_rggb14be",
      Self::BayerGbrg14Le => "bayer_gbrg14le",
      Self::BayerGbrg14Be => "bayer_gbrg14be",
      Self::BayerGrbg14Le => "bayer_grbg14le",
      Self::BayerGrbg14Be => "bayer_grbg14be",
      Self::BayerBggr16Le => "bayer_bggr16le",
      Self::BayerBggr16Be => "bayer_bggr16be",
      Self::BayerRggb16Le => "bayer_rggb16le",
      Self::BayerRggb16Be => "bayer_rggb16be",
      Self::BayerGbrg16Le => "bayer_gbrg16le",
      Self::BayerGbrg16Be => "bayer_gbrg16be",
      Self::BayerGrbg16Le => "bayer_grbg16le",
      Self::BayerGrbg16Be => "bayer_grbg16be",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }
}

roster!(
  PixelFormat,
  "pixel format",
  [
    None, Yuv420p, Yuv422p, Yuv440p, Yuv444p, Yuv411p, Yuv410p, Yuvj411p,
    Yuvj420p, Yuvj422p, Yuvj440p, Yuvj444p, Yuv420p9Le, Yuv420p9Be,
    Yuv420p10Le, Yuv420p10Be, Yuv420p12Le, Yuv420p12Be, Yuv420p14Le,
    Yuv420p14Be, Yuv420p16Le, Yuv420p16Be, Yuv422p9Le, Yuv422p9Be,
    Yuv422p10Le, Yuv422p10Be, Yuv422p12Le, Yuv422p12Be, Yuv422p14Le,
    Yuv422p14Be, Yuv422p16Le, Yuv422p16Be, Yuv440p10Le, Yuv440p10Be,
    Yuv440p12Le, Yuv440p12Be, Yuv444p9Le, Yuv444p9Be, Yuv444p10Le,
    Yuv444p10Be, Yuv444p12Le, Yuv444p12Be, Yuv444p14Le, Yuv444p14Be,
    Yuv444p16Le, Yuv444p16Be, Yuv444p10MsbLe, Yuv444p10MsbBe, Yuv444p12MsbLe,
    Yuv444p12MsbBe, Yuva420p, Yuva422p, Yuva444p, Yuva420p9Le, Yuva420p9Be,
    Yuva422p9Le, Yuva422p9Be, Yuva444p9Le, Yuva444p9Be, Yuva420p10Le,
    Yuva420p10Be, Yuva422p10Le, Yuva422p10Be, Yuva444p10Le, Yuva444p10Be,
    Yuva420p12Le, Yuva420p12Be, Yuva422p12Le, Yuva422p12Be, Yuva444p12Le,
    Yuva444p12Be, Yuva444p14Le, Yuva420p16Le, Yuva420p16Be, Yuva422p16Le,
    Yuva422p16Be, Yuva444p16Le, Yuva444p16Be, Nv12, Nv21, Nv16, Nv24, Nv42,
    Nv20Le, Nv20Be, P010Le, P010Be, P012Le, P012Be, P016Le, P016Be, P210Le,
    P210Be, P212Le, P212Be, P216Le, P216Be, P410Le, P410Be, P412Le, P412Be,
    P416Le, P416Be, Yuyv422, Uyvy422, Yvyu422, Uyyvyy411, Y210Le, Y210Be,
    Y212Le, Y212Be, Y216Le, Y216Be, V210, V410Le, V410Be, Xv30Le, Xv30Be,
    V30xLe, V30xBe, Xv36Le, Xv36Be, Xv48Le, Xv48Be, Vuya, Vuyx, Ayuv,
    Ayuv64Le, Ayuv64Be, Uyva, Vyu444, Xyz12Le, Xyz12Be, Rgb24, Bgr24, Rgba,
    Bgra, Argb, Abgr, Rgbx, Bgrx, Xrgb, Xbgr, X2Rgb10Le, X2Rgb10Be, X2Bgr10Le,
    X2Bgr10Be, Gbr24p, Rgb4, Rgb4Byte, Rgb8, Bgr4, Bgr4Byte, Bgr8, Rgb444Le,
    Rgb444Be, Bgr444Le, Bgr444Be, Rgb555Le, Rgb555Be, Bgr555Le, Bgr555Be,
    Rgb565Le, Rgb565Be, Bgr565Le, Bgr565Be, Rgb48Le, Rgb48Be, Bgr48Le,
    Bgr48Be, Rgba64Le, Rgba64Be, Bgra64Le, Bgra64Be, Rgb96Le, Rgb96Be,
    Rgba128Le, Rgba128Be, Rgbf16Le, Rgbf16Be, Rgbf32Le, Rgbf32Be, Rgbaf16Le,
    Rgbaf16Be, Rgbaf32Le, Rgbaf32Be, Gbrp, Gbrp9Le, Gbrp9Be, Gbrp10Le,
    Gbrp10Be, Gbrp10MsbLe, Gbrp10MsbBe, Gbrp12Le, Gbrp12Be, Gbrp12MsbLe,
    Gbrp12MsbBe, Gbrp14Le, Gbrp14Be, Gbrp16Le, Gbrp16Be, Gbrpf16Le, Gbrpf16Be,
    Gbrpf32Le, Gbrpf32Be, Gbrap, Gbrap10Le, Gbrap10Be, Gbrap12Le, Gbrap12Be,
    Gbrap14Le, Gbrap14Be, Gbrap16Le, Gbrap16Be, Gbrap32Le, Gbrap32Be,
    Gbrapf16Le, Gbrapf16Be, Gbrapf32Le, Gbrapf32Be, Gray8, Gray8a, Gray9Le,
    Gray9Be, Gray10Le, Gray10Be, Gray12Le, Gray12Be, Gray14Le, Gray14Be,
    Gray16Le, Gray16Be, Gray32Le, Gray32Be, Grayf32Le, Grayf32Be, Grayf16Le,
    Grayf16Be, Ya8, Y400a, Ya16Le, Ya16Be, Yaf16Le, Yaf16Be, Yaf32Le, Yaf32Be,
    Monowhite, Monoblack, Pal8, BayerBggr8, BayerRggb8, BayerGbrg8,
    BayerGrbg8, BayerBggr10Le, BayerBggr10Be, BayerRggb10Le, BayerRggb10Be,
    BayerGbrg10Le, BayerGbrg10Be, BayerGrbg10Le, BayerGrbg10Be, BayerBggr12Le,
    BayerBggr12Be, BayerRggb12Le, BayerRggb12Be, BayerGbrg12Le, BayerGbrg12Be,
    BayerGrbg12Le, BayerGrbg12Be, BayerBggr14Le, BayerBggr14Be, BayerRggb14Le,
    BayerRggb14Be, BayerGbrg14Le, BayerGbrg14Be, BayerGrbg14Le, BayerGrbg14Be,
    BayerBggr16Le, BayerBggr16Be, BayerRggb16Le, BayerRggb16Be, BayerGbrg16Le,
    BayerGbrg16Be, BayerGrbg16Le, BayerGrbg16Be
  ],
  alloc_escape: Other
);

#[cfg(test)]
mod tests;
