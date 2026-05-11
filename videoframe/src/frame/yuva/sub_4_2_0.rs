use crate::frame::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, OddWidth, UnsupportedBits, ZeroDimension,
};
use derive_more::{Display, IsVariant};
use thiserror::Error;

/// Errors returned by [`Yuva420pFrame::try_new`].
///
/// Variant shape mirrors `Yuv420pFrameError` (geometry, plane-too-short)
/// extended with [`Self::InsufficientAStride`] / [`Self::InsufficientAPlane`]
/// for the 4:2:0 alpha plane (full-width × full-height — alpha is at
/// luma resolution, only chroma is subsampled).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Yuva420pFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `width` was odd. YUVA420p / 4:2:0 subsamples chroma 2:1 in width.
  #[error(transparent)]
  OddWidth(OddWidth),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `u_stride < ceil(width / 2)`.
  #[error(transparent)]
  InsufficientUStride(InsufficientStride),

  /// `v_stride < ceil(width / 2)`.
  #[error(transparent)]
  InsufficientVStride(InsufficientStride),

  /// `a_stride < width`. The alpha plane is full-width × full-height
  /// (1:1 with Y, like Yuv444p planes — only chroma is subsampled in
  /// 4:2:0).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` bytes.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * height.div_ceil(2)` bytes.
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * height.div_ceil(2)` bytes.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` bytes.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

/// A validated YUVA 4:2:0 planar frame at 8 bits per sample
/// (`AV_PIX_FMT_YUVA420P`).
///
/// Four planes:
/// - `y` — full-size luma (same as `Yuv420pFrame::y`).
/// - `u` / `v` — half-width × half-height chroma (same as the parent
///   YUV 4:2:0 layout).
/// - `a` — **full-width × full-height** alpha (1:1 with Y; only chroma
///   is subsampled in 4:2:0).
///
/// `width` must be even (4:2:0 subsamples chroma 2:1 in width).
/// `height` may be odd (chroma row sizing uses `height.div_ceil(2)`,
/// alpha sizing uses `height` since alpha is full-resolution).
#[derive(Debug, Clone, Copy)]
pub struct Yuva420pFrame<'a> {
  y: &'a [u8],
  u: &'a [u8],
  v: &'a [u8],
  a: &'a [u8],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
  a_stride: u32,
}

impl<'a> Yuva420pFrame<'a> {
  /// Constructs a new [`Yuva420pFrame`], validating dimensions and
  /// plane lengths.
  ///
  /// Returns [`Yuva420pFrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - `width` is odd,
  /// - `y_stride < width`, `u_stride < (width + 1) / 2`,
  ///   `v_stride < (width + 1) / 2`, or `a_stride < width`,
  /// - any plane is too short to cover its declared rows.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u8],
    u: &'a [u8],
    v: &'a [u8],
    a: &'a [u8],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
    a_stride: u32,
  ) -> Result<Self, Yuva420pFrameError> {
    if width == 0 || height == 0 {
      return Err(Yuva420pFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Yuva420pFrameError::OddWidth(OddWidth::new(width)));
    }
    if y_stride < width {
      return Err(Yuva420pFrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let chroma_width = width.div_ceil(2);
    if u_stride < chroma_width {
      return Err(Yuva420pFrameError::InsufficientUStride(
        InsufficientStride::new(u_stride, chroma_width),
      ));
    }
    if v_stride < chroma_width {
      return Err(Yuva420pFrameError::InsufficientVStride(
        InsufficientStride::new(v_stride, chroma_width),
      ));
    }
    // Alpha is full-width (1:1 with Y).
    if a_stride < width {
      return Err(Yuva420pFrameError::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Yuva420pFrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    let chroma_height = height.div_ceil(2);
    let u_min = match (u_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrameError::GeometryOverflow(GeometryOverflow::new(
          u_stride,
          chroma_height,
        )));
      }
    };
    if u.len() < u_min {
      return Err(Yuva420pFrameError::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrameError::GeometryOverflow(GeometryOverflow::new(
          v_stride,
          chroma_height,
        )));
      }
    };
    if v.len() < v_min {
      return Err(Yuva420pFrameError::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }
    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrameError::GeometryOverflow(GeometryOverflow::new(
          a_stride, height,
        )));
      }
    };
    if a.len() < a_min {
      return Err(Yuva420pFrameError::InsufficientAPlane(
        InsufficientPlane::new(a_min, a.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      a,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
      a_stride,
    })
  }

  /// Constructs a new [`Yuva420pFrame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u8],
    u: &'a [u8],
    v: &'a [u8],
    a: &'a [u8],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
    a_stride: u32,
  ) -> Self {
    match Self::try_new(
      y, u, v, a, width, height, y_stride, u_stride, v_stride, a_stride,
    ) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuva420pFrame dimensions or plane lengths"),
    }
  }

  /// Y (luma) plane bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u8] {
    self.y
  }
  /// U (Cb) plane bytes — half-width × half-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u8] {
    self.u
  }
  /// V (Cr) plane bytes — half-width × half-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u8] {
    self.v
  }
  /// A (alpha) plane bytes — full-width × full-height (1:1 with Y).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a(&self) -> &'a [u8] {
    self.a
  }
  /// Frame width in pixels. Always even.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Frame height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }
  /// Byte stride of the Y plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
  /// Byte stride of the U plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }
  /// Byte stride of the V plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }
  /// Byte stride of the A plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a_stride(&self) -> u32 {
    self.a_stride
  }
}

