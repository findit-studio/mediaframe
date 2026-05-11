use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, WidthNotMultipleOf4, WidthOverflow,
  ZeroDimension,
};
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

// ============================================================
// Tier 5.25 — Packed YUV 4:1:1 8-bit (UYYVYY411 / DV legacy)
// ============================================================
//
// FFmpeg `AV_PIX_FMT_UYYVYY411`: packed YUV 4:1:1 with one chroma
// pair shared by 4 luma samples. Layout per 4-pixel group, 6 bytes:
//
//   `[U0, Y0, Y1, V0, Y2, Y3]`
//
// Reference: FFmpeg `libavutil/pixdesc.c` entry for
// `AV_PIX_FMT_UYYVYY411` — descriptor declares
// `log2_chroma_w = 2`, `log2_chroma_h = 0`, `nb_components = 3`,
// `comp[0] (Y) offset = 1, depth = 8, step = 3` (Y bytes at offsets
// 1, 2, 4, 5 of each 6-byte group), `comp[1] (U) offset = 0, depth =
// 8, step = 6`, `comp[2] (V) offset = 3, depth = 8, step = 6`. That
// resolves to U at byte 0, Y0/Y1 at bytes 1/2, V at byte 3, Y2/Y3 at
// bytes 4/5 — matching the canonical "U Y Y V Y Y" naming.
//
// Bytes per pixel: 12 / 8 = 1.5 → row stride must be ≥ `width * 3 / 2`.
// Width must be a multiple of 4 (one complete chroma pair per 4 luma
// pixels).
//
// Common in DV 4:1:1 NTSC capture (legacy). Treated as a P3 format —
// API surface mirrors the Tier 3 packed YUV 4:2:2 frames in
// [`super::packed_yuv_8bit`].

/// Errors returned by [`Uyyvyy411Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Uyyvyy411FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `width` was not a multiple of 4. Packed YUV 4:1:1 shares one
  /// chroma pair across 4 luma samples, so each 6-byte block covers
  /// exactly 4 pixels — widths not divisible by 4 can't form a
  /// complete final block.
  #[error(transparent)]
  WidthNotMultipleOf4(WidthNotMultipleOf4),

  /// `stride < width * 3 / 2`. Each row needs `width * 3 / 2` bytes
  /// (6 bytes per 4-pixel block, 12 bpp).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `width * 3` overflows `u32` (the checked op prior to the exact
  /// `/ 2` that yields the row stride). Reachable only at extreme
  /// widths — well beyond practical raster sizes.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **UYYVYY411** frame (`AV_PIX_FMT_UYYVYY411`).
/// Single plane, 6 bytes per 4-pixel block, byte order
/// `U0, Y0, Y1, V0, Y2, Y3` — one (U, V) chroma pair shared by four
/// luma samples (4:1:1 horizontal subsampling). 12 bpp.
///
/// `stride` is in **bytes** (≥ `width * 3 / 2`). `width` must be a
/// multiple of 4. Common in DV 4:1:1 NTSC capture (legacy).
#[derive(Debug, Clone, Copy)]
pub struct Uyyvyy411Frame<'a> {
  uyyvyy: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Uyyvyy411Frame<'a> {
  /// Constructs a new [`Uyyvyy411Frame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    uyyvyy: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Uyyvyy411FrameError> {
    if width == 0 || height == 0 {
      return Err(Uyyvyy411FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 3 != 0 {
      return Err(Uyyvyy411FrameError::WidthNotMultipleOf4(
        WidthNotMultipleOf4::new(width),
      ));
    }
    // `width * 3 / 2`. `width` is divisible by 4 above, so the
    // `* 3 / 2` is exact (no rounding) — but check the `* 3`
    // multiplication for u32 overflow at extreme widths.
    let min_stride = match width.checked_mul(3) {
      Some(v) => v / 2,
      None => {
        return Err(Uyyvyy411FrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(Uyyvyy411FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Uyyvyy411FrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if uyyvyy.len() < plane_min {
      return Err(Uyyvyy411FrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, uyyvyy.len()),
      ));
    }
    Ok(Self {
      uyyvyy,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Uyyvyy411Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(uyyvyy: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(uyyvyy, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Uyyvyy411Frame dimensions or plane length"),
    }
  }

  /// Packed UYYVYY plane bytes
  /// (`U0, Y0, Y1, V0, Y2, Y3, U1, Y4, Y5, V1, Y6, Y7, …`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn uyyvyy(&self) -> &'a [u8] {
    self.uyyvyy
  }
  /// Frame width in pixels (multiple of 4).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Frame height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }
  /// Byte stride (`>= width * 3 / 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}
