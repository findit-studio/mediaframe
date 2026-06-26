//! High-bit-depth planar GBR (`AV_PIX_FMT_GBRP{9,10,12,14,16}{LE,BE}`) and
//! planar GBR+A (`AV_PIX_FMT_GBRAP{10,12,14,16}{LE,BE}`) source frames, plus
//! the MSB-packed 10/12-bit GBR pair (`AV_PIX_FMT_GBRP{10,12}MSB{LE,BE}`) and
//! the 32-bit-per-channel GBR+A frame (`AV_PIX_FMT_GBRAP32{LE,BE}`).
//!
//! All formats are *planar RGB* — three (or four) full-resolution planes,
//! one per channel, in **G, B, R** order (FFmpeg convention).
//! `GbrapHighBitFrame` / [`Gbrap32Frame`] add a fourth full-resolution alpha
//! plane.
//!
//! # Bit packing
//!
//! - [`GbrpHighBitFrame`] / [`GbrapHighBitFrame`] store their samples in the
//!   **low `BITS` bits** of each `u16` element (upper `16 − BITS` bits zero),
//!   matching FFmpeg's `gbrp{9,10,12,14,16}{le,be}` / `gbrap{10,12,14,16}{le,be}`
//!   descriptors (`shift = 0`).
//! - [`GbrpMsbFrame`] stores its samples in the **high `BITS` bits**
//!   (MSB-aligned, low `16 − BITS` bits zero), matching FFmpeg's
//!   `gbrp{10,12}msb{le,be}` descriptors (`shift = 6` at 10-bit, `shift = 4`
//!   at 12-bit). This is the exact inverse of [`GbrpHighBitFrame`], so it is a
//!   **dedicated** type with its own [`GbrpMsbFrame::try_new_checked`] that
//!   rejects stray **low** bits — mirroring how low-bit [`Nv20Frame`] is kept
//!   separate from high-bit `P210Frame`, but inverted (here this type is the
//!   high-bit one and [`GbrpHighBitFrame`] is the low-bit one).
//! - [`Gbrap32Frame`] uses four `u32` planes with **all 32 bits active** (full
//!   range, `shift = 0`, `depth = 32`); there is no stray-bit contract.
//!
//! (FFmpeg has no `gbrap9` — only the 3-plane `gbrp9` exists at 9 bits, so
//! `GbrapHighBitFrame` accepts only `BITS ∈ {10, 12, 14, 16}`.)
//! Callers with `u16` byte buffers from FFmpeg must cast via
//! [`bytemuck::cast_slice`] and divide `linesize[i]` by 2 before construction
//! (by 4 for the `u32` [`Gbrap32Frame`]).
//!
//! [`Nv20Frame`]: crate::frame::Nv20Frame
//! [`P210Frame`]: crate::frame::P210Frame
//!
//! # Endian contract — `<const BE: bool = false>`
//!
//! Each frame type carries a `<const BE: bool>` parameter that defaults to
//! `false` (LE-encoded plane bytes, matching the FFmpeg `*LE` suffix). Set
//! `BE = true` to consume `*BE`-encoded plane bytes; row kernels perform the
//! byte-swap (or no-op) under the hood — callers do **not** pre-swap. The
//! `BE` parameter on `Frame` propagates through the walker
//! (`gbrpN_to::<BE>(...)`) into the sinker dispatch
//! (`MixedSinker<GbrpN<BE>>`), which monomorphizes the kernel call as
//! `gbr_to_*_high_bit_row::<BITS, BE>(...)`.
//!
//! # Stride semantics
//!
//! **Stride is in samples (`u16` elements)**, not bytes. Each plane row
//! `r` starts at sample offset `r * *_stride`.
//!
//! # Sample-value range
//!
//! `try_new` validates geometry only. Out-of-range samples (upper bits
//! set) are masked by `& ((1 << BITS) - 1)` inside every kernel, giving
//! stable deterministic output. Scanning every sample at video rates is
//! prohibitive — same rationale as `Yuv420pFrame16`.

use super::{GeometryOverflow, InsufficientPlane, InsufficientStride, ZeroDimension};
use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

