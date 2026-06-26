use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, UnsupportedBits, WidthAlignment,
  ZeroDimension,
};
use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

/// A validated YUV 4:2:0 planar frame at bit depths > 8 (10/12/14).
///
/// Structurally identical to `Yuv420pFrame` — three planes, half‑
/// size chroma — but sample storage is **`u16`** so every pixel
/// carries up to 16 bits of payload. `BITS` is the active bit depth
/// (10, 12, 14, or 16). Callers are **expected** to store each sample in
/// the **low** `BITS` bits of its `u16` (upper `16 - BITS` bits zero),
/// matching FFmpeg's little‑endian `yuv420p10le` / `yuv420p12le` /
/// `yuv420p14le` convention, where each plane is a byte buffer
/// reinterpretable as `u16` little‑endian. `try_new` validates plane
/// geometry / strides / lengths but does **not** inspect sample
/// values to verify this packing.
///
/// This is **not** the FFmpeg `p010` layout — `p010` stores samples
/// in the **high** 10 bits of each `u16` (`sample << 6`). Callers
/// holding a p010 buffer must shift right by `16 - BITS` before
/// construction.
///
/// # Input sample range
///
/// The kernels assume every input sample is in `[0, (1 << BITS) - 1]`
/// — i.e., upper `16 - BITS` bits zero. Validating this at
/// construction would require scanning every sample of every plane
/// (megabytes per frame at video rates); instead the constructor
/// validates geometry only and the contract falls on the caller.
/// Decoders and FFmpeg output satisfy this by construction.
///
/// **Output for out‑of‑range samples is equivalent to pre‑masking
/// every sample to the low `BITS` bits.** Every kernel (scalar + all
/// 5 SIMD tiers) AND‑masks each `u16` load to `(1 << BITS) - 1`
/// before the Q15 path, so a sample like `0xFFC0` (p010 white =
/// `1023 << 6`) is treated identically to `0x03C0` on every backend
/// when `BITS == 10`. This gives deterministic, backend‑independent
/// output for mispacked input — feeding `p010` data into a
/// `yuv420p10le`‑shaped frame produces severely distorted, but stable,
/// pixel values across scalar / NEON / SSE4.1 / AVX2 / AVX‑512 /
/// wasm simd128, which is an obvious signal for downstream diffing.
/// The mask is a single AND per load and a no‑op on valid input
/// (upper bits already zero).
///
/// Callers who want the mispacking to surface as a loud error
/// instead of silent color corruption should use
/// [`Self::try_new_checked`] — it scans every sample and returns
/// [`Yuv420pFrame16Error::SampleOutOfRange`] on the first violation.
///
/// All four supported depths — `BITS == 10` (HDR10 / 10‑bit SDR
/// keystone), `BITS == 12` (HEVC Main 12 / VP9 Profile 3),
/// `BITS == 14` (grading / mastering pipelines), and `BITS == 16`
/// (reference / intermediate HDR) — share this frame struct but
/// **use two kernel families**:
///
/// - 10 / 12 / 14 run on a single const-generic Q15 i32 pipeline
///   (`scalar::yuv_420p_n_to_rgb_*<BITS>` + matching SIMD kernels
///   across NEON / SSE4.1 / AVX2 / AVX-512 / wasm simd128).
/// - 16 runs on a parallel i64 kernel family
///   (`scalar::yuv_420p16_to_rgb_*` + matching SIMD) because the
///   Q15 chroma multiply-add overflows i32 at 16 bits.
///
/// The constructor validates `BITS ∈ {10, 12, 14, 16}` up front;
/// kernel selection is at the public dispatcher boundary
/// (`yuv420pNN_to_rgb_*`). The selection is free — each dispatcher
/// is a dedicated function that knows which family to call.
///
/// Stride is in **samples** (`u16` elements), not bytes. Users
/// holding a byte buffer from FFmpeg should cast via
/// `bytemuck::cast_slice` and divide `linesize[i]` by 2 before
/// constructing.
///
/// `width` must be even (same 4:2:0 rationale as `Yuv420pFrame`);
/// `height` may be odd and is handled via `height.div_ceil(2)` in
/// chroma‑row sizing.
#[derive(Debug, Clone, Copy)]
pub struct Yuv420pFrame16<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  u: &'a [u16],
  v: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> Yuv420pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuv420pFrame16`], validating dimensions, plane
  /// lengths, and the `BITS` parameter.
  ///
  /// Returns [`Yuv420pFrame16Error`] if any of:
  /// - `BITS` is not 10, 12, 14, or 16 — use [`Yuv420p10Frame`],
  ///   [`Yuv420p12Frame`], [`Yuv420p14Frame`], or [`Yuv420p16Frame`]
  ///   at call sites for readability, all four are type aliases
  ///   over this struct,
  /// - `width` or `height` is zero,
  /// - `width` is odd,
  /// - any stride is smaller than the plane's declared pixel width,
  /// - any plane is too short to cover its declared rows, or
  /// - `stride * rows` overflows `usize` (32‑bit targets only).
  ///
  /// All strides are in **samples** (`u16` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv420pFrame16Error> {
    // Guard the `BITS` parameter at the top. 10/12/14 share the Q15
    // i32 kernel family; 16 uses a parallel i64 kernel family (see
    // [`Yuv420p16Frame`] and `yuv_420p16_to_rgb_*`). 8 has its own
    // (non-generic) 8-bit kernels in `Yuv420pFrame`.
    if BITS != 9 && BITS != 10 && BITS != 12 && BITS != 14 && BITS != 16 {
      return Err(Yuv420pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuv420pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Yuv420pFrame16Error::WidthAlignment(WidthAlignment::odd(
        width as usize,
      )));
    }
    if y_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let chroma_width = width.div_ceil(2);
    if u_stride < chroma_width {
      return Err(Yuv420pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, chroma_width),
      ));
    }
    if v_stride < chroma_width {
      return Err(Yuv420pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, chroma_width),
      ));
    }

    // Plane sizes are in `u16` elements, so the overflow guard runs
    // against the sample count — callers converting from byte strides
    // should have already divided by 2.
    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuv420pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    let chroma_height = height.div_ceil(2);
    let u_min = match (u_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, chroma_height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuv420pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, chroma_height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuv420pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
    })
  }

  /// Constructs a new [`Yuv420pFrame16`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Self {
    match Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuv420pFrame16 dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample of
  /// every plane and rejects values above `(1 << BITS) - 1`. Use this
  /// on untrusted input (e.g., a `u16` buffer of unknown provenance
  /// that might be `p010`‑packed or otherwise dirty) where accepting
  /// out-of-range samples would be unacceptable because they violate
  /// the expected bit-depth contract and can produce invalid results.
  ///
  /// Cost: one O(plane_size) linear scan per plane — a few megabytes
  /// per 1080p frame at 10 bits. The default [`Self::try_new`] skips
  /// this so the hot path (decoder output, already-conforming
  /// buffers) stays O(1).
  ///
  /// Returns [`Yuv420pFrame16Error::SampleOutOfRange`] on the first
  /// offending sample — the error carries the plane, element index
  /// within that plane's slice, offending value, and the valid
  /// maximum so the caller can pinpoint the bad sample. All of
  /// [`Self::try_new`]'s geometry errors are still possible.
  ///
  /// Per the LE-encoded byte contract documented on the type, samples
  /// are validated **after** `u16::from_le` normalization so the range
  /// check operates on the intended logical sample value on every host.
  /// On little-endian hosts `from_le` is a no-op (the host-native `u16`
  /// already matches the wire); on big-endian hosts it byte-swaps each
  /// `u16` back into host-native form before the comparison. Without
  /// this normalization a valid `yuv420p10le` plane on a BE host would
  /// have its samples appear byte-swapped (e.g. `1023` encoded LE as
  /// bytes `[0xFF, 0x03]` reads as host-native `0xFF03` on BE) and the
  /// validator would falsely reject every row. The reported `value` in
  /// the error is the normalized logical sample so callers can match it
  /// against the declared `max_valid`. Mirrors the `Y2xxFrame::try_new_checked`
  /// pattern.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv420pFrame16Error> {
    let frame = Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride)?;
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    // Scan the declared-payload region of each plane. Stride may add
    // unused padding past the declared width; we don't inspect that —
    // callers often pass buffers whose padding bytes are arbitrary,
    // and the kernels never read them.
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::Y,
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::U,
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::V,
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

  /// Y (luma) plane samples. Row `r` starts at sample offset
  /// `r * y_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }

  /// U (Cb) plane samples. Row `r` starts at sample offset
  /// `r * u_stride()`. U has half the width and half the height of the
  /// frame (chroma row index for output row `r` is `r / 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }

  /// V (Cr) plane samples. Row `r` starts at sample offset
  /// `r * v_stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
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

  /// Sample stride of the U plane (`>= width / 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }

  /// Sample stride of the V plane (`>= width / 2`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }

  /// Active bit depth — 10, 12, 14, or 16. Mirrors the `BITS` const
  /// parameter so generic code can read it without naming the type.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are
  /// BE-encoded (`AV_PIX_FMT_YUV420P*BE`), `false` if LE-encoded
  /// (`AV_PIX_FMT_YUV420P*LE`). Runtime mirror of the
  /// `<const BE: bool>` type parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded type alias for a validated YUV 4:2:0 planar frame at 10