/// Errors returned by [`Yuva420pFrame16::try_new`] and
/// [`Yuva420pFrame16::try_new_checked`].
///
/// Variant shape mirrors `Yuv420pFrame16Error` extended with the
/// `A`-plane variants ([`Self::InsufficientAStride`] /
/// [`Self::InsufficientAPlane`]) for the 4:2:0 alpha plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Yuva420pFrame16Error {
  /// `BITS` was not one of the supported depths (9, 10, 16). FFmpeg
  /// only ships `yuva420p9le`, `yuva420p10le`, `yuva420p16le` — no
  /// 12/14-bit YUVA 4:2:0 pixel formats exist.
  #[error(transparent)]
  UnsupportedBits(UnsupportedBits),

  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `width` was odd.
  #[error(transparent)]
  OddWidth(OddWidth),

  /// `y_stride < width` (in samples).
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `u_stride < ceil(width / 2)` (in samples).
  #[error(transparent)]
  InsufficientUStride(InsufficientStride),

  /// `v_stride < ceil(width / 2)` (in samples).
  #[error(transparent)]
  InsufficientVStride(InsufficientStride),

  /// `a_stride < width` (in samples).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * ceil(height / 2)` samples.
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * ceil(height / 2)` samples.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` samples.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A plane sample exceeds `(1 << BITS) - 1`. Only
  /// [`Yuva420pFrame16::try_new_checked`] can produce this error.
  #[error(
    "sample {} on plane {} at element {} exceeds {} ((1 << BITS) - 1)", .0.value(), .0.plane(), .0.index(), .0.max_valid()
  )]
  SampleOutOfRange(Yuva420pFrame16SampleOutOfRange),
}

/// Identifies which plane of a [`Yuva420pFrame16`] an error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Yuva420pFrame16Plane {
  /// Luma plane.
  Y,
  /// U (Cb) chroma plane.
  U,
  /// V (Cr) chroma plane.
  V,
  /// Alpha plane.
  A,
}

