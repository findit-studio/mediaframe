use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, WidthOverflow, ZeroDimension,
};
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

// ============================================================
// Tier 7 — Legacy 16-bit packed RGB/BGR frame types
// (Ship Tier 7 — Rgb565, Bgr565, Rgb555, Bgr555, Rgb444, Bgr444)
// ============================================================

/// Errors returned by any of the legacy 16-bit packed-RGB frame constructors.
///
/// All six frame types (`Rgb565Frame`, `Bgr565Frame`, `Rgb555Frame`,
/// `Bgr555Frame`, `Rgb444Frame`, `Bgr444Frame`) share this error enum and
/// perform validation in the same order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum LegacyRgbFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < 2 * width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// `2 * width` overflows `u32`.
  #[error(transparent)]
  WidthOverflow(WidthOverflow),
}

// ---- Rgb565Frame -----------------------------------------------------------

/// A validated packed **RGB565** frame (`AV_PIX_FMT_RGB565LE`) — 2 bytes per
/// pixel, 16-bit little-endian word with bits \[15:11\]=R5, \[10:5\]=G6, \[4:0\]=B5.
///
/// `stride` is in **bytes** (≥ `2 * width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb565Frame<'a> {
  rgb565: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb565Frame<'a> {
  /// Constructs a new [`Rgb565Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb565: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, LegacyRgbFrameError> {
    if width == 0 || height == 0 {
      return Err(LegacyRgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(LegacyRgbFrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(LegacyRgbFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(LegacyRgbFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if rgb565.len() < plane_min {
      return Err(LegacyRgbFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgb565.len()),
      ));
    }
    Ok(Self {
      rgb565,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb565Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb565: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb565, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb565Frame dimensions or plane length"),
    }
  }

  /// Packed RGB565 plane bytes — each 2-byte group is one LE `u16` pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb565(&self) -> &'a [u8] {
    self.rgb565
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

  /// Byte stride (`>= 2 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Bgr565Frame -----------------------------------------------------------

/// A validated packed **BGR565** frame (`AV_PIX_FMT_BGR565LE`) — 2 bytes per
/// pixel, 16-bit little-endian word with bits \[15:11\]=B5, \[10:5\]=G6, \[4:0\]=R5.
///
/// `stride` is in **bytes** (≥ `2 * width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr565Frame<'a> {
  bgr565: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr565Frame<'a> {
  /// Constructs a new [`Bgr565Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr565: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, LegacyRgbFrameError> {
    if width == 0 || height == 0 {
      return Err(LegacyRgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(LegacyRgbFrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(LegacyRgbFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(LegacyRgbFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if bgr565.len() < plane_min {
      return Err(LegacyRgbFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, bgr565.len()),
      ));
    }
    Ok(Self {
      bgr565,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr565Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr565: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr565, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr565Frame dimensions or plane length"),
    }
  }

  /// Packed BGR565 plane bytes — each 2-byte group is one LE `u16` pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr565(&self) -> &'a [u8] {
    self.bgr565
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

  /// Byte stride (`>= 2 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Rgb555Frame -----------------------------------------------------------

/// A validated packed **RGB555** frame (`AV_PIX_FMT_RGB555LE`) — 2 bytes per
/// pixel, 16-bit little-endian word with bit 15 as unused padding, bits
/// \[14:10\]=R5, \[9:5\]=G5, \[4:0\]=B5.
///
/// `stride` is in **bytes** (≥ `2 * width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb555Frame<'a> {
  rgb555: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb555Frame<'a> {
  /// Constructs a new [`Rgb555Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb555: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, LegacyRgbFrameError> {
    if width == 0 || height == 0 {
      return Err(LegacyRgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(LegacyRgbFrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(LegacyRgbFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(LegacyRgbFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if rgb555.len() < plane_min {
      return Err(LegacyRgbFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgb555.len()),
      ));
    }
    Ok(Self {
      rgb555,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb555Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb555: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb555, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb555Frame dimensions or plane length"),
    }
  }

  /// Packed RGB555 plane bytes — each 2-byte group is one LE `u16` pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb555(&self) -> &'a [u8] {
    self.rgb555
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

  /// Byte stride (`>= 2 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Bgr555Frame -----------------------------------------------------------

/// A validated packed **BGR555** frame (`AV_PIX_FMT_BGR555LE`) — 2 bytes per
/// pixel, 16-bit little-endian word with bit 15 as unused padding, bits
/// \[14:10\]=B5, \[9:5\]=G5, \[4:0\]=R5.
///
/// `stride` is in **bytes** (≥ `2 * width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr555Frame<'a> {
  bgr555: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr555Frame<'a> {
  /// Constructs a new [`Bgr555Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr555: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, LegacyRgbFrameError> {
    if width == 0 || height == 0 {
      return Err(LegacyRgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(LegacyRgbFrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(LegacyRgbFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(LegacyRgbFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if bgr555.len() < plane_min {
      return Err(LegacyRgbFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, bgr555.len()),
      ));
    }
    Ok(Self {
      bgr555,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr555Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr555: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr555, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr555Frame dimensions or plane length"),
    }
  }

  /// Packed BGR555 plane bytes — each 2-byte group is one LE `u16` pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr555(&self) -> &'a [u8] {
    self.bgr555
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

  /// Byte stride (`>= 2 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Rgb444Frame -----------------------------------------------------------

/// A validated packed **RGB444** frame (`AV_PIX_FMT_RGB444LE`) — 2 bytes per
/// pixel, 16-bit little-endian word with bits \[15:12\] as unused padding, bits
/// \[11:8\]=R4, \[7:4\]=G4, \[3:0\]=B4.
///
/// `stride` is in **bytes** (≥ `2 * width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb444Frame<'a> {
  rgb444: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb444Frame<'a> {
  /// Constructs a new [`Rgb444Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb444: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, LegacyRgbFrameError> {
    if width == 0 || height == 0 {
      return Err(LegacyRgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(LegacyRgbFrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(LegacyRgbFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(LegacyRgbFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if rgb444.len() < plane_min {
      return Err(LegacyRgbFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgb444.len()),
      ));
    }
    Ok(Self {
      rgb444,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb444Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb444: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb444, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb444Frame dimensions or plane length"),
    }
  }

  /// Packed RGB444 plane bytes — each 2-byte group is one LE `u16` pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb444(&self) -> &'a [u8] {
    self.rgb444
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

  /// Byte stride (`>= 2 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Bgr444Frame -----------------------------------------------------------

/// A validated packed **BGR444** frame (`AV_PIX_FMT_BGR444LE`) — 2 bytes per
/// pixel, 16-bit little-endian word with bits \[15:12\] as unused padding, bits
/// \[11:8\]=B4, \[7:4\]=G4, \[3:0\]=R4.
///
/// `stride` is in **bytes** (≥ `2 * width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr444Frame<'a> {
  bgr444: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr444Frame<'a> {
  /// Constructs a new [`Bgr444Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr444: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, LegacyRgbFrameError> {
    if width == 0 || height == 0 {
      return Err(LegacyRgbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    let min_stride = match width.checked_mul(2) {
      Some(v) => v,
      None => {
        return Err(LegacyRgbFrameError::WidthOverflow(WidthOverflow::new(
          width,
        )));
      }
    };
    if stride < min_stride {
      return Err(LegacyRgbFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(LegacyRgbFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if bgr444.len() < plane_min {
      return Err(LegacyRgbFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, bgr444.len()),
      ));
    }
    Ok(Self {
      bgr444,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr444Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr444: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr444, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr444Frame dimensions or plane length"),
    }
  }

  /// Packed BGR444 plane bytes — each 2-byte group is one LE `u16` pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr444(&self) -> &'a [u8] {
    self.bgr444
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

  /// Byte stride (`>= 2 * width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}
