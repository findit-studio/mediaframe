use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, WidthOverflow, ZeroDimension,
};
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

// ============================================================
// Tier 9 — Packed float RGB source-side frame (Rgbf32)
// ============================================================

/// Errors returned by [`Rgbf32Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Rgbf32FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 3 * width` `f32` elements. Each row needs `3 * width`
  /// `f32` samples for packed RGB float.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` `f32` elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `3 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGBF32** frame.
/// One plane, 3 × `f32` per pixel, channel order `R, G, B`.
///
/// Values are **linear** RGB by convention — no gamma / OETF handling
/// is applied by `colconv`. Caller is responsible for applying any
/// gamma / OETF transforms before constructing the frame.
///
/// HDR values (> 1.0) are permitted in the buffer; output paths that
/// target u8 / u16 saturate them to the output range. Output paths
/// targeting `f32` (`with_rgb_f32`) preserve them losslessly.
///
/// `stride` is in **`f32` elements** (≥ `3 * width`), matching the
/// per-format convention that stride aligns with the underlying slice
/// element type. No width parity constraint.
///
/// # Endian contract — `<const BE: bool = false>`
///
/// The `<const BE: bool>` parameter selects the plane byte order, matching
/// the FFmpeg `*LE` / `*BE` pixel-format suffix in the format name:
///
/// - `BE = false` (`Rgbf32Frame<'_, false>` aka [`Rgbf32LeFrame`]) — plane
///   bytes are LE-encoded, matching `AV_PIX_FMT_RGBF32LE`. On a
///   little-endian host (every CI runner today) LE bytes _are_ host-native,
///   so `&[f32]` is also a host-native float slice; on a big-endian host
///   the bytes have to be byte-swapped back to host-native (via
///   `f32::from_bits(u32::from_le(elem.to_bits()))`) before arithmetic.
/// - `BE = true` (`Rgbf32Frame<'_, true>` aka [`Rgbf32BeFrame`]) — plane
///   bytes are BE-encoded, matching `AV_PIX_FMT_RGBF32BE`. On a
///   little-endian host the bytes are byte-swapped before arithmetic; on a
///   big-endian host they are host-native.
///
/// FFmpeg also defines an unsuffixed `AV_PIX_FMT_RGBF32` alias that is
/// **target-endian** (resolves to `RGBF32LE` on LE hosts and `RGBF32BE` on
/// BE hosts). Callers holding target-endian bytes should pick the
/// `<const BE>` parameter that matches the host they were produced on.
///
/// Downstream row kernels handle the byte-swap (or no-op) under the hood —
/// callers do **not** pre-swap. The `BE` parameter on `Frame` propagates
/// through the walker (`rgbf32_to::<BE>(...)`) into the sinker dispatch
/// (`MixedSinker<Rgbf32<BE>>`), which monomorphizes the kernel call as
/// `rgbf32_to_*_row::<BE>(...)`.
///
/// Stride is in **f32 elements** (not bytes). Callers holding a byte buffer
/// from FFmpeg should cast via `bytemuck::cast_slice` and divide
/// `linesize[0]` by 4 before constructing.
#[derive(Debug, Clone, Copy)]
pub struct Rgbf32Frame<'a, const BE: bool = false> {
  rgb: &'a [f32],
  width: u32,
  height: u32,
  stride: u32,
}

/// LE-encoded `Rgbf32Frame` (`AV_PIX_FMT_RGBF32LE`). Equivalent to the
/// default `Rgbf32Frame<'a>`; provided as an explicit alias for callers who
/// want to document the endianness at the type level.
pub type Rgbf32LeFrame<'a> = Rgbf32Frame<'a, false>;

/// BE-encoded `Rgbf32Frame` (`AV_PIX_FMT_RGBF32BE`). Plane bytes are
/// big-endian-encoded `f32` samples; downstream row kernels byte-swap under
/// the hood.
pub type Rgbf32BeFrame<'a> = Rgbf32Frame<'a, true>;