/// A validated planar GBR frame at high bit depth (`AV_PIX_FMT_GBRP{9,10,12,14,16}{LE,BE}`).
///
/// Three full-resolution `u16` planes in **G, B, R** order:
/// - `g` — green plane.
/// - `b` — blue plane.
/// - `r` — red plane.
///
/// `BITS ∈ {9, 10, 12, 14, 16}` — validated by a compile-time
/// `const` assertion at construction. Stride is in **samples** (`u16`
/// elements); each plane requires `*_stride >= width` and
/// `len >= *_stride * height`. No width/height parity constraint.
///
/// The `<const BE: bool>` parameter selects the plane byte order:
/// `false` (default) → LE-encoded `*LE` bytes, `true` → BE-encoded
/// `*BE` bytes. Downstream row kernels handle the byte-swap (or no-op)
/// under the hood — callers do **not** pre-swap.
///
/// Use the per-depth type aliases [`Gbrp9Frame`], [`Gbrp10Frame`],
/// [`Gbrp12Frame`], [`Gbrp14Frame`], [`Gbrp16Frame`] at call sites,
/// or the `*Le*`/`*Be*` aliases for explicit endianness.
#[derive(Debug, Clone, Copy)]
pub struct GbrpHighBitFrame<'a, const BITS: u32, const BE: bool = false> {
  g: &'a [u16],
  b: &'a [u16],
  r: &'a [u16],
  width: u32,
  height: u32,
  g_stride: u32,
  b_stride: u32,
  r_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> GbrpHighBitFrame<'a, BITS, BE> {
  /// Constructs a new [`GbrpHighBitFrame`], validating dimensions and
  /// plane lengths. Returns [`GbrpHighBitFrameError`] if any of:
  /// - `BITS ∉ {9, 10, 12, 14, 16}` — caught at compile time via
  ///   `const { assert!(…) }`, so misuse is a compile error rather than
  ///   a runtime error,
  /// - `width` or `height` is zero,
  /// - any stride is smaller than `width` (in samples),
  /// - `stride * height` overflows `usize` (32-bit targets only),
  /// - any plane is shorter than `stride * height` samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
  ) -> Result<Self, GbrpHighBitFrameError> {
    const {
      assert!(
        matches!(BITS, 9 | 10 | 12 | 14 | 16),
        "BITS must be one of 9, 10, 12, 14, or 16",
      );
    }

    if width == 0 || height == 0 {
      return Err(GbrpHighBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if g_stride < width {
      return Err(GbrpHighBitFrameError::InsufficientGStride(
        InsufficientStride::new(g_stride, width),
      ));
    }
    if b_stride < width {
      return Err(GbrpHighBitFrameError::InsufficientBStride(
        InsufficientStride::new(b_stride, width),
      ));
    }
    if r_stride < width {
      return Err(GbrpHighBitFrameError::InsufficientRStride(
        InsufficientStride::new(r_stride, width),
      ));
    }

    let g_min = match (g_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrpHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(g_stride, height),
        ));
      }
    };
    if g.len() < g_min {
      return Err(GbrpHighBitFrameError::InsufficientGPlane(
        InsufficientPlane::new(g_min, g.len()),
      ));
    }

    let b_min = match (b_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrpHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(b_stride, height),
        ));
      }
    };
    if b.len() < b_min {
      return Err(GbrpHighBitFrameError::InsufficientBPlane(
        InsufficientPlane::new(b_min, b.len()),
      ));
    }

    let r_min = match (r_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrpHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(r_stride, height),
        ));
      }
    };
    if r.len() < r_min {
      return Err(GbrpHighBitFrameError::InsufficientRPlane(
        InsufficientPlane::new(r_min, r.len()),
      ));
    }

    Ok(Self {
      g,
      b,
      r,
      width,
      height,
      g_stride,
      b_stride,
      r_stride,
    })
  }

  /// Constructs a new [`GbrpHighBitFrame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
  ) -> Self {
    match Self::try_new(g, b, r, width, height, g_stride, b_stride, r_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid GbrpHighBitFrame dimensions or plane lengths"),
    }
  }

  /// Green plane samples. Row `r` starts at sample offset `r * g_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(&self) -> &'a [u16] {
    self.g
  }
  /// Blue plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(&self) -> &'a [u16] {
    self.b
  }
  /// Red plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(&self) -> &'a [u16] {
    self.r
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
  /// Sample stride of the green plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g_stride(&self) -> u32 {
    self.g_stride
  }
  /// Sample stride of the blue plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b_stride(&self) -> u32 {
    self.b_stride
  }
  /// Sample stride of the red plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r_stride(&self) -> u32 {
    self.r_stride
  }
  /// Active bit depth — one of 9, 10, 12, 14, or 16. Mirrors the `BITS`
  /// const parameter so generic code can read it without naming the type.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`*BE`), `false` if LE-encoded (`*LE`). Runtime mirror of the
  /// `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }

  // ---- crate-internal Y/U/V aliases ------------------------------------
  //
  // The shared `walker!` macro uses fixed `y/u/v` field-name conventions
  // (`src.y()`, `src.u_stride()`, etc.). To reuse the macro verbatim for
  // planar GBR — whose externally-correct accessor names are `g/b/r` —
  // we expose `pub(crate)` aliases: `y == g`, `u == b`, `v == r`.
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y(&self) -> &'a [u16] {
    self.g
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u(&self) -> &'a [u16] {
    self.b
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v(&self) -> &'a [u16] {
    self.r
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y_stride(&self) -> u32 {
    self.g_stride
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u_stride(&self) -> u32 {
    self.b_stride
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v_stride(&self) -> u32 {
    self.r_stride
  }
}

/// Errors returned by [`GbrpHighBitFrame::try_new`].
///
/// Variant shape mirrors [`super::GbrpFrameError`] but with all sizes
/// expressed in **samples** (`u16` elements) instead of bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum GbrpHighBitFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `g_stride < width` (in samples).
  #[error(transparent)]
  InsufficientGStride(InsufficientStride),

  /// `b_stride < width` (in samples).
  #[error(transparent)]
  InsufficientBStride(InsufficientStride),

  /// `r_stride < width` (in samples).
  #[error(transparent)]
  InsufficientRStride(InsufficientStride),

  /// G plane is shorter than `g_stride * height` samples.
  #[error(transparent)]
  InsufficientGPlane(InsufficientPlane),

  /// B plane is shorter than `b_stride * height` samples.
  #[error(transparent)]
  InsufficientBPlane(InsufficientPlane),

  /// R plane is shorter than `r_stride * height` samples.
  #[error(transparent)]
  InsufficientRPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

/// Type alias for a validated planar GBR 9-bit frame
/// (`AV_PIX_FMT_GBRP9{LE,BE}`). Samples in the low 9 bits of each `u16`.
/// Defaults to LE; use [`Gbrp9BeFrame`] / [`Gbrp9LeFrame`] for explicit
/// endianness.
pub type Gbrp9Frame<'a, const BE: bool = false> = GbrpHighBitFrame<'a, 9, BE>;
/// LE-encoded `Gbrp9Frame` (`AV_PIX_FMT_GBRP9LE`).
pub type Gbrp9LeFrame<'a> = GbrpHighBitFrame<'a, 9, false>;
/// BE-encoded `Gbrp9Frame` (`AV_PIX_FMT_GBRP9BE`).
pub type Gbrp9BeFrame<'a> = GbrpHighBitFrame<'a, 9, true>;

/// Type alias for a validated planar GBR 10-bit frame
/// (`AV_PIX_FMT_GBRP10{LE,BE}`). Samples in the low 10 bits of each `u16`.
pub type Gbrp10Frame<'a, const BE: bool = false> = GbrpHighBitFrame<'a, 10, BE>;
/// LE-encoded `Gbrp10Frame` (`AV_PIX_FMT_GBRP10LE`).
pub type Gbrp10LeFrame<'a> = GbrpHighBitFrame<'a, 10, false>;
/// BE-encoded `Gbrp10Frame` (`AV_PIX_FMT_GBRP10BE`).
pub type Gbrp10BeFrame<'a> = GbrpHighBitFrame<'a, 10, true>;

/// Type alias for a validated planar GBR 12-bit frame
/// (`AV_PIX_FMT_GBRP12{LE,BE}`). Samples in the low 12 bits of each `u16`.
pub type Gbrp12Frame<'a, const BE: bool = false> = GbrpHighBitFrame<'a, 12, BE>;
/// LE-encoded `Gbrp12Frame` (`AV_PIX_FMT_GBRP12LE`).
pub type Gbrp12LeFrame<'a> = GbrpHighBitFrame<'a, 12, false>;
/// BE-encoded `Gbrp12Frame` (`AV_PIX_FMT_GBRP12BE`).
pub type Gbrp12BeFrame<'a> = GbrpHighBitFrame<'a, 12, true>;

/// Type alias for a validated planar GBR 14-bit frame
/// (`AV_PIX_FMT_GBRP14{LE,BE}`). Samples in the low 14 bits of each `u16`.
pub type Gbrp14Frame<'a, const BE: bool = false> = GbrpHighBitFrame<'a, 14, BE>;
/// LE-encoded `Gbrp14Frame` (`AV_PIX_FMT_GBRP14LE`).
pub type Gbrp14LeFrame<'a> = GbrpHighBitFrame<'a, 14, false>;
/// BE-encoded `Gbrp14Frame` (`AV_PIX_FMT_GBRP14BE`).
pub type Gbrp14BeFrame<'a> = GbrpHighBitFrame<'a, 14, true>;

/// Type alias for a validated planar GBR 16-bit frame
/// (`AV_PIX_FMT_GBRP16{LE,BE}`). Full `u16` range — all 16 bits active.
pub type Gbrp16Frame<'a, const BE: bool = false> = GbrpHighBitFrame<'a, 16, BE>;
/// LE-encoded `Gbrp16Frame` (`AV_PIX_FMT_GBRP16LE`).
pub type Gbrp16LeFrame<'a> = GbrpHighBitFrame<'a, 16, false>;
/// BE-encoded `Gbrp16Frame` (`AV_PIX_FMT_GBRP16BE`).
pub type Gbrp16BeFrame<'a> = GbrpHighBitFrame<'a, 16, true>;

// ---------------------------------------------------------------------------

/// A validated planar GBR+A frame at high bit depth
/// (`AV_PIX_FMT_GBRAP{10,12,14,16}{LE,BE}`).
///
/// Four full-resolution `u16` planes in **G, B, R, A** order:
/// - `g` / `b` / `r` — colour planes.
/// - `a` — alpha plane (1:1 with G; real per-pixel alpha).
///
/// `BITS ∈ {10, 12, 14, 16}` — validated at compile time. Stride is
/// in **samples** (`u16` elements); each plane requires
/// `*_stride >= width` and `len >= *_stride * height`.
///
/// The `<const BE: bool>` parameter selects the plane byte order; see
/// [`GbrpHighBitFrame`] for the contract.
///
/// Use the per-depth aliases [`Gbrap10Frame`] through [`Gbrap16Frame`].
/// (FFmpeg has no GBRAP9 variant — only the 3-plane GBRP9 exists at 9 bits.)
#[derive(Debug, Clone, Copy)]
pub struct GbrapHighBitFrame<'a, const BITS: u32, const BE: bool = false> {
  g: &'a [u16],
  b: &'a [u16],
  r: &'a [u16],
  a: &'a [u16],
  width: u32,
  height: u32,
  g_stride: u32,
  b_stride: u32,
  r_stride: u32,
  a_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> GbrapHighBitFrame<'a, BITS, BE> {
  /// Constructs a new [`GbrapHighBitFrame`], validating dimensions and
  /// plane lengths.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    a: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
    a_stride: u32,
  ) -> Result<Self, GbrapHighBitFrameError> {
    const {
      assert!(
        matches!(BITS, 10 | 12 | 14 | 16),
        "BITS must be one of 10, 12, 14, or 16 (FFmpeg has no GBRAP9 variant)",
      );
    }

    if width == 0 || height == 0 {
      return Err(GbrapHighBitFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if g_stride < width {
      return Err(GbrapHighBitFrameError::InsufficientGStride(
        InsufficientStride::new(g_stride, width),
      ));
    }
    if b_stride < width {
      return Err(GbrapHighBitFrameError::InsufficientBStride(
        InsufficientStride::new(b_stride, width),
      ));
    }
    if r_stride < width {
      return Err(GbrapHighBitFrameError::InsufficientRStride(
        InsufficientStride::new(r_stride, width),
      ));
    }
    if a_stride < width {
      return Err(GbrapHighBitFrameError::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let g_min = match (g_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrapHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(g_stride, height),
        ));
      }
    };
    if g.len() < g_min {
      return Err(GbrapHighBitFrameError::InsufficientGPlane(
        InsufficientPlane::new(g_min, g.len()),
      ));
    }

    let b_min = match (b_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrapHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(b_stride, height),
        ));
      }
    };
    if b.len() < b_min {
      return Err(GbrapHighBitFrameError::InsufficientBPlane(
        InsufficientPlane::new(b_min, b.len()),
      ));
    }

    let r_min = match (r_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrapHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(r_stride, height),
        ));
      }
    };
    if r.len() < r_min {
      return Err(GbrapHighBitFrameError::InsufficientRPlane(
        InsufficientPlane::new(r_min, r.len()),
      ));
    }

    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrapHighBitFrameError::GeometryOverflow(
          GeometryOverflow::new(a_stride, height),
        ));
      }
    };
    if a.len() < a_min {
      return Err(GbrapHighBitFrameError::InsufficientAPlane(
        InsufficientPlane::new(a_min, a.len()),
      ));
    }

    Ok(Self {
      g,
      b,
      r,
      a,
      width,
      height,
      g_stride,
      b_stride,
      r_stride,
      a_stride,
    })
  }

  /// Constructs a new [`GbrapHighBitFrame`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    a: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
    a_stride: u32,
  ) -> Self {
    match Self::try_new(
      g, b, r, a, width, height, g_stride, b_stride, r_stride, a_stride,
    ) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid GbrapHighBitFrame dimensions or plane lengths"),
    }
  }

  /// Green plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(&self) -> &'a [u16] {
    self.g
  }
  /// Blue plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(&self) -> &'a [u16] {
    self.b
  }
  /// Red plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(&self) -> &'a [u16] {
    self.r
  }
  /// Alpha plane samples — full-width × full-height (1:1 with G).
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
  /// Sample stride of the green plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g_stride(&self) -> u32 {
    self.g_stride
  }
  /// Sample stride of the blue plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b_stride(&self) -> u32 {
    self.b_stride
  }
  /// Sample stride of the red plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r_stride(&self) -> u32 {
    self.r_stride
  }
  /// Sample stride of the alpha plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a_stride(&self) -> u32 {
    self.a_stride
  }
  /// Active bit depth — one of 10, 12, 14, or 16.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`*BE`), `false` if LE-encoded (`*LE`). Runtime mirror of the
  /// `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }

  // ---- crate-internal Y/U/V aliases ------------------------------------
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y(&self) -> &'a [u16] {
    self.g
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u(&self) -> &'a [u16] {
    self.b
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v(&self) -> &'a [u16] {
    self.r
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y_stride(&self) -> u32 {
    self.g_stride
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u_stride(&self) -> u32 {
    self.b_stride
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v_stride(&self) -> u32 {
    self.r_stride
  }
  // `a_stride` already has the right name — no alias needed.
}

/// Errors returned by [`GbrapHighBitFrame::try_new`].
///
/// Mirrors [`GbrpHighBitFrameError`] extended with `A`-plane variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum GbrapHighBitFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `g_stride < width` (in samples).
  #[error(transparent)]
  InsufficientGStride(InsufficientStride),

  /// `b_stride < width` (in samples).
  #[error(transparent)]
  InsufficientBStride(InsufficientStride),

  /// `r_stride < width` (in samples).
  #[error(transparent)]
  InsufficientRStride(InsufficientStride),

  /// `a_stride < width` (in samples).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// G plane is shorter than `g_stride * height` samples.
  #[error(transparent)]
  InsufficientGPlane(InsufficientPlane),

  /// B plane is shorter than `b_stride * height` samples.
  #[error(transparent)]
  InsufficientBPlane(InsufficientPlane),

  /// R plane is shorter than `r_stride * height` samples.
  #[error(transparent)]
  InsufficientRPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` samples.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

/// Type alias for a validated planar GBR+A 10-bit frame
/// (`AV_PIX_FMT_GBRAP10{LE,BE}`). Samples in the low 10 bits of each `u16`.
pub type Gbrap10Frame<'a, const BE: bool = false> = GbrapHighBitFrame<'a, 10, BE>;
/// LE-encoded `Gbrap10Frame` (`AV_PIX_FMT_GBRAP10LE`).
pub type Gbrap10LeFrame<'a> = GbrapHighBitFrame<'a, 10, false>;
/// BE-encoded `Gbrap10Frame` (`AV_PIX_FMT_GBRAP10BE`).
pub type Gbrap10BeFrame<'a> = GbrapHighBitFrame<'a, 10, true>;

/// Type alias for a validated planar GBR+A 12-bit frame
/// (`AV_PIX_FMT_GBRAP12{LE,BE}`). Samples in the low 12 bits of each `u16`.
pub type Gbrap12Frame<'a, const BE: bool = false> = GbrapHighBitFrame<'a, 12, BE>;
/// LE-encoded `Gbrap12Frame` (`AV_PIX_FMT_GBRAP12LE`).
pub type Gbrap12LeFrame<'a> = GbrapHighBitFrame<'a, 12, false>;
/// BE-encoded `Gbrap12Frame` (`AV_PIX_FMT_GBRAP12BE`).
pub type Gbrap12BeFrame<'a> = GbrapHighBitFrame<'a, 12, true>;

/// Type alias for a validated planar GBR+A 14-bit frame
/// (`AV_PIX_FMT_GBRAP14{LE,BE}`). Samples in the low 14 bits of each `u16`.
pub type Gbrap14Frame<'a, const BE: bool = false> = GbrapHighBitFrame<'a, 14, BE>;
/// LE-encoded `Gbrap14Frame` (`AV_PIX_FMT_GBRAP14LE`).
pub type Gbrap14LeFrame<'a> = GbrapHighBitFrame<'a, 14, false>;
/// BE-encoded `Gbrap14Frame` (`AV_PIX_FMT_GBRAP14BE`).
pub type Gbrap14BeFrame<'a> = GbrapHighBitFrame<'a, 14, true>;

/// Type alias for a validated planar GBR+A 16-bit frame
/// (`AV_PIX_FMT_GBRAP16{LE,BE}`). Full `u16` range — all 16 bits active.
pub type Gbrap16Frame<'a, const BE: bool = false> = GbrapHighBitFrame<'a, 16, BE>;
/// LE-encoded `Gbrap16Frame` (`AV_PIX_FMT_GBRAP16LE`).
pub type Gbrap16LeFrame<'a> = GbrapHighBitFrame<'a, 16, false>;
/// BE-encoded `Gbrap16Frame` (`AV_PIX_FMT_GBRAP16BE`).
pub type Gbrap16BeFrame<'a> = GbrapHighBitFrame<'a, 16, true>;

// ---------------------------------------------------------------------------
// GbrpMsbFrame — three u16 planes, MSB-packed (high `BITS` bits active)
// ---------------------------------------------------------------------------

/// A validated **MSB-packed** planar GBR frame at high bit depth
/// (`AV_PIX_FMT_GBRP{10,12}MSB{LE,BE}`).
///
/// Three full-resolution `u16` planes in **G, B, R** order, with each
/// sample's `BITS` active bits stored in the **high** `BITS` positions of
/// the `u16` (low `16 − BITS` bits zero). This is the exact inverse of
/// [`GbrpHighBitFrame`], whose samples sit in the **low** `BITS`:
///
/// - **`GbrpMsbFrame`** — `value & ((1 << (16 − BITS)) − 1) == 0`; the active
///   bits are the high `BITS`. Matches FFmpeg's `gbrp{10,12}msb{le,be}`
///   descriptors (`shift = 6` at 10-bit, `shift = 4` at 12-bit).
/// - **[`GbrpHighBitFrame`]** — `value >> BITS == 0`; the active bits are the
///   low `BITS`. Matches `gbrp{10,12}{le,be}` (`shift = 0`).
///
/// A dedicated type is required because the two contracts are mirror images:
/// the low-bit [`GbrpHighBitFrame`] validates (and downstream kernels mask)
/// the opposite bit positions, so handing MSB-packed data to it would garble
/// every sample. This mirrors how low-bit [`Nv20Frame`](crate::frame::Nv20Frame)
/// is kept separate from high-bit `P210Frame`, here with the roles inverted
/// (this type is the high-bit one).
///
/// `BITS ∈ {10, 12}` — validated by a compile-time `const` assertion at
/// construction (FFmpeg defines `MSB` variants only at 10 and 12 bits).
/// Stride is in **samples** (`u16` elements); each plane requires
/// `*_stride >= width` and `len >= *_stride * height`. No width/height
/// parity constraint.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded `*LE` bytes, `true` → BE-encoded `*BE` bytes.
/// Downstream row kernels handle the byte-swap (or no-op) under the hood —
/// callers do **not** pre-swap.
///
/// Use the per-depth type aliases [`Gbrp10MsbFrame`] / [`Gbrp12MsbFrame`] at
/// call sites, or the `*Le*` / `*Be*` aliases for explicit endianness.
///
/// [`Nv20Frame`]: crate::frame::Nv20Frame
#[derive(Debug, Clone, Copy)]
pub struct GbrpMsbFrame<'a, const BITS: u32, const BE: bool = false> {
  g: &'a [u16],
  b: &'a [u16],
  r: &'a [u16],
  width: u32,
  height: u32,
  g_stride: u32,
  b_stride: u32,
  r_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> GbrpMsbFrame<'a, BITS, BE> {
  /// Constructs a new [`GbrpMsbFrame`], validating dimensions and plane
  /// lengths (geometry only — samples are **not** scanned). Returns
  /// [`GbrpMsbFrameError`] if any of:
  /// - `BITS ∉ {10, 12}` — caught at compile time via `const { assert!(…) }`,
  /// - `width` or `height` is zero,
  /// - any stride is smaller than `width` (in samples),
  /// - `stride * height` overflows `usize` (32-bit targets only),
  /// - any plane is shorter than `stride * height` samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
  ) -> Result<Self, GbrpMsbFrameError> {
    const {
      assert!(
        matches!(BITS, 10 | 12),
        "BITS must be one of 10 or 12 (FFmpeg defines GBRP MSB variants only at 10 and 12 bits)",
      );
    }

    if width == 0 || height == 0 {
      return Err(GbrpMsbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if g_stride < width {
      return Err(GbrpMsbFrameError::InsufficientGStride(
        InsufficientStride::new(g_stride, width),
      ));
    }
    if b_stride < width {
      return Err(GbrpMsbFrameError::InsufficientBStride(
        InsufficientStride::new(b_stride, width),
      ));
    }
    if r_stride < width {
      return Err(GbrpMsbFrameError::InsufficientRStride(
        InsufficientStride::new(r_stride, width),
      ));
    }

    let g_min = match (g_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrpMsbFrameError::GeometryOverflow(GeometryOverflow::new(
          g_stride, height,
        )));
      }
    };
    if g.len() < g_min {
      return Err(GbrpMsbFrameError::InsufficientGPlane(
        InsufficientPlane::new(g_min, g.len()),
      ));
    }

    let b_min = match (b_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrpMsbFrameError::GeometryOverflow(GeometryOverflow::new(
          b_stride, height,
        )));
      }
    };
    if b.len() < b_min {
      return Err(GbrpMsbFrameError::InsufficientBPlane(
        InsufficientPlane::new(b_min, b.len()),
      ));
    }

    let r_min = match (r_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(GbrpMsbFrameError::GeometryOverflow(GeometryOverflow::new(
          r_stride, height,
        )));
      }
    };
    if r.len() < r_min {
      return Err(GbrpMsbFrameError::InsufficientRPlane(
        InsufficientPlane::new(r_min, r.len()),
      ));
    }

    Ok(Self {
      g,
      b,
      r,
      width,
      height,
      g_stride,
      b_stride,
      r_stride,
    })
  }

  /// Constructs a new [`GbrpMsbFrame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
  ) -> Self {
    match Self::try_new(g, b, r, width, height, g_stride, b_stride, r_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid GbrpMsbFrame dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample and rejects
  /// any whose **low `16 − BITS` bits** are non-zero. An MSB-packed sample
  /// carries its `BITS` active bits in the **high** `BITS` of each `u16`, so a
  /// valid sample always satisfies `value & ((1 << (16 − BITS)) − 1) == 0`
  /// (low 6 bits zero at 10-bit, low 4 zero at 12-bit). Non-zero low bits are
  /// evidence the buffer isn't MSB-packed — most often a low-bit-packed
  /// [`GbrpHighBitFrame`] buffer (active bits in the low `BITS`) handed to the
  /// MSB path, whose high-bit extraction would silently misread it. This is
  /// the exact inverse of [`Nv20Frame::try_new_checked`], which rejects
  /// non-zero **high** bits for the low-bit-packed format, and mirrors
  /// `PnFrame422::try_new_checked` (the high-bit P210 family).
  ///
  /// **This is a packing sanity check, not a provenance validator.** It
  /// catches noisy low-bit-packed data (where most samples carry low-bit
  /// content), but it **cannot** distinguish MSB-packed data from a low-bit
  /// buffer whose samples all happen to be exact multiples of
  /// `1 << (16 − BITS)`. Callers needing strict provenance must rely on their
  /// source format metadata and pick the right frame type
  /// ([`GbrpMsbFrame`] vs [`GbrpHighBitFrame`]) at construction.
  ///
  /// Cost: one O(width × height) scan per plane. The default [`Self::try_new`]
  /// skips this so the hot path stays O(1).
  ///
  /// Returns [`GbrpMsbFrameError::StrayLowBits`] on the first offending sample
  /// — carries the plane, element index, and offending value.
  ///
  /// Per the byte-order contract recorded by `<const BE: bool>`, samples are
  /// validated **after** `u16::from_le` / `u16::from_be` normalization so the
  /// bit check operates on the intended logical sample value on every host.
  /// The reported `value` in the error is the normalized logical sample.
  ///
  /// [`Nv20Frame::try_new_checked`]: crate::frame::Nv20Frame::try_new_checked
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    g: &'a [u16],
    b: &'a [u16],
    r: &'a [u16],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
  ) -> Result<Self, GbrpMsbFrameError> {
    let frame = Self::try_new(g, b, r, width, height, g_stride, b_stride, r_stride)?;
    // MSB-packed: the active bits are the HIGH `BITS`; the low `16 - BITS`
    // must be zero. This is the inverse of the NV20 high-bit mask.
    const {
      assert!(
        BITS < 16,
        "MSB low-bit mask is only meaningful for BITS < 16"
      )
    };
    let low_mask: u16 = (1u16 << (16 - BITS)) - 1;
    let w = width as usize;
    let h = height as usize;
    let planes: [(&[u16], u32, GbrpMsbFramePlane); 3] = [
      (g, g_stride, GbrpMsbFramePlane::G),
      (b, b_stride, GbrpMsbFramePlane::B),
      (r, r_stride, GbrpMsbFramePlane::R),
    ];
    for (plane, stride, which) in planes {
      let stride = stride as usize;
      for row in 0..h {
        let start = row * stride;
        for (col, &s) in plane[start..start + w].iter().enumerate() {
          // Normalize from the recorded byte order to host-native before the
          // bit check (no-op on a matching-endian host, byte-swap otherwise).
          let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
          if logical & low_mask != 0 {
            return Err(GbrpMsbFrameError::StrayLowBits(GbrpMsbStrayLowBits::new(
              which,
              start + col,
              logical,
            )));
          }
        }
      }
    }
    Ok(frame)
  }

  /// Green plane samples. Row `r` starts at sample offset `r * g_stride()`.
  /// Each sample's `BITS` active bits sit in the **high** `BITS` positions of
  /// the `u16` (low `16 − BITS` bits zero).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(&self) -> &'a [u16] {
    self.g
  }
  /// Blue plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(&self) -> &'a [u16] {
    self.b
  }
  /// Red plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(&self) -> &'a [u16] {
    self.r
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
  /// Sample stride of the green plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g_stride(&self) -> u32 {
    self.g_stride
  }
  /// Sample stride of the blue plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b_stride(&self) -> u32 {
    self.b_stride
  }
  /// Sample stride of the red plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r_stride(&self) -> u32 {
    self.r_stride
  }
  /// Active bit depth — one of 10 or 12. Mirrors the `BITS` const parameter
  /// so generic code can read it without naming the type.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`*BE`), `false` if LE-encoded (`*LE`). Runtime mirror of the
  /// `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }

  // ---- crate-internal Y/U/V aliases ------------------------------------
  //
  // The shared `walker!` macro uses fixed `y/u/v` field-name conventions.
  // To reuse the macro verbatim for planar GBR — whose externally-correct
  // accessor names are `g/b/r` — we expose `pub(crate)` aliases:
  // `y == g`, `u == b`, `v == r`.
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y(&self) -> &'a [u16] {
    self.g
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u(&self) -> &'a [u16] {
    self.b
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v(&self) -> &'a [u16] {
    self.r
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y_stride(&self) -> u32 {
    self.g_stride
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u_stride(&self) -> u32 {
    self.b_stride
  }
  #[allow(dead_code)] // walker_macro planar3 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v_stride(&self) -> u32 {
    self.r_stride
  }
}

