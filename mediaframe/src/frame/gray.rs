//! Validated gray-scale frame types: [`Gray8Frame`], [`GrayNFrame`]
//! (covers Gray9/10/12/14), [`Gray16Frame`], [`Gray32Frame`],
//! [`Grayf16Frame`], [`Grayf32Frame`], [`Ya8Frame`], [`Ya16Frame`],
//! [`Yaf16Frame`], and [`Yaf32Frame`].
//!
//! All are 1-plane formats — the single plane carries the entire pixel
//! payload (the `Ya*` family interleaves luma and alpha within that one
//! plane). No chroma planes exist.
//!
//! - `Grayf32Frame<const BE>` — single f32 plane (FFmpeg `grayf32{le,be}`),
//!   stride in f32 elements.
//! - `Grayf16Frame<const BE>` — single `half::f16` plane (FFmpeg
//!   `grayf16{le,be}`), stride in f16 elements.
//! - `Ya8Frame` — single u8 packed plane `[Y, A, Y, A, ...]` (FFmpeg `ya8`).
//!   No `<const BE>` — 8-bit byte order is identity.
//! - `Ya16Frame<const BE>` — single u16 packed plane `[Y, A, Y, A, ...]`
//!   (FFmpeg `ya16{le,be}`).
//! - `Yaf16Frame<const BE>` — single `half::f16` packed plane
//!   `[Y, A, Y, A, ...]` (FFmpeg `yaf16{le,be}`).
//! - `Yaf32Frame<const BE>` — single f32 packed plane `[Y, A, Y, A, ...]`
//!   (FFmpeg `yaf32{le,be}`).
//!
//! - `Gray8Frame` — 1 plane of `u8` (FFmpeg `gray` / `AV_PIX_FMT_GRAY8`). No
//!   `<const BE>` — 8-bit byte order is identity.
//! - `GrayNFrame<BITS, const BE>` — 1 plane of `u16`, `BITS` active low bits
//!   (FFmpeg `gray9{le,be}` / `gray10{le,be}` / `gray12{le,be}` / `gray14{le,be}`).
//! - `Gray16Frame<const BE>` — 1 plane of `u16`, all 16 bits active
//!   (FFmpeg `gray16{le,be}`).
//! - `Gray32Frame<const BE>` — 1 plane of `u32`, all 32 bits active
//!   (FFmpeg `gray32{le,be}`).
//!
//! # Endian contract — `<const BE: bool = false>`
//!
//! Each high-bit / float frame type carries a `<const BE: bool>` parameter
//! that defaults to `false` (LE-encoded bytes). The parameter encodes the
//! **byte order of the plane bytes**, matching the FFmpeg `*LE` / `*BE`
//! pixel-format suffix. Downstream row kernels handle the byte-swap (or
//! no-op) under the hood — callers do **not** pre-swap. The `BE` parameter
//! propagates through the walker (e.g. `gray16_to::<BE>(...)`) into the
//! sinker dispatch (e.g. `MixedSinker<Gray16<BE>>`), which monomorphizes
//! the kernel call as `gray16_to_*_row::<BE>(...)`.
//!
//! 8-bit formats (`Gray8`, `Ya8`) are **not** const-generic on `BE` because
//! single-byte values have no byte order to swap.

use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, UnsupportedBits, WidthOverflow,
  ZeroDimension,
};
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

// ---- Gray8Frame -----------------------------------------------------------

/// A validated 8-bit gray-scale frame.
///
/// Single plane:
/// - `y` — full-size luma, `y_stride >= width`, length `>= y_stride * height`.
///
/// No width-parity constraint (gray has no chroma to subsample).
#[derive(Debug, Clone, Copy)]
pub struct Gray8Frame<'a> {
  y: &'a [u8],
  width: u32,
  height: u32,
  y_stride: u32,
}

