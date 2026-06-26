//! NV20 — semi-planar 4:2:2, 10-bit, **low-bit-packed**
//! (`AV_PIX_FMT_NV20LE` / `AV_PIX_FMT_NV20BE`).
//!
//! 4:2:2 semi-planar twin of [`P210Frame`](crate::frame::P210Frame):
//! the exact same 2-plane shape — one full-size luma plane plus one
//! interleaved-UV plane at half width and **full height** (one chroma
//! row per Y row) — and the same `u16` element type. The **only**
//! difference from P210 is the bit-alignment within each `u16`:
//!
//! - **NV20** packs its 10 active bits in the **low** 10 of each `u16`
//!   (`value & 0x03FF`; the high 6 bits are zero). This matches
//!   FFmpeg's `AV_PIX_FMT_NV20LE` descriptor — `shift = 0` for every
//!   component.
//! - **P210** packs its 10 active bits in the **high** 10
//!   (`value << 6`; the low 6 bits are zero) — `shift = 6` in FFmpeg's
//!   `AV_PIX_FMT_P210LE` descriptor.
//!
//! # Layout is two planes, NOT a tight bit-packed stream
//!
//! Despite the loose "8 channels in 5 16-bit words" shorthand sometimes
//! attached to this format, NV20 is **not** tight bit-packing (that
//! would put 8 ten-bit samples into 80 bits). FFmpeg's `pixdesc.c`
//! descriptor gives `step = 2` *bytes* per component — i.e. each 10-bit
//! sample occupies a full 16-bit word — across two planes:
//!
//! ```text
//! Y  { plane = 0, step = 2, offset = 0, shift = 0, depth = 10 }
//! U  { plane = 1, step = 4, offset = 0, shift = 0, depth = 10 }
//! V  { plane = 1, step = 4, offset = 2, shift = 0, depth = 10 }
//! flags = PLANAR,  log2_chroma_w = 1,  log2_chroma_h = 0
//! ```
//!
//! So the storage is identical to [`P210Frame`] (genuinely semi-planar,
//! one `u16` per sample), and only the per-`u16` bit position differs.
//!
//! # Why a dedicated frame type instead of reusing `PnFrame422`
//!
//! [`PnFrame422`](crate::frame::PnFrame422) bakes in the **high**-bit
//! convention: its docs promise "10 active bits in the high 10
//! positions, low 6 zero" and its `try_new_checked` rejects samples
//! whose **low** `16 - BITS` bits are non-zero. For NV20 the active
//! bits live in the **low** 10, so that check is exactly inverted — it
//! would reject valid NV20 content (anything with low-bit signal) and
//! wave through high-bit garbage. NV20 therefore carries its own frame
//! type with the low-bit contract, mirroring how high-bit
//! [`PnFrame`](crate::frame::PnFrame) (P010) is deliberately kept
//! separate from low-bit `Yuv420pFrame16` (yuv420p10le).
//!
//! Stride is in **`u16` samples**, not bytes. Callers holding an FFmpeg
//! byte buffer must cast via `bytemuck::cast_slice` and divide
//! `linesize[i]` by 2 before constructing.
//!
//! Two planes:
//! - `y` — full-size luma, `y_stride >= width`, length
//!   `>= y_stride * height` (all in `u16` samples).
//! - `uv` — interleaved chroma (`U0, V0, U1, V1, …`) at **half width ×
//!   full height**, so each chroma row holds `width` `u16` elements
//!   (= `width / 2` interleaved `(U, V)` pairs); `uv_stride >= width`,
//!   length `>= uv_stride * height`.
//!
//! `width` must be even (4:2:2 subsamples chroma 2:1 horizontally and
//! pairs `(U, V)`); `height` has no parity constraint.

use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, WidthAlignment, ZeroDimension,
};
use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

/// A validated NV20 (semi-planar 4:2:2, 10-bit, low-bit-packed `u16`)
/// frame.
///
/// See this module's top-level docs for the full layout and the
/// high-vs-low-bit distinction from [`P210Frame`].
///
/// # Endian contract — `<const BE: bool = false>`
///
/// The `<const BE: bool>` parameter records the per-`u16` byte order of
/// the plane data: `false` (default) → LE-encoded words
/// (`AV_PIX_FMT_NV20LE`), `true` → BE-encoded words
/// (`AV_PIX_FMT_NV20BE`). Geometry validation is endian-independent;
/// downstream row kernels normalize each `u16` from the recorded byte
/// order before masking out the low 10 active bits.
///
/// # Aliases
/// - [`Nv20LeFrame`] = `Nv20Frame<'a, false>` — explicit LE (default).
/// - [`Nv20BeFrame`] = `Nv20Frame<'a, true>` — explicit BE.
///
/// [`P210Frame`]: crate::frame::P210Frame
#[derive(Debug, Clone, Copy)]
pub struct Nv20Frame<'a, const BE: bool = false> {
  y: &'a [u16],
  uv: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  uv_stride: u32,
}