/// Identifies which plane of a [`GbrpMsbFrame`] a
/// [`GbrpMsbFrameError::StrayLowBits`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum GbrpMsbFramePlane {
  /// Green plane.
  G,
  /// Blue plane.
  B,
  /// Red plane.
  R,
}

/// Payload for [`GbrpMsbFrameError::StrayLowBits`]. Records an MSB-packed
/// sample whose **low** `16 − BITS` bits were set (the active bits are the
/// high `BITS`, so the low bits must be zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GbrpMsbStrayLowBits {
  plane: GbrpMsbFramePlane,
  index: usize,
  value: u16,
}

impl GbrpMsbStrayLowBits {
  /// Constructs a new `GbrpMsbStrayLowBits`.
  #[inline]
  pub const fn new(plane: GbrpMsbFramePlane, index: usize, value: u16) -> Self {
    Self {
      plane,
      index,
      value,
    }
  }
  /// Returns the `plane` the offending sample lives on.
  #[inline]
  pub const fn plane(&self) -> GbrpMsbFramePlane {
    self.plane
  }
  /// Returns the element `index` (in `u16` samples, from the plane base) of
  /// the offending sample.
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

/// Errors returned by [`GbrpMsbFrame::try_new`] and
/// [`GbrpMsbFrame::try_new_checked`].
///
/// Variant shape mirrors [`GbrpHighBitFrameError`] with the added
/// [`Self::StrayLowBits`] packing-sanity variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum GbrpMsbFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `g_stride < width` (in samples).
  #[error(transparent)]
  InsufficientGStride(InsufficientStride),