impl<'a> Gray8Frame<'a> {
  /// Constructs a new [`Gray8Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Gray8FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `y_stride < width`, or
  /// - `y.len() < y_stride * height` (with overflow check on 32-bit targets).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [u8],
    width: u32,
    height: u32,
    y_stride: u32,
  ) -> Result<Self, Gray8FrameError> {
    if width == 0 || height == 0 {
      return Err(Gray8FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Gray8FrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gray8FrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Gray8FrameError::InsufficientYPlane(InsufficientPlane::new(
        y_min,
        y.len(),
      )));
    }
    Ok(Self {
      y,
      width,
      height,
      y_stride,
    })
  }

  /// Constructs a new [`Gray8Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(y: &'a [u8], width: u32, height: u32, y_stride: u32) -> Self {
    match Self::try_new(y, width, height, y_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Gray8Frame dimensions or plane length"),
    }
  }

  /// Y (luma) plane bytes. Row `r` starts at byte offset `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u8] {
    self.y
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

  /// Byte stride of the Y plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
}

/// Errors returned by [`Gray8Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Gray8FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` bytes.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- GrayNFrame<BITS> ------------------------------------------------------

/// A validated high-bit-depth gray-scale frame (9/10/12/14 bits).
///
/// Single `u16` plane with `BITS` active low bits per sample (low-bit-packed,
/// matching FFmpeg `gray9{le,be}` / `gray10{le,be}` / `gray12{le,be}` /
/// `gray14{le,be}`). Upper `16 - BITS` bits of each sample are expected to
/// be zero; the kernels AND-mask every load to `(1 << BITS) - 1` for backend
/// consistency.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes, `true` → BE-encoded bytes. Downstream row
/// kernels perform the byte-swap (or no-op) under the hood — callers do
/// **not** pre-swap.
///
/// Stride is in **samples** (`u16` elements), not bytes. Callers with byte
/// buffers from FFmpeg should cast via `bytemuck::cast_slice` and divide
/// `linesize[0]` by 2 before constructing.
#[derive(Debug, Clone, Copy)]
pub struct GrayNFrame<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> GrayNFrame<'a, BITS, BE> {
  /// Constructs a new [`GrayNFrame`], validating dimensions, plane length,
  /// and the `BITS` parameter (`BITS` must be 9, 10, 12, or 14).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
  ) -> Result<Self, GrayNFrameError> {
    if BITS != 9 && BITS != 10 && BITS != 12 && BITS != 14 {
      return Err(GrayNFrameError::UnsupportedBits(UnsupportedBits::new(BITS)));
    }
    if width == 0 || height == 0 {
      return Err(GrayNFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(GrayNFrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GrayNFrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(GrayNFrameError::InsufficientYPlane(InsufficientPlane::new(
        y_min,
        y.len(),
      )));
    }
    Ok(Self {
      y,
      width,
      height,
      y_stride,
    })
  }

  /// Constructs a new [`GrayNFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(y: &'a [u16], width: u32, height: u32, y_stride: u32) -> Self {
    match Self::try_new(y, width, height, y_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid GrayNFrame dimensions or plane length"),
    }
  }

  /// Y (luma) plane samples. Row `r` starts at element offset `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
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

  /// Sample stride of the Y plane (`>= width`, in `u16` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`gray*be`), `false` if LE-encoded (`gray*le`). Runtime mirror of the
  /// `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// 9-bit low-packed gray frame (FFmpeg `gray9{le,be}`). Each sample is a `u16`
/// with the low 9 bits active; the upper 7 bits are zero (or ignored).
/// `<const BE>` defaults to `false` (LE).
pub type Gray9Frame<'a, const BE: bool = false> = GrayNFrame<'a, 9, BE>;
/// 10-bit low-packed gray frame (FFmpeg `gray10{le,be}`). `<const BE>`
/// defaults to `false` (LE).
pub type Gray10Frame<'a, const BE: bool = false> = GrayNFrame<'a, 10, BE>;
/// 12-bit low-packed gray frame (FFmpeg `gray12{le,be}`). `<const BE>`
/// defaults to `false` (LE).
pub type Gray12Frame<'a, const BE: bool = false> = GrayNFrame<'a, 12, BE>;
/// 14-bit low-packed gray frame (FFmpeg `gray14{le,be}`). `<const BE>`
/// defaults to `false` (LE).
pub type Gray14Frame<'a, const BE: bool = false> = GrayNFrame<'a, 14, BE>;

/// LE-encoded `Gray9Frame` (`AV_PIX_FMT_GRAY9LE`).
pub type Gray9LeFrame<'a> = GrayNFrame<'a, 9, false>;
/// BE-encoded `Gray9Frame` (`AV_PIX_FMT_GRAY9BE`).
pub type Gray9BeFrame<'a> = GrayNFrame<'a, 9, true>;
/// LE-encoded `Gray10Frame` (`AV_PIX_FMT_GRAY10LE`).
pub type Gray10LeFrame<'a> = GrayNFrame<'a, 10, false>;
/// BE-encoded `Gray10Frame` (`AV_PIX_FMT_GRAY10BE`).
pub type Gray10BeFrame<'a> = GrayNFrame<'a, 10, true>;
/// LE-encoded `Gray12Frame` (`AV_PIX_FMT_GRAY12LE`).
pub type Gray12LeFrame<'a> = GrayNFrame<'a, 12, false>;
/// BE-encoded `Gray12Frame` (`AV_PIX_FMT_GRAY12BE`).
pub type Gray12BeFrame<'a> = GrayNFrame<'a, 12, true>;
/// LE-encoded `Gray14Frame` (`AV_PIX_FMT_GRAY14LE`).
pub type Gray14LeFrame<'a> = GrayNFrame<'a, 14, false>;
/// BE-encoded `Gray14Frame` (`AV_PIX_FMT_GRAY14BE`).
pub type Gray14BeFrame<'a> = GrayNFrame<'a, 14, true>;

/// Errors returned by [`GrayNFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum GrayNFrameError {
  /// `BITS` must be 9, 10, 12, or 14.
  #[error(transparent)]
  UnsupportedBits(UnsupportedBits),

  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- Gray16Frame -----------------------------------------------------------

/// A validated 16-bit gray-scale frame.
///
/// Single `u16` plane, all 16 bits active (FFmpeg `gray16{le,be}`).
/// Stride is in **samples** (`u16` elements), not bytes.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes (`AV_PIX_FMT_GRAY16LE`), `true` → BE-encoded
/// bytes (`AV_PIX_FMT_GRAY16BE`). Downstream row kernels handle the byte-swap.
///
/// # Aliases
/// - [`Gray16LeFrame`] = `Gray16Frame<'a, false>`.
/// - [`Gray16BeFrame`] = `Gray16Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Gray16Frame<'a, const BE: bool = false> {
  y: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
}