/// LE-encoded `Nv20Frame` (`AV_PIX_FMT_NV20LE`). Equivalent to
/// `Nv20Frame<'a>` (the default `BE = false`); provided as an explicit
/// alias for callers who want to document the endianness at the type
/// level.
pub type Nv20LeFrame<'a> = Nv20Frame<'a, false>;

/// BE-encoded `Nv20Frame` (`AV_PIX_FMT_NV20BE`) — each `u16`'s bytes are
/// big-endian; the downstream row kernels byte-swap before masking the
/// low 10 active bits.
pub type Nv20BeFrame<'a> = Nv20Frame<'a, true>;

impl<'a, const BE: bool> Nv20Frame<'a, BE> {
  /// Constructs a new [`Nv20Frame`], validating dimensions and plane
  /// lengths. Strides are in `u16` **samples**.
  ///
  /// Returns [`Nv20FrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - `width` is odd (4:2:2 subsamples chroma 2:1 in width and pairs
  ///   `(U, V)`),
  /// - `y_stride < width`,
  /// - `uv_stride < width` (the chroma row holds `width / 2`
  ///   interleaved pairs = `width` `u16` elements),
  /// - `uv_stride` is odd (an odd `u16`-element stride would start
  ///   alternate chroma rows on the `V` element of the previous pair,
  ///   swapping the U / V interpretation),
  /// - either plane is too short, or
  /// - `stride * rows` overflows `usize` (32-bit targets only).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    y: &'a [u16],
    uv: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    uv_stride: u32,
  ) -> Result<Self, Nv20FrameError> {
    if width == 0 || height == 0 {
      return Err(Nv20FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Nv20FrameError::WidthAlignment(WidthAlignment::odd(
        width as usize,
      )));
    }
    if y_stride < width {
      return Err(Nv20FrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    // Each chroma row carries `width / 2` interleaved `(U, V)` pairs =
    // `width` `u16` elements.
    let uv_row_elems = width;
    if uv_stride < uv_row_elems {
      return Err(Nv20FrameError::InsufficientUvStride(
        InsufficientStride::new(uv_stride, uv_row_elems),
      ));
    }
    // Interleaved UV is consecutive `(U, V)` u16 pairs. An odd
    // u16-element stride would start every other chroma row on the V
    // element of the previous pair, swapping the U / V interpretation
    // deterministically and producing wrong colors on alternate rows.
    if uv_stride & 1 != 0 {
      return Err(Nv20FrameError::UvStrideOdd(Nv20UvStrideOdd::new(uv_stride)));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Nv20FrameError::GeometryOverflow(GeometryOverflow::new(
          y_stride, height,
        )));
      }
    };
    if y.len() < y_min {
      return Err(Nv20FrameError::InsufficientYPlane(InsufficientPlane::new(
        y_min,
        y.len(),
      )));
    }
    // 4:2:2 chroma is full-height — one chroma row per Y row (no
    // `div_ceil(2)`).
    let uv_min = match (uv_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Nv20FrameError::GeometryOverflow(GeometryOverflow::new(
          uv_stride, height,
        )));
      }
    };
    if uv.len() < uv_min {
      return Err(Nv20FrameError::InsufficientUvPlane(InsufficientPlane::new(
        uv_min,
        uv.len(),
      )));
    }

    Ok(Self {
      y,
      uv,
      width,
      height,
      y_stride,
      uv_stride,
    })
  }

  /// Constructs a new [`Nv20Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    y: &'a [u16],
    uv: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    uv_stride: u32,
  ) -> Self {
    match Self::try_new(y, uv, width, height, y_stride, uv_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Nv20Frame dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample and
  /// rejects any whose **high 6 bits** are non‑zero. NV20 packs its 10
  /// active bits in the **low** 10 of each `u16` (`value & 0x03FF`), so
  /// a valid sample always has `value & 0xFC00 == 0`; non‑zero high bits
  /// is evidence the buffer isn't NV20‑shaped — most often a
  /// high‑bit‑packed [`P210Frame`] buffer (active bits in the high 10)
  /// handed to the NV20 path, whose low‑10 mask would silently misread
  /// it. This is the exact **inverse** of
  /// [`PnFrame422::try_new_checked`], which rejects non‑zero **low** 6
  /// bits for the high‑bit‑packed P210 family.
  ///
  /// **This is a packing sanity check, not a provenance validator.** It
  /// catches noisy high‑bit‑packed data (where most samples carry
  /// high‑bit content), but it **cannot** distinguish NV20 from a
  /// high‑bit‑packed buffer whose samples all happen to be `<= 0x03FF`
  /// (e.g. a near‑black P210 region). At 10‑bit the check rejects 63/64
  /// random high‑bit patterns and is a strong signal; callers needing
  /// strict provenance must rely on their source format metadata and
  /// pick the right frame type (`Nv20Frame` vs [`P210Frame`]) at
  /// construction.
  ///
  /// Cost: one O(width × height) scan per plane. The default
  /// [`Self::try_new`] skips this so the hot path stays O(1).
  ///
  /// Returns [`Nv20FrameError::StrayHighBits`] on the first offending
  /// sample — carries the plane, element index, and offending value.
  ///
  /// Per the byte-order contract recorded by `<const BE: bool>`, samples
  /// are validated **after** `u16::from_le` / `u16::from_be`
  /// normalization so the bit check operates on the intended logical
  /// sample value on every host. On an LE-encoded frame (`BE == false`)
  /// `from_le` is a no-op on LE hosts and byte-swaps on BE hosts; on a
  /// BE-encoded frame (`BE == true`) `from_be` byte-swaps on LE hosts.
  /// Without this normalization a valid `NV20LE` plane on a BE host
  /// would have its low-aligned samples appear byte-swapped (active bits
  /// landing in the high byte) and the validator would falsely reject
  /// every row. The reported `value` in the error is the normalized
  /// logical sample.
  ///
  /// [`P210Frame`]: crate::frame::P210Frame
  /// [`PnFrame422::try_new_checked`]: crate::frame::PnFrame422::try_new_checked
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_new_checked(
    y: &'a [u16],
    uv: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    uv_stride: u32,
  ) -> Result<Self, Nv20FrameError> {
    let frame = Self::try_new(y, uv, width, height, y_stride, uv_stride)?;
    // NV20 carries its 10 active bits in the LOW 10 of each `u16`; the
    // high 6 must be zero. This is the inverse of the P210 low-bit mask.
    const HIGH_MASK: u16 = 0xFC00;
    let w = width as usize;
    let h = height as usize;
    let uv_w = w; // half-width × 2 elements per pair = width u16 elements per row
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        // Normalize from the recorded byte order to host-native before
        // the bit check (no-op on a matching-endian host, byte-swap
        // otherwise).
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical & HIGH_MASK != 0 {
          return Err(Nv20FrameError::StrayHighBits(Nv20StrayHighBits::new(
            Nv20FramePlane::Y,
            start + col,
            logical,
          )));
        }
      }
    }
    // 4:2:2: scan every chroma row (full-height).
    for row in 0..h {
      let start = row * uv_stride as usize;
      for (col, &s) in uv[start..start + uv_w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical & HIGH_MASK != 0 {
          return Err(Nv20FrameError::StrayHighBits(Nv20StrayHighBits::new(
            Nv20FramePlane::Uv,
            start + col,
            logical,
          )));
        }
      }
    }
    Ok(frame)
  }

  /// Y (luma) plane samples. Row `r` starts at sample offset
  /// `r * y_stride()`. Each sample's 10 active bits sit in the **low**
  /// 10 positions of the `u16` (high 6 bits zero).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }

  /// Interleaved UV plane samples. Each chroma row starts at sample
  /// offset `row * uv_stride()` (4:2:2 — one chroma row per Y row) and
  /// contains `width` `u16` elements laid out as
  /// `U0, V0, U1, V1, …, U_{w/2-1}, V_{w/2-1}`. Each element's 10
  /// active bits sit in the **low** 10 positions.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn uv(&self) -> &'a [u16] {
    self.uv
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

  /// Sample stride of the Y plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }

  /// Sample stride of the interleaved UV plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn uv_stride(&self) -> u32 {
    self.uv_stride
  }

  /// Compile-time BE flag mirror — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_NV20BE`), `false` if LE-encoded
  /// (`AV_PIX_FMT_NV20LE`). Runtime mirror of the `<const BE: bool>`
  /// type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Identifies which plane of an [`Nv20Frame`] a