  /// `b_stride < width` (in samples).
  #[error(transparent)]
  InsufficientBStride(InsufficientStride),

  /// `r_stride < width` (in samples).
  #[error(transparent)]
  InsufficientRStride(InsufficientStride),

  /// G plane is shorter than `g_stride * height` samples.
  #[error(transparent)]
  InsufficientGPlane(InsufficientPlane),

  /// B plane is shorter than `b_stride * height` samples.
  #[error(transparent)]
  InsufficientBPlane(InsufficientPlane),

  /// R plane is shorter than `r_stride * height` samples.
  #[error(transparent)]
  InsufficientRPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A sample's low `16 − BITS` bits were non-zero — an MSB-packed sample
  /// carries its `BITS` active bits in the **high** `BITS` of each `u16`, so
  /// valid samples satisfy `value & ((1 << (16 − BITS)) − 1) == 0`. Only
  /// [`GbrpMsbFrame::try_new_checked`] can produce this error; the
  /// geometry-only [`GbrpMsbFrame::try_new`] never inspects samples.
  ///
  /// Note: the absence of this error does **not** prove the buffer is
  /// MSB-packed. A low-bit-packed buffer whose samples are all exact
  /// multiples of `1 << (16 − BITS)` passes the check silently. See
  /// [`GbrpMsbFrame::try_new_checked`] for the full discussion.
  #[error(
    "sample {:#06x} on plane {} at element {} has non-zero low bits (not a valid GBRP MSB-packed sample)", .0.value(), .0.plane(), .0.index()
  )]
  StrayLowBits(GbrpMsbStrayLowBits),
}