/// bits per sample (`AV_PIX_FMT_YUV420P10LE`). Tight wrapper over
/// [`Yuv420pFrame16`] with `BITS == 10`, `BE == false`. The BE-encoded
/// counterpart is [`Yuv420p10BeFrame`].
pub type Yuv420p10Frame<'a> = Yuv420pFrame16<'a, 10>;

/// LE-encoded YUV 4:2:0 planar frame at 9 bits per sample
/// (`AV_PIX_FMT_YUV420P9LE`). BE counterpart: [`Yuv420p9BeFrame`].
pub type Yuv420p9Frame<'a> = Yuv420pFrame16<'a, 9>;

/// LE-encoded YUV 4:2:0 planar frame at 12 bits per sample
/// (`AV_PIX_FMT_YUV420P12LE`). BE counterpart: [`Yuv420p12BeFrame`].
pub type Yuv420p12Frame<'a> = Yuv420pFrame16<'a, 12>;

/// LE-encoded YUV 4:2:0 planar frame at 14 bits per sample
/// (`AV_PIX_FMT_YUV420P14LE`). BE counterpart: [`Yuv420p14BeFrame`].
pub type Yuv420p14Frame<'a> = Yuv420pFrame16<'a, 14>;

/// LE-encoded YUV 4:2:0 planar frame at 16 bits per sample
/// (`AV_PIX_FMT_YUV420P16LE`). BE counterpart: [`Yuv420p16BeFrame`].
/// **Uses a parallel i64 kernel family** because the Q15 chroma sum
/// overflows i32 at 16 bits.
pub type Yuv420p16Frame<'a> = Yuv420pFrame16<'a, 16>;

// ---- Phase 4 — explicit LE/BE aliases for the 4:2:0 planar HB family ----
//
// The original aliases (`Yuv420p10Frame` etc.) default to `BE = false`
// (LE-encoded plane bytes); the `*BeFrame` aliases pin `BE = true` for
// callers who hold BE-encoded byte buffers (`AV_PIX_FMT_YUV420P*BE`).
// The `*LeFrame` aliases mirror the default explicitly so callers that
// want to document endianness at the type level can do so symmetrically
// to the BE aliases.

/// LE-encoded `Yuv420p9Frame` (`AV_PIX_FMT_YUV420P9LE`).
pub type Yuv420p9LeFrame<'a> = Yuv420pFrame16<'a, 9, false>;
/// BE-encoded `Yuv420p9Frame` (`AV_PIX_FMT_YUV420P9BE`). Plane bytes
/// are byte-swapped at load by the row kernels.
pub type Yuv420p9BeFrame<'a> = Yuv420pFrame16<'a, 9, true>;
/// LE-encoded `Yuv420p10Frame` (`AV_PIX_FMT_YUV420P10LE`).
pub type Yuv420p10LeFrame<'a> = Yuv420pFrame16<'a, 10, false>;
/// BE-encoded `Yuv420p10Frame` (`AV_PIX_FMT_YUV420P10BE`).
pub type Yuv420p10BeFrame<'a> = Yuv420pFrame16<'a, 10, true>;
/// LE-encoded `Yuv420p12Frame` (`AV_PIX_FMT_YUV420P12LE`).
pub type Yuv420p12LeFrame<'a> = Yuv420pFrame16<'a, 12, false>;
/// BE-encoded `Yuv420p12Frame` (`AV_PIX_FMT_YUV420P12BE`).
pub type Yuv420p12BeFrame<'a> = Yuv420pFrame16<'a, 12, true>;
/// LE-encoded `Yuv420p14Frame` (`AV_PIX_FMT_YUV420P14LE`).
pub type Yuv420p14LeFrame<'a> = Yuv420pFrame16<'a, 14, false>;
/// BE-encoded `Yuv420p14Frame` (`AV_PIX_FMT_YUV420P14BE`).
pub type Yuv420p14BeFrame<'a> = Yuv420pFrame16<'a, 14, true>;
/// LE-encoded `Yuv420p16Frame` (`AV_PIX_FMT_YUV420P16LE`).
pub type Yuv420p16LeFrame<'a> = Yuv420pFrame16<'a, 16, false>;
/// BE-encoded `Yuv420p16Frame` (`AV_PIX_FMT_YUV420P16BE`).
pub type Yuv420p16BeFrame<'a> = Yuv420pFrame16<'a, 16, true>;

