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
//! Stable wire format: [`PixelFormat::to_u32`] returns the underlying
//! discriminant (this enum is `#[repr(u32)]`); [`PixelFormat::from_u32`]
//! reverses the mapping. Unrecognised values map to
//! [`PixelFormat::Unknown`].

use derive_more::{Display, IsVariant};

/// Pixel format identifier covering FFmpeg + Bayer + cinema-RAW.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(with = "crate::quickcheck_helpers::coded::pixel_format")
)]
pub enum PixelFormat {
  /// Unknown / unset format. The wrapped `u32` is the original
  /// wire value passed to [`PixelFormat::from_u32`] — preserved so
  /// round-tripping unknown values is lossless.
  Unknown(u32),

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
  /// 10-bit semi-planar 4:2:2 YUV (8 channels in 5 16-bit words), little-endian.
  Nv20Le,
  /// 10-bit semi-planar 4:2:2 YUV (8 channels in 5 16-bit words), big-endian.
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
  Monowhite,
  /// 1-bit monochrome, black = 0 (`AV_PIX_FMT_MONOBLACK`).
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
  /// Bayer RGGB pattern, 10-bit little-endian.
  BayerRggb10Le,
  /// Bayer GBRG pattern, 10-bit little-endian.
  BayerGbrg10Le,
  /// Bayer GRBG pattern, 10-bit little-endian.
  BayerGrbg10Le,
  /// Bayer BGGR pattern, 12-bit little-endian.
  BayerBggr12Le,
  /// Bayer RGGB pattern, 12-bit little-endian.
  BayerRggb12Le,
  /// Bayer GBRG pattern, 12-bit little-endian.
  BayerGbrg12Le,
  /// Bayer GRBG pattern, 12-bit little-endian.
  BayerGrbg12Le,
  /// Bayer BGGR pattern, 14-bit little-endian.
  BayerBggr14Le,
  /// Bayer RGGB pattern, 14-bit little-endian.
  BayerRggb14Le,
  /// Bayer GBRG pattern, 14-bit little-endian.
  BayerGbrg14Le,
  /// Bayer GRBG pattern, 14-bit little-endian.
  BayerGrbg14Le,
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
}

impl Default for PixelFormat {
  #[inline]
  fn default() -> Self {
    Self::Unknown(0)
  }
}

impl PixelFormat {
  /// Stable wire representation. Known variants return their
  /// assigned wire id; [`PixelFormat::Unknown`] carries its original
  /// `u32` value through unchanged so `from_u32(to_u32(x)) == x` for
  /// every `x`.
  #[inline]
  pub const fn to_u32(self) -> u32 {
    match self {
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
      Self::BayerBggr12Le => 920,
      Self::BayerRggb12Le => 921,
      Self::BayerGbrg12Le => 922,
      Self::BayerGrbg12Le => 923,
      Self::BayerBggr14Le => 930,
      Self::BayerRggb14Le => 931,
      Self::BayerGbrg14Le => 932,
      Self::BayerGrbg14Le => 933,
      Self::BayerBggr16Le => 940,
      Self::BayerBggr16Be => 944,
      Self::BayerRggb16Le => 941,
      Self::BayerRggb16Be => 945,
      Self::BayerGbrg16Le => 942,
      Self::BayerGbrg16Be => 946,
      Self::BayerGrbg16Le => 943,
      Self::BayerGrbg16Be => 947,
      Self::Unknown(value) => value,
    }
  }

  /// Decodes from the stable `u32` representation produced by
  /// [`Self::to_u32`]. Unrecognised values map to [`Self::Unknown`].
  #[inline]
  pub const fn from_u32(value: u32) -> Self {
    match value {
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
      920 => Self::BayerBggr12Le,
      921 => Self::BayerRggb12Le,
      922 => Self::BayerGbrg12Le,
      923 => Self::BayerGrbg12Le,
      930 => Self::BayerBggr14Le,
      931 => Self::BayerRggb14Le,
      932 => Self::BayerGbrg14Le,
      933 => Self::BayerGrbg14Le,
      940 => Self::BayerBggr16Le,
      941 => Self::BayerRggb16Le,
      942 => Self::BayerGbrg16Le,
      943 => Self::BayerGrbg16Le,
      944 => Self::BayerBggr16Be,
      945 => Self::BayerRggb16Be,
      946 => Self::BayerGbrg16Be,
      947 => Self::BayerGrbg16Be,
      _ => Self::Unknown(value),
    }
  }

