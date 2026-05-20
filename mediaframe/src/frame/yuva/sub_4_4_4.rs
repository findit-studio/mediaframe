use crate::frame::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, UnsupportedBits, ZeroDimension,
};
use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

/// Errors returned by [`Yuva444pFrame16::try_new`] and
/// [`Yuva444pFrame16::try_new_checked`].
///
/// Variant shape mirrors `Yuv420pFrame16Error` (geometry,
/// `UnsupportedBits`, `SampleOutOfRange`, plane-too-short),
/// extended with the `A`-plane variants ([`Self::InsufficientAStride`],
/// [`Self::InsufficientAPlane`]) for the 4:4:4 alpha plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Yuva444pFrame16Error {
  /// `BITS` was not one of the supported depths. Yuva444p shipped
  /// progressively — 8b‑1 (10), 8b‑3 (9), 8b‑4 (12 / 14), 8b‑5a (16,
  /// scalar; SIMD lands in 8b‑5b/c).
  #[error(transparent)]
  UnsupportedBits(UnsupportedBits),

  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width` (in samples).
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `u_stride < width` (in samples). 4:4:4 chroma is full-width.
  #[error(transparent)]
  InsufficientUStride(InsufficientStride),

  /// `v_stride < width` (in samples). 4:4:4 chroma is full-width.
  #[error(transparent)]
  InsufficientVStride(InsufficientStride),

  /// `a_stride < width` (in samples). The alpha plane is full-width
  /// at the source's bit depth (1:1 with Y, like the chroma planes).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * height` samples.
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * height` samples.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` samples.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (32‑bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A plane sample exceeds `(1 << BITS) - 1`. Only
  /// [`Yuva444pFrame16::try_new_checked`] can produce this error.
  #[error(
    "sample {} on plane {} at element {} exceeds {} ((1 << BITS) - 1)", .0.value(), .0.plane(), .0.index(), .0.max_valid()
  )]
  SampleOutOfRange(Yuva444pFrame16SampleOutOfRange),
}

/// Identifies which plane of a [`Yuva444pFrame16`] an error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Yuva444pFrame16Plane {
  /// Luma plane.
  Y,
  /// U (Cb) chroma plane.
  U,
  /// V (Cr) chroma plane.
  V,
  /// Alpha plane.
  A,
}

/// A validated planar 4:4:4 `u16`-backed frame **with an alpha plane**,
/// generic over `const BITS: u32`. Tranche 1 ships `BITS == 10` only
/// (`AV_PIX_FMT_YUVA444P10LE`); later tranches will admit additional
/// depths as the corresponding YUVA pixel formats land.
///
/// Four planes — Y, U, V, A — all full-width × full-height (the alpha
/// plane is at the source's bit depth, low-bit-packed in `u16`,
/// matching the Y/U/V planes).
#[derive(Debug, Clone, Copy)]
pub struct Yuva444pFrame16<'a, const BITS: u32, const BE: bool = false> {
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

