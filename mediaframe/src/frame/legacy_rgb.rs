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

// ============================================================
// Tier 7 — Legacy bit-packed RGB/BGR frame types (8bpp + 4bpp)
// (Rgb8, Bgr8, Rgb4Byte, Bgr4Byte — 1 byte/pixel;
//  Rgb4, Bgr4 — 4 bits/pixel, two pixels per byte)
// ============================================================

/// Errors returned by any of the legacy bit-packed-RGB frame
/// constructors (the 8-bits-per-pixel `Rgb8` / `Bgr8` / `Rgb4Byte` /
/// `Bgr4Byte` and the 4-bits-per-pixel `Rgb4` / `Bgr4`).
///
/// All six frame types share this error enum and perform validation in
/// the same order. Unlike [`LegacyRgbFrameError`] there is no
/// `WidthOverflow` variant: the minimum stride for these formats is at
/// most `width` bytes (`width` for the byte-aligned formats,
/// `width.div_ceil(2)` for the sub-byte 4-bpp formats), so it can never
/// overflow `u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum PackedRgbBitFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride` is smaller than the format's minimum row length in bytes
  /// (`width` for the byte-aligned formats, `width.div_ceil(2)` for the
  /// sub-byte 4-bpp formats).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * height` overflows `usize`.
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

// ---- Rgb8Frame -------------------------------------------------------------