/// Errors returned by [`Yuv420pFrame16::try_new`]. Variant shape
/// mirrors `Yuv420pFrameError`, with `UnsupportedBits` added for
/// the new `BITS` parameter and all sizes expressed in **samples**
/// (`u16` elements) instead of bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Yuv420pFrame16Error {
  /// `BITS` was not one of the supported depths (10, 12, 14, 16).
  /// 8‑bit frames should use `Yuv420pFrame`; 16‑bit is supported,
  /// but uses a different kernel family (see [`Yuv420pFrame16`] docs).
  #[error(transparent)]
  UnsupportedBits(UnsupportedBits),

  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `width` was odd. Same 4:2:0 rationale as
  /// `Yuv420pFrameError::WidthAlignment`.
  #[error(transparent)]
  WidthAlignment(WidthAlignment),

  /// `y_stride < width` (in samples).
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `u_stride < ceil(width / 2)` (in samples).
  #[error(transparent)]
  InsufficientUStride(InsufficientStride),

  /// `v_stride < ceil(width / 2)` (in samples).
  #[error(transparent)]
  InsufficientVStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * ceil(height / 2)` samples.
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * ceil(height / 2)` samples.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// `stride * rows` overflows `usize` (32‑bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A plane sample exceeds `(1 << BITS) - 1` — i.e., a bit above the
  /// declared active depth is set. Only [`Yuv420pFrame16::try_new_checked`]
  /// can produce this error; [`Yuv420pFrame16::try_new`] validates
  /// geometry only and treats the low‑bit‑packing contract as an
  /// expectation. Use the checked constructor for untrusted input
  /// (e.g., a buffer that might be `p010`‑packed instead of
  /// `yuv420p10le`‑packed).
  #[error(
    "sample {} on plane {} at element {} exceeds {} ((1 << BITS) - 1)",
    .0.value(), .0.plane(), .0.index(), .0.max_valid()
  )]
  SampleOutOfRange(Yuv420pFrame16SampleOutOfRange),
}

/// Payload for [`Yuv420pFrame16Error::SampleOutOfRange`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Yuv420pFrame16SampleOutOfRange {
  plane: Yuv420pFrame16Plane,
  index: usize,
  value: u16,
  max_valid: u16,
}

impl Yuv420pFrame16SampleOutOfRange {
  /// Constructs a new payload.
  #[inline]
  pub const fn new(plane: Yuv420pFrame16Plane, index: usize, value: u16, max_valid: u16) -> Self {
    Self {
      plane,
      index,
      value,
      max_valid,
    }
  }
  /// Which plane the offending sample lives on.
  #[inline]
  pub const fn plane(&self) -> Yuv420pFrame16Plane {
    self.plane
  }
  /// Element index within the plane's slice.
  #[inline]
  pub const fn index(&self) -> usize {
    self.index
  }
  /// The offending sample value.
  #[inline]
  pub const fn value(&self) -> u16 {
    self.value
  }
  /// Maximum allowed value for the declared `BITS`.
  #[inline]
  pub const fn max_valid(&self) -> u16 {
    self.max_valid
  }
}

/// Identifies which plane of a [`Yuv420pFrame16`] an error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Yuv420pFrame16Plane {
  /// Luma plane.
  Y,
  /// U (Cb) chroma plane.
  U,
  /// V (Cr) chroma plane.
  V,
}