/// [`Nv20FrameError::StrayHighBits`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Nv20FramePlane {
  /// Luma plane.
  Y,
  /// Interleaved UV plane.
  Uv,
}

/// Errors returned by [`Nv20Frame::try_new`] and
/// [`Nv20Frame::try_new_checked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Nv20FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `width` was odd. 4:2:2 subsamples chroma 2:1 in width, so each
  /// chroma column pairs two Y columns; odd widths leave the last Y
  /// column without a paired chroma sample.
  #[error(transparent)]
  WidthAlignment(WidthAlignment),

  /// `y_stride < width` (in `u16` samples).
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `uv_stride` is smaller than the `width` `u16` elements of
  /// interleaved UV payload one chroma row must hold.
  #[error(transparent)]
  InsufficientUvStride(InsufficientStride),

  /// `uv_stride` is odd. Each interleaved chroma row is laid out as
  /// `(U, V)` pairs of `u16` elements; an odd stride starts every other
  /// row on the opposite element of the pair, swapping the U / V
  /// interpretation deterministically and producing wrong colors on
  /// alternate rows.
  #[error(
    "uv_stride ({}) is odd; semi-planar interleaved UV requires an even u16-element stride", .0.uv_stride()
  )]
  UvStrideOdd(Nv20UvStrideOdd),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// UV plane is shorter than `uv_stride * height` samples (4:2:2
  /// chroma is full-height).
  #[error(transparent)]
  InsufficientUvPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (only reachable on 32-bit
  /// targets — wasm32, i686 — with extreme dimensions).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A sample's high 6 bits were non‑zero — an NV20 sample packs its 10
  /// active bits in the **low** 10 of each `u16`, so valid samples are
  /// always `<= 0x03FF` (`value & 0xFC00 == 0`). Only
  /// [`Nv20Frame::try_new_checked`] can produce this error; the
  /// geometry-only [`Nv20Frame::try_new`] never inspects samples.
  ///
  /// Note: the absence of this error does **not** prove the buffer is
  /// NV20. A high‑bit‑packed buffer of samples that all happen to be
  /// `<= 0x03FF` passes the check silently. See
  /// [`Nv20Frame::try_new_checked`] for the full discussion. This is the
  /// inverse of `PnFrameError::SampleLowBitsSet` on the high‑bit‑packed
  /// P210 family.
  #[error(
    "sample {:#06x} on plane {} at element {} has non-zero high 6 bits (not a valid NV20 low-bit-packed 10-bit sample)", .0.value(), .0.plane(), .0.index()
  )]
  StrayHighBits(Nv20StrayHighBits),
}

