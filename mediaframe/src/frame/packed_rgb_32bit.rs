//! Packed 32-bit-per-channel integer RGB / RGBA source frames:
//! - `AV_PIX_FMT_RGB96{LE,BE}`  → [`Rgb96Frame`]   (R, G, B;    stride in u32 elements ≥ 3 × width)
//! - `AV_PIX_FMT_RGBA128{LE,BE}` → [`Rgba128Frame`] (R, G, B, A; stride in u32 elements ≥ 4 × width)
//!
//! These are the full-bit `u32` twins of the 16-bit
//! [`Rgb48Frame`](super::Rgb48Frame) / [`Rgba64Frame`](super::Rgba64Frame)
//! families — all 32 bits per channel are active, with no stray-bit contract.
//! Unlike the float `RGBAF32` format, these carry *integer* samples (FFmpeg
//! does not set `AV_PIX_FMT_FLAG_FLOAT` on them).
//!
//! # Endian contract — `<const BE: bool = false>`
//!
//! Each frame type carries a `<const BE: bool>` parameter that defaults to
//! `false` (LE-encoded bytes). The parameter encodes the **byte order of the
//! plane bytes**, matching the FFmpeg `*LE` / `*BE` pixel-format suffix in the
//! format name:
//!
//! - `BE = false` (`Rgb96Frame<'_, false>` aka [`Rgb96LeFrame`]) — plane bytes
//!   are LE-encoded, matching `AV_PIX_FMT_RGB96LE`. On a little-endian host
//!   (every CI runner today) LE bytes _are_ host-native, so `&[u32]` is also a
//!   host-native u32 slice; on a big-endian host the bytes have to be
//!   byte-swapped back to host-native before arithmetic.
//! - `BE = true` (`Rgb96Frame<'_, true>` aka [`Rgb96BeFrame`]) — plane bytes
//!   are BE-encoded, matching `AV_PIX_FMT_RGB96BE`. On a little-endian host
//!   the bytes are byte-swapped before arithmetic; on a big-endian host they
//!   are host-native.
//!
//! Downstream row kernels handle the byte-swap (or no-op) under the hood —
//! callers do **not** pre-swap. The `BE` parameter on `Frame` propagates
//! through the walker (`rgb96_to::<BE>(...)`) into the sinker dispatch
//! (`MixedSinker<Rgb96<BE>>`), which monomorphizes the kernel call as
//! `rgb96_to_*_row_endian::<BE>(...)`.
//!
//! Stride is in **u32 elements** (not bytes). Callers holding a raw FFmpeg
//! byte buffer should cast via `bytemuck::cast_slice` (which checks alignment
//! at runtime) and divide `linesize[0]` by 4 before constructing. Direct
//! pointer casts to `&[u32]` are undefined behaviour if the byte buffer is
//! not 4-byte aligned.

use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, WidthOverflow, ZeroDimension,
};
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

// ---- Rgb96Frame --------------------------------------------------------------

/// Errors returned by [`Rgb96Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Rgb96FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 3 * width` (in u32 elements).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` u32 elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `3 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGB96** frame (`AV_PIX_FMT_RGB96{LE,BE}`) — three
/// `u32` samples per pixel in `R, G, B` order. All 32 bits per channel are
/// active (full-bit; no stray-bit contract).
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes (`AV_PIX_FMT_RGB96LE`), `true` → BE-encoded
/// bytes (`AV_PIX_FMT_RGB96BE`). Downstream row kernels handle the byte-swap
/// (or no-op) under the hood — callers do **not** pre-swap.
///
/// `stride` is in **u32 elements** (≥ `3 * width`). Callers holding byte
/// buffers from FFmpeg should cast via `bytemuck::cast_slice` and divide
/// `linesize[0]` by 4 before constructing.
///
/// # Aliases
/// - [`Rgb96LeFrame`] = `Rgb96Frame<'a, false>` — explicit LE.
/// - [`Rgb96BeFrame`] = `Rgb96Frame<'a, true>` — explicit BE.
#[derive(Debug, Clone, Copy)]
pub struct Rgb96Frame<'a, const BE: bool = false> {
  rgb96: &'a [u32],
  width: u32,
  height: u32,
  stride: u32,
}