/// LE-encoded `Gray16Frame` (`AV_PIX_FMT_GRAY16LE`).
pub type Gray16LeFrame<'a> = Gray16Frame<'a, false>;

/// BE-encoded `Gray16Frame` (`AV_PIX_FMT_GRAY16BE`).
pub type Gray16BeFrame<'a> = Gray16Frame<'a, true>;

impl<'a, const BE: bool> Gray16Frame<'a, BE> {
  /// Constructs a new [`Gray16Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
  ) -> Result<Self, Gray16FrameError> {
    if width == 0 || height == 0 {
      return Err(Gray16FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Gray16FrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gray16FrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Gray16FrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    Ok(Self {
      y,
      width,
      height,
      y_stride,
    })
  }

  /// Constructs a new [`Gray16Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(y: &'a [u16], width: u32, height: u32, y_stride: u32) -> Self {
    match Self::try_new(y, width, height, y_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Gray16Frame dimensions or plane length"),
    }
  }

  /// Y (luma) plane samples. Row `r` starts at element offset `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
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

  /// Sample stride of the Y plane (`>= width`, in `u16` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_GRAY16BE`), `false` if LE-encoded (`AV_PIX_FMT_GRAY16LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Gray16Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Gray16FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- Grayf32Frame -----------------------------------------------------------

/// A validated 32-bit float gray-scale frame (FFmpeg `grayf32{le,be}`).
///
/// Single `f32` plane. Nominal luma range `[0.0, 1.0]`; HDR > 1.0 is permitted
/// and not rejected at construction. Out-of-range values are clamped during
/// output conversion.
///
/// The `<const BE: bool>` parameter selects the **bit-pattern byte order** of
/// each `f32` element: `false` (default) → LE-encoded bytes
/// (`AV_PIX_FMT_GRAYF32LE`), `true` → BE-encoded bytes
/// (`AV_PIX_FMT_GRAYF32BE`). Downstream row kernels load each `f32` via a
/// byte-swapped `u32` bit pattern when `BE = true` — callers do **not**
/// pre-swap.
///
/// Stride is in **f32 elements** (not bytes). Callers holding a byte buffer
/// from FFmpeg should cast via `bytemuck::cast_slice` and divide
/// `linesize[0]` by 4 before constructing.
///
/// # Aliases
/// - [`Grayf32LeFrame`] = `Grayf32Frame<'a, false>`.
/// - [`Grayf32BeFrame`] = `Grayf32Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Grayf32Frame<'a, const BE: bool = false> {
  y: &'a [f32],
  width: u32,
  height: u32,
  y_stride: u32, // in f32 elements
}