/// Type alias for a validated MSB-packed planar GBR 10-bit frame
/// (`AV_PIX_FMT_GBRP10MSB{LE,BE}`). Samples in the high 10 bits of each `u16`
/// (low 6 zero). Defaults to LE; use [`Gbrp10MsbLeFrame`] / [`Gbrp10MsbBeFrame`]
/// for explicit endianness.
pub type Gbrp10MsbFrame<'a, const BE: bool = false> = GbrpMsbFrame<'a, 10, BE>;
/// LE-encoded `Gbrp10MsbFrame` (`AV_PIX_FMT_GBRP10MSBLE`).
pub type Gbrp10MsbLeFrame<'a> = GbrpMsbFrame<'a, 10, false>;
/// BE-encoded `Gbrp10MsbFrame` (`AV_PIX_FMT_GBRP10MSBBE`).
pub type Gbrp10MsbBeFrame<'a> = GbrpMsbFrame<'a, 10, true>;

/// Type alias for a validated MSB-packed planar GBR 12-bit frame
/// (`AV_PIX_FMT_GBRP12MSB{LE,BE}`). Samples in the high 12 bits of each `u16`
/// (low 4 zero).
pub type Gbrp12MsbFrame<'a, const BE: bool = false> = GbrpMsbFrame<'a, 12, BE>;
/// LE-encoded `Gbrp12MsbFrame` (`AV_PIX_FMT_GBRP12MSBLE`).
pub type Gbrp12MsbLeFrame<'a> = GbrpMsbFrame<'a, 12, false>;
/// BE-encoded `Gbrp12MsbFrame` (`AV_PIX_FMT_GBRP12MSBBE`).
pub type Gbrp12MsbBeFrame<'a> = GbrpMsbFrame<'a, 12, true>;