/// A validated planar 4:2:0 `u16`-backed frame **with an alpha plane**,
/// generic over `const BITS: u32 ∈ {9, 10, 16}`. FFmpeg ships
/// `yuva420p9le`, `yuva420p10le`, and `yuva420p16le` — no 12/14-bit
/// YUVA 4:2:0 pixel formats exist, so [`Self::try_new`] rejects them.
///
/// Four planes — Y full-width × full-height, U/V half-width ×
/// half-height (4:2:0 chroma subsampling), A full-width × full-height
/// (alpha is at luma resolution; only chroma is subsampled).
#[derive(Debug, Clone, Copy)]
pub struct Yuva420pFrame16<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  u: &'a [u16],
  v: &'a [u16],
  a: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
  a_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> Yuva420pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuva420pFrame16`], validating dimensions,
  /// plane lengths, and the `BITS` parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    a: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
    a_stride: u32,
  ) -> Result<Self, Yuva420pFrame16Error> {
    // FFmpeg's only YUVA 4:2:0 high-bit pixel formats: yuva420p9le,
    // yuva420p10le, yuva420p16le. No 12/14-bit variants exist.
    if BITS != 9 && BITS != 10 && BITS != 16 {
      return Err(Yuva420pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuva420pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Yuva420pFrame16Error::OddWidth(OddWidth::new(width)));
    }
    if y_stride < width {
      return Err(Yuva420pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let chroma_width = width.div_ceil(2);
    if u_stride < chroma_width {
      return Err(Yuva420pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, chroma_width),
      ));
    }
    if v_stride < chroma_width {
      return Err(Yuva420pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, chroma_width),
      ));
    }
    if a_stride < width {
      return Err(Yuva420pFrame16Error::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuva420pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    let chroma_height = height.div_ceil(2);
    let u_min = match (u_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, chroma_height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuva420pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, chroma_height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuva420pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }
    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(a_stride, height),
        ));
      }
    };
    if a.len() < a_min {
      return Err(Yuva420pFrame16Error::InsufficientAPlane(
        InsufficientPlane::new(a_min, a.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      a,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
      a_stride,
    })
  }

  /// Constructs a new [`Yuva420pFrame16`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    a: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
    a_stride: u32,
  ) -> Self {
    match Self::try_new(
      y, u, v, a, width, height, y_stride, u_stride, v_stride, a_stride,
    ) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuva420pFrame16 dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample of
  /// every plane and rejects values above `(1 << BITS) - 1`. Use this
  /// on untrusted input.
  ///
  /// Cost: one O(plane_size) linear scan per plane (Y, U, V, A —
  /// four planes total).
  ///
  /// Per the LE-encoded byte contract documented on the type, samples
  /// are validated **after** `u16::from_le` normalization so the range
  /// check operates on the intended logical sample value on every host.
  /// On little-endian hosts `from_le` is a no-op (the host-native `u16`
  /// already matches the wire); on big-endian hosts it byte-swaps each
  /// `u16` back into host-native form before the comparison. Without
  /// this normalization a valid `yuva420p10le` plane on a BE host would
  /// have its samples appear byte-swapped (e.g. `1023` encoded LE as
  /// bytes `[0xFF, 0x03]` reads as host-native `0xFF03` on BE) and the
  /// validator would falsely reject every row. The reported `value` in
  /// the error is the normalized logical sample so callers can match it
  /// against the declared `max_valid`. Mirrors the
  /// `Yuv420pFrame16::try_new_checked` pattern.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    a: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
    a_stride: u32,
  ) -> Result<Self, Yuva420pFrame16Error> {
    let frame = Self::try_new(
      y, u, v, a, width, height, y_stride, u_stride, v_stride, a_stride,
    )?;
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    let chroma_w = w / 2;
    let chroma_h = height.div_ceil(2) as usize;
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        // Normalize from LE-encoded wire to host-native before the
        // range check (no-op on LE host, byte-swap on BE host).
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva420pFrame16Error::SampleOutOfRange(
            Yuva420pFrame16SampleOutOfRange::new(
              Yuva420pFrame16Plane::Y,
              start + col,
              logical,
              max_valid,
            ),
          ));
        }
      }
    }
    for row in 0..chroma_h {
      let start = row * u_stride as usize;
      for (col, &s) in u[start..start + chroma_w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva420pFrame16Error::SampleOutOfRange(
            Yuva420pFrame16SampleOutOfRange::new(
              Yuva420pFrame16Plane::U,
              start + col,
              logical,
              max_valid,
            ),
          ));
        }
      }
    }
    for row in 0..chroma_h {
      let start = row * v_stride as usize;
      for (col, &s) in v[start..start + chroma_w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva420pFrame16Error::SampleOutOfRange(
            Yuva420pFrame16SampleOutOfRange::new(
              Yuva420pFrame16Plane::V,
              start + col,
              logical,
              max_valid,
            ),
          ));
        }
      }
    }
    for row in 0..h {
      let start = row * a_stride as usize;
      for (col, &s) in a[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva420pFrame16Error::SampleOutOfRange(
            Yuva420pFrame16SampleOutOfRange::new(
              Yuva420pFrame16Plane::A,
              start + col,
              logical,
              max_valid,
            ),
          ));
        }
      }
    }
    Ok(frame)
  }

  /// Y plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }
  /// U plane samples — half-width × half-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }
  /// V plane samples — half-width × half-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
  }
  /// A plane samples — full-width × full-height, native bit depth,
  /// low-bit-packed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a(&self) -> &'a [u16] {
    self.a
  }
  /// Frame width in pixels. Always even.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Frame height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }
  /// Y-plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
  /// U-plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }
  /// V-plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }
  /// A-plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a_stride(&self) -> u32 {
    self.a_stride
  }
  /// The `BITS` const parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Compile-time BE flag mirror — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YUVA420P*BE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded 4:2:0 planar with alpha, 9-bit (`AV_PIX_FMT_YUVA420P9LE`).
pub type Yuva420p9Frame<'a> = Yuva420pFrame16<'a, 9>;

/// LE-encoded 4:2:0 planar with alpha, 10-bit (`AV_PIX_FMT_YUVA420P10LE`).
pub type Yuva420p10Frame<'a> = Yuva420pFrame16<'a, 10>;

/// LE-encoded 4:2:0 planar with alpha, 16-bit (`AV_PIX_FMT_YUVA420P16LE`).
/// Uses the parallel i64 kernel family for the u16 RGBA path.
pub type Yuva420p16Frame<'a> = Yuva420pFrame16<'a, 16>;

// ---- Phase 4 — explicit LE/BE aliases for the YUVA 4:2:0 HB family ----

/// LE-encoded `Yuva420p9Frame` (`AV_PIX_FMT_YUVA420P9LE`).
pub type Yuva420p9LeFrame<'a> = Yuva420pFrame16<'a, 9, false>;
/// BE-encoded `Yuva420p9Frame` (`AV_PIX_FMT_YUVA420P9BE`).
pub type Yuva420p9BeFrame<'a> = Yuva420pFrame16<'a, 9, true>;
/// LE-encoded `Yuva420p10Frame` (`AV_PIX_FMT_YUVA420P10LE`).
pub type Yuva420p10LeFrame<'a> = Yuva420pFrame16<'a, 10, false>;
/// BE-encoded `Yuva420p10Frame` (`AV_PIX_FMT_YUVA420P10BE`).
pub type Yuva420p10BeFrame<'a> = Yuva420pFrame16<'a, 10, true>;
/// LE-encoded `Yuva420p16Frame` (`AV_PIX_FMT_YUVA420P16LE`).
pub type Yuva420p16LeFrame<'a> = Yuva420pFrame16<'a, 16, false>;
/// BE-encoded `Yuva420p16Frame` (`AV_PIX_FMT_YUVA420P16BE`).
pub type Yuva420p16BeFrame<'a> = Yuva420pFrame16<'a, 16, true>;

/// Payload struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Yuva420pFrame16SampleOutOfRange {
  plane: Yuva420pFrame16Plane,
  index: usize,
  value: u16,
  max_valid: u16,
}

impl Yuva420pFrame16SampleOutOfRange {
  /// Constructs a new `Yuva420pFrame16SampleOutOfRange`.
  #[inline]
  pub const fn new(plane: Yuva420pFrame16Plane, index: usize, value: u16, max_valid: u16) -> Self {
    Self {
      plane,
      index,
      value,
      max_valid,
    }
  }
  /// Returns the `plane` field.
  #[inline]
  pub const fn plane(&self) -> Yuva420pFrame16Plane {
    self.plane
  }
  /// Returns the `index` field.
  #[inline]
  pub const fn index(&self) -> usize {
    self.index
  }
  /// Returns the `value` field.
  #[inline]
  pub const fn value(&self) -> u16 {
    self.value
  }
  /// Returns the `max_valid` field.
  #[inline]
  pub const fn max_valid(&self) -> u16 {
    self.max_valid
  }
}
