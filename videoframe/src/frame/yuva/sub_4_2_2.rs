use crate::frame::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, OddWidth, UnsupportedBits, ZeroDimension,
};
use derive_more::{Display, IsVariant};
use thiserror::Error;

/// Errors returned by [`Yuva422pFrame::try_new`].
///
/// Variant shape mirrors `Yuva420pFrameError`; the only semantic
/// difference is that 4:2:2 chroma is full-height, so plane-size
/// validation uses `u_stride * height` / `v_stride * height` rather
/// than `_stride * height.div_ceil(2)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Yuva422pFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `width` was odd. YUVA422p / 4:2:2 subsamples chroma 2:1 in width.
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
  /// (1:1 with Y, like Yuv422p planes — only chroma is subsampled in
  /// 4:2:2 horizontally, alpha is at luma resolution).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` bytes.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * height` bytes (chroma is
  /// full-height in 4:2:2).
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * height` bytes.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` bytes.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

/// A validated planar 4:2:2 `u8`-backed frame **with an alpha plane**
/// (`AV_PIX_FMT_YUVA422P`).
///
/// Storage mirrors `Yuv422pFrame` (Y full-size, U / V half-width ×
/// full-height — 4:2:2 only subsamples chroma horizontally) plus a
/// fourth full-resolution alpha plane (1:1 with Y).
///
/// `width` must be even (4:2:2 subsamples chroma 2:1 in width).
/// `height` may be any positive value.
#[derive(Debug, Clone, Copy)]
pub struct Yuva422pFrame<'a> {
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

impl<'a> Yuva422pFrame<'a> {
  /// Constructs a new [`Yuva422pFrame`], validating dimensions and
  /// plane lengths.
  ///
  /// Returns [`Yuva422pFrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - `width` is odd,
  /// - `y_stride < width`, `u_stride < (width + 1) / 2`,
  ///   `v_stride < (width + 1) / 2`, or `a_stride < width`,
  /// - any plane is too short to cover its declared rows
  ///   (chroma uses `_stride * height` because 4:2:2 chroma is
  ///   full-height; alpha uses `a_stride * height`).
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
  ) -> Result<Self, Yuva422pFrameError> {
    if width == 0 || height == 0 {
      return Err(Yuva422pFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Yuva422pFrameError::OddWidth(OddWidth::new(width)));
    }
    if y_stride < width {
      return Err(Yuva422pFrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let chroma_width = width.div_ceil(2);
    if u_stride < chroma_width {
      return Err(Yuva422pFrameError::InsufficientUStride(
        InsufficientStride::new(u_stride, chroma_width),
      ));
    }
    if v_stride < chroma_width {
      return Err(Yuva422pFrameError::InsufficientVStride(
        InsufficientStride::new(v_stride, chroma_width),
      ));
    }
    if a_stride < width {
      return Err(Yuva422pFrameError::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Yuva422pFrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    // 4:2:2: chroma is full-height (only subsamples horizontally).
    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrameError::GeometryOverflow(GeometryOverflow::new(
          u_stride, height,
        )));
      }
    };
    if u.len() < u_min {
      return Err(Yuva422pFrameError::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrameError::GeometryOverflow(GeometryOverflow::new(
          v_stride, height,
        )));
      }
    };
    if v.len() < v_min {
      return Err(Yuva422pFrameError::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }
    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrameError::GeometryOverflow(GeometryOverflow::new(
          a_stride, height,
        )));
      }
    };
    if a.len() < a_min {
      return Err(Yuva422pFrameError::InsufficientAPlane(
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

  /// Constructs a new [`Yuva422pFrame`], panicking on invalid inputs.
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
      Err(_) => panic!("invalid Yuva422pFrame dimensions or plane lengths"),
    }
  }

  /// Y (luma) plane bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u8] {
    self.y
  }
  /// U (Cb) plane bytes — half-width × full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u8] {
    self.u
  }
  /// V (Cr) plane bytes — half-width × full-height.
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

/// Errors returned by [`Yuva422pFrame16::try_new`] and
/// [`Yuva422pFrame16::try_new_checked`]. Variant shape mirrors
/// `Yuva420pFrame16Error` — only the semantic difference is in
/// chroma row count (4:2:2 chroma is full-height; the
/// `InsufficientUPlane` / `InsufficientVPlane` docs document
/// `_stride * height` rather than `_stride * height.div_ceil(2)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Error)]
#[non_exhaustive]
pub enum Yuva422pFrame16Error {
  /// `BITS` was not one of the supported depths (9, 10, 12, 16).
  /// FFmpeg ships `yuva422p9le`, `yuva422p10le`, `yuva422p12le`,
  /// `yuva422p16le`; Ship 8b‑4 wired 12-bit through the existing
  /// BITS-generic 4:2:2 row kernel templates.
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