/// A validated packed **RGB8** frame (`AV_PIX_FMT_RGB8`) — 1 byte per
/// pixel, packed RGB 3:3:2 with bits \[7:5\]=R3, \[4:2\]=G3, \[1:0\]=B2
/// (`(msb)3R 3G 2B(lsb)`). No unused bits.
///
/// `stride` is in **bytes** (≥ `width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb8Frame<'a> {
  rgb8: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb8Frame<'a> {
  /// Constructs a new [`Rgb8Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb8: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, PackedRgbBitFrameError> {
    if width == 0 || height == 0 {
      return Err(PackedRgbBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if stride < width {
      return Err(PackedRgbBitFrameError::InsufficientStride(
        InsufficientStride::new(stride, width),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(PackedRgbBitFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if rgb8.len() < plane_min {
      return Err(PackedRgbBitFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgb8.len()),
      ));
    }
    Ok(Self {
      rgb8,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb8Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb8: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb8, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb8Frame dimensions or plane length"),
    }
  }

  /// Packed RGB8 plane bytes — each byte is one 3:3:2 pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb8(&self) -> &'a [u8] {
    self.rgb8
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

  /// Byte stride (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Bgr8Frame -------------------------------------------------------------

/// A validated packed **BGR8** frame (`AV_PIX_FMT_BGR8`) — 1 byte per
/// pixel, packed RGB 3:3:2 with bits \[7:6\]=B2, \[5:3\]=G3, \[2:0\]=R3
/// (`(msb)2B 3G 3R(lsb)`). No unused bits.
///
/// `stride` is in **bytes** (≥ `width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr8Frame<'a> {
  bgr8: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr8Frame<'a> {
  /// Constructs a new [`Bgr8Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr8: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, PackedRgbBitFrameError> {
    if width == 0 || height == 0 {
      return Err(PackedRgbBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if stride < width {
      return Err(PackedRgbBitFrameError::InsufficientStride(
        InsufficientStride::new(stride, width),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(PackedRgbBitFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if bgr8.len() < plane_min {
      return Err(PackedRgbBitFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, bgr8.len()),
      ));
    }
    Ok(Self {
      bgr8,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr8Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr8: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr8, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr8Frame dimensions or plane length"),
    }
  }

  /// Packed BGR8 plane bytes — each byte is one 3:3:2 pixel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr8(&self) -> &'a [u8] {
    self.bgr8
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

  /// Byte stride (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Rgb4ByteFrame ---------------------------------------------------------

/// A validated packed **RGB4_BYTE** frame (`AV_PIX_FMT_RGB4_BYTE`) — 1
/// byte per pixel, packed RGB 1:2:1 with bit \[3\]=R1, bits \[2:1\]=G2,
/// bit \[0\]=B1 (`(msb)1R 2G 1B(lsb)`). The top 4 bits \[7:4\] are unused
/// padding (only the low nibble carries data).
///
/// `stride` is in **bytes** (≥ `width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb4ByteFrame<'a> {
  rgb4_byte: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb4ByteFrame<'a> {
  /// Constructs a new [`Rgb4ByteFrame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb4_byte: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, PackedRgbBitFrameError> {
    if width == 0 || height == 0 {
      return Err(PackedRgbBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if stride < width {
      return Err(PackedRgbBitFrameError::InsufficientStride(
        InsufficientStride::new(stride, width),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(PackedRgbBitFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if rgb4_byte.len() < plane_min {
      return Err(PackedRgbBitFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgb4_byte.len()),
      ));
    }
    Ok(Self {
      rgb4_byte,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb4ByteFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb4_byte: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb4_byte, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb4ByteFrame dimensions or plane length"),
    }
  }

  /// Packed RGB4_BYTE plane bytes — each byte is one 1:2:1 pixel in its
  /// low nibble.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb4_byte(&self) -> &'a [u8] {
    self.rgb4_byte
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

  /// Byte stride (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Bgr4ByteFrame ---------------------------------------------------------

/// A validated packed **BGR4_BYTE** frame (`AV_PIX_FMT_BGR4_BYTE`) — 1
/// byte per pixel, packed RGB 1:2:1 with bit \[3\]=B1, bits \[2:1\]=G2,
/// bit \[0\]=R1 (`(msb)1B 2G 1R(lsb)`). The top 4 bits \[7:4\] are unused
/// padding (only the low nibble carries data).
///
/// `stride` is in **bytes** (≥ `width`). No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr4ByteFrame<'a> {
  bgr4_byte: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr4ByteFrame<'a> {
  /// Constructs a new [`Bgr4ByteFrame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr4_byte: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, PackedRgbBitFrameError> {
    if width == 0 || height == 0 {
      return Err(PackedRgbBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if stride < width {
      return Err(PackedRgbBitFrameError::InsufficientStride(
        InsufficientStride::new(stride, width),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(PackedRgbBitFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if bgr4_byte.len() < plane_min {
      return Err(PackedRgbBitFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, bgr4_byte.len()),
      ));
    }
    Ok(Self {
      bgr4_byte,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr4ByteFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr4_byte: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr4_byte, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr4ByteFrame dimensions or plane length"),
    }
  }

  /// Packed BGR4_BYTE plane bytes — each byte is one 1:2:1 pixel in its
  /// low nibble.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr4_byte(&self) -> &'a [u8] {
    self.bgr4_byte
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

  /// Byte stride (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Rgb4Frame -------------------------------------------------------------

/// A validated packed **RGB4** frame (`AV_PIX_FMT_RGB4`) — 4 bits per
/// pixel, bitstream-packed two pixels per byte. Each 4-bit nibble holds
/// one 1:2:1 pixel: bit \[3\]=R1, bits \[2:1\]=G2, bit \[0\]=B1
/// (`(msb)1R 2G 1B(lsb)`). Within each byte the **first (even) pixel is
/// the high nibble \[7:4\]** and the second (odd) pixel is the low nibble
/// \[3:0\].
///
/// `stride` is in **bytes** (≥ `width.div_ceil(2)`, i.e. `(4 * width + 7)
/// / 8`). Odd widths leave the final byte's low nibble unused. No width
/// parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Rgb4Frame<'a> {
  rgb4: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Rgb4Frame<'a> {
  /// Constructs a new [`Rgb4Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    rgb4: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, PackedRgbBitFrameError> {
    if width == 0 || height == 0 {
      return Err(PackedRgbBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    // Bitstream 4-bpp row length in bytes: `(4 * width + 7) / 8`, which
    // for 4 bpp is exactly `width.div_ceil(2)`. `width <= u32::MAX`, so
    // the ceiling division cannot overflow `u32`.
    let min_stride = width.div_ceil(2);
    if stride < min_stride {
      return Err(PackedRgbBitFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(PackedRgbBitFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if rgb4.len() < plane_min {
      return Err(PackedRgbBitFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, rgb4.len()),
      ));
    }
    Ok(Self {
      rgb4,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Rgb4Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rgb4: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(rgb4, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Rgb4Frame dimensions or plane length"),
    }
  }

  /// Packed RGB4 plane bytes — each byte holds two 1:2:1 pixels (first
  /// pixel in the high nibble).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rgb4(&self) -> &'a [u8] {
    self.rgb4
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

  /// Byte stride (`>= width.div_ceil(2)`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

// ---- Bgr4Frame -------------------------------------------------------------

/// A validated packed **BGR4** frame (`AV_PIX_FMT_BGR4`) — 4 bits per
/// pixel, bitstream-packed two pixels per byte. Each 4-bit nibble holds
/// one 1:2:1 pixel: bit \[3\]=B1, bits \[2:1\]=G2, bit \[0\]=R1
/// (`(msb)1B 2G 1R(lsb)`). Within each byte the **first (even) pixel is
/// the high nibble \[7:4\]** and the second (odd) pixel is the low nibble
/// \[3:0\].
///
/// `stride` is in **bytes** (≥ `width.div_ceil(2)`, i.e. `(4 * width + 7)
/// / 8`). Odd widths leave the final byte's low nibble unused. No width
/// parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Bgr4Frame<'a> {
  bgr4: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> Bgr4Frame<'a> {
  /// Constructs a new [`Bgr4Frame`], validating dimensions and plane length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    bgr4: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, PackedRgbBitFrameError> {
    if width == 0 || height == 0 {
      return Err(PackedRgbBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    // Bitstream 4-bpp row length in bytes: `(4 * width + 7) / 8`, which
    // for 4 bpp is exactly `width.div_ceil(2)`. `width <= u32::MAX`, so
    // the ceiling division cannot overflow `u32`.
    let min_stride = width.div_ceil(2);
    if stride < min_stride {
      return Err(PackedRgbBitFrameError::InsufficientStride(
        InsufficientStride::new(stride, min_stride),
      ));
    }
    let plane_min = match (stride as u64).checked_mul(height as u64) {
      Some(v) if v <= usize::MAX as u64 => v as usize,
      _ => {
        return Err(PackedRgbBitFrameError::GeometryOverflow(
          GeometryOverflow::new(stride, height),
        ));
      }
    };
    if bgr4.len() < plane_min {
      return Err(PackedRgbBitFrameError::InsufficientPlane(
        InsufficientPlane::new(plane_min, bgr4.len()),
      ));
    }
    Ok(Self {
      bgr4,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`Bgr4Frame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(bgr4: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(bgr4, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Bgr4Frame dimensions or plane length"),
    }
  }

  /// Packed BGR4 plane bytes — each byte holds two 1:2:1 pixels (first
  /// pixel in the high nibble).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bgr4(&self) -> &'a [u8] {
    self.bgr4
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

  /// Byte stride (`>= width.div_ceil(2)`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}
