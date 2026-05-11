use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, WidthOverflow, ZeroDimension,
};
use derive_more::IsVariant;
use thiserror::Error;

// ============================================================
// Tier 6 — Packed RGB / BGR 8-bit source-side frames (Ship 9a)
// ============================================================

/// Errors returned by [`Rgb24Frame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Rgb24FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 3 * width`. Each row needs `3 * width` bytes for packed RGB.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `3 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGB24** frame at 8 bits per channel
/// (`AV_PIX_FMT_RGB24`). One plane, 3 bytes per pixel, byte order
/// `R, G, B`.
///
/// `stride` is in **bytes** (≥ `3 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb24Frame<'a> {
  rgb: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb24Frame<'a> {
  /// Constructs a new [`Rgb24Frame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Rgb24FrameError> {
    if width == 0 || height == 0 {
      return Err(Rgb24FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(3) {
      Some(v) => v,
      None => return Err(Rgb24FrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(Rgb24FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Rgb24FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgb.len() < plane_min {
      return Err(Rgb24FrameError::InsufficientPlane(InsufficientPlane::new(
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

  /// Constructs a new [`Rgb24Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb24Frame dimensions or plane length"),
    }
  }

  /// Packed RGB plane bytes (`R, G, B, R, G, B, …` per row).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb(&self) -> &'a [u8] {
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
  /// Byte stride (`>= 3 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`Bgr24Frame::try_new`]. Variant shape mirrors
/// [`Rgb24FrameError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Bgr24FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 3 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `3 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **BGR24** frame at 8 bits per channel
/// (`AV_PIX_FMT_BGR24`). One plane, 3 bytes per pixel, byte order
/// `B, G, R` — only the channel-order distinction differentiates
/// this from [`Rgb24Frame`].
///
/// `stride` is in **bytes** (≥ `3 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr24Frame<'a> {
  bgr: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr24Frame<'a> {
  /// Constructs a new [`Bgr24Frame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, Bgr24FrameError> {
    if width == 0 || height == 0 {
      return Err(Bgr24FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(3) {
      Some(v) => v,
      None => return Err(Bgr24FrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(Bgr24FrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Bgr24FrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if bgr.len() < plane_min {
      return Err(Bgr24FrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        bgr.len(),
      )));
    }
    Ok(Self {
      bgr,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr24Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr24Frame dimensions or plane length"),
    }
  }

  /// Packed BGR plane bytes (`B, G, R, B, G, R, …` per row).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr(&self) -> &'a [u8] {
    self.bgr
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
  /// Byte stride (`>= 3 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ============================================================
// Tier 6 — Packed RGBA / BGRA 8-bit source-side frames (Ship 9b)
// ============================================================
//
// Both formats are single-plane 8 bits per channel, 4 bytes per
// pixel. The 4th byte is real alpha (not padding) — for the
// `0rgb` / `rgb0` / `0bgr` / `bgr0` family where the 4th byte is
// padding, the planned `RgbPaddingFrame` (Ship 9d) handles that
// case so callers can't accidentally treat padding as alpha.

/// Errors returned by [`RgbaFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum RgbaFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`. Each row needs `4 * width` bytes for packed RGBA.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGBA** frame at 8 bits per channel
/// (`AV_PIX_FMT_RGBA`). One plane, 4 bytes per pixel, byte order
/// `R, G, B, A`.
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint. The 4th byte is real alpha — for the `0rgb` / `rgb0`
/// / `0bgr` / `bgr0` padding-byte family (where the 4th byte is
/// ignored padding, not alpha) see the planned Ship 9d
/// `RgbPaddingFrame` type.
#[derive(Debug, Clone, Copy)]
pub struct RgbaFrame<'a> {
  rgba: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> RgbaFrame<'a> {
  /// Constructs a new [`RgbaFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgba: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, RgbaFrameError> {
    if width == 0 || height == 0 {
      return Err(RgbaFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(RgbaFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(RgbaFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(RgbaFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgba.len() < plane_min {
      return Err(RgbaFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        rgba.len(),
      )));
    }
    Ok(Self {
      rgba,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`RgbaFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgba: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgba, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid RgbaFrame dimensions or plane length"),
    }
  }

  /// Packed RGBA plane bytes (`R, G, B, A` per pixel).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgba(&self) -> &'a [u8] {
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`BgraFrame::try_new`]. Variant shape mirrors
/// [`RgbaFrameError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum BgraFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **BGRA** frame at 8 bits per channel
/// (`AV_PIX_FMT_BGRA`). One plane, 4 bytes per pixel, byte order
/// `B, G, R, A` — channel-order distinction from [`RgbaFrame`]
/// is at the kernel level (sinker swaps `R↔B` while keeping `A`).
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct BgraFrame<'a> {
  bgra: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> BgraFrame<'a> {
  /// Constructs a new [`BgraFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgra: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, BgraFrameError> {
    if width == 0 || height == 0 {
      return Err(BgraFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(BgraFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(BgraFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(BgraFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if bgra.len() < plane_min {
      return Err(BgraFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        bgra.len(),
      )));
    }
    Ok(Self {
      bgra,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`BgraFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgra: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgra, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid BgraFrame dimensions or plane length"),
    }
  }

  /// Packed BGRA plane bytes (`B, G, R, A` per pixel).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgra(&self) -> &'a [u8] {
    self.bgra
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`ArgbFrame::try_new`]. Variant shape mirrors
/// [`RgbaFrameError`] — only the channel order on the four bytes
/// per pixel differs at the kernel level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum ArgbFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **ARGB** frame at 8 bits per channel
/// (`AV_PIX_FMT_ARGB`). One plane, 4 bytes per pixel, byte order
/// `A, R, G, B` — alpha is at the **leading** position (byte 0),
/// vs trailing for [`RgbaFrame`].
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint. The 1st byte is real alpha — for the `0rgb` / `rgb0`
/// / `0bgr` / `bgr0` padding-byte family (where the alpha-position
/// byte is ignored padding, not alpha) see the planned Ship 9d
/// `RgbPaddingFrame` type.
#[derive(Debug, Clone, Copy)]
pub struct ArgbFrame<'a> {
  argb: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> ArgbFrame<'a> {
  /// Constructs a new [`ArgbFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    argb: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, ArgbFrameError> {
    if width == 0 || height == 0 {
      return Err(ArgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(ArgbFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(ArgbFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(ArgbFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if argb.len() < plane_min {
      return Err(ArgbFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        argb.len(),
      )));
    }
    Ok(Self {
      argb,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`ArgbFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(argb: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(argb, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid ArgbFrame dimensions or plane length"),
    }
  }

  /// Packed ARGB plane bytes (`A, R, G, B` per pixel).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn argb(&self) -> &'a [u8] {
    self.argb
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`AbgrFrame::try_new`]. Variant shape mirrors
/// [`ArgbFrameError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum AbgrFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **ABGR** frame at 8 bits per channel
/// (`AV_PIX_FMT_ABGR`). One plane, 4 bytes per pixel, byte order
/// `A, B, G, R` — leading alpha + reversed RGB order vs
/// [`ArgbFrame`].
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct AbgrFrame<'a> {
  abgr: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> AbgrFrame<'a> {
  /// Constructs a new [`AbgrFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    abgr: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, AbgrFrameError> {
    if width == 0 || height == 0 {
      return Err(AbgrFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(AbgrFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(AbgrFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(AbgrFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if abgr.len() < plane_min {
      return Err(AbgrFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        abgr.len(),
      )));
    }
    Ok(Self {
      abgr,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`AbgrFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(abgr: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(abgr, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid AbgrFrame dimensions or plane length"),
    }
  }

  /// Packed ABGR plane bytes (`A, B, G, R` per pixel).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn abgr(&self) -> &'a [u8] {
    self.abgr
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`XrgbFrame::try_new`]. Variant shape mirrors
/// the [`RgbaFrameError`] family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum XrgbFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **0RGB** frame at 8 bits per channel
/// (`AV_PIX_FMT_0RGB`). One plane, 4 bytes per pixel, byte order
/// `X, R, G, B` — the leading byte is **ignored padding** (not real
/// alpha — see [`ArgbFrame`] for the alpha-bearing analogue).
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct XrgbFrame<'a> {
  xrgb: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> XrgbFrame<'a> {
  /// Constructs a new [`XrgbFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    xrgb: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, XrgbFrameError> {
    if width == 0 || height == 0 {
      return Err(XrgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(XrgbFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(XrgbFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(XrgbFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if xrgb.len() < plane_min {
      return Err(XrgbFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        xrgb.len(),
      )));
    }
    Ok(Self {
      xrgb,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`XrgbFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(xrgb: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(xrgb, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid XrgbFrame dimensions or plane length"),
    }
  }

  /// Packed XRGB plane bytes (leading padding byte ignored).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn xrgb(&self) -> &'a [u8] {
    self.xrgb
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`RgbxFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum RgbxFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **RGB0** frame at 8 bits per channel
/// (`AV_PIX_FMT_RGB0`). One plane, 4 bytes per pixel, byte order
/// `R, G, B, X` — the trailing byte is **ignored padding**.
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct RgbxFrame<'a> {
  rgbx: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> RgbxFrame<'a> {
  /// Constructs a new [`RgbxFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgbx: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, RgbxFrameError> {
    if width == 0 || height == 0 {
      return Err(RgbxFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(RgbxFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(RgbxFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(RgbxFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if rgbx.len() < plane_min {
      return Err(RgbxFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        rgbx.len(),
      )));
    }
    Ok(Self {
      rgbx,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`RgbxFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgbx: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgbx, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid RgbxFrame dimensions or plane length"),
    }
  }

  /// Packed RGBX plane bytes (trailing padding byte ignored).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgbx(&self) -> &'a [u8] {
    self.rgbx
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`XbgrFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum XbgrFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **0BGR** frame at 8 bits per channel
/// (`AV_PIX_FMT_0BGR`). One plane, 4 bytes per pixel, byte order
/// `X, B, G, R` — leading padding + reversed RGB order vs
/// [`XrgbFrame`].
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct XbgrFrame<'a> {
  xbgr: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> XbgrFrame<'a> {
  /// Constructs a new [`XbgrFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    xbgr: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, XbgrFrameError> {
    if width == 0 || height == 0 {
      return Err(XbgrFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(XbgrFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(XbgrFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(XbgrFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if xbgr.len() < plane_min {
      return Err(XbgrFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        xbgr.len(),
      )));
    }
    Ok(Self {
      xbgr,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`XbgrFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(xbgr: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(xbgr, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid XbgrFrame dimensions or plane length"),
    }
  }

  /// Packed XBGR plane bytes (leading padding byte ignored).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn xbgr(&self) -> &'a [u8] {
    self.xbgr
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// Errors returned by [`BgrxFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum BgrxFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 4 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `4 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

/// A validated packed **BGR0** frame at 8 bits per channel
/// (`AV_PIX_FMT_BGR0`). One plane, 4 bytes per pixel, byte order
/// `B, G, R, X` — trailing padding + reversed RGB order vs
/// [`RgbxFrame`].
///
/// `stride` is in **bytes** (≥ `4 * width`). No width parity
/// constraint.
#[derive(Debug, Clone, Copy)]
pub struct BgrxFrame<'a> {
  bgrx: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> BgrxFrame<'a> {
  /// Constructs a new [`BgrxFrame`], validating dimensions and
  /// plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgrx: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, BgrxFrameError> {
    if width == 0 || height == 0 {
      return Err(BgrxFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(4) {
      Some(v) => v,
      None => return Err(BgrxFrameError::WidthOverflow(WidthOverflow::new(width))),
    };
    if stride < min_stride {
      return Err(BgrxFrameError::InsufficientStride(InsufficientStride::new(
        stride, min_stride,
      )));
    }
    let plane_min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(BgrxFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if bgrx.len() < plane_min {
      return Err(BgrxFrameError::InsufficientPlane(InsufficientPlane::new(
        plane_min,
        bgrx.len(),
      )));
    }
    Ok(Self {
      bgrx,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`BgrxFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgrx: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgrx, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid BgrxFrame dimensions or plane length"),
    }
  }

  /// Packed BGRX plane bytes (trailing padding byte ignored).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgrx(&self) -> &'a [u8] {
    self.bgrx
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
  /// Byte stride (`>= 4 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}