/// LE-encoded `Grayf32Frame` (`AV_PIX_FMT_GRAYF32LE`).
pub type Grayf32LeFrame<'a> = Grayf32Frame<'a, false>;

/// BE-encoded `Grayf32Frame` (`AV_PIX_FMT_GRAYF32BE`).
pub type Grayf32BeFrame<'a> = Grayf32Frame<'a, true>;

impl<'a, const BE: bool> Grayf32Frame<'a, BE> {
  /// Constructs a new [`Grayf32Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Grayf32FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `y_stride < width`, or
  /// - `y.len() < y_stride * height` (with overflow check on 32-bit targets).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [f32],
    width: u32,
    height: u32,
    y_stride: u32,
  ) -> Result<Self, Grayf32FrameError> {
    if width == 0 || height == 0 {
      return Err(Grayf32FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Grayf32FrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Grayf32FrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Grayf32FrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    Ok(Self {
      y,
      width,
      height,
      y_stride,
    })
  }

  /// Constructs a new [`Grayf32Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(y: &'a [f32], width: u32, height: u32, y_stride: u32) -> Self {
    match Self::try_new(y, width, height, y_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Grayf32Frame dimensions or plane length"),
    }
  }

  /// Y (luma) plane f32 elements. Row `r` starts at element offset `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [f32] {
    self.y
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

  /// Stride of the Y plane in f32 elements (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_GRAYF32BE`), `false` if LE-encoded
  /// (`AV_PIX_FMT_GRAYF32LE`). Runtime mirror of the `<const BE: bool>` type
  /// parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Grayf32Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Grayf32FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` f32 elements.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- Ya8Frame ---------------------------------------------------------------

/// A validated 8-bit gray + alpha packed frame (FFmpeg `ya8` / `AV_PIX_FMT_YA8`).
///
/// Single `u8` plane in packed `[Y0, A0, Y1, A1, ...]` layout. Each pixel
/// occupies 2 bytes: the luma Y byte followed by the alpha A byte.
///
/// Stride is in **bytes** (stride covers `width × 2` bytes per active row,
/// plus any padding). Callers from FFmpeg can use `linesize[0]` directly.
#[derive(Debug, Clone, Copy)]
pub struct Ya8Frame<'a> {
  packed: &'a [u8],
  width: u32,
  height: u32,
  stride: u32, // in bytes
}