/// Payload for [`Nv20FrameError::UvStrideOdd`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nv20UvStrideOdd {
  uv_stride: u32,
}

impl Nv20UvStrideOdd {
  /// Constructs a new `Nv20UvStrideOdd`.
  #[inline]
  pub const fn new(uv_stride: u32) -> Self {
    Self { uv_stride }
  }
  /// Returns the offending `uv_stride`.
  #[inline]
  pub const fn uv_stride(&self) -> u32 {
    self.uv_stride
  }
}

/// Payload for [`Nv20FrameError::StrayHighBits`]. Mirrors the high‑bit
/// P210 family's `PnSampleLowBitsSet` payload with the contract
/// inverted: NV20's active bits are the low 10, so this records a sample
/// whose **high** 6 bits were set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nv20StrayHighBits {
  plane: Nv20FramePlane,
  index: usize,
  value: u16,
}

impl Nv20StrayHighBits {
  /// Constructs a new `Nv20StrayHighBits`.
  #[inline]
  pub const fn new(plane: Nv20FramePlane, index: usize, value: u16) -> Self {
    Self {
      plane,
      index,
      value,
    }
  }
  /// Returns the `plane` the offending sample lives on.
  #[inline]
  pub const fn plane(&self) -> Nv20FramePlane {
    self.plane
  }
  /// Returns the element `index` (in `u16` samples, from the plane base)
  /// of the offending sample.
  #[inline]
  pub const fn index(&self) -> usize {
    self.index
  }
  /// Returns the offending (byte-order-normalized) sample `value`.
  #[inline]
  pub const fn value(&self) -> u16 {
    self.value
  }
}