// ---------------------------------------------------------------------------
// Gbrap32Frame — four u32 planes, all 32 bits active (full range)
// ---------------------------------------------------------------------------

/// A validated planar GBR+A 32-bit-per-channel frame
/// (`AV_PIX_FMT_GBRAP32{LE,BE}`).
///
/// Four full-resolution `u32` planes in **G, B, R, A** order:
/// - `g` / `b` / `r` — colour planes.
/// - `a` — alpha plane (1:1 with G; real per-pixel alpha).
///
/// **All 32 bits of each `u32` element are active** (full `u32` range,
/// `depth = 32`, `shift = 0`), so — unlike [`GbrpMsbFrame`] /
/// [`GbrpHighBitFrame`] — there is **no** stray-bit contract and hence no
/// `try_new_checked`: [`Self::try_new`] validates geometry only, exactly as
/// the full-range 16-bit [`Gbrap16Frame`] does. FFmpeg added this planar
/// 32-bit RGBA format for Vulkan FFv1 decoding.
///
/// Stride is in **`u32` elements** (not bytes); each plane requires
/// `*_stride >= width` and `len >= *_stride * height`. No width/height parity
/// constraint.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded bytes (`AV_PIX_FMT_GBRAP32LE`), `true` → BE-encoded
/// bytes (`AV_PIX_FMT_GBRAP32BE`). Downstream row kernels handle the
/// per-`u32` byte-swap (or no-op) under the hood — callers do **not**
/// pre-swap. Callers holding byte buffers from FFmpeg should cast via
/// `bytemuck::cast_slice` and divide each `linesize[i]` by 4 before
/// constructing.
///
/// # Aliases
/// - [`Gbrap32LeFrame`] = `Gbrap32Frame<'a, false>` (explicit LE; the default).
/// - [`Gbrap32BeFrame`] = `Gbrap32Frame<'a, true>`.
#[derive(Debug, Clone, Copy)]
pub struct Gbrap32Frame<'a, const BE: bool = false> {
  g: &'a [u32],
  b: &'a [u32],
  r: &'a [u32],
  a: &'a [u32],
  width: u32,
  height: u32,
  g_stride: u32,
  b_stride: u32,
  r_stride: u32,
  a_stride: u32,
}