/// A validated planar 4:2:2 `u16`-backed frame, generic over
/// `const BITS: u32 ∈ {10, 12, 14, 16}`. Samples are low-bit-packed
/// (the `BITS` active bits sit in the **low** bits of each `u16`).
///
/// Layout mirrors [`Yuv420pFrame16`] but with chroma half-width,
/// **full-height**: `u.len() >= u_stride * height`. The per-row
/// kernel contract is identical to the 4:2:0 family — the 4:2:2
/// difference lives in the walker (chroma row matches Y row instead
/// of `Y / 2`).
///
/// All strides are in **samples** (`u16` elements). Use the
/// [`Yuv422p10Frame`] / [`Yuv422p12Frame`] / [`Yuv422p14Frame`] /
/// [`Yuv422p16Frame`] aliases at call sites.
#[derive(Debug, Clone, Copy)]
pub struct Yuv422pFrame16<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  u: &'a [u16],
  v: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> Yuv422pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuv422pFrame16`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv420pFrame16Error> {
    if BITS != 9 && BITS != 10 && BITS != 12 && BITS != 14 && BITS != 16 {
      return Err(Yuv420pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuv420pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if width & 1 != 0 {
      return Err(Yuv420pFrame16Error::WidthAlignment(WidthAlignment::odd(
        width as usize,
      )));
    }
    if y_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    let chroma_width = width.div_ceil(2);
    if u_stride < chroma_width {
      return Err(Yuv420pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, chroma_width),
      ));
    }
    if v_stride < chroma_width {
      return Err(Yuv420pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, chroma_width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuv420pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    // 4:2:2: chroma is **full-height** (no `div_ceil(2)`).
    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuv420pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuv420pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
    })
  }

  /// Constructs a new [`Yuv422pFrame16`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Self {
    match Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuv422pFrame16 dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample of
  /// every plane and rejects values above `(1 << BITS) - 1`. Use this
  /// on untrusted input where accepting out-of-range samples would
  /// silently corrupt the conversion via the kernels' bit-mask.
  ///
  /// Returns [`Yuv420pFrame16Error::SampleOutOfRange`] on the first
  /// offending sample. All of [`Self::try_new`]'s geometry errors are
  /// still possible. At `BITS == 16` the check is a no-op (every
  /// `u16` value is valid) — same convention as
  /// [`Yuv420pFrame16::try_new_checked`].
  ///
  /// Per the LE-encoded byte contract on the type, samples are validated
  /// **after** `u16::from_le` normalization so the range check operates
  /// on the intended logical sample on both LE and BE hosts. See
  /// [`Yuv420pFrame16::try_new_checked`] for the full rationale.
  ///
  /// Cost: one O(plane_size) linear scan per plane. The default
  /// [`Self::try_new`] skips this so the hot path (decoder output,
  /// already-conforming buffers) stays O(1).
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv420pFrame16Error> {
    let frame = Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride)?;
    if BITS == 16 {
      return Ok(frame);
    }
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    // 4:2:2: chroma is half-width, FULL-height.
    let chroma_w = w / 2;
    let chroma_h = h;
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::Y,
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::U,
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::V,
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

  /// Y plane (`u16` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }
  /// U plane. Half-width, full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }
  /// V plane. Half-width, full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
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
  /// The `BITS` const parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Compile-time BE flag mirror — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YUV422P*BE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded 4:2:2 planar, 9-bit (`AV_PIX_FMT_YUV422P9LE`). BE
/// counterpart: [`Yuv422p9BeFrame`].
pub type Yuv422p9Frame<'a> = Yuv422pFrame16<'a, 9>;
/// LE-encoded 4:2:2 planar, 10-bit (`AV_PIX_FMT_YUV422P10LE`). BE
/// counterpart: [`Yuv422p10BeFrame`].
pub type Yuv422p10Frame<'a> = Yuv422pFrame16<'a, 10>;
/// LE-encoded 4:2:2 planar, 12-bit (`AV_PIX_FMT_YUV422P12LE`). BE
/// counterpart: [`Yuv422p12BeFrame`].
pub type Yuv422p12Frame<'a> = Yuv422pFrame16<'a, 12>;
/// LE-encoded 4:2:2 planar, 14-bit (`AV_PIX_FMT_YUV422P14LE`). BE
/// counterpart: [`Yuv422p14BeFrame`].
pub type Yuv422p14Frame<'a> = Yuv422pFrame16<'a, 14>;
/// LE-encoded 4:2:2 planar, 16-bit (`AV_PIX_FMT_YUV422P16LE`). BE
/// counterpart: [`Yuv422p16BeFrame`]. Uses the parallel i64 kernel
/// family.
pub type Yuv422p16Frame<'a> = Yuv422pFrame16<'a, 16>;

// ---- Phase 4 — explicit LE/BE aliases for the 4:2:2 planar HB family ----

/// LE-encoded `Yuv422p9Frame` (`AV_PIX_FMT_YUV422P9LE`).
pub type Yuv422p9LeFrame<'a> = Yuv422pFrame16<'a, 9, false>;
/// BE-encoded `Yuv422p9Frame` (`AV_PIX_FMT_YUV422P9BE`).
pub type Yuv422p9BeFrame<'a> = Yuv422pFrame16<'a, 9, true>;
/// LE-encoded `Yuv422p10Frame` (`AV_PIX_FMT_YUV422P10LE`).
pub type Yuv422p10LeFrame<'a> = Yuv422pFrame16<'a, 10, false>;
/// BE-encoded `Yuv422p10Frame` (`AV_PIX_FMT_YUV422P10BE`).
pub type Yuv422p10BeFrame<'a> = Yuv422pFrame16<'a, 10, true>;
/// LE-encoded `Yuv422p12Frame` (`AV_PIX_FMT_YUV422P12LE`).
pub type Yuv422p12LeFrame<'a> = Yuv422pFrame16<'a, 12, false>;
/// BE-encoded `Yuv422p12Frame` (`AV_PIX_FMT_YUV422P12BE`).
pub type Yuv422p12BeFrame<'a> = Yuv422pFrame16<'a, 12, true>;
/// LE-encoded `Yuv422p14Frame` (`AV_PIX_FMT_YUV422P14LE`).
pub type Yuv422p14LeFrame<'a> = Yuv422pFrame16<'a, 14, false>;
/// BE-encoded `Yuv422p14Frame` (`AV_PIX_FMT_YUV422P14BE`).
pub type Yuv422p14BeFrame<'a> = Yuv422pFrame16<'a, 14, true>;
/// LE-encoded `Yuv422p16Frame` (`AV_PIX_FMT_YUV422P16LE`).
pub type Yuv422p16LeFrame<'a> = Yuv422pFrame16<'a, 16, false>;
/// BE-encoded `Yuv422p16Frame` (`AV_PIX_FMT_YUV422P16BE`).
pub type Yuv422p16BeFrame<'a> = Yuv422pFrame16<'a, 16, true>;

/// A validated planar 4:4:4 `u16`-backed frame, generic over
/// `const BITS: u32 ∈ {10, 12, 14, 16}`. All three planes are
/// full-size. No width parity constraint.
#[derive(Debug, Clone, Copy)]
pub struct Yuv444pFrame16<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  u: &'a [u16],
  v: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> Yuv444pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuv444pFrame16`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv420pFrame16Error> {
    if BITS != 9 && BITS != 10 && BITS != 12 && BITS != 14 && BITS != 16 {
      return Err(Yuv420pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuv420pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    // 4:4:4: chroma stride ≥ width (not width / 2).
    if u_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, width),
      ));
    }
    if v_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuv420pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuv420pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuv420pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
    })
  }

  /// Constructs a new [`Yuv444pFrame16`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Self {
    match Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuv444pFrame16 dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample of
  /// every plane and rejects values above `(1 << BITS) - 1`. Use this
  /// on untrusted input where accepting out-of-range samples would
  /// silently corrupt the conversion via the kernels' bit-mask.
  ///
  /// Returns [`Yuv420pFrame16Error::SampleOutOfRange`] on the first
  /// offending sample. All of [`Self::try_new`]'s geometry errors are
  /// still possible. At `BITS == 16` the check is a no-op (every
  /// `u16` value is valid) — same convention as
  /// [`Yuv420pFrame16::try_new_checked`].
  ///
  /// Per the LE-encoded byte contract on the type, samples are validated
  /// **after** `u16::from_le` normalization so the range check operates
  /// on the intended logical sample on both LE and BE hosts. See
  /// [`Yuv420pFrame16::try_new_checked`] for the full rationale.
  ///
  /// Cost: one O(plane_size) linear scan per plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv420pFrame16Error> {
    let frame = Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride)?;
    if BITS == 16 {
      return Ok(frame);
    }
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    // 4:4:4: chroma is full-width, full-height (1:1 with Y).
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::Y,
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::U,
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
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::V,
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
  /// The `BITS` const parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Compile-time BE flag mirror — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YUV444P*BE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded 4:4:4 planar, 9-bit (`AV_PIX_FMT_YUV444P9LE`).
