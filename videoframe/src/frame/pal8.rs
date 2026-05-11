//! Validated source frame for the 8-bit indexed-color (`AV_PIX_FMT_PAL8`) format.
//!
//! Each pixel is a `u8` index into a 256-entry BGRA palette carried
//! alongside the pixel buffer. See [`Pal8Frame::try_new`] for layout details.

use super::{GeometryOverflow, InsufficientPlane, InsufficientStride, ZeroDimension};
use derive_more::IsVariant;
use thiserror::Error;

/// Error returned by [`Pal8Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Pal8FrameError {
  /// `width` or `height` was zero.
  #[error("zero width or height: {}×{}", .0.width(), .0.height())]
  ZeroDimension(ZeroDimension),

  /// `stride < width`.
  #[error("stride {} < width {}", .0.stride(), .0.min())]
  InsufficientStride(InsufficientStride),

  /// `stride * height` overflows `usize` (can only fire on 32-bit targets).
  #[error("geometry overflow: stride {} × rows {} exceeds usize", .0.stride(), .0.rows())]
  GeometryOverflow(GeometryOverflow),

  /// Pixel data is shorter than `stride * height` bytes.
  #[error("pixel data too short: expected >= {} bytes, got {}", .0.expected(), .0.actual())]
  InsufficientPlane(InsufficientPlane),
}

/// A validated 8-bit indexed-color (`AV_PIX_FMT_PAL8`) source frame.
///
/// `data` holds one `u8` index per pixel (row-major, with `stride`
/// bytes per row). `palette` is the 256-entry BGRA lookup table:
/// each entry is `[B, G, R, A]` per FFmpeg's `AV_PIX_FMT_PAL8`
/// convention.
#[derive(Debug, Clone, Copy)]
pub struct Pal8Frame<'a> {
  data: &'a [u8],
  palette: &'a [[u8; 4]; 256],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Pal8Frame<'a> {
  /// Constructs and validates a [`Pal8Frame`].
  ///
  /// Returns [`Pal8FrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - `stride < width`,
  /// - `stride * height` overflows `usize`, or
  /// - `data.len() < stride * height`.
  ///
  /// The `palette` slice is always exactly 256 entries — Rust's type
  /// system enforces this; no runtime palette-length check is needed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    data: &'a [u8],
    palette: &'a [[u8; 4]; 256],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Pal8FrameError> {
    if width == 0 || height == 0 {
      return Err(Pal8FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if stride < width {
      return Err(Pal8FrameError::InsufficientStride(InsufficientStride::new(
        stride, width,
      )));
    }
    let min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Pal8FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if data.len() < min {
      return Err(Pal8FrameError::InsufficientPlane(InsufficientPlane::new(
        min,
        data.len(),
      )));
    }
    Ok(Self {
      data,
      palette,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Pal8Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    data: &'a [u8],
    palette: &'a [[u8; 4]; 256],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Self {
    match Self::try_new(data, palette, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Pal8Frame dimensions or plane length"),
    }
  }

  /// The pixel index buffer. Row `r` starts at byte offset `r * stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data(&self) -> &'a [u8] {
    self.data
  }

  /// The 256-entry BGRA palette. Each entry is `[B, G, R, A]` per
  /// FFmpeg's `AV_PIX_FMT_PAL8` convention.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn palette(&self) -> &'a [[u8; 4]; 256] {
    self.palette
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

  /// Byte stride of the pixel plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}