  /// U plane is shorter than `u_stride * height` samples (chroma is
  /// full-height in 4:2:2).
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * height` samples.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` samples.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A plane sample exceeds `(1 << BITS) - 1`. Only
  /// [`Yuva422pFrame16::try_new_checked`] can produce this error.
  #[error(
    "sample {} on plane {} at element {} exceeds {} ((1 << BITS) - 1)", .0.value(), .0.plane(), .0.index(), .0.max_valid()
  )]
  SampleOutOfRange(Yuva422pFrame16SampleOutOfRange),
}

/// Identifies which plane of a [`Yuva422pFrame16`] an error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Yuva422pFrame16Plane {
  /// Luma plane.
  Y,
  /// U (Cb) chroma plane.
  U,
  /// V (Cr) chroma plane.
  V,
  /// Alpha plane.
  A,
}

/// A validated planar 4:2:2 `u16`-backed frame **with an alpha plane**,
/// generic over `const BITS: u32 ∈ {9, 10, 12, 16}`. Matches the full
/// FFmpeg set — `yuva422p9le`, `yuva422p10le`, `yuva422p12le`,
/// `yuva422p16le`.
///
/// Four planes — Y full-width × full-height, U / V half-width ×
/// full-height (4:2:2 chroma subsamples horizontally only), A
/// full-width × full-height (alpha is at luma resolution).
#[derive(Debug, Clone, Copy)]
pub struct Yuva422pFrame16<'a, const BITS: u32, const BE: bool = false> {
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