pub type Yuv444p9Frame<'a> = Yuv444pFrame16<'a, 9>;
/// LE-encoded 4:4:4 planar, 10-bit (`AV_PIX_FMT_YUV444P10LE`).
pub type Yuv444p10Frame<'a> = Yuv444pFrame16<'a, 10>;
/// LE-encoded 4:4:4 planar, 12-bit (`AV_PIX_FMT_YUV444P12LE`).
pub type Yuv444p12Frame<'a> = Yuv444pFrame16<'a, 12>;
/// LE-encoded 4:4:4 planar, 14-bit (`AV_PIX_FMT_YUV444P14LE`).
pub type Yuv444p14Frame<'a> = Yuv444pFrame16<'a, 14>;
/// LE-encoded 4:4:4 planar, 16-bit (`AV_PIX_FMT_YUV444P16LE`). Uses
/// the parallel i64 kernel family.
pub type Yuv444p16Frame<'a> = Yuv444pFrame16<'a, 16>;

// ---- Phase 4 — explicit LE/BE aliases for the 4:4:4 planar HB family ----

/// LE-encoded `Yuv444p9Frame` (`AV_PIX_FMT_YUV444P9LE`).
pub type Yuv444p9LeFrame<'a> = Yuv444pFrame16<'a, 9, false>;
/// BE-encoded `Yuv444p9Frame` (`AV_PIX_FMT_YUV444P9BE`).
pub type Yuv444p9BeFrame<'a> = Yuv444pFrame16<'a, 9, true>;
/// LE-encoded `Yuv444p10Frame` (`AV_PIX_FMT_YUV444P10LE`).
pub type Yuv444p10LeFrame<'a> = Yuv444pFrame16<'a, 10, false>;
/// BE-encoded `Yuv444p10Frame` (`AV_PIX_FMT_YUV444P10BE`).
pub type Yuv444p10BeFrame<'a> = Yuv444pFrame16<'a, 10, true>;
/// LE-encoded `Yuv444p12Frame` (`AV_PIX_FMT_YUV444P12LE`).
pub type Yuv444p12LeFrame<'a> = Yuv444pFrame16<'a, 12, false>;
/// BE-encoded `Yuv444p12Frame` (`AV_PIX_FMT_YUV444P12BE`).
pub type Yuv444p12BeFrame<'a> = Yuv444pFrame16<'a, 12, true>;
/// LE-encoded `Yuv444p14Frame` (`AV_PIX_FMT_YUV444P14LE`).
pub type Yuv444p14LeFrame<'a> = Yuv444pFrame16<'a, 14, false>;
/// BE-encoded `Yuv444p14Frame` (`AV_PIX_FMT_YUV444P14BE`).
pub type Yuv444p14BeFrame<'a> = Yuv444pFrame16<'a, 14, true>;
/// LE-encoded `Yuv444p16Frame` (`AV_PIX_FMT_YUV444P16LE`).
pub type Yuv444p16LeFrame<'a> = Yuv444pFrame16<'a, 16, false>;
/// BE-encoded `Yuv444p16Frame` (`AV_PIX_FMT_YUV444P16BE`).
pub type Yuv444p16BeFrame<'a> = Yuv444pFrame16<'a, 16, true>;

// ---------------------------------------------------------------------------
// Yuv444pMsbFrame — three u16 planes, MSB-packed (high `BITS` bits active)
// ---------------------------------------------------------------------------

/// Identifies which plane of a [`Yuv444pMsbFrame`] a
/// [`Yuv444pMsbFrameError::StrayLowBits`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Yuv444pMsbFramePlane {
  /// Luma plane.
  Y,
  /// U (Cb) chroma plane.
  U,
  /// V (Cr) chroma plane.
  V,
}

/// Payload for [`Yuv444pMsbFrameError::StrayLowBits`]. Records an MSB-packed
/// sample whose **low** `16 − BITS` bits were set (the active bits are the
/// high `BITS`, so the low bits must be zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Yuv444pMsbStrayLowBits {
  plane: Yuv444pMsbFramePlane,
  index: usize,
  value: u16,
}

impl Yuv444pMsbStrayLowBits {
  /// Constructs a new `Yuv444pMsbStrayLowBits`.
  #[inline]
  pub const fn new(plane: Yuv444pMsbFramePlane, index: usize, value: u16) -> Self {
    Self {
      plane,
      index,
      value,
    }
  }
  /// Returns the `plane` the offending sample lives on.
  #[inline]
  pub const fn plane(&self) -> Yuv444pMsbFramePlane {
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

/// Errors returned by [`Yuv444pMsbFrame::try_new`] and
/// [`Yuv444pMsbFrame::try_new_checked`].
///
/// Variant shape mirrors [`Yuv420pFrame16Error`] (geometry) with the added
/// [`Self::StrayLowBits`] packing-sanity variant. Unlike the low-bit
/// [`Yuv444pFrame16`], whose checked constructor rejects samples with stray
/// **high** bits, this MSB-packed type rejects stray **low** bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Yuv444pMsbFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `y_stride < width` (in samples).
  #[error(transparent)]
  InsufficientYStride(InsufficientStride),

  /// `u_stride < width` (in samples; 4:4:4 chroma is full-width).
  #[error(transparent)]
  InsufficientUStride(InsufficientStride),

  /// `v_stride < width` (in samples; 4:4:4 chroma is full-width).
  #[error(transparent)]
  InsufficientVStride(InsufficientStride),

  /// Y plane is shorter than `y_stride * height` samples.
  #[error(transparent)]
  InsufficientYPlane(InsufficientPlane),

  /// U plane is shorter than `u_stride * height` samples.
  #[error(transparent)]
  InsufficientUPlane(InsufficientPlane),

  /// V plane is shorter than `v_stride * height` samples.
  #[error(transparent)]
  InsufficientVPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32-bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A sample's low `16 − BITS` bits were non-zero — an MSB-packed sample
  /// carries its `BITS` active bits in the **high** `BITS` of each `u16`, so
  /// valid samples satisfy `value & ((1 << (16 − BITS)) − 1) == 0`. Only
  /// [`Yuv444pMsbFrame::try_new_checked`] can produce this error; the
  /// geometry-only [`Yuv444pMsbFrame::try_new`] never inspects samples.
  ///
  /// Note: the absence of this error does **not** prove the buffer is
  /// MSB-packed. A low-bit-packed buffer whose samples are all exact
  /// multiples of `1 << (16 − BITS)` passes the check silently. See
  /// [`Yuv444pMsbFrame::try_new_checked`] for the full discussion.
  #[error(
    "sample {:#06x} on plane {} at element {} has non-zero low bits (not a valid YUV 4:4:4 MSB-packed sample)", .0.value(), .0.plane(), .0.index()
  )]
  StrayLowBits(Yuv444pMsbStrayLowBits),
}

