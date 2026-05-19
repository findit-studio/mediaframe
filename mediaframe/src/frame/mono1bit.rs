//! Validated monochrome 1-bit frame types: [`MonoFrame`],
//! with type aliases [`MonoblackFrame`] and [`MonowhiteFrame`].
//!
//! Both formats store 1 bit per pixel in a byte buffer (8 pixels per byte),
//! MSB first. The only difference is polarity:
//!
//! - `MonoblackFrame` (INVERT=false) — bit=0 → Y=0 (black), bit=1 → Y=255 (white)
//! - `MonowhiteFrame` (INVERT=true) — bit=0 → Y=255 (white), bit=1 → Y=0 (black)
//!
//! FFmpeg names: `AV_PIX_FMT_MONOBLACK`, `AV_PIX_FMT_MONOWHITE`.

use super::{GeometryOverflow, InsufficientPlane, InsufficientStride, ZeroDimension};
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

/// A validated 1-bit-per-pixel monochrome frame.
///
/// Single plane: `&[u8]`, 8 pixels per byte, MSB first.
/// Each pixel is 1 bit (0 or 1), unpacked to u8 luma on output:
///
/// - If `INVERT=false` (Monoblack): bit=0 → 0, bit=1 → 255
/// - If `INVERT=true` (Monowhite): bit=0 → 255, bit=1 → 0
///
/// Stride is in **bytes**; each row is `(width + 7) / 8` bytes minimum,
/// padded to the stride boundary.
///
/// No width-parity constraint (no chroma subsampling).
#[derive(Debug, Clone, Copy)]
pub struct MonoFrame<'a, const INVERT: bool> {
  data: &'a [u8],
  width: u32,
  height: u32,
  stride: u32, // in bytes
}

impl<'a, const INVERT: bool> MonoFrame<'a, INVERT> {
  /// Constructs a new [`MonoFrame`], validating dimensions and plane length.
  ///
  /// Returns [`MonoFrameError`] if:
  /// - `width` or `height` is zero,
  /// - `stride < width.div_ceil(8)`, or
  /// - `data.len() < stride * height` (with overflow check).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    data: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, MonoFrameError> {
    if width == 0 || height == 0 {
      return Err(MonoFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = width.div_ceil(8);
    if stride < min_stride {
      return Err(MonoFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let data_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(MonoFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if data.len() < data_min {
      return Err(MonoFrameError::InsufficientDataPlane(
        InsufficientPlane::new(data_min, data.len()),
      ));
    }
    Ok(Self {
      data,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`MonoFrame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(data: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(data, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid MonoFrame dimensions or plane length"),
    }
  }

  /// Data (monochrome bit buffer) plane. Row `r` starts at byte offset
  /// `r * stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data(&self) -> &'a [u8] {
    self.data
  }

  /// Frame width in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Frame height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Byte stride of the data plane (`>= width.div_ceil(8)`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Data plane (monochrome bit buffer). Same as [`Self::data`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u8] {
    self.data
  }

  /// Byte stride of the data plane. Same as [`Self::stride`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.stride
  }
}

/// Type alias for Monoblack frames (INVERT=false): bit=0 → black, bit=1 → white.
pub type MonoblackFrame<'a> = MonoFrame<'a, false>;

/// Type alias for Monowhite frames (INVERT=true): bit=0 → white, bit=1 → black.
pub type MonowhiteFrame<'a> = MonoFrame<'a, true>;

/// Errors returned by [`MonoFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum MonoFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < ceil(width / 8)`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Data plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientDataPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}