impl<'a, const BITS: u32, const BE: bool> Yuva444pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuva444pFrame16`].
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
  ) -> Result<Self, Yuva444pFrame16Error> {
    // Ship 8b‑1 shipped 10-bit; 8b‑3 added 9; 8b‑4 added 12/14;
    // 8b‑5a opens 16. The 16-bit path uses the dedicated i64 4:4:4
    // kernel family (separate from the BITS-generic Q15 i32
    // template that covers {9,10,12,14}).
    if BITS != 9 && BITS != 10 && BITS != 12 && BITS != 14 && BITS != 16 {
      return Err(Yuva444pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuva444pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Yuva444pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    // 4:4:4: chroma stride ≥ width.
    if u_stride < width {
      return Err(Yuva444pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, width),
      ));
    }
    if v_stride < width {
      return Err(Yuva444pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, width),
      ));
    }
    // Alpha is full-width (1:1 with Y).
    if a_stride < width {
      return Err(Yuva444pFrame16Error::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuva444pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuva444pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuva444pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }
    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(a_stride, height),
        ));
      }
    };
    if a.len() < a_min {
      return Err(Yuva444pFrame16Error::InsufficientAPlane(
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

  /// Constructs a new [`Yuva444pFrame16`], panicking on invalid inputs.
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
      Err(_) => panic!("invalid Yuva444pFrame16 dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample of
  /// every plane and rejects values above `(1 << BITS) - 1`. Use this
  /// on untrusted input where accepting out-of-range samples would
  /// silently corrupt the conversion via the kernels' bit-mask.
  ///
  /// Returns [`Yuva444pFrame16Error::SampleOutOfRange`] on the first
  /// offending sample. All of [`Self::try_new`]'s geometry errors are
  /// still possible.
  ///
  /// Cost: one O(plane_size) linear scan per plane (Y, U, V, A — four
  /// planes total).
  ///
  /// Per the LE-encoded byte contract documented on the type, samples
  /// are validated **after** `u16::from_le` normalization so the range
  /// check operates on the intended logical sample value on every host.
  /// On little-endian hosts `from_le` is a no-op (the host-native `u16`
  /// already matches the wire); on big-endian hosts it byte-swaps each
  /// `u16` back into host-native form before the comparison. Without
  /// this normalization a valid `yuva444p10le` plane on a BE host would
  /// have its samples appear byte-swapped (e.g. `1023` encoded LE as
  /// bytes `[0xFF, 0x03]` reads as host-native `0xFF03` on BE) and the
  /// validator would falsely reject every row. The reported `value` in
  /// the error is the normalized logical sample so callers can match it
  /// against the declared `max_valid`. Mirrors the
  /// `Yuv444pFrame16::try_new_checked` pattern.
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
  ) -> Result<Self, Yuva444pFrame16Error> {
    let frame = Self::try_new(
      y, u, v, a, width, height, y_stride, u_stride, v_stride, a_stride,
    )?;
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        // Normalize from LE-encoded wire to host-native before the
        // range check (no-op on LE host, byte-swap on BE host).
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva444pFrame16Error::SampleOutOfRange(
            Yuva444pFrame16SampleOutOfRange::new(
              Yuva444pFrame16Plane::Y,
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
      for (col, &s) in u[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva444pFrame16Error::SampleOutOfRange(
            Yuva444pFrame16SampleOutOfRange::new(
              Yuva444pFrame16Plane::U,
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
      for (col, &s) in v[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuva444pFrame16Error::SampleOutOfRange(
            Yuva444pFrame16SampleOutOfRange::new(
              Yuva444pFrame16Plane::V,
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
          return Err(Yuva444pFrame16Error::SampleOutOfRange(
            Yuva444pFrame16SampleOutOfRange::new(
              Yuva444pFrame16Plane::A,
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

  /// Y plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }
  /// U plane. Full-width, full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }
  /// V plane. Full-width, full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
  }
  /// A plane. Full-width, full-height. Native bit depth, low-bit-packed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a(&self) -> &'a [u16] {
    self.a
  }
  /// Frame width in pixels. No parity constraint.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Frame height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }
  /// Y‑plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
  /// U‑plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }
  /// V‑plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }
  /// A‑plane stride in samples.
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
  /// (`AV_PIX_FMT_YUVA444P*BE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded 4:4:4 planar with alpha, 9-bit (`AV_PIX_FMT_YUVA444P9LE`).
pub type Yuva444p9Frame<'a> = Yuva444pFrame16<'a, 9>;
/// LE-encoded 4:4:4 planar with alpha, 10-bit (`AV_PIX_FMT_YUVA444P10LE`).
pub type Yuva444p10Frame<'a> = Yuva444pFrame16<'a, 10>;
/// LE-encoded 4:4:4 planar with alpha, 12-bit (`AV_PIX_FMT_YUVA444P12LE`).
pub type Yuva444p12Frame<'a> = Yuva444pFrame16<'a, 12>;
/// LE-encoded 4:4:4 planar with alpha, 14-bit. FFmpeg does not ship
/// this depth, but the colconv 4:4:4 BITS-generic kernel templates
/// already cover it.
pub type Yuva444p14Frame<'a> = Yuva444pFrame16<'a, 14>;
/// LE-encoded 4:4:4 planar with alpha, 16-bit (`AV_PIX_FMT_YUVA444P16LE`).
/// Uses the dedicated i64 4:4:4 16-bit kernel family.
pub type Yuva444p16Frame<'a> = Yuva444pFrame16<'a, 16>;

// ---- Phase 4 — explicit LE/BE aliases for the YUVA 4:4:4 HB family ----

/// LE-encoded `Yuva444p9Frame` (`AV_PIX_FMT_YUVA444P9LE`).
pub type Yuva444p9LeFrame<'a> = Yuva444pFrame16<'a, 9, false>;
/// BE-encoded `Yuva444p9Frame` (`AV_PIX_FMT_YUVA444P9BE`).
pub type Yuva444p9BeFrame<'a> = Yuva444pFrame16<'a, 9, true>;
/// LE-encoded `Yuva444p10Frame` (`AV_PIX_FMT_YUVA444P10LE`).
pub type Yuva444p10LeFrame<'a> = Yuva444pFrame16<'a, 10, false>;
/// BE-encoded `Yuva444p10Frame` (`AV_PIX_FMT_YUVA444P10BE`).
pub type Yuva444p10BeFrame<'a> = Yuva444pFrame16<'a, 10, true>;
/// LE-encoded `Yuva444p12Frame` (`AV_PIX_FMT_YUVA444P12LE`).
pub type Yuva444p12LeFrame<'a> = Yuva444pFrame16<'a, 12, false>;
/// BE-encoded `Yuva444p12Frame` (`AV_PIX_FMT_YUVA444P12BE`).
pub type Yuva444p12BeFrame<'a> = Yuva444pFrame16<'a, 12, true>;
/// LE-encoded `Yuva444p14Frame` (no FFmpeg-shipped BE variant; provided
/// for symmetry with the rest of the family).
pub type Yuva444p14LeFrame<'a> = Yuva444pFrame16<'a, 14, false>;
/// BE-encoded `Yuva444p14Frame`.
pub type Yuva444p14BeFrame<'a> = Yuva444pFrame16<'a, 14, true>;
/// LE-encoded `Yuva444p16Frame` (`AV_PIX_FMT_YUVA444P16LE`).
pub type Yuva444p16LeFrame<'a> = Yuva444pFrame16<'a, 16, false>;
/// BE-encoded `Yuva444p16Frame` (`AV_PIX_FMT_YUVA444P16BE`).
pub type Yuva444p16BeFrame<'a> = Yuva444pFrame16<'a, 16, true>;

/// Errors returned by [`Yuva444pFrame::try_new`].
///
/// Variant shape mirrors `Yuva420pFrameError` (geometry, plane-too-short,
/// `InsufficientAStride` / `InsufficientAPlane` for the alpha plane) but
/// without `OddWidth` because 4:4:4 has no chroma subsampling, so any
/// width is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Yuva444pFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width`.
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `u_stride < width`. 4:4:4 chroma is full-width.
  #[error(transparent)]
  InsufficientUStride(InsufficientStride),

  /// `v_stride < width`. 4:4:4 chroma is full-width.
  #[error(transparent)]
  InsufficientVStride(InsufficientStride),

  /// `a_stride < width`. The alpha plane is full-width × full-height
  /// (1:1 with Y).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` bytes.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * height` bytes.
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

/// A validated YUVA 4:4:4 planar frame at 8 bits per sample
/// (`AV_PIX_FMT_YUVA444P`).
///
/// Four planes — all full-width × full-height (4:4:4 has no chroma
/// subsampling): Y, U, V, and A. Mirrors `Yuv444pFrame` plus the
/// alpha plane.
#[derive(Debug, Clone, Copy)]
pub struct Yuva444pFrame<'a> {
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

impl<'a> Yuva444pFrame<'a> {
  /// Constructs a new [`Yuva444pFrame`], validating dimensions and
  /// plane lengths.
  ///
  /// Returns [`Yuva444pFrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - any stride is smaller than `width`,
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
  ) -> Result<Self, Yuva444pFrameError> {
    if width == 0 || height == 0 {
      return Err(Yuva444pFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Yuva444pFrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    if u_stride < width {
      return Err(Yuva444pFrameError::InsufficientUStride(
        InsufficientStride::new(u_stride, width),
      ));
    }
    if v_stride < width {
      return Err(Yuva444pFrameError::InsufficientVStride(
        InsufficientStride::new(v_stride, width),
      ));
    }
    if a_stride < width {
      return Err(Yuva444pFrameError::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Yuva444pFrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrameError::GeometryOverflow(GeometryOverflow::new(
          u_stride, height,
        )));
      }
    };
    if u.len() < u_min {
      return Err(Yuva444pFrameError::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrameError::GeometryOverflow(GeometryOverflow::new(
          v_stride, height,
        )));
      }
    };
    if v.len() < v_min {
      return Err(Yuva444pFrameError::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }
    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuva444pFrameError::GeometryOverflow(GeometryOverflow::new(
          a_stride, height,
        )));
      }
    };
    if a.len() < a_min {
      return Err(Yuva444pFrameError::InsufficientAPlane(
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

  /// Constructs a new [`Yuva444pFrame`], panicking on invalid inputs.
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
      Err(_) => panic!("invalid Yuva444pFrame dimensions or plane lengths"),
    }
  }

  /// Y (luma) plane bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u8] {
    self.y
  }
  /// U (Cb) plane bytes — full-width × full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u8] {
    self.u
  }
  /// V (Cr) plane bytes — full-width × full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u8] {
    self.v
  }
  /// A (alpha) plane bytes — full-width × full-height (1:1 with Y).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a(&self) -> &'a [u8] {
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

/// Payload struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Yuva444pFrame16SampleOutOfRange {
  plane: Yuva444pFrame16Plane,
  index: usize,
  value: u16,
  max_valid: u16,
}

impl Yuva444pFrame16SampleOutOfRange {
  /// Constructs a new `Yuva444pFrame16SampleOutOfRange`.
  #[inline]
  pub const fn new(plane: Yuva444pFrame16Plane, index: usize, value: u16, max_valid: u16) -> Self {
    Self {
      plane,
      index,
      value,
      max_valid,
    }
  }
  /// Returns the `plane` field.
  #[inline]
  pub const fn plane(&self) -> Yuva444pFrame16Plane {
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