/// LE-encoded `Gbrap32Frame` (`AV_PIX_FMT_GBRAP32LE`).
pub type Gbrap32LeFrame<'a> = Gbrap32Frame<'a, false>;
/// BE-encoded `Gbrap32Frame` (`AV_PIX_FMT_GBRAP32BE`).
pub type Gbrap32BeFrame<'a> = Gbrap32Frame<'a, true>;

impl<'a, const BE: bool> Gbrap32Frame<'a, BE> {
  /// Constructs a new [`Gbrap32Frame`], validating dimensions and plane
  /// lengths (geometry only — all 32 bits are active, so there is nothing to
  /// scan). Returns [`Gbrap32FrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - any stride is smaller than `width` (in `u32` elements),
  /// - `stride * height` overflows `usize` (32-bit targets only),
  /// - any plane is shorter than `stride * height` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    g: &'a [u32],
    b: &'a [u32],
    r: &'a [u32],
    a: &'a [u32],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
    a_stride: u32,
  ) -> Result<Self, Gbrap32FrameError> {
    if width == 0 || height == 0 {
      return Err(Gbrap32FrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if g_stride < width {
      return Err(Gbrap32FrameError::InsufficientGStride(
        InsufficientStride::new(g_stride, width),
      ));
    }
    if b_stride < width {
      return Err(Gbrap32FrameError::InsufficientBStride(
        InsufficientStride::new(b_stride, width),
      ));
    }
    if r_stride < width {
      return Err(Gbrap32FrameError::InsufficientRStride(
        InsufficientStride::new(r_stride, width),
      ));
    }
    if a_stride < width {
      return Err(Gbrap32FrameError::InsufficientAStride(
        InsufficientStride::new(a_stride, width),
      ));
    }

    let g_min = match (g_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gbrap32FrameError::GeometryOverflow(GeometryOverflow::new(
          g_stride, height,
        )));
      }
    };
    if g.len() < g_min {
      return Err(Gbrap32FrameError::InsufficientGPlane(
        InsufficientPlane::new(g_min, g.len()),
      ));
    }

    let b_min = match (b_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gbrap32FrameError::GeometryOverflow(GeometryOverflow::new(
          b_stride, height,
        )));
      }
    };
    if b.len() < b_min {
      return Err(Gbrap32FrameError::InsufficientBPlane(
        InsufficientPlane::new(b_min, b.len()),
      ));
    }

    let r_min = match (r_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gbrap32FrameError::GeometryOverflow(GeometryOverflow::new(
          r_stride, height,
        )));
      }
    };
    if r.len() < r_min {
      return Err(Gbrap32FrameError::InsufficientRPlane(
        InsufficientPlane::new(r_min, r.len()),
      ));
    }

    let a_min = match (a_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Gbrap32FrameError::GeometryOverflow(GeometryOverflow::new(
          a_stride, height,
        )));
      }
    };
    if a.len() < a_min {
      return Err(Gbrap32FrameError::InsufficientAPlane(
        InsufficientPlane::new(a_min, a.len()),
      ));
    }

    Ok(Self {
      g,
      b,
      r,
      a,
      width,
      height,
      g_stride,
      b_stride,
      r_stride,
      a_stride,
    })
  }

  /// Constructs a new [`Gbrap32Frame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    g: &'a [u32],
    b: &'a [u32],
    r: &'a [u32],
    a: &'a [u32],
    width: u32,
    height: u32,
    g_stride: u32,
    b_stride: u32,
    r_stride: u32,
    a_stride: u32,
  ) -> Self {
    match Self::try_new(
      g, b, r, a, width, height, g_stride, b_stride, r_stride, a_stride,
    ) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Gbrap32Frame dimensions or plane lengths"),
    }
  }

  /// Green plane samples. Row `r` starts at element offset `r * g_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(&self) -> &'a [u32] {
    self.g
  }
  /// Blue plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(&self) -> &'a [u32] {
    self.b
  }
  /// Red plane samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(&self) -> &'a [u32] {
    self.r
  }
  /// Alpha plane samples — full-width × full-height (1:1 with G).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a(&self) -> &'a [u32] {
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
  /// Sample stride of the green plane (`>= width`, in `u32` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g_stride(&self) -> u32 {
    self.g_stride
  }
  /// Sample stride of the blue plane (`>= width`, in `u32` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b_stride(&self) -> u32 {
    self.b_stride
  }
  /// Sample stride of the red plane (`>= width`, in `u32` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r_stride(&self) -> u32 {
    self.r_stride
  }
  /// Sample stride of the alpha plane (`>= width`, in `u32` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn a_stride(&self) -> u32 {
    self.a_stride
  }
  /// Returns the compile-time BE flag — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_GBRAP32BE`), `false` if LE-encoded (`AV_PIX_FMT_GBRAP32LE`).
  /// Runtime mirror of the `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }

  // ---- crate-internal Y/U/V aliases ------------------------------------
  //
  // The shared walker plumbing uses fixed `y/u/v` field-name conventions;
  // expose `pub(crate)` aliases `y == g`, `u == b`, `v == r` (alpha already
  // has the right name).
  #[allow(dead_code)] // walker planar4 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y(&self) -> &'a [u32] {
    self.g
  }
  #[allow(dead_code)] // walker planar4 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u(&self) -> &'a [u32] {
    self.b
  }
  #[allow(dead_code)] // walker planar4 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v(&self) -> &'a [u32] {
    self.r
  }
  #[allow(dead_code)] // walker planar4 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn y_stride(&self) -> u32 {
    self.g_stride
  }
  #[allow(dead_code)] // walker planar4 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn u_stride(&self) -> u32 {
    self.b_stride
  }
  #[allow(dead_code)] // walker planar4 alias
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn v_stride(&self) -> u32 {
    self.r_stride
  }
}