/// A validated **MSB-packed** planar YUV 4:4:4 frame at high bit depth
/// (`AV_PIX_FMT_YUV444P{10,12}MSB{LE,BE}`).
///
/// Three full-resolution `u16` planes (Y, U, V — chroma is 1:1 with luma in
/// 4:4:4), with each sample's `BITS` active bits stored in the **high** `BITS`
/// positions of the `u16` (low `16 − BITS` bits zero). This is the exact
/// inverse of the low-bit [`Yuv444pFrame16`], whose samples sit in the **low**
/// `BITS`:
///
/// - **`Yuv444pMsbFrame`** — `value & ((1 << (16 − BITS)) − 1) == 0`; the
///   active bits are the high `BITS`. Matches FFmpeg's `yuv444p{10,12}msb{le,be}`
///   descriptors (`shift = 6` at 10-bit, `shift = 4` at 12-bit).
/// - **[`Yuv444pFrame16`]** — `value >> BITS == 0`; the active bits are the
///   low `BITS`. Matches `yuv444p{10,12}{le,be}` (`shift = 0`).
///
/// A dedicated type is required because the two contracts are mirror images:
/// the low-bit [`Yuv444pFrame16`] validates (and downstream kernels mask) the
/// opposite bit positions, so handing MSB-packed data to it would garble every
/// sample. This mirrors how the low-bit [`Nv20Frame`](crate::frame::Nv20Frame)
/// is kept separate from the high-bit `P210Frame`, and how the GBR family pairs
/// [`GbrpMsbFrame`](crate::frame::GbrpMsbFrame) against
/// [`GbrpHighBitFrame`](crate::frame::GbrpHighBitFrame).
///
/// `BITS ∈ {10, 12}` — validated by a compile-time `const` assertion at
/// construction (FFmpeg defines `yuv444p` MSB variants only at 10 and 12
/// bits). All three planes are full-size; stride is in **samples** (`u16`
/// elements), each plane requiring `*_stride >= width` and
/// `len >= *_stride * height`. No width/height parity constraint.
///
/// The `<const BE: bool>` parameter selects the plane byte order: `false`
/// (default) → LE-encoded `*LE` bytes, `true` → BE-encoded `*BE` bytes.
/// Downstream row kernels handle the byte-swap (or no-op) under the hood —
/// callers do **not** pre-swap.
///
/// Use the per-depth type aliases [`Yuv444p10MsbFrame`] / [`Yuv444p12MsbFrame`]
/// at call sites, or the `*Le*` / `*Be*` aliases for explicit endianness.
#[derive(Debug, Clone, Copy)]
pub struct Yuv444pMsbFrame<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  u: &'a [u16],
  v: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> Yuv444pMsbFrame<'a, BITS, BE> {
  /// Constructs a new [`Yuv444pMsbFrame`], validating dimensions and plane
  /// lengths (geometry only — samples are **not** scanned). Returns
  /// [`Yuv444pMsbFrameError`] if any of:
  /// - `BITS ∉ {10, 12}` — caught at compile time via `const { assert!(…) }`,
  /// - `width` or `height` is zero,
  /// - any stride is smaller than `width` (in samples),
  /// - `stride * height` overflows `usize` (32-bit targets only),
  /// - any plane is shorter than `stride * height` samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv444pMsbFrameError> {
    const {
      assert!(
        matches!(BITS, 10 | 12),
        "BITS must be one of 10 or 12 (FFmpeg defines YUV 4:4:4 MSB variants only at 10 and 12 bits)",
      );
    }

    if width == 0 || height == 0 {
      return Err(Yuv444pMsbFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Yuv444pMsbFrameError::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    // 4:4:4: chroma stride ≥ width (full-width chroma).
    if u_stride < width {
      return Err(Yuv444pMsbFrameError::InsufficientUStride(
        InsufficientStride::new(u_stride, width),
      ));
    }
    if v_stride < width {
      return Err(Yuv444pMsbFrameError::InsufficientVStride(
        InsufficientStride::new(v_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv444pMsbFrameError::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuv444pMsbFrameError::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }

    let u_min = match (u_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv444pMsbFrameError::GeometryOverflow(
          GeometryOverflow::new(u_stride, height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuv444pMsbFrameError::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }

    let v_min = match (v_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv444pMsbFrameError::GeometryOverflow(
          GeometryOverflow::new(v_stride, height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuv444pMsbFrameError::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
    })
  }

  /// Constructs a new [`Yuv444pMsbFrame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Self {
    match Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuv444pMsbFrame dimensions or plane lengths"),
    }
  }

  /// Like [`Self::try_new`] but additionally scans every sample and rejects
  /// any whose **low `16 − BITS` bits** are non-zero. An MSB-packed sample
  /// carries its `BITS` active bits in the **high** `BITS` of each `u16`, so a
  /// valid sample always satisfies `value & ((1 << (16 − BITS)) − 1) == 0`
  /// (low 6 bits zero at 10-bit, low 4 zero at 12-bit). Non-zero low bits are
  /// evidence the buffer isn't MSB-packed — most often a low-bit-packed
  /// [`Yuv444pFrame16`] buffer (active bits in the low `BITS`) handed to the
  /// MSB path, whose high-bit extraction would silently misread it. This is
  /// the exact inverse of [`Nv20Frame::try_new_checked`], which rejects
  /// non-zero **high** bits for the low-bit-packed format, and mirrors
  /// [`GbrpMsbFrame::try_new_checked`].
  ///
  /// **This is a packing sanity check, not a provenance validator.** It
  /// catches noisy low-bit-packed data (where most samples carry low-bit
  /// content), but it **cannot** distinguish MSB-packed data from a low-bit
  /// buffer whose samples all happen to be exact multiples of
  /// `1 << (16 − BITS)`. Callers needing strict provenance must rely on their
  /// source format metadata and pick the right frame type
  /// ([`Yuv444pMsbFrame`] vs [`Yuv444pFrame16`]) at construction.
  ///
  /// Cost: one O(width × height) scan per plane. The default [`Self::try_new`]
  /// skips this so the hot path stays O(1).
  ///
  /// Returns [`Yuv444pMsbFrameError::StrayLowBits`] on the first offending
  /// sample — carries the plane, element index, and offending value.
  ///
  /// Per the byte-order contract recorded by `<const BE: bool>`, samples are
  /// validated **after** `u16::from_le` / `u16::from_be` normalization so the
  /// bit check operates on the intended logical sample value on every host.
  /// The reported `value` in the error is the normalized logical sample.
  ///
  /// [`Nv20Frame::try_new_checked`]: crate::frame::Nv20Frame::try_new_checked
  /// [`GbrpMsbFrame::try_new_checked`]: crate::frame::GbrpMsbFrame::try_new_checked
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv444pMsbFrameError> {
    let frame = Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride)?;
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
    // 4:4:4: every plane is full-width × full-height (1:1 with Y).
    let planes: [(&[u16], u32, Yuv444pMsbFramePlane); 3] = [
      (y, y_stride, Yuv444pMsbFramePlane::Y),
      (u, u_stride, Yuv444pMsbFramePlane::U),
      (v, v_stride, Yuv444pMsbFramePlane::V),
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
            return Err(Yuv444pMsbFrameError::StrayLowBits(
              Yuv444pMsbStrayLowBits::new(which, start + col, logical),
            ));
          }
        }
      }
    }
    Ok(frame)
  }

  /// Y plane samples. Each sample's `BITS` active bits sit in the **high**
  /// `BITS` positions of the `u16` (low `16 − BITS` bits zero).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }
  /// U plane samples. Full-width, full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }
  /// V plane samples. Full-width, full-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
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
  /// Y‑plane stride in samples (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
  /// U‑plane stride in samples (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }
  /// V‑plane stride in samples (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }
  /// Active bit depth — one of 10 or 12. Mirrors the `BITS` const parameter
  /// so generic code can read it without naming the type.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Compile-time BE flag mirror — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YUV444P*MSBBE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Type alias for a validated MSB-packed planar YUV 4:4:4 10-bit frame
/// (`AV_PIX_FMT_YUV444P10MSB{LE,BE}`). Samples in the high 10 bits of each
/// `u16` (low 6 zero). Defaults to LE; use [`Yuv444p10MsbLeFrame`] /
/// [`Yuv444p10MsbBeFrame`] for explicit endianness.
pub type Yuv444p10MsbFrame<'a, const BE: bool = false> = Yuv444pMsbFrame<'a, 10, BE>;
/// LE-encoded `Yuv444p10MsbFrame` (`AV_PIX_FMT_YUV444P10MSBLE`).
pub type Yuv444p10MsbLeFrame<'a> = Yuv444pMsbFrame<'a, 10, false>;
/// BE-encoded `Yuv444p10MsbFrame` (`AV_PIX_FMT_YUV444P10MSBBE`).
pub type Yuv444p10MsbBeFrame<'a> = Yuv444pMsbFrame<'a, 10, true>;

/// Type alias for a validated MSB-packed planar YUV 4:4:4 12-bit frame
/// (`AV_PIX_FMT_YUV444P12MSB{LE,BE}`). Samples in the high 12 bits of each
/// `u16` (low 4 zero).
pub type Yuv444p12MsbFrame<'a, const BE: bool = false> = Yuv444pMsbFrame<'a, 12, BE>;
/// LE-encoded `Yuv444p12MsbFrame` (`AV_PIX_FMT_YUV444P12MSBLE`).
pub type Yuv444p12MsbLeFrame<'a> = Yuv444pMsbFrame<'a, 12, false>;
/// BE-encoded `Yuv444p12MsbFrame` (`AV_PIX_FMT_YUV444P12MSBBE`).
pub type Yuv444p12MsbBeFrame<'a> = Yuv444pMsbFrame<'a, 12, true>;

/// Errors returned by [`Yuv440pFrame16::try_new`] and
/// [`Yuv440pFrame16::try_new_checked`]. Transparent alias of
/// [`Yuv420pFrame16Error`] — same `UnsupportedBits` /
/// `SampleOutOfRange` / geometry variants apply. The alias keeps the
/// public 4:4:0 surface self-descriptive without duplicating an
/// otherwise-identical enum.
pub type Yuv440pFrame16Error = Yuv420pFrame16Error;

/// A validated planar 4:4:0 `u16`-backed frame, generic over
/// `const BITS: u32 ∈ {10, 12}`. Samples are low-bit-packed (the
/// `BITS` active bits sit in the **low** bits of each `u16`).
///
/// Layout: Y full-size, U/V **full-width × half-height** — same
/// vertical subsampling as 4:2:0, no horizontal subsampling like
/// 4:4:4. Per-row kernel reuses the 4:4:4 family
/// (`yuv_444p_n_to_rgb_*<BITS>`) verbatim — only the walker reads
/// chroma row `r / 2` instead of `r`.
///
/// FFmpeg variants: `yuv440p10le`, `yuv440p12le`. No 9/14/16-bit
/// variants exist in FFmpeg, so [`Self::try_new`] rejects them.
#[derive(Debug, Clone, Copy)]
pub struct Yuv440pFrame16<'a, const BITS: u32, const BE: bool = false> {
  y: &'a [u16],
  u: &'a [u16],
  v: &'a [u16],
  width: u32,
  height: u32,
  y_stride: u32,
  u_stride: u32,
  v_stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> Yuv440pFrame16<'a, BITS, BE> {
  /// Constructs a new [`Yuv440pFrame16`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn try_new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv440pFrame16Error> {
    if BITS != 10 && BITS != 12 {
      return Err(Yuv420pFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(Yuv420pFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    if y_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientYStride(
        InsufficientStride::new(y_stride, width),
      ));
    }
    // 4:4:0 chroma is full-width — chroma_width == width.
    if u_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientUStride(
        InsufficientStride::new(u_stride, width),
      ));
    }
    if v_stride < width {
      return Err(Yuv420pFrame16Error::InsufficientVStride(
        InsufficientStride::new(v_stride, width),
      ));
    }

    let y_min = match (y_stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(y_stride, height),
        ));
      }
    };
    if y.len() < y_min {
      return Err(Yuv420pFrame16Error::InsufficientYPlane(
        InsufficientPlane::new(y_min, y.len()),
      ));
    }
    // 4:4:0: chroma is half-height (same axis as 4:2:0 vertical).
    let chroma_height = height.div_ceil(2);
    let u_min = match (u_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(u_stride, chroma_height),
        ));
      }
    };
    if u.len() < u_min {
      return Err(Yuv420pFrame16Error::InsufficientUPlane(
        InsufficientPlane::new(u_min, u.len()),
      ));
    }
    let v_min = match (v_stride as usize).checked_mul(chroma_height as usize) {
      Some(v) => v,
      None => {
        return Err(Yuv420pFrame16Error::GeometryOverflow(
          GeometryOverflow::new(v_stride, chroma_height),
        ));
      }
    };
    if v.len() < v_min {
      return Err(Yuv420pFrame16Error::InsufficientVPlane(
        InsufficientPlane::new(v_min, v.len()),
      ));
    }

    Ok(Self {
      y,
      u,
      v,
      width,
      height,
      y_stride,
      u_stride,
      v_stride,
    })
  }

  /// Constructs a new [`Yuv440pFrame16`], panicking on invalid inputs.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Self {
    match Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid Yuv440pFrame16 dimensions or plane lengths"),
    }
  }

  /// Constructs a new [`Yuv440pFrame16`] and additionally rejects any
  /// sample whose value exceeds `(1 << BITS) - 1`. Mirrors
  /// [`Yuv420pFrame16::try_new_checked`] /
  /// [`Yuv444pFrame16::try_new_checked`]; downstream row kernels mask
  /// the high bits at load time, so out-of-range samples otherwise
  /// produce silently wrong output. Use this constructor on untrusted
  /// inputs (custom decoders, unchecked FFI buffers, etc.).
  ///
  /// Per the LE-encoded byte contract on the type, samples are validated
  /// **after** `u16::from_le` normalization so the range check operates
  /// on the intended logical sample on both LE and BE hosts. See
  /// [`Yuv420pFrame16::try_new_checked`] for the full rationale.
  ///
  /// Cost: one O(plane_size) linear scan per plane. The chroma planes
  /// here are full-width × half-height (4:4:0 layout).
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub fn try_new_checked(
    y: &'a [u16],
    u: &'a [u16],
    v: &'a [u16],
    width: u32,
    height: u32,
    y_stride: u32,
    u_stride: u32,
    v_stride: u32,
  ) -> Result<Self, Yuv440pFrame16Error> {
    let frame = Self::try_new(y, u, v, width, height, y_stride, u_stride, v_stride)?;
    // No BITS == 16 early-return: `try_new` rejects everything outside
    // {10, 12}, so unlike Yuv420p/444p (which both accept 16) the
    // u16-saturating-noop case can't occur here.
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    let chroma_h = (height as usize).div_ceil(2);
    for row in 0..h {
      let start = row * y_stride as usize;
      for (col, &s) in y[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::Y,
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
      for (col, &s) in u[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::U,
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
      for (col, &s) in v[start..start + w].iter().enumerate() {
        let logical = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if logical > max_valid {
          return Err(Yuv420pFrame16Error::SampleOutOfRange(
            Yuv420pFrame16SampleOutOfRange::new(
              Yuv420pFrame16Plane::V,
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

  /// Y plane (`u16` elements).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> &'a [u16] {
    self.y
  }
  /// U plane. **Full-width, half-height.**
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u(&self) -> &'a [u16] {
    self.u
  }
  /// V plane. Full-width, half-height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u16] {
    self.v
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
  /// Y plane stride in samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y_stride(&self) -> u32 {
    self.y_stride
  }
  /// U plane stride in samples (full-width).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn u_stride(&self) -> u32 {
    self.u_stride
  }
  /// V plane stride in samples (full-width).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v_stride(&self) -> u32 {
    self.v_stride
  }
  /// The `BITS` const parameter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
  /// Compile-time BE flag mirror — `true` if plane bytes are BE-encoded
  /// (`AV_PIX_FMT_YUV440P*BE`), `false` if LE-encoded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// LE-encoded 4:4:0 planar, 10-bit (`AV_PIX_FMT_YUV440P10LE`).
pub type Yuv440p10Frame<'a> = Yuv440pFrame16<'a, 10>;
/// LE-encoded 4:4:0 planar, 12-bit (`AV_PIX_FMT_YUV440P12LE`).
pub type Yuv440p12Frame<'a> = Yuv440pFrame16<'a, 12>;

// ---- Phase 4 — explicit LE/BE aliases for the 4:4:0 planar HB family ----

/// LE-encoded `Yuv440p10Frame` (`AV_PIX_FMT_YUV440P10LE`).
pub type Yuv440p10LeFrame<'a> = Yuv440pFrame16<'a, 10, false>;
/// BE-encoded `Yuv440p10Frame` (`AV_PIX_FMT_YUV440P10BE`).
pub type Yuv440p10BeFrame<'a> = Yuv440pFrame16<'a, 10, true>;
/// LE-encoded `Yuv440p12Frame` (`AV_PIX_FMT_YUV440P12LE`).
pub type Yuv440p12LeFrame<'a> = Yuv440pFrame16<'a, 12, false>;
/// BE-encoded `Yuv440p12Frame` (`AV_PIX_FMT_YUV440P12BE`).
pub type Yuv440p12BeFrame<'a> = Yuv440pFrame16<'a, 12, true>;