/// LE-encoded `Rgb96Frame` (`AV_PIX_FMT_RGB96LE`). Equivalent to the default
/// `Rgb96Frame<'a>`; provided as an explicit alias for callers who want to
/// document the endianness at the type level.
pub type Rgb96LeFrame<'a> = Rgb96Frame<'a, false>;

/// BE-encoded `Rgb96Frame` (`AV_PIX_FMT_RGB96BE`). Plane bytes are
/// big-endian-encoded `u32` samples; downstream row kernels byte-swap under
/// the hood.
pub type Rgb96BeFrame<'a> = Rgb96Frame<'a, true>;

impl<'a, const BE: bool> Rgb96Frame<'a, BE> {
  /// Constructs a new [`Rgb96Frame`], validating dimensions and plane length.
  ///
  /// The `<const BE: bool>` parameter selects whether the supplied `rgb96`
  /// slice is interpreted as LE-encoded bytes (`BE = false`, default) or
  /// BE-encoded bytes (`BE = true`). The byte-swap is performed inside the
  /// row kernels — this constructor does no I/O on the bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb96: &'a [u32],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Rgb96FrameError> {
    if width == 0 || height == 0 {
      return Err(Rgb96FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(3) {
      Some(v) => v,
      None => return Err(Rgb96FrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(Rgb96FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Rgb96FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgb96.len() < plane_min {
      return Err(Rgb96FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        rgb96.len(),
      )));
    }
    Ok(Self {
      rgb96,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb96Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb96: &'a [u32], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb96, width, height, stride) {
      Ok(f) => f,
      Err(_) => panic!("invalid Rgb96Frame dimensions or plane length"),
    }
  }

  /// Packed RGB96 plane — `width * 3` u32 elements per row (`R, G, B` per pixel).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb96(&self) -> &'a [u32] {
    self.rgb96
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
  /// Stride in u32 elements (≥ `3 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_RGB96BE`), `false` if LE-encoded (`AV_PIX_FMT_RGB96LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

// ---- Rgba128Frame ------------------------------------------------------------

/// Errors returned by [`Rgba128Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Rgba128FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width` (in u32 elements).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` u32 elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGBA128** frame (`AV_PIX_FMT_RGBA128{LE,BE}`) — four
/// `u32` samples per pixel in `R, G, B, A` order. The alpha channel is real
/// (not padding) and is passed through by `with_rgba` / `with_rgba_u16`. All
/// 32 bits per channel are active (full-bit; no stray-bit contract).
///
/// The `<const BE: bool>` parameter selects the plane byte order; see
/// [`Rgb96Frame`] for the full contract.
///
/// # Aliases
/// - [`Rgba128LeFrame`] = `Rgba128Frame<'a, false>`.
/// - [`Rgba128BeFrame`] = `Rgba128Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Rgba128Frame<'a, const BE: bool = false> {
  rgba128: &'a [u32],
  width: u32,
  height: u32,
  stride: u32,
}

/// LE-encoded `Rgba128Frame` (`AV_PIX_FMT_RGBA128LE`).
pub type Rgba128LeFrame<'a> = Rgba128Frame<'a, false>;

/// BE-encoded `Rgba128Frame` (`AV_PIX_FMT_RGBA128BE`).
pub type Rgba128BeFrame<'a> = Rgba128Frame<'a, true>;

impl<'a, const BE: bool> Rgba128Frame<'a, BE> {
  /// Constructs a new [`Rgba128Frame`], validating dimensions and plane length.
  /// `<const BE: bool>` selects LE (`false`, default) vs BE (`true`) plane
  /// byte order; row kernels perform the byte-swap.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgba128: &'a [u32],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Rgba128FrameError> {
    if width == 0 || height == 0 {
      return Err(Rgba128FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(Rgba128FrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(Rgba128FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Rgba128FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgba128.len() < plane_min {
      return Err(Rgba128FrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgba128.len()),
      ));
    }
    Ok(Self {
      rgba128,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgba128Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgba128: &'a [u32], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgba128, width, height, stride) {
      Ok(f) => f,
      Err(_) => panic!("invalid Rgba128Frame dimensions or plane length"),
    }
  }

  /// Packed RGBA128 plane — `width * 4` u32 elements per row (`R, G, B, A` per pixel).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgba128(&self) -> &'a [u32] {
    self.rgba128
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
  /// Stride in u32 elements (≥ `4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}