impl<'a, const BITS: u32, const BE: bool> Yuva422pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuva422pFrame16`], validating dimensions,
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
  ) -> Result<Self, Yuva422pFrame16Error> {
    if BITS != 9 && BITS != 10 && BITS != 12 && BITS != 16 {
      return Err(Yuva422pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuva422pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Yuva422pFrame16Error::OddWidth(OddWidth::new(width)));
    }
    if y_stride < width {
      return Err(Yuva422pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let chroma_width = width.div_ceil(2);
    if u_stride < chroma_width {
      return Err(Yuva422pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, chroma_width),
      ));
    }
    if v_stride < chroma_width {
      return Err(Yuva422pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, chroma_width),
      ));
    }
    if a_stride < width {
      return Err(Yuva422pFrame16Error::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuva422pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    // 4:2:2: chroma full-height.
    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuva422pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuva422pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }
    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva422pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(a_stride, height),
        ));
      }
    };
    if a.len() < a_min {
      return Err(Yuva422pFrame16Error::InsufficientAPlane(
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

  /// Constructs a new [`Yuva422pFrame16`], panicking on invalid
  /// inputs. Prefer [`Self::try_new`] when inputs may be invalid at
  /// runtime.
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
      Err(_) => panic!("invalid Yuva422pFrame16 dimensions, plane lengths, or BITS"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample of
  /// every plane and rejects values above `(1 << BITS) - 1`. Use this
  /// on untrusted input where accepting out-of-range samples would
  /// silently corrupt the conversion via the kernels' bit-mask.
  ///
  /// Returns [`Yuva422pFrame16Error::SampleOutOfRange`] on the first
  /// offending sample. All of [`Self::try_new`]'s geometry errors are
  /// still possible.
  ///
  /// 4:2:2 geometry: Y and A are full-width × full-height; U and V
  /// are half-width × full-height (chroma subsamples horizontally
  /// only).
  ///
  /// Cost: one O(plane_size) linear scan per plane (Y, U, V, A —
  /// four planes total). The default [`Self::try_new`] skips this so
  /// the hot path (decoder output, already-conforming buffers) stays
  /// O(1).
  ///
  /// Per the LE-encoded byte contract documented on the type, samples
  /// are validated **after** `u16::from_le` normalization so the range
  /// check operates on the intended logical sample value on every host.
  /// On little-endian hosts `from_le` is a no-op (the host-native `u16`
  /// already matches the wire); on big-endian hosts it byte-swaps each
  /// `u16` back into host-native form before the comparison. Without
  /// this normalization a valid `yuva422p10le` plane on a BE host would
  /// have its samples appear byte-swapped (e.g. `1023` encoded LE as
  /// bytes `[0xFF, 0x03]` reads as host-native `0xFF03` on BE) and the
  /// validator would falsely reject every row. The reported `value` in
  /// the error is the normalized logical sample so callers can match it
  /// against the declared `max_valid`. Mirrors the
  /// `Yuv422pFrame16::try_new_checked` pattern.
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
  ) -> Result<Self, Yuva422pFrame16Error> {
    let frame = Self::try_new(
      y, u, v, a, width, height, y_stride, u_stride, v_stride, a_stride,
    )?;
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    let chroma_w = w / 2;
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        // Normalize from LE-encoded wire to host-native before the
        // range check (no-op on LE host, byte-swap on BE host).
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva422pFrame16Error::SampleOutOfRange(
            Yuva422pFrame16SampleOutOfRange::new(
              Yuva422pFrame16Plane::Y,
              start + col,
              logical,
              max_valid,
            ),
          ));
        }
      }
    }
    for row in 0..h {
      let start = row * u_stride as usize;
      for (col, &s) in u[start..start + chroma_w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva422pFrame16Error::SampleOutOfRange(
            Yuva422pFrame16SampleOutOfRange::new(
              Yuva422pFrame16Plane::U,
              start + col,
              logical,
              max_valid,
            ),
          ));
        }
      }
    }
    for row in 0..h {
      let start = row * v_stride as usize;
      for (col, &s) in v[start..start + chroma_w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva422pFrame16Error::SampleOutOfRange(
            Yuva422pFrame16SampleOutOfRange::new(
              Yuva422pFrame16Plane::V,
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
          return Err(Yuva422pFrame16Error::SampleOutOfRange(
            Yuva422pFrame16SampleOutOfRange::new(
              Yuva422pFrame16Plane::A,
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

  /// Y (luma) plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }
  /// U (Cb) plane samples — half-width × full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }
  /// V (Cr) plane samples — half-width × full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
  }
  /// A (alpha) plane samples — full-width × full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a(&self) -> &'a [u16] {
    self.a
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
  /// Sample stride of the Y plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
  /// Sample stride of the U plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }
  /// Sample stride of the V plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }
  /// Sample stride of the A plane.
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
  /// (`AV_PIX_FMT_YUVA422P*BE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded 4:2:2 planar with alpha, 9-bit (`AV_PIX_FMT_YUVA422P9LE`).
pub type Yuva422p9Frame<'a> = Yuva422pFrame16<'a, 9>;

/// LE-encoded 4:2:2 planar with alpha, 10-bit (`AV_PIX_FMT_YUVA422P10LE`).
pub type Yuva422p10Frame<'a> = Yuva422pFrame16<'a, 10>;

/// LE-encoded 4:2:2 planar with alpha, 12-bit (`AV_PIX_FMT_YUVA422P12LE`).
pub type Yuva422p12Frame<'a> = Yuva422pFrame16<'a, 12>;

/// LE-encoded 4:2:2 planar with alpha, 16-bit (`AV_PIX_FMT_YUVA422P16LE`).
/// Uses the parallel i64 kernel family for the u16 RGBA path.
pub type Yuva422p16Frame<'a> = Yuva422pFrame16<'a, 16>;

// ---- Phase 4 — explicit LE/BE aliases for the YUVA 4:2:2 HB family ----

/// LE-encoded `Yuva422p9Frame` (`AV_PIX_FMT_YUVA422P9LE`).
pub type Yuva422p9LeFrame<'a> = Yuva422pFrame16<'a, 9, false>;
/// BE-encoded `Yuva422p9Frame` (`AV_PIX_FMT_YUVA422P9BE`).
pub type Yuva422p9BeFrame<'a> = Yuva422pFrame16<'a, 9, true>;
/// LE-encoded `Yuva422p10Frame` (`AV_PIX_FMT_YUVA422P10LE`).
pub type Yuva422p10LeFrame<'a> = Yuva422pFrame16<'a, 10, false>;
/// BE-encoded `Yuva422p10Frame` (`AV_PIX_FMT_YUVA422P10BE`).
pub type Yuva422p10BeFrame<'a> = Yuva422pFrame16<'a, 10, true>;
/// LE-encoded `Yuva422p12Frame` (`AV_PIX_FMT_YUVA422P12LE`).
pub type Yuva422p12LeFrame<'a> = Yuva422pFrame16<'a, 12, false>;
/// BE-encoded `Yuva422p12Frame` (`AV_PIX_FMT_YUVA422P12BE`).
pub type Yuva422p12BeFrame<'a> = Yuva422pFrame16<'a, 12, true>;
/// LE-encoded `Yuva422p16Frame` (`AV_PIX_FMT_YUVA422P16LE`).
pub type Yuva422p16LeFrame<'a> = Yuva422pFrame16<'a, 16, false>;
/// BE-encoded `Yuva422p16Frame` (`AV_PIX_FMT_YUVA422P16BE`).
pub type Yuva422p16BeFrame<'a> = Yuva422pFrame16<'a, 16, true>;

/// Payload struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Yuva422pFrame16SampleOutOfRange {
  plane: Yuva422pFrame16Plane,
  index: usize,
  value: u16,
  max_valid: u16,
}

impl Yuva422pFrame16SampleOutOfRange {
  /// Constructs a new `Yuva422pFrame16SampleOutOfRange`.
  #[inline]
  pub const fn new(plane: Yuva422pFrame16Plane, index: usize, value: u16, max_valid: u16) -> Self {
    Self {
      plane,
      index,
      value,
      max_valid,
    }
  }
  /// Returns the `plane` field.
  #[inline]
  pub const fn plane(&self) -> Yuva422pFrame16Plane {
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