impl<'a> Ya8Frame<'a> {
  /// Constructs a new [`Ya8Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Ya8FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `stride < width * 2` (too narrow for 2 bytes/pixel),
  /// - `stride * height` overflows `usize`, or
  /// - `packed.len() < stride * height`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    packed: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Ya8FrameError> {
    if width == 0 || height == 0 {
      return Err(Ya8FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(Ya8FrameError::WidthOverflow(WidthOverflow::new(width)));
      }
    };
    if stride < min_stride {
      return Err(Ya8FrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Ya8FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if packed.len() < plane_min {
      return Err(Ya8FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        packed.len(),
      )));
    }
    Ok(Self {
      packed,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Ya8Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(packed: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(packed, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Ya8Frame dimensions or plane length"),
    }
  }

  /// Packed `[Y, A, Y, A, ...]` u8 plane. Row `r` starts at byte offset `r * stride()`.
  /// Each active row contains `width * 2` bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn packed(&self) -> &'a [u8] {
    self.packed
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

  /// Row stride in bytes (`>= width * 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`Ya8Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Ya8FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < width * 2` (too narrow to fit 2 bytes per pixel).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Packed plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `width * 2` overflows `u32` (only reachable when `width > 2^31`).
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

// ---- Ya16Frame --------------------------------------------------------------

/// A validated 16-bit gray + alpha packed frame
/// (FFmpeg `ya16{le,be}` / `AV_PIX_FMT_YA16{LE,BE}`).
///
/// Single `u16` plane in packed `[Y0, A0, Y1, A1, ...]` layout. Each pixel
/// occupies 2 u16 elements: the luma Y element followed by the alpha A element.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes (`AV_PIX_FMT_YA16LE`), `true` → BE-encoded
/// bytes (`AV_PIX_FMT_YA16BE`). Downstream row kernels handle the byte-swap.
///
/// Stride is in **u16 elements** (stride covers `width × 2` elements per active
/// row, plus any padding). Callers from FFmpeg should divide `linesize[0]` by 2.
///
/// # Aliases
/// - [`Ya16LeFrame`] = `Ya16Frame<'a, false>`.
/// - [`Ya16BeFrame`] = `Ya16Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Ya16Frame<'a, const BE: bool = false> {
  packed: &'a [u16],
  width: u32,
  height: u32,
  stride: u32, // in u16 elements
}

/// LE-encoded `Ya16Frame` (`AV_PIX_FMT_YA16LE`).
pub type Ya16LeFrame<'a> = Ya16Frame<'a, false>;

/// BE-encoded `Ya16Frame` (`AV_PIX_FMT_YA16BE`).
pub type Ya16BeFrame<'a> = Ya16Frame<'a, true>;

impl<'a, const BE: bool> Ya16Frame<'a, BE> {
  /// Constructs a new [`Ya16Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Ya16FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `stride < width * 2` (too narrow for 2 u16/pixel),
  /// - `stride * height` overflows `usize`, or
  /// - `packed.len() < stride * height`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    packed: &'a [u16],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Ya16FrameError> {
    if width == 0 || height == 0 {
      return Err(Ya16FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(Ya16FrameError::WidthOverflow(WidthOverflow::new(width)));
      }
    };
    if stride < min_stride {
      return Err(Ya16FrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Ya16FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if packed.len() < plane_min {
      return Err(Ya16FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        packed.len(),
      )));
    }
    Ok(Self {
      packed,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Ya16Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(packed: &'a [u16], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(packed, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Ya16Frame dimensions or plane length"),
    }
  }

  /// Packed `[Y, A, Y, A, ...]` u16 plane. Row `r` starts at element offset
  /// `r * stride()`. Each active row contains `width * 2` u16 elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn packed(&self) -> &'a [u16] {
    self.packed
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

  /// Row stride in u16 elements (`>= width * 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YA16BE`), `false` if LE-encoded (`AV_PIX_FMT_YA16LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Ya16Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Ya16FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < width * 2` (too narrow to fit 2 u16 per pixel).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Packed plane is shorter than `stride * height` u16 elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `width * 2` overflows `u32` (only reachable when `width > 2^31`).
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

// ---- Gray32Frame -----------------------------------------------------------

/// A validated 32-bit (integer) gray-scale frame.
///
/// Single `u32` plane, all 32 bits active (FFmpeg `gray32{le,be}`). The
/// full-bit integer twin of [`Gray16Frame`] — no stray-bit contract,
/// since every bit of each sample carries luma signal. Stride is in
/// **samples** (`u32` elements), not bytes.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes (`AV_PIX_FMT_GRAY32LE`), `true` → BE-encoded
/// bytes (`AV_PIX_FMT_GRAY32BE`). Downstream row kernels handle the byte-swap.
///
/// # Aliases
/// - [`Gray32LeFrame`] = `Gray32Frame<'a, false>`.
/// - [`Gray32BeFrame`] = `Gray32Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Gray32Frame<'a, const BE: bool = false> {
  y: &'a [u32],
  width: u32,
  height: u32,
  y_stride: u32,
}

/// LE-encoded `Gray32Frame` (`AV_PIX_FMT_GRAY32LE`).
pub type Gray32LeFrame<'a> = Gray32Frame<'a, false>;

/// BE-encoded `Gray32Frame` (`AV_PIX_FMT_GRAY32BE`).
pub type Gray32BeFrame<'a> = Gray32Frame<'a, true>;

impl<'a, const BE: bool> Gray32Frame<'a, BE> {
  /// Constructs a new [`Gray32Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [u32],
    width: u32,
    height: u32,
    y_stride: u32,
  ) -> Result<Self, Gray32FrameError> {
    if width == 0 || height == 0 {
      return Err(Gray32FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Gray32FrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gray32FrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Gray32FrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    Ok(Self {
      y,
      width,
      height,
      y_stride,
    })
  }

  /// Constructs a new [`Gray32Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(y: &'a [u32], width: u32, height: u32, y_stride: u32) -> Self {
    match Self::try_new(y, width, height, y_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Gray32Frame dimensions or plane length"),
    }
  }

  /// Y (luma) plane samples. Row `r` starts at element offset `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u32] {
    self.y
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

  /// Sample stride of the Y plane (`>= width`, in `u32` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_GRAY32BE`), `false` if LE-encoded (`AV_PIX_FMT_GRAY32LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Gray32Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Gray32FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- Grayf16Frame -----------------------------------------------------------

/// A validated 16-bit half-float gray-scale frame (FFmpeg `grayf16{le,be}`).
///
/// Single [`half::f16`] plane. Nominal luma range `[0.0, 1.0]`; HDR > 1.0 is
/// permitted and not rejected at construction. Out-of-range values are clamped
/// during output conversion. The half-float twin of [`Grayf32Frame`].
///
/// The `<const BE: bool>` parameter selects the **bit-pattern byte order** of
/// each `f16` element: `false` (default) → LE-encoded bytes
/// (`AV_PIX_FMT_GRAYF16LE`), `true` → BE-encoded bytes
/// (`AV_PIX_FMT_GRAYF16BE`). Downstream row kernels load each `f16` via a
/// byte-swapped `u16` bit pattern when `BE = true` — callers do **not**
/// pre-swap.
///
/// Stride is in **f16 elements** (not bytes). Callers holding a byte buffer
/// from FFmpeg should cast via `bytemuck::cast_slice` and divide
/// `linesize[0]` by 2 before constructing.
///
/// # Aliases
/// - [`Grayf16LeFrame`] = `Grayf16Frame<'a, false>`.
/// - [`Grayf16BeFrame`] = `Grayf16Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Grayf16Frame<'a, const BE: bool = false> {
  y: &'a [half::f16],
  width: u32,
  height: u32,
  y_stride: u32, // in f16 elements
}

/// LE-encoded `Grayf16Frame` (`AV_PIX_FMT_GRAYF16LE`).
pub type Grayf16LeFrame<'a> = Grayf16Frame<'a, false>;

/// BE-encoded `Grayf16Frame` (`AV_PIX_FMT_GRAYF16BE`).
pub type Grayf16BeFrame<'a> = Grayf16Frame<'a, true>;

impl<'a, const BE: bool> Grayf16Frame<'a, BE> {
  /// Constructs a new [`Grayf16Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Grayf16FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `y_stride < width`, or
  /// - `y.len() < y_stride * height` (with overflow check on 32-bit targets).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [half::f16],
    width: u32,
    height: u32,
    y_stride: u32,
  ) -> Result<Self, Grayf16FrameError> {
    if width == 0 || height == 0 {
      return Err(Grayf16FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Grayf16FrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Grayf16FrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Grayf16FrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    Ok(Self {
      y,
      width,
      height,
      y_stride,
    })
  }

  /// Constructs a new [`Grayf16Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(y: &'a [half::f16], width: u32, height: u32, y_stride: u32) -> Self {
    match Self::try_new(y, width, height, y_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Grayf16Frame dimensions or plane length"),
    }
  }

  /// Y (luma) plane f16 elements. Row `r` starts at element offset `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [half::f16] {
    self.y
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

  /// Stride of the Y plane in f16 elements (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_GRAYF16BE`), `false` if LE-encoded
  /// (`AV_PIX_FMT_GRAYF16LE`). Runtime mirror of the `<const BE: bool>` type
  /// parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Grayf16Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Grayf16FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` f16 elements.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- Yaf16Frame -------------------------------------------------------------

/// A validated 16-bit half-float gray + alpha packed frame
/// (FFmpeg `yaf16{le,be}` / `AV_PIX_FMT_YAF16{LE,BE}`).
///
/// Single [`half::f16`] plane in packed `[Y0, A0, Y1, A1, ...]` layout. Each
/// pixel occupies 2 f16 elements: the luma Y element followed by the alpha A
/// element (FFmpeg pixdesc: both components in plane 0, Y at byte offset 0, A
/// at byte offset 2, `step = 4` bytes). The half-float twin of [`Ya16Frame`].
///
/// Nominal range `[0.0, 1.0]` (opaque alpha = 1.0); HDR > 1.0 is permitted and
/// clamped at output, not at construction.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes (`AV_PIX_FMT_YAF16LE`), `true` → BE-encoded
/// bytes (`AV_PIX_FMT_YAF16BE`). Downstream row kernels handle the byte-swap.
///
/// Stride is in **f16 elements** (stride covers `width × 2` elements per active
/// row, plus any padding). Callers from FFmpeg should divide `linesize[0]` by 2.
///
/// # Aliases
/// - [`Yaf16LeFrame`] = `Yaf16Frame<'a, false>`.
/// - [`Yaf16BeFrame`] = `Yaf16Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Yaf16Frame<'a, const BE: bool = false> {
  packed: &'a [half::f16],
  width: u32,
  height: u32,
  stride: u32, // in f16 elements
}

/// LE-encoded `Yaf16Frame` (`AV_PIX_FMT_YAF16LE`).
pub type Yaf16LeFrame<'a> = Yaf16Frame<'a, false>;

/// BE-encoded `Yaf16Frame` (`AV_PIX_FMT_YAF16BE`).
pub type Yaf16BeFrame<'a> = Yaf16Frame<'a, true>;

impl<'a, const BE: bool> Yaf16Frame<'a, BE> {
  /// Constructs a new [`Yaf16Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Yaf16FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `stride < width * 2` (too narrow for 2 f16/pixel),
  /// - `stride * height` overflows `usize`, or
  /// - `packed.len() < stride * height`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    packed: &'a [half::f16],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Yaf16FrameError> {
    if width == 0 || height == 0 {
      return Err(Yaf16FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(Yaf16FrameError::WidthOverflow(WidthOverflow::new(width)));
      }
    };
    if stride < min_stride {
      return Err(Yaf16FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yaf16FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if packed.len() < plane_min {
      return Err(Yaf16FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        packed.len(),
      )));
    }
    Ok(Self {
      packed,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Yaf16Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(packed: &'a [half::f16], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(packed, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yaf16Frame dimensions or plane length"),
    }
  }

  /// Packed `[Y, A, Y, A, ...]` f16 plane. Row `r` starts at element offset
  /// `r * stride()`. Each active row contains `width * 2` f16 elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn packed(&self) -> &'a [half::f16] {
    self.packed
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

  /// Row stride in f16 elements (`>= width * 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YAF16BE`), `false` if LE-encoded (`AV_PIX_FMT_YAF16LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Yaf16Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Yaf16FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < width * 2` (too narrow to fit 2 f16 per pixel).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Packed plane is shorter than `stride * height` f16 elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `width * 2` overflows `u32` (only reachable when `width > 2^31`).
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

// ---- Yaf32Frame -------------------------------------------------------------

/// A validated 32-bit float gray + alpha packed frame
/// (FFmpeg `yaf32{le,be}` / `AV_PIX_FMT_YAF32{LE,BE}`).
///
/// Single `f32` plane in packed `[Y0, A0, Y1, A1, ...]` layout. Each pixel
/// occupies 2 f32 elements: the luma Y element followed by the alpha A element
/// (FFmpeg pixdesc: both components in plane 0, Y at byte offset 0, A at byte
/// offset 4, `step = 8` bytes). The single-precision twin of [`Yaf16Frame`].
///
/// Nominal range `[0.0, 1.0]` (opaque alpha = 1.0); HDR > 1.0 is permitted and
/// clamped at output, not at construction.
///
/// The `<const BE: bool>` parameter selects the **bit-pattern byte order** of
/// each `f32` element: `false` (default) → LE-encoded bytes
/// (`AV_PIX_FMT_YAF32LE`), `true` → BE-encoded bytes (`AV_PIX_FMT_YAF32BE`).
/// Downstream row kernels handle the byte-swap of the float bit pattern.
///
/// Stride is in **f32 elements** (stride covers `width × 2` elements per active
/// row, plus any padding). Callers from FFmpeg should divide `linesize[0]` by 4.
///
/// # Aliases
/// - [`Yaf32LeFrame`] = `Yaf32Frame<'a, false>`.
/// - [`Yaf32BeFrame`] = `Yaf32Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Yaf32Frame<'a, const BE: bool = false> {
  packed: &'a [f32],
  width: u32,
  height: u32,
  stride: u32, // in f32 elements
}

/// LE-encoded `Yaf32Frame` (`AV_PIX_FMT_YAF32LE`).
pub type Yaf32LeFrame<'a> = Yaf32Frame<'a, false>;

/// BE-encoded `Yaf32Frame` (`AV_PIX_FMT_YAF32BE`).
pub type Yaf32BeFrame<'a> = Yaf32Frame<'a, true>;

impl<'a, const BE: bool> Yaf32Frame<'a, BE> {
  /// Constructs a new [`Yaf32Frame`], validating dimensions and plane length.
  ///
  /// Returns [`Yaf32FrameError`] if:
  /// - `width` or `height` is zero,
  /// - `stride < width * 2` (too narrow for 2 f32/pixel),
  /// - `stride * height` overflows `usize`, or
  /// - `packed.len() < stride * height`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    packed: &'a [f32],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Yaf32FrameError> {
    if width == 0 || height == 0 {
      return Err(Yaf32FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(Yaf32FrameError::WidthOverflow(WidthOverflow::new(width)));
      }
    };
    if stride < min_stride {
      return Err(Yaf32FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yaf32FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if packed.len() < plane_min {
      return Err(Yaf32FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        packed.len(),
      )));
    }
    Ok(Self {
      packed,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Yaf32Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(packed: &'a [f32], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(packed, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yaf32Frame dimensions or plane length"),
    }
  }

  /// Packed `[Y, A, Y, A, ...]` f32 plane. Row `r` starts at element offset
  /// `r * stride()`. Each active row contains `width * 2` f32 elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn packed(&self) -> &'a [f32] {
    self.packed
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

  /// Row stride in f32 elements (`>= width * 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YAF32BE`), `false` if LE-encoded (`AV_PIX_FMT_YAF32LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Errors returned by [`Yaf32Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Yaf32FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < width * 2` (too narrow to fit 2 f32 per pixel).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Packed plane is shorter than `stride * height` f32 elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `width * 2` overflows `u32` (only reachable when `width > 2^31`).
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}