/// Errors returned by [`Gbrap32Frame::try_new`].
///
/// Mirrors [`GbrapHighBitFrameError`] with all sizes expressed in `u32`
/// elements. There is no stray-bit variant — every bit of a 32-bit sample is
/// active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Gbrap32FrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `g_stride < width` (in `u32` elements).
  #[error(transparent)]
  InsufficientGStride(InsufficientStride),

  /// `b_stride < width` (in `u32` elements).
  #[error(transparent)]
  InsufficientBStride(InsufficientStride),

  /// `r_stride < width` (in `u32` elements).
  #[error(transparent)]
  InsufficientRStride(InsufficientStride),

  /// `a_stride < width` (in `u32` elements).
  #[error(transparent)]
  InsufficientAStride(InsufficientStride),

  /// G plane is shorter than `g_stride * height` elements.
  #[error(transparent)]
  InsufficientGPlane(InsufficientPlane),

  /// B plane is shorter than `b_stride * height` elements.
  #[error(transparent)]
  InsufficientBPlane(InsufficientPlane),

  /// R plane is shorter than `r_stride * height` elements.
  #[error(transparent)]
  InsufficientRPlane(InsufficientPlane),

  /// A plane is shorter than `a_stride * height` elements.
  #[error(transparent)]
  InsufficientAPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}