  /// Returns `true` for Bayer-mosaic formats (any pattern, any bit
  /// depth). Bayer frames carry undebayered sensor data; downstream
  /// consumers (e.g. `colconv::raw`) demosaic + white-balance + colour-
  /// correct to produce RGB.
  #[inline]
  pub const fn is_bayer(self) -> bool {
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
        | Self::BayerBggr12Le
        | Self::BayerRggb12Le
        | Self::BayerGbrg12Le
        | Self::BayerGrbg12Le
        | Self::BayerBggr14Le
        | Self::BayerRggb14Le
        | Self::BayerGbrg14Le
        | Self::BayerGrbg14Le
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
}

impl PixelFormat {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AV_PIX_FMT_*` lowercase slug). Matches the enum's [`Display`]
  /// output exactly — single source of truth.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
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
      Self::BayerRggb10Le => "bayer_rggb10le",
      Self::BayerGbrg10Le => "bayer_gbrg10le",
      Self::BayerGrbg10Le => "bayer_grbg10le",
      Self::BayerBggr12Le => "bayer_bggr12le",
      Self::BayerRggb12Le => "bayer_rggb12le",
      Self::BayerGbrg12Le => "bayer_gbrg12le",
      Self::BayerGrbg12Le => "bayer_grbg12le",
      Self::BayerBggr14Le => "bayer_bggr14le",
      Self::BayerRggb14Le => "bayer_rggb14le",
      Self::BayerGbrg14Le => "bayer_gbrg14le",
      Self::BayerGrbg14Le => "bayer_grbg14le",
      Self::BayerBggr16Le => "bayer_bggr16le",
      Self::BayerBggr16Be => "bayer_bggr16be",
      Self::BayerRggb16Le => "bayer_rggb16le",
      Self::BayerRggb16Be => "bayer_rggb16be",
      Self::BayerGbrg16Le => "bayer_gbrg16le",
      Self::BayerGbrg16Be => "bayer_gbrg16be",
      Self::BayerGrbg16Le => "bayer_grbg16le",
      Self::BayerGrbg16Be => "bayer_grbg16be",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // `format!` lives in `alloc` under no_std + alloc; bring it in via the
  // crate-root `extern crate alloc as std;` alias. Under `feature = "std"`
  // this resolves to the real `std::format`.
  #[cfg(any(feature = "alloc", feature = "std"))]
  use std::format;

  #[test]
  fn default_is_unknown() {
    assert!(matches!(PixelFormat::default(), PixelFormat::Unknown(0)));
  }

  #[test]
  fn round_trip_u32_for_known_variants() {
    let all = [
      PixelFormat::Unknown(0),
      PixelFormat::Yuv420p,
      PixelFormat::Yuv444p,
      PixelFormat::Yuv420p10Le,
      PixelFormat::Yuv422p16Le,
      PixelFormat::Yuva444p,
      PixelFormat::Nv12,
      PixelFormat::P010Le,
      PixelFormat::P416Le,
      PixelFormat::Yuyv422,
      PixelFormat::V210,
      PixelFormat::Ayuv64Le,
      PixelFormat::Rgb24,
      PixelFormat::Bgra,
      PixelFormat::Rgb565Le,
      PixelFormat::Rgba64Le,
      PixelFormat::Rgbf32Le,
      PixelFormat::Gbrp,
      PixelFormat::Gbrap16Le,
      PixelFormat::Gbrapf32Le,
      PixelFormat::Gray8,
      PixelFormat::Gray16Le,
      PixelFormat::Ya16Le,
      PixelFormat::Monowhite,
      PixelFormat::Pal8,
      PixelFormat::BayerBggr8,
      PixelFormat::BayerRggb16Le,
    ];
    for fmt in all {
      assert_eq!(
        PixelFormat::from_u32(fmt.to_u32()),
        fmt,
        "round-trip failed for {fmt:?}"
      );
    }
  }

  #[test]
  fn unknown_for_garbage_u32() {
    assert_eq!(PixelFormat::from_u32(99_999), PixelFormat::Unknown(99_999));
    assert_eq!(PixelFormat::from_u32(1), PixelFormat::Unknown(1));
  }

  #[test]
  fn unknown_round_trip_is_lossless() {
    // Pre-Unknown(u32) refactor: from_u32(99) → Unknown → to_u32 → 0.
    // Post: from_u32(99) → Unknown(99) → to_u32 → 99. Lossless.
    for n in [1u32, 42, 99_999, u32::MAX, 0] {
      let p = PixelFormat::from_u32(n);
      assert_eq!(p.to_u32(), n, "lossless round-trip broken for {n}");
    }
  }

  // `format!` requires an allocator; gate to alloc-or-std builds.
  // The `Display` impl itself works in bare-core mode via
  // `write!`-style sinks — only this test's assertion strategy needs
  // alloc.
  #[cfg(any(feature = "alloc", feature = "std"))]
  #[test]
  fn display_uses_ffmpeg_lowercase_names() {
    assert_eq!(format!("{}", PixelFormat::Yuv420p), "yuv420p");
    assert_eq!(format!("{}", PixelFormat::Nv12), "nv12");
    assert_eq!(format!("{}", PixelFormat::P010Le), "p010le");
    assert_eq!(format!("{}", PixelFormat::Rgba64Le), "rgba64le");
    assert_eq!(format!("{}", PixelFormat::BayerBggr12Le), "bayer_bggr12le");
    assert_eq!(format!("{}", PixelFormat::Unknown(0)), "unknown");
  }

  #[test]
  fn is_bayer_partition() {
    assert!(PixelFormat::BayerBggr8.is_bayer());
    assert!(PixelFormat::BayerRggb16Le.is_bayer());
    assert!(PixelFormat::BayerGrbg12Le.is_bayer());
    assert!(!PixelFormat::Yuv420p.is_bayer());
    assert!(!PixelFormat::Rgb24.is_bayer());
    assert!(!PixelFormat::Unknown(0).is_bayer());
  }

  #[test]
  fn is_variant_helpers_compile() {
    assert!(PixelFormat::Yuv420p.is_yuv_420_p());
    assert!(PixelFormat::Nv12.is_nv_12());
    assert!(PixelFormat::P010Le.is_p_010_le());
    assert!(!PixelFormat::Yuv420p.is_unknown());
  }

  #[test]
  fn copy_and_eq() {
    let p = PixelFormat::Nv12;
    let q = p; // Copy
    assert_eq!(p, q);
    assert_ne!(p, PixelFormat::Yuv420p);
  }
}