impl<'a, const BE: bool> Rgbf32Frame<'a, BE> {
  /// Constructs a new [`Rgbf32Frame`], validating dimensions and
  /// plane length.
  ///
  /// The `<const BE: bool>` parameter selects whether the supplied `rgb`
  /// slice is interpreted as LE-encoded bytes (`BE = false`, default) or
  /// BE-encoded bytes (`BE = true`). The byte-swap is performed inside the
  /// row kernels — this constructor does no I/O on the bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb: &'a [f32],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Rgbf32FrameError> {
    if width == 0 || height == 0 {
      return Err(Rgbf32FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(3) {
      Some(v) => v,
      None => return Err(Rgbf32FrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(Rgbf32FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Rgbf32FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgb.len() < plane_min {
      return Err(Rgbf32FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        rgb.len(),
      )));
    }
    Ok(Self {
      rgb,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgbf32Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb: &'a [f32], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgbf32Frame dimensions or plane length"),
    }
  }

  /// Packed RGB plane (`R, G, B, R, G, B, …` per row, each value an
  /// `f32`). Length is at least `stride * height` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb(&self) -> &'a [f32] {
    self.rgb
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
  /// Stride in **`f32` elements** per row (`>= 3 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_RGBF32BE`), `false` if LE-encoded (`AV_PIX_FMT_RGBF32LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

// ============================================================
// Packed float RGBA source-side frame (Rgbaf32)
// ============================================================

/// Errors returned by [`Rgbaf32Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Rgbaf32FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width` `f32` elements. Each row needs `4 * width`
  /// `f32` samples for packed RGBA float.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` `f32` elements.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGBAF32** frame (FFmpeg `AV_PIX_FMT_RGBAF32{LE,BE}`).
/// One plane, 4 × `f32` per pixel, channel order `R, G, B, A`. The alpha
/// channel is real (not padding) and is passed through by `with_rgba` /
/// `with_rgba_u16`.
///
/// Values are **linear** RGB by convention — no gamma / OETF handling
/// is applied by `colconv`. Caller is responsible for applying any
/// gamma / OETF transforms before constructing the frame.
///
/// HDR values (> 1.0) are permitted in the buffer; output paths that
/// target u8 / u16 saturate them to the output range. Output paths
/// targeting `f32` (`with_rgb_f32`) preserve them losslessly.
///
/// `stride` is in **`f32` elements** (≥ `4 * width`), matching the
/// per-format convention that stride aligns with the underlying slice
/// element type. No width parity constraint.
///
/// # Endian contract — `<const BE: bool = false>`
///
/// The `<const BE: bool>` parameter selects the plane byte order, matching
/// the FFmpeg `*LE` / `*BE` pixel-format suffix in the format name:
///
/// - `BE = false` (`Rgbaf32Frame<'_, false>` aka [`Rgbaf32LeFrame`]) — plane
///   bytes are LE-encoded, matching `AV_PIX_FMT_RGBAF32LE`. On a
///   little-endian host (every CI runner today) LE bytes _are_ host-native,
///   so `&[f32]` is also a host-native float slice; on a big-endian host
///   the bytes have to be byte-swapped back to host-native (via
///   `f32::from_bits(u32::from_le(elem.to_bits()))`) before arithmetic.
/// - `BE = true` (`Rgbaf32Frame<'_, true>` aka [`Rgbaf32BeFrame`]) — plane
///   bytes are BE-encoded, matching `AV_PIX_FMT_RGBAF32BE`. On a
///   little-endian host the bytes are byte-swapped before arithmetic; on a
///   big-endian host they are host-native.
///
/// FFmpeg also defines an unsuffixed `AV_PIX_FMT_RGBAF32` alias that is
/// **target-endian** (resolves to `RGBAF32LE` on LE hosts and `RGBAF32BE` on
/// BE hosts). Callers holding target-endian bytes should pick the
/// `<const BE>` parameter that matches the host they were produced on.
///
/// Downstream row kernels handle the byte-swap (or no-op) under the hood —
/// callers do **not** pre-swap. The `BE` parameter on `Frame` propagates
/// through the walker (`rgbaf32_to::<BE>(...)`) into the sinker dispatch
/// (`MixedSinker<Rgbaf32<BE>>`), which monomorphizes the kernel call as
/// `rgbaf32_to_*_row::<BE>(...)`.
///
/// Stride is in **f32 elements** (not bytes). Callers holding a byte buffer
/// from FFmpeg should cast via `bytemuck::cast_slice` and divide
/// `linesize[0]` by 4 before constructing.
#[derive(Debug, Clone, Copy)]
pub struct Rgbaf32Frame<'a, const BE: bool = false> {
  rgba: &'a [f32],
  width: u32,
  height: u32,
  stride: u32,
}

/// LE-encoded `Rgbaf32Frame` (`AV_PIX_FMT_RGBAF32LE`). Equivalent to the
/// default `Rgbaf32Frame<'a>`; provided as an explicit alias for callers who
/// want to document the endianness at the type level.
pub type Rgbaf32LeFrame<'a> = Rgbaf32Frame<'a, false>;

/// BE-encoded `Rgbaf32Frame` (`AV_PIX_FMT_RGBAF32BE`). Plane bytes are
/// big-endian-encoded `f32` samples; downstream row kernels byte-swap under
/// the hood.
pub type Rgbaf32BeFrame<'a> = Rgbaf32Frame<'a, true>;

impl<'a, const BE: bool> Rgbaf32Frame<'a, BE> {
  /// Constructs a new [`Rgbaf32Frame`], validating dimensions and
  /// plane length.
  ///
  /// The `<const BE: bool>` parameter selects whether the supplied `rgba`
  /// slice is interpreted as LE-encoded bytes (`BE = false`, default) or
  /// BE-encoded bytes (`BE = true`). The byte-swap is performed inside the
  /// row kernels — this constructor does no I/O on the bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgba: &'a [f32],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Rgbaf32FrameError> {
    if width == 0 || height == 0 {
      return Err(Rgbaf32FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(Rgbaf32FrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(Rgbaf32FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Rgbaf32FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgba.len() < plane_min {
      return Err(Rgbaf32FrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgba.len()),
      ));
    }
    Ok(Self {
      rgba,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgbaf32Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgba: &'a [f32], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgba, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgbaf32Frame dimensions or plane length"),
    }
  }

  /// Packed RGBA plane (`R, G, B, A, R, G, B, A, …` per row, each value an
  /// `f32`). Length is at least `stride * height` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgba(&self) -> &'a [f32] {
    self.rgba
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
  /// Stride in **`f32` elements** per row (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_RGBAF32BE`), `false` if LE-encoded (`AV_PIX_FMT_RGBAF32LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}
