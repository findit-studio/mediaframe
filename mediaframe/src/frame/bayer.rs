use super::{
  GeometryOverflow, InsufficientPlane, InsufficientStride, UnsupportedBits, ZeroDimension,
};
use crate::PixelSink;
use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use thiserror::Error;

/// A validated Bayer-mosaic frame at 8 bits per sample.
///
/// Single plane: each `u8` element is one sensor sample, with the
/// color (R / G / B) determined by the `BayerPattern`
/// passed at the walker boundary and the sample's `(row, column)`
/// position within the repeating 2×2 tile.
///
/// Odd `width` and `height` are accepted: a cropped Bayer plane
/// (post-production crop, sensor-specific active area) legitimately
/// exhibits a partial 2×2 tile at the right column or bottom row.
/// The walker clamps top / bottom rows and the demosaic kernel
/// clamps left / right columns, so the math is defined for every
/// site regardless of dimension parity.
///
/// `stride` is the sample stride of the plane — `>= width`,
/// permitting the upstream decoder to pad rows.
///
/// Source: FFmpeg's `bayer_bggr8` / `bayer_rggb8` / `bayer_grbg8` /
/// `bayer_gbrg8` decoders, vendor-SDK Bayer ingest paths (R3D /
/// BRAW / NRAW), and any custom RAW pipeline that has already
/// extracted a Bayer plane from the camera bitstream.
#[derive(Debug, Clone, Copy)]
pub struct BayerFrame<'a> {
  data: &'a [u8],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a> BayerFrame<'a> {
  /// Constructs a new [`BayerFrame`], validating dimensions and
  /// plane length.
  ///
  /// Returns [`BayerFrameError`] if any of:
  /// - `width` or `height` is zero,
  /// - `stride < width`,
  /// - `data.len() < stride * height`, or
  /// - `stride * height` overflows `usize` (32‑bit targets only).
  ///
  /// Odd widths and heights are accepted; see the type-level docs
  /// for the rationale.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(
    data: &'a [u8],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, BayerFrameError> {
    if width == 0 || height == 0 {
      return Err(BayerFrameError::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    // Odd Bayer widths and heights are accepted: a cropped Bayer
    // plane (post-production crop, sensor-specific active area)
    // legitimately exhibits a partial 2×2 tile at the right column
    // or bottom row. The walker clamps top / bottom rows and the
    // demosaic kernel clamps left / right columns, so the math is
    // defined for every site regardless of dimension parity.
    if stride < width {
      return Err(BayerFrameError::InsufficientStride(
        InsufficientStride::new(stride, width),
      ));
    }
    let min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(BayerFrameError::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if data.len() < min {
      return Err(BayerFrameError::InsufficientPlane(InsufficientPlane::new(
        min,
        data.len(),
      )));
    }
    Ok(Self {
      data,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`BayerFrame`], panicking on invalid inputs.
  /// Prefer [`Self::try_new`] when inputs may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(data: &'a [u8], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(data, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => panic!("invalid BayerFrame dimensions or plane length"),
    }
  }

  /// The Bayer plane bytes. Row `r` starts at byte offset
  /// `r * stride()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data(&self) -> &'a [u8] {
    self.data
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

  /// Byte stride of the Bayer plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
}

/// A validated Bayer-mosaic frame at 10 / 12 / 14 / 16 bits per
/// sample, **low-packed** (each sample's *logical* value occupies the
/// low bits of its `u16`).
///
/// `BITS` ∈ {10, 12, 14, 16}; each `u16`'s **logical** value — after
/// byte-order normalization (see the endian contract below) — occupies
/// the **low** `BITS` bits (range `[0, (1 << BITS) - 1]`), with the high
/// `16 - BITS` bits zero. (The stored `u16` is the raw wire element, not
/// this logical value; see [`Self::data`].) This matches the planar
/// high-bit-depth convention used by `Yuv420pFrame16`, `Yuv422p*`, and
/// `Yuv444p*`. Note that this is **not** the `PnFrame`
/// (`P010` / `P012`) convention, which is high-bit-packed
/// (semi-planar `u16` containers carry samples in the *high* bits);
/// Bayer is single-plane and tracks the planar family instead.
///
/// # Endian contract — `<const BE: bool = false>`
///
/// The `<const BE: bool>` parameter selects the plane byte order,
/// mirroring the Y2xx (`Y210`/`Y212`/`Y216`) family. FFmpeg defines a
/// Bayer LE/BE split only at **16-bit** (`AV_PIX_FMT_BAYER_*16LE` /
/// `*16BE`); the 10 / 12 / 14-bit forms are **mediaframe extensions**
/// with no FFmpeg pixel format, endian-aware for decoders that emit
/// high-bit Bayer:
///
/// - `BE = false` (default; e.g. [`Bayer10Frame`]) — the `&[u16]` plane
///   is the **LE-encoded byte layout** reinterpreted as `u16` (at
///   16-bit, `AV_PIX_FMT_BAYER_*16LE`). On a little-endian host LE bytes
///   are host-native; on a big-endian host they are byte-swapped before
///   arithmetic.
/// - `BE = true` (e.g. [`Bayer10BeFrame`]) — the plane is the
///   **BE-encoded byte layout** (at 16-bit, `AV_PIX_FMT_BAYER_*16BE`).
///   On a little-endian host the bytes are byte-swapped before
///   arithmetic; on a big-endian host they are host-native.
///
/// Downstream row kernels handle the byte-swap (or no-op) under the
/// hood — callers do **not** pre-swap. The `BE` flag propagates from
/// the frame through [`bayer16_to_endian`] into the sink dispatch,
/// exactly as Y216's `BE` propagates through `y216_to_endian`. Callers
/// holding a raw FFmpeg byte buffer should cast via
/// `bytemuck::cast_slice` (which checks alignment) before constructing.
///
/// **Wire-bytes contract, and the change from the pre-endian-aware
/// behavior.** The plane `&[u16]` is now interpreted as **wire bytes**
/// — LE-encoded for `BE = false`, BE-encoded for `BE = true` — exactly
/// like [`Y2xxFrame`](crate::frame::Y2xxFrame) and the high-bit YUV
/// frames, and both [`Self::try_new`]'s sample check and the row kernel
/// normalize that byte order (`from_le` / `from_be`) before touching the
/// value. This is the host-independent, FFmpeg-matching contract.
///
/// It is **not** byte-identical on every host to the original,
/// endian-naive `BayerFrame16`, which compared the host-native `u16`
/// reading directly with no byte-order normalization. The correct
/// characterization is:
///
/// - On a **little-endian host** the behavior is **identical** to the
///   pre-endian-aware code for `BE = false`: the LE wire order coincides
///   with the host-native order, so `from_le` is a no-op and the
///   compared value is unchanged.
/// - On a **big-endian host** the behavior **differs** from the
///   pre-endian-aware code: a `BE = false` plane is now correctly read
///   as LE wire bytes (byte-swapped before the range check and the
///   arithmetic) rather than as raw native `u16`. This is the *intended*
///   fix — an LE-encoded plane is LE on every host — not a regression.
///
/// (For `BE = true` the same logic holds with the endianness roles
/// swapped: identical to a hypothetical native reading on a big-endian
/// host, byte-swapped on a little-endian host.)
///
/// **Type-level guarantee.** [`Self::try_new`] validates every
/// active sample against the low-packed range as part of
/// construction (after normalizing the wire byte order to
/// host-native), so an existing `BayerFrame16<BITS, BE>` value is
/// guaranteed to carry only in-range samples. Downstream
/// `bayer16_to` / `bayer16_to_endian` therefore needs no further
/// runtime validation and never panics on bad sample data —
/// any `Result::Err` from the conversion comes from the sink,
/// never from the frame's contents.
///
/// Diverges from the rest of the high-bit-depth crate
/// (`Yuv420pFrame16` / `Yuv422pFrame16` / `Yuv444pFrame16` ship a
/// cheap `try_new` + opt-in `try_new_checked`) because Bayer16
/// frames typically come from less-trusted RAW pipelines (vendor
/// SDKs, file loaders) and have no hot-path performance pressure
/// to skip the per-sample check. Mandatory validation makes the
/// `bayer16_to` walker fully fallible.
///
/// Odd widths and heights are accepted (cropped Bayer is a real
/// workflow; the kernel handles partial 2×2 tiles via edge
/// clamping).
///
/// Source: FFmpeg's `bayer_*16le` / `bayer_*16be` decoders, vendor-SDK
/// 10/12/14/16-bit RAW ingest paths. If your upstream provides
/// high-bit-packed Bayer (active bits in the *high* `BITS`),
/// right-shift each sample by `(16 - BITS)` before constructing
/// [`BayerFrame16`].
#[derive(Debug, Clone, Copy)]
pub struct BayerFrame16<'a, const BITS: u32, const BE: bool = false> {
  data: &'a [u16],
  width: u32,
  height: u32,
  stride: u32,
}

impl<'a, const BITS: u32, const BE: bool> BayerFrame16<'a, BITS, BE> {
  /// Constructs a new [`BayerFrame16`], validating dimensions,
  /// plane length, the `BITS` parameter, **and every active
  /// sample's value**.
  ///
  /// Unlike the rest of the high-bit-depth crate (`Yuv420pFrame16`,
  /// `Yuv422pFrame16`, etc.) which split the validation into
  /// `try_new` (geometry) + `try_new_checked` (samples), Bayer16
  /// always validates samples here. RAW pipelines often surface
  /// trusted-but-actually-mispacked input (MSB-aligned bytes from
  /// a sensor SDK, stale high bits from a copy that didn't mask
  /// the source), and downstream demosaic / WB / CCM math has no
  /// well-defined behavior on out-of-range samples. Catching at
  /// construction lets callers handle the failure as a normal
  /// `Result` instead of risking a panic later in
  /// `bayer16_to`.
  ///
  /// Per the byte-encoding contract on the type-level docs, each
  /// sample is normalized from its wire byte order to host-native
  /// (`u16::from_le` for `BE = false`, `u16::from_be` for `BE = true`)
  /// **before** the range check, so the check always operates on the
  /// intended logical value. On a little-endian host `from_le` is a
  /// no-op and `from_be` byte-swaps; on a big-endian host the roles
  /// flip. Without this a valid BE plane on an LE host would have its
  /// low-packed samples appear byte-swapped (high bits set in the
  /// host-native reading) and the validator would falsely reject every
  /// row — and vice-versa. This mirrors `Y2xxFrame::try_new_checked`.
  ///
  /// `stride` is in **samples** (`u16` elements). Returns
  /// [`BayerFrame16Error`] if any of:
  /// - `BITS` is not 10, 12, 14, or 16,
  /// - `width` or `height` is zero,
  /// - `stride < width`,
  /// - `data.len() < stride * height`,
  /// - `stride * height` overflows `usize`, or
  /// - any sample's value exceeds `(1 << BITS) - 1` (returned as
  ///   [`BayerFrame16Error::SampleOutOfRange`]).
  ///
  /// Odd widths and heights are accepted; see the type-level docs
  /// for the rationale.
  ///
  /// Cost: O(width × height) sample scan in addition to the
  /// O(1) geometry checks. The scan is a tight loop over `u16`
  /// values per row and runs once per frame; downstream
  /// `bayer16_to` therefore needs no further
  /// sample validation.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_new(
    data: &'a [u16],
    width: u32,
    height: u32,
    stride: u32,
  ) -> Result<Self, BayerFrame16Error> {
    if BITS != 10 && BITS != 12 && BITS != 14 && BITS != 16 {
      return Err(BayerFrame16Error::UnsupportedBits(UnsupportedBits::new(
        BITS,
      )));
    }
    if width == 0 || height == 0 {
      return Err(BayerFrame16Error::ZeroDimension(ZeroDimension::new(
        width, height,
      )));
    }
    // Odd Bayer widths and heights are accepted; see
    // [`BayerFrame::try_new`] for the rationale (cropped Bayer is
    // a real workflow, edge clamping handles partial tiles).
    if stride < width {
      return Err(BayerFrame16Error::InsufficientStride(
        InsufficientStride::new(stride, width),
      ));
    }
    let min = match (stride as usize).checked_mul(height as usize) {
      Some(v) => v,
      None => {
        return Err(BayerFrame16Error::GeometryOverflow(GeometryOverflow::new(
          stride, height,
        )));
      }
    };
    if data.len() < min {
      return Err(BayerFrame16Error::InsufficientPlane(
        InsufficientPlane::new(min, data.len()),
      ));
    }
    // Sample range scan — only the **active** per-row region
    // (`r * stride .. r * stride + width`) is checked. Row padding
    // and trailing storage are deliberately skipped because the
    // walker never reads them, matching the boundary contract of
    // the row dispatchers.
    //
    // Each sample is normalized from wire byte order to host-native
    // before the range check (`from_le` for `BE = false`, `from_be`
    // for `BE = true`) so the low-packed range is checked against the
    // intended logical value on every host; see the method docs.
    let max_valid: u16 = ((1u32 << BITS) - 1) as u16;
    let w = width as usize;
    let h = height as usize;
    for row in 0..h {
      let start = row * stride as usize;
      for (col, &s) in data[start..start + w].iter().enumerate() {
        let host = if BE { u16::from_be(s) } else { u16::from_le(s) };
        if host > max_valid {
          return Err(BayerFrame16Error::SampleOutOfRange(
            BayerSampleOutOfRange::new(start + col, host, max_valid),
          ));
        }
      }
    }
    Ok(Self {
      data,
      width,
      height,
      stride,
    })
  }

  /// Constructs a new [`BayerFrame16`], panicking on invalid inputs.
  /// Includes sample-range validation; see [`Self::try_new`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(data: &'a [u16], width: u32, height: u32, stride: u32) -> Self {
    match Self::try_new(data, width, height, stride) {
      Ok(frame) => frame,
      Err(_) => {
        panic!("invalid BayerFrame16 dimensions, plane length, BITS value, or sample range")
      }
    }
  }

  /// The Bayer plane as raw wire `u16` elements. Row `r` starts at
  /// sample offset `r * stride()`. Each element is in the frame's
  /// wire byte order (LE-encoded for `BE = false`, BE-encoded for
  /// `BE = true`) and is **not** pre-normalized. Its **logical**
  /// value — after `from_le` / `from_be` per [`is_be`](Self::is_be)
  /// — carries the `BITS` active bits in the low `BITS` positions,
  /// the high `16 - BITS` zero on well-formed input. Downstream
  /// kernels normalize byte order; callers do **not** pre-swap.
  /// See the type-level endian contract.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data(&self) -> &'a [u16] {
    self.data
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

  /// Sample stride of the Bayer plane (`>= width`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Active bit depth — 10, 12, 14, or 16.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }

  /// Returns the compile-time BE flag — `true` if plane bytes are
  /// BE-encoded, `false` if LE-encoded (at 16-bit, the FFmpeg
  /// `AV_PIX_FMT_BAYER_*16BE` / `*16LE`). Runtime mirror of the
  /// `<const BE: bool>` type parameter; mirrors `Y2xxFrame::is_be`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_be(&self) -> bool {
    BE
  }
}

/// Type alias for a 10-bit LE Bayer frame — LE-wire `u16` elements whose
/// **logical** values (after `from_le`) are in `[0, 1023]` (high 6 bits
/// zero). See [`BayerFrame16`] for the wire-bytes / endian contract.
pub type Bayer10Frame<'a> = BayerFrame16<'a, 10, false>;
/// Type alias for a 12-bit LE Bayer frame.
pub type Bayer12Frame<'a> = BayerFrame16<'a, 12, false>;
/// Type alias for a 14-bit LE Bayer frame.
pub type Bayer14Frame<'a> = BayerFrame16<'a, 14, false>;
/// Type alias for a 16-bit LE Bayer frame.
pub type Bayer16Frame<'a> = BayerFrame16<'a, 16, false>;

/// Type alias for a 10-bit **BE-encoded** Bayer frame (mediaframe
/// extension) — plane bytes are big-endian `u16` samples; downstream
/// kernels byte-swap under the hood.
pub type Bayer10BeFrame<'a> = BayerFrame16<'a, 10, true>;
/// Type alias for a 12-bit **BE-encoded** Bayer frame (mediaframe extension).
pub type Bayer12BeFrame<'a> = BayerFrame16<'a, 12, true>;
/// Type alias for a 14-bit **BE-encoded** Bayer frame (mediaframe extension).
pub type Bayer14BeFrame<'a> = BayerFrame16<'a, 14, true>;
/// Type alias for a 16-bit **BE-encoded** Bayer frame
/// (`AV_PIX_FMT_BAYER_*16BE`).
pub type Bayer16BeFrame<'a> = BayerFrame16<'a, 16, true>;

/// Errors returned by [`BayerFrame::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum BayerFrameError {
  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < width`.
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` bytes.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (can only fire on
  /// 32‑bit targets — wasm32, i686 — with extreme dimensions).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),
}

/// Errors returned by [`BayerFrame16::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, TryUnwrap, Unwrap, Error)]
#[non_exhaustive]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum BayerFrame16Error {
  /// `BITS` const-generic parameter is not one of `{10, 12, 14, 16}`.
  #[error(transparent)]
  UnsupportedBits(UnsupportedBits),

  /// `width` or `height` was zero.
  #[error(transparent)]
  ZeroDimension(ZeroDimension),

  /// `stride < width` (in `u16` samples).
  #[error(transparent)]
  InsufficientStride(InsufficientStride),

  /// Plane is shorter than `stride * height` samples.
  #[error(transparent)]
  InsufficientPlane(InsufficientPlane),

  /// `stride * rows` does not fit in `usize` (32‑bit targets only).
  #[error(transparent)]
  GeometryOverflow(GeometryOverflow),

  /// A sample's value exceeds `(1 << BITS) - 1` — the sample's
  /// high `16 - BITS` bits are non-zero, which is invalid under
  /// the low-packed Bayer16 convention. Returned by
  /// [`BayerFrame16::try_new`] (and [`BayerFrame16::new`] which
  /// wraps it) — sample-range validation is part of standard
  /// frame construction so the `bayer16_to` walker
  /// is fully fallible.
  #[error(transparent)]
  SampleOutOfRange(BayerSampleOutOfRange),
}

/// Payload struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[error("bayer sample {} at element {} exceeds {} ((1 << BITS) - 1)", self.value(), self.index(), self.max_valid())]
pub struct BayerSampleOutOfRange {
  index: usize,
  value: u16,
  max_valid: u16,
}

impl BayerSampleOutOfRange {
  /// Constructs a new `BayerSampleOutOfRange`.
  #[inline]
  pub const fn new(index: usize, value: u16, max_valid: u16) -> Self {
    Self {
      index,
      value,
      max_valid,
    }
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

/// Bayer pattern — which sensor color sits at the top-left of the
/// repeating 2×2 tile.
///
/// In BGGR / RGGB the green diagonal runs top-left → bottom-right; in
/// GRBG / GBRG the green diagonal runs top-right → bottom-left. Each
/// 2×2 cell carries two greens (one on the red row, one on the blue
/// row), one red, and one blue.
///
/// Source: read from the camera's metadata (R3D `ImagerCFA`, BRAW
/// `cfa_pattern`, NRAW SDK accessor). FFmpeg's bayer pixel formats
/// (`AV_PIX_FMT_BAYER_BGGR8` / `RGGB8` / `GRBG8` / `GBRG8` and the
/// `*_16LE` siblings) carry the pattern in the format identifier
/// itself.
///
/// **Scope.** This enum covers the four standard 2×2 Bayer
/// arrangements only. Other CFA families used by modern
/// professional cameras (Quad Bayer / Sony, X-Trans / Fujifilm,
/// RGBW / BMD URSA 12K, Foveon stacked photosites / Sigma,
/// monochrome / Leica) are tracked separately as future RAW
/// pixel-buffer types — they need different walker shapes
/// and / or completely different demosaic algorithms, so they
/// won't ride on this enum. See
/// `docs/color-conversion-functions.md` § "Cleanup follow-ups
/// → Tier 14 RAW family extensions" for the full roadmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Display)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum BayerPattern {
  /// `B G / G R` — top-left is **B**, bottom-right is **R**.
  Bggr,
  /// `R G / G B` — top-left is **R**, bottom-right is **B**.
  Rggb,
  /// `G R / B G` — top-left is **G** (on the red row), top-right is
  /// **R**.
  Grbg,
  /// `G B / R G` — top-left is **G** (on the blue row), top-right is
  /// **B**.
  Gbrg,
}

impl BayerPattern {
  /// Returns the Bayer pattern's name as a string.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Bggr => "bggr",
      Self::Rggb => "rggb",
      Self::Grbg => "grbg",
      Self::Gbrg => "gbrg",
    }
  }
}

/// Demosaic algorithm.
///
/// Selects the per-pixel reconstruction kernel the walker uses to
/// fill in the two missing color channels at each Bayer site.
///
/// Currently only [`BayerDemosaic::Bilinear`] is wired up. The enum
/// is `#[non_exhaustive]` so future variants (Malvar-He-Cutler /
/// MHC for sharper output, DCB / VNG / AHD for edge-aware
/// high-quality reconstruction) can land without a breaking
/// change. The MHC variant is the smallest next step (5-row
/// window, ~3× bilinear cost); DCB / VNG / AHD are larger
/// follow-ups that need a different walker shape than the per-row
/// model. See `docs/color-conversion-functions.md` §
/// "Cleanup follow-ups → Higher-quality Bayer demosaic algorithms"
/// for the full design notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, IsVariant, Display)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum BayerDemosaic {
  /// Bilinear demosaic — 3×3 row window, 4-tap horizontal/vertical
  /// average for the missing color channels. Soft but fast and
  /// numerically stable; the standard "first pass" reconstruction.
  #[default]
  Bilinear,
}

impl BayerDemosaic {
  /// Returns the demosaic algorithm's name as a string.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Bilinear => "Bilinear",
    }
  }
}

/// Per-channel white-balance gains.
///
/// Each gain is a **finite, non-negative** `f32` multiplier applied
/// to the corresponding raw color channel before the
/// [`ColorCorrectionMatrix`] is applied. Source: camera metadata
/// (`WB_RGGB_LEVELS` family, RED `Kelvin` / `Tint` resolved to
/// gains by the SDK, BRAW `whiteBalanceKelvin` resolved similarly).
/// [`WhiteBalance::try_new`] enforces the invariant; any NaN, ±∞,
/// or negative gain is rejected via [`WhiteBalanceError`].
///
/// Zero is permitted (zeroes that channel — degenerate but
/// well-defined).
///
/// A neutral [`WhiteBalance::neutral`] (`R = G = B = 1.0`) means
/// "no white-balance correction" — the sensor's native primaries are
/// passed through unchanged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiteBalance {
  r: f32,
  g: f32,
  b: f32,
}

impl WhiteBalance {
  /// Constructs a [`WhiteBalance`] from explicit R / G / B gains,
  /// validating that each is **finite and non-negative**. Camera
  /// metadata pipelines occasionally surface NaN / ±∞ (failed Kelvin
  /// → gain conversions, missing sensor metadata) and a single such
  /// value would propagate through the fused 3×3 transform and
  /// produce silently-corrupt output (NaN clamps to 0 on cast,
  /// turning unrelated channels black). Reject upstream instead.
  ///
  /// Returns [`WhiteBalanceError`] if any gain is non-finite or
  /// negative. A gain of `0` is permitted (zeroes out that channel —
  /// degenerate but well-defined).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(r: f32, g: f32, b: f32) -> Result<Self, WhiteBalanceError> {
    if !r.is_finite() {
      return Err(WhiteBalanceError::NonFinite(
        NonFiniteWhiteBalanceChannel::r(r),
      ));
    }
    if !g.is_finite() {
      return Err(WhiteBalanceError::NonFinite(
        NonFiniteWhiteBalanceChannel::g(g),
      ));
    }
    if !b.is_finite() {
      return Err(WhiteBalanceError::NonFinite(
        NonFiniteWhiteBalanceChannel::b(b),
      ));
    }
    if r < 0.0 {
      return Err(WhiteBalanceError::Negative(NegativeWhiteBalanceChannel::r(
        r,
      )));
    }
    if g < 0.0 {
      return Err(WhiteBalanceError::Negative(NegativeWhiteBalanceChannel::g(
        g,
      )));
    }
    if b < 0.0 {
      return Err(WhiteBalanceError::Negative(NegativeWhiteBalanceChannel::b(
        b,
      )));
    }
    // Magnitude bound. Real WB gains rarely exceed 10× (extreme
    // tungsten correction); the bound is generous (`1e6`) but
    // closes the door on finite-but-pathological metadata that
    // would overflow per-pixel f32 math during the matmul. With
    // gains ≤ 1e6 and 16-bit samples (≤ 65535) and CCM coefficients
    // bounded by [`ColorCorrectionMatrix::MAX_COEFFICIENT`],
    // the largest per-channel sum stays well under `f32::MAX`,
    // so the kernel can never produce Inf or NaN from validated
    // inputs.
    if r > Self::MAX_GAIN {
      return Err(WhiteBalanceError::OutOfBounds(
        WhiteBalanceChannelOutOfBounds::r(r, Self::MAX_GAIN),
      ));
    }
    if g > Self::MAX_GAIN {
      return Err(WhiteBalanceError::OutOfBounds(
        WhiteBalanceChannelOutOfBounds::g(g, Self::MAX_GAIN),
      ));
    }
    if b > Self::MAX_GAIN {
      return Err(WhiteBalanceError::OutOfBounds(
        WhiteBalanceChannelOutOfBounds::b(b, Self::MAX_GAIN),
      ));
    }
    Ok(Self { r, g, b })
  }

  /// Maximum permitted gain magnitude. `1e6` is far above any
  /// realistic camera-metadata value (real WB gains are O(1–10))
  /// and far below the value at which per-pixel f32 matmul could
  /// overflow given sample range `[0, 65535]` and CCM coefficient
  /// bounds — see [`Self::try_new`] for the full overflow analysis.
  pub const MAX_GAIN: f32 = 1.0e6;

  /// Constructs a [`WhiteBalance`], panicking on invalid input.
  /// Prefer [`Self::try_new`] when gains may be invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(r: f32, g: f32, b: f32) -> Self {
    match Self::try_new(r, g, b) {
      Ok(wb) => wb,
      Err(_) => panic!("invalid WhiteBalance gains (non-finite, negative, or > MAX_GAIN)"),
    }
  }

  /// Neutral white-balance (`R = G = B = 1.0`) — sensor primaries
  /// pass through unchanged.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn neutral() -> Self {
    Self {
      r: 1.0,
      g: 1.0,
      b: 1.0,
    }
  }

  /// Red-channel gain.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(&self) -> f32 {
    self.r
  }

  /// Green-channel gain.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(&self) -> f32 {
    self.g
  }

  /// Blue-channel gain.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(&self) -> f32 {
    self.b
  }
}

impl Default for WhiteBalance {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::neutral()
  }
}

/// 3×3 color-correction matrix applied after white balance.
///
/// Maps the sensor's white-balanced RGB into a target working space
/// (sRGB / Rec.709 / Rec.2020). Stored row-major: `m[i][j]` is the
/// coefficient of the input column `j` contributing to the output
/// channel `i`. Applying the matrix to an input vector
/// `[R_in, G_in, B_in]` yields:
///
/// ```text
///   R_out = m[0][0]*R_in + m[0][1]*G_in + m[0][2]*B_in
///   G_out = m[1][0]*R_in + m[1][1]*G_in + m[1][2]*B_in
///   B_out = m[2][0]*R_in + m[2][1]*G_in + m[2][2]*B_in
/// ```
///
/// A neutral [`ColorCorrectionMatrix::identity`] (1.0 on the
/// diagonal, 0 off) means "no color correction" — the
/// white-balanced sensor RGB is passed through.
///
/// Source: RED / BMD / Nikon SDKs hand a 3×3 back natively.
///
/// **Color-space note.** This matrix is *opaque* about the target
/// gamut — the caller decides whether the output is in Rec.709 /
/// Rec.2020 / DCI-P3 / ACES AP0 or AP1 / sensor-native primaries
/// by choosing the coefficients accordingly. The output is always
/// **scene-linear** (no transfer-function / log / gamma encoding
/// applied; the demosaic kernel does linear arithmetic).
/// Downstream gamut transforms and transfer-function encoding
/// (sRGB, Rec.709 OETF, log, HLG, PQ) are not in `colconv`'s
/// current scope — typically handled via OCIO or a dedicated
/// tonemap layer. See `docs/color-conversion-functions.md` §
/// "Cleanup follow-ups → Color-space handling" for the deferred
/// in-crate convenience-layer roadmap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorCorrectionMatrix {
  m: [[f32; 3]; 3],
}

impl ColorCorrectionMatrix {
  /// Constructs a [`ColorCorrectionMatrix`] from a row-major 3×3,
  /// validating that every element is **finite** (not NaN, not
  /// ±∞) and bounded by `|value| <= [`Self::MAX_COEFFICIENT_ABS`]
  /// (= 1e6). CCM elements may legitimately be negative (color
  /// matrices regularly subtract crosstalk), and the magnitude
  /// bound is well above any realistic camera value (real CCMs
  /// are O(1–5)) but closes the door on finite-but-pathological
  /// metadata that would overflow per-pixel f32 math.
  ///
  /// Returns [`ColorCorrectionMatrixError`] on the first
  /// out-of-spec element, naming its `(row, col)` coordinates.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(m: [[f32; 3]; 3]) -> Result<Self, ColorCorrectionMatrixError> {
    let mut row = 0;
    while row < 3 {
      let mut col = 0;
      while col < 3 {
        let v = m[row][col];
        if !v.is_finite() {
          return Err(ColorCorrectionMatrixError::NonFinite(
            NonFiniteColorCorrectionMatrixElement::new(row, col, v),
          ));
        }

        // Magnitude bound — see the type-level docs for the
        // overflow analysis. With `|coeff| <= 1e6`, gain ≤ 1e6,
        // and sample range `[0, 65535]`, the largest per-channel
        // sum is `3 * 1e6 * 1e6 * 65535 ≈ 1.97e17`, ~21 orders
        // of magnitude under `f32::MAX ≈ 3.4e38`. No Inf, no NaN.
        if !(v >= -Self::MAX_COEFFICIENT_ABS && v <= Self::MAX_COEFFICIENT_ABS) {
          return Err(ColorCorrectionMatrixError::OutOfBounds(
            ColorCorrectionMatrixElementOutOfBounds::new(row, col, v, Self::MAX_COEFFICIENT_ABS),
          ));
        }
        col += 1;
      }
      row += 1;
    }
    Ok(Self { m })
  }

  /// Maximum permitted absolute value of any CCM element. `1e6`
  /// is far above any realistic camera-metadata value (real CCMs
  /// are O(1–5)) and closes the door on finite-but-pathological
  /// metadata. See [`Self::try_new`] for the overflow analysis.
  pub const MAX_COEFFICIENT_ABS: f32 = 1.0e6;

  /// Constructs a [`ColorCorrectionMatrix`], panicking on invalid
  /// input. Prefer [`Self::try_new`] when matrix elements may be
  /// invalid at runtime.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(m: [[f32; 3]; 3]) -> Self {
    match Self::try_new(m) {
      Ok(ccm) => ccm,
      Err(_) => panic!(
        "invalid ColorCorrectionMatrix element (non-finite or |value| > MAX_COEFFICIENT_ABS)"
      ),
    }
  }

  /// The identity matrix — no color correction. Equivalent to
  /// passing the white-balanced sensor RGB straight through.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn identity() -> Self {
    Self {
      m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    }
  }

  /// Borrows the underlying row-major 3×3.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_array(&self) -> &[[f32; 3]; 3] {
    &self.m
  }
}

impl Default for ColorCorrectionMatrix {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::identity()
  }
}

/// Identifies which white-balance channel failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Display)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum WbChannel {
  /// Red gain.
  R,
  /// Green gain.
  G,
  /// Blue gain.
  B,
}

impl WbChannel {
  /// Returns a human-readable name for the channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      WbChannel::R => "R",
      WbChannel::G => "G",
      WbChannel::B => "B",
    }
  }
}

/// Errors returned by [`WhiteBalance::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, IsVariant, Unwrap, TryUnwrap, Error)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum WhiteBalanceError {
  /// A gain is non-finite (NaN, +∞, or -∞).
  #[error(transparent)]
  NonFinite(#[from] NonFiniteWhiteBalanceChannel),
  /// A gain is negative. Zero is allowed (zeroes the channel).
  #[error(transparent)]
  Negative(#[from] NegativeWhiteBalanceChannel),
  /// A gain exceeds [`WhiteBalance::MAX_GAIN`] (`1e6`). The bound
  /// is far above any realistic camera value but closes the door
  /// on finite-but-pathological metadata that would overflow
  /// per-pixel f32 matmul.
  #[error(transparent)]
  OutOfBounds(#[from] WhiteBalanceChannelOutOfBounds),
}

/// non-finite white balance channel (NaN, +∞, or -∞)
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error("white balance channel {} is non-finite (got {})", .channel.as_str(), .value)]
pub struct NonFiniteWhiteBalanceChannel {
  channel: WbChannel,
  value: f32,
}

impl NonFiniteWhiteBalanceChannel {
  /// Constructs a new `NonFiniteWhiteBalance`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new(channel: WbChannel, value: f32) -> Self {
    Self { channel, value }
  }

  /// Constructs a `NonFiniteWhiteBalance` for the red channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(val: f32) -> Self {
    Self::new(WbChannel::R, val)
  }

  /// Constructs a `NonFiniteWhiteBalance` for the green channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(val: f32) -> Self {
    Self::new(WbChannel::G, val)
  }

  /// Constructs a `NonFiniteWhiteBalance` for the blue channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(val: f32) -> Self {
    Self::new(WbChannel::B, val)
  }

  /// Returns the `channel` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn channel(&self) -> WbChannel {
    self.channel
  }

  /// Returns the `value` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn value(&self) -> f32 {
    self.value
  }
}

/// negative white balance channel
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error("white balance channel {} is negative (got {})", .channel.as_str(), .value)]
pub struct NegativeWhiteBalanceChannel {
  channel: WbChannel,
  value: f32,
}

impl NegativeWhiteBalanceChannel {
  /// Constructs a new `NonFiniteWhiteBalance`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new(channel: WbChannel, value: f32) -> Self {
    Self { channel, value }
  }

  /// Constructs a `NonFiniteWhiteBalance` for the red channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(val: f32) -> Self {
    Self::new(WbChannel::R, val)
  }

  /// Constructs a `NonFiniteWhiteBalance` for the green channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(val: f32) -> Self {
    Self::new(WbChannel::G, val)
  }

  /// Constructs a `NonFiniteWhiteBalance` for the blue channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(val: f32) -> Self {
    Self::new(WbChannel::B, val)
  }

  /// Returns the `channel` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn channel(&self) -> WbChannel {
    self.channel
  }

  /// Returns the `value` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn value(&self) -> f32 {
    self.value
  }
}

/// A gain exceeds [`WhiteBalance::MAX_GAIN`] (`1e6`). The bound
/// is far above any realistic camera value but closes the door
/// on finite-but-pathological metadata that would overflow
/// per-pixel f32 matmul.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error("white balance channel ({} = {value}) exceeds the magnitude bound ({max})", .channel.as_str())]
pub struct WhiteBalanceChannelOutOfBounds {
  channel: WbChannel,
  value: f32,
  max: f32,
}

impl WhiteBalanceChannelOutOfBounds {
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new(channel: WbChannel, value: f32, max: f32) -> Self {
    Self {
      channel,
      value,
      max,
    }
  }

  /// Constructs a `WhiteBalanceChannelOutOfBounds` for the red channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn r(val: f32, max: f32) -> Self {
    Self::new(WbChannel::R, val, max)
  }

  /// Constructs a `WhiteBalanceChannelOutOfBounds` for the green channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn g(val: f32, max: f32) -> Self {
    Self::new(WbChannel::G, val, max)
  }

  /// Constructs a `WhiteBalanceChannelOutOfBounds` for the blue channel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn b(val: f32, max: f32) -> Self {
    Self::new(WbChannel::B, val, max)
  }

  /// Returns the `channel` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn channel(&self) -> WbChannel {
    self.channel
  }

  /// Returns the `value` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn value(&self) -> f32 {
    self.value
  }

  /// Returns the `max` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn max(&self) -> f32 {
    self.max
  }
}

/// Errors returned by [`ColorCorrectionMatrix::try_new`].
#[derive(Debug, Clone, Copy, PartialEq, IsVariant, Unwrap, TryUnwrap, Error)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum ColorCorrectionMatrixError {
  /// An element is non-finite (NaN, +∞, or -∞).
  #[error(transparent)]
  NonFinite(#[from] NonFiniteColorCorrectionMatrixElement),
  /// An element's absolute value exceeds
  /// [`ColorCorrectionMatrix::MAX_COEFFICIENT_ABS`] (`1e6`). The
  /// bound is far above any realistic camera value but closes the
  /// door on finite-but-pathological metadata.
  #[error(transparent)]
  OutOfBounds(#[from] ColorCorrectionMatrixElementOutOfBounds),
}

/// ColorCorrectionMatrix element is non-finite (NaN, +∞, or -∞).
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error("ColorCorrectionMatrix[{row}][{col}] is non-finite (got {value})")]
pub struct NonFiniteColorCorrectionMatrixElement {
  row: usize,
  col: usize,
  value: f32,
}

impl NonFiniteColorCorrectionMatrixElement {
  /// Constructs a new `NonFiniteColorCorrectionMatrixElement`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(row: usize, col: usize, value: f32) -> Self {
    Self { row, col, value }
  }

  /// Returns the `row` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row(&self) -> usize {
    self.row
  }

  /// Returns the `col` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn col(&self) -> usize {
    self.col
  }

  /// Returns the `value` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn value(&self) -> f32 {
    self.value
  }
}

/// An element's absolute value exceeds
/// [`ColorCorrectionMatrix::MAX_COEFFICIENT_ABS`] (`1e6`). The
/// bound is far above any realistic camera value but closes the
/// door on finite-but-pathological metadata.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error(
  "ColorCorrectionMatrix[{row}][{col}] = {value} exceeds the magnitude bound (|coeff| ≤ {max_abs})"
)]
pub struct ColorCorrectionMatrixElementOutOfBounds {
  row: usize,
  col: usize,
  value: f32,
  max_abs: f32,
}

impl ColorCorrectionMatrixElementOutOfBounds {
  /// Constructs a new `ColorCorrectionMatrixElementOutOfBounds`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(row: usize, col: usize, value: f32, max_abs: f32) -> Self {
    Self {
      row,
      col,
      value,
      max_abs,
    }
  }

  /// Returns the `row` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row(&self) -> usize {
    self.row
  }

  /// Returns the `col` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn col(&self) -> usize {
    self.col
  }

  /// Returns the `value` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn value(&self) -> f32 {
    self.value
  }

  /// Returns the `max_abs` field.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn max_abs(&self) -> f32 {
    self.max_abs
  }
}

/// One output row of a Bayer source handed to a [`BayerSink`].
///
/// Carries the three row-aligned slices the demosaic kernel needs,
/// the row index, the pattern, the demosaic algorithm, and the
/// fused 3×3 transform.
///
/// **Boundary contract: mirror-by-2.** At the top edge (row 0) the
/// walker supplies `above = mid_row(1)`, and at the bottom edge
/// (row `h - 1`) it supplies `below = mid_row(h - 2)` — *not* a
/// replicate clamp. This preserves CFA parity across the row
/// boundary because Bayer tiles in 2×2: skipping two rows lands on
/// the same color the missing-tap site would have provided.
/// Falls back to replicate when `height < 2`. Custom sinks must
/// honor this convention; calling a bayer to rgb row
/// from a sink that supplies replicate-clamped row borrows will
/// produce different border pixels than [`super::bayer_to`] does.
///
/// Sinks call into bayer to rgb row (or directly
/// the scalar / SIMD primitive of their choice) with these slices to
/// produce one row of packed RGB output.
#[derive(Debug, Clone, Copy)]
pub struct BayerRow<'a> {
  above: &'a [u8],
  mid: &'a [u8],
  below: &'a [u8],
  row: usize,
  pattern: BayerPattern,
  demosaic: BayerDemosaic,
  m: [[f32; 3]; 3],
}

impl<'a> BayerRow<'a> {
  /// Bundles one row of an 8-bit Bayer source for a [`BayerSink`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    above: &'a [u8],
    mid: &'a [u8],
    below: &'a [u8],
    row: usize,
    pattern: BayerPattern,
    demosaic: BayerDemosaic,
    m: [[f32; 3]; 3],
  ) -> Self {
    Self {
      above,
      mid,
      below,
      row,
      pattern,
      demosaic,
      m,
    }
  }

  /// Row above `mid` per the **mirror-by-2** boundary contract:
  /// for an interior row this is `mid_row(row - 1)`; at the top
  /// edge (`row == 0`) it is `mid_row(1)`. Falls back to `mid` when
  /// `height < 2`. Same length as [`Self::mid`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn above(&self) -> &'a [u8] {
    self.above
  }

  /// The row currently being produced — `width` bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn mid(&self) -> &'a [u8] {
    self.mid
  }

  /// Row below `mid` per the **mirror-by-2** boundary contract:
  /// for an interior row this is `mid_row(row + 1)`; at the bottom
  /// edge (`row == h - 1`) it is `mid_row(h - 2)`. Falls back to
  /// `mid` when `height < 2`. Same length as [`Self::mid`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn below(&self) -> &'a [u8] {
    self.below
  }

  /// Output row index within the frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row(&self) -> usize {
    self.row
  }

  /// Row parity (`row & 1`) — needed by the demosaic kernel to pick
  /// which Bayer site each pixel sits on.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row_parity(&self) -> u32 {
    (self.row & 1) as u32
  }

  /// The Bayer pattern this frame uses.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pattern(&self) -> BayerPattern {
    self.pattern
  }

  /// The demosaic algorithm requested by the caller.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn demosaic(&self) -> BayerDemosaic {
    self.demosaic
  }

  /// Borrow the fused `M = CCM · diag(wb)` transform.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn m(&self) -> &[[f32; 3]; 3] {
    &self.m
  }
}

/// Sinks that consume 8-bit Bayer rows.
///
/// A subtrait of [`PixelSink`] that pins the row shape to
/// [`BayerRow`].
pub trait BayerSink: for<'a> PixelSink<Input<'a> = BayerRow<'a>> {}

/// Walks an 8-bit [`BayerFrame`] row by row, handing each row to the
/// sink along with the precomputed `M = CCM · diag(wb)` transform.
///
/// **Boundary contract.** `above` / `below` use **mirror-by-2** at
/// the top and bottom edges (`row 0 → above = row 1`, `row h-1 →
/// below = row h-2`); see [`BayerRow`] for the full discussion.
///
/// **Allocation profile.** Zero per-row and zero per-frame heap
/// allocation. The walker computes `M` once on the stack at entry,
/// slices three row borrows into the source plane, and hands them
/// to the sink. The sink owns the RGB output buffer.
pub fn bayer_to<S: BayerSink>(
  src: &BayerFrame<'_>,
  pattern: BayerPattern,
  demosaic: BayerDemosaic,
  wb: WhiteBalance,
  ccm: ColorCorrectionMatrix,
  sink: &mut S,
) -> Result<(), S::Error> {
  sink.begin_frame(src.width(), src.height())?;

  let m = fuse_wb_ccm(&wb, &ccm);

  let w = src.width() as usize;
  let h = src.height() as usize;
  let stride = src.stride() as usize;
  let plane = src.data();

  for row in 0..h {
    // **Mirror-by-2** row clamp at the top / bottom edges. See the
    // [`scalar::bayer_to_rgb_row`] kernel docs for the rationale
    // (preserves CFA parity across the boundary; replicate clamp
    // would mix wrong-color samples into the missing-channel
    // averages). Falls back to replicate when `h < 2`.
    let above_row = if row == 0 {
      if h >= 2 { 1 } else { 0 }
    } else {
      row - 1
    };
    let below_row = if row + 1 == h {
      if h >= 2 { h - 2 } else { h - 1 }
    } else {
      row + 1
    };

    let above = &plane[above_row * stride..above_row * stride + w];
    let mid = &plane[row * stride..row * stride + w];
    let below = &plane[below_row * stride..below_row * stride + w];

    sink.process(BayerRow::new(above, mid, below, row, pattern, demosaic, m))?;
  }
  Ok(())
}

/// Internal: fuse white-balance and CCM into a single 3×3 transform
/// `M = CCM · diag(wb)`. The walker calls this once per frame; the
/// per-row kernel applies a single 3×3 matmul per pixel.
#[cfg_attr(not(tarpaulin), inline(always))]
pub const fn fuse_wb_ccm(wb: &WhiteBalance, ccm: &ColorCorrectionMatrix) -> [[f32; 3]; 3] {
  let m = ccm.as_array();
  let (wr, wg, wb_) = (wb.r(), wb.g(), wb.b());
  [
    [m[0][0] * wr, m[0][1] * wg, m[0][2] * wb_],
    [m[1][0] * wr, m[1][1] * wg, m[1][2] * wb_],
    [m[2][0] * wr, m[2][1] * wg, m[2][2] * wb_],
  ]
}

/// One output row of a high-bit-depth Bayer source handed to a
/// [`BayerSink16<BITS>`].
///
/// Carries `&[u16]` slices for `above` / `mid` / `below`, the row
/// index, the pattern, the demosaic algorithm, and the **unscaled**
/// fused `M = CCM · diag(wb)` 3×3. Output-bit-depth scaling
/// (multiply by `255 / ((1 << BITS) - 1)` for u8 output; identity
/// for low-packed u16 output) is the kernel's job.
///
/// **Boundary contract: mirror-by-2** — see [`super::BayerRow`]
/// for the full discussion. Top edge supplies `above = mid_row(1)`,
/// bottom edge supplies `below = mid_row(h - 2)`; replicate
/// fallback applies only when `height < 2`. Custom sinks must
/// honor this convention.
#[derive(Debug, Clone, Copy)]
pub struct BayerRow16<'a, const BITS: u32> {
  above: &'a [u16],
  mid: &'a [u16],
  below: &'a [u16],
  row: usize,
  pattern: BayerPattern,
  demosaic: BayerDemosaic,
  m: [[f32; 3]; 3],
}

impl<'a, const BITS: u32> BayerRow16<'a, BITS> {
  /// Bundles one row of a high-bit-depth Bayer source for a
  /// [`BayerSink16<BITS>`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub const fn new(
    above: &'a [u16],
    mid: &'a [u16],
    below: &'a [u16],
    row: usize,
    pattern: BayerPattern,
    demosaic: BayerDemosaic,
    m: [[f32; 3]; 3],
  ) -> Self {
    Self {
      above,
      mid,
      below,
      row,
      pattern,
      demosaic,
      m,
    }
  }

  /// Row above `mid` per the **mirror-by-2** boundary contract:
  /// `mid_row(row - 1)` for interior rows; `mid_row(1)` at the top
  /// edge. See [`super::BayerRow::above`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn above(&self) -> &'a [u16] {
    self.above
  }

  /// The row currently being produced — `width` `u16` samples.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn mid(&self) -> &'a [u16] {
    self.mid
  }

  /// Row below `mid` per the **mirror-by-2** boundary contract:
  /// `mid_row(row + 1)` for interior rows; `mid_row(h - 2)` at the
  /// bottom edge. See [`super::BayerRow::below`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn below(&self) -> &'a [u16] {
    self.below
  }

  /// Output row index within the frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row(&self) -> usize {
    self.row
  }

  /// Row parity (`row & 1`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row_parity(&self) -> u32 {
    (self.row & 1) as u32
  }

  /// The Bayer pattern this frame uses.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pattern(&self) -> BayerPattern {
    self.pattern
  }

  /// The demosaic algorithm requested by the caller.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn demosaic(&self) -> BayerDemosaic {
    self.demosaic
  }

  /// Borrow the fused `M = CCM · diag(wb)` transform. Unscaled —
  /// kernels apply the input/output bit-depth scaling themselves.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn m(&self) -> &[[f32; 3]; 3] {
    &self.m
  }

  /// Active bit depth — 10, 12, 14, or 16.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bits(&self) -> u32 {
    BITS
  }
}

/// Sinks that consume high-bit-depth Bayer rows at a fixed `BITS`.
///
/// The `<const BE: bool>` parameter encodes the source plane byte
/// order, mirroring `Y216Sink<BE>`: a sink typically impls for the one
/// `BE` matching its kernel monomorphization. The [`BayerRow16`] type
/// itself does **not** carry `BE` — Row is just borrowed `u16` samples;
/// the BE-aware byte-swap happens inside the kernel dispatch keyed off
/// the sink's `<const BE>` parameter. `BE` defaults to `false` (LE) so
/// existing LE-only custom sinks keep writing
/// `impl BayerSink16<BITS> for MySink` / `S: BayerSink16<BITS>` without
/// migrating to an explicit const argument.
pub trait BayerSink16<const BITS: u32, const BE: bool = false>:
  for<'a> PixelSink<Input<'a> = BayerRow16<'a, BITS>>
{
}

/// Walks a [`BayerFrame16<BITS, BE>`] row by row, handing each row to
/// the sink along with the precomputed `M = CCM · diag(wb)` 3×3,
/// propagating the frame's `<const BE>` byte order into the sink
/// dispatch.
///
/// This is the byte-order-generic walker, mirroring `y216_to_endian`:
/// it accepts both LE (`BE = false`) and BE (`BE = true`) frames. The
/// [`bayer16_to`] wrapper forwards here with `BE = false` and preserves
/// the pre-endian `<const BITS, S>` public signature for source compat.
///
/// The `BE` flag rides on the frame and the [`BayerSink16<BITS, BE>`]
/// bound; the row slices are handed through untouched (the [`BayerRow16`]
/// type does not carry `BE`). The actual byte-swap of the `u16` samples
/// happens inside the sink's kernel dispatch, keyed off `BE` — exactly
/// as the Y2xx family threads `BE` from `Y2xxFrame<'_, BITS, BE>` into
/// `Y216Sink<BE>`.
///
/// **Fully fallible.** The walker performs no data-dependent
/// validation — every panic surface that previously existed has
/// been moved to [`BayerFrame16::try_new`], which validates
/// dimensions *and* every active sample's range at construction.
/// Once you hold a `BayerFrame16<BITS, BE>`, the conversion can only
/// fail through `S::Error` (sink-side I/O, geometry-mismatch,
/// etc.); bad sample data is reported as
/// [`crate::frame::BayerFrame16Error::SampleOutOfRange`] from the
/// frame constructor instead of as a runtime panic here.
///
/// **Allocation profile.** Zero per-row and zero per-frame heap
/// allocation, identical to the 8-bit [`super::bayer_to`].
pub fn bayer16_to_endian<const BITS: u32, const BE: bool, S>(
  src: &BayerFrame16<'_, BITS, BE>,
  pattern: BayerPattern,
  demosaic: BayerDemosaic,
  wb: WhiteBalance,
  ccm: ColorCorrectionMatrix,
  sink: &mut S,
) -> Result<(), S::Error>
where
  S: BayerSink16<BITS, BE>,
{
  let w = src.width() as usize;
  let h = src.height() as usize;
  let stride = src.stride() as usize;
  let plane = src.data();

  sink.begin_frame(src.width(), src.height())?;

  let m = fuse_wb_ccm(&wb, &ccm);

  for row in 0..h {
    // Mirror-by-2 row clamp; see [`super::bayer::bayer_to`] for
    // the rationale (CFA-parity preservation at boundaries).
    let above_row = if row == 0 {
      if h >= 2 { 1 } else { 0 }
    } else {
      row - 1
    };
    let below_row = if row + 1 == h {
      if h >= 2 { h - 2 } else { h - 1 }
    } else {
      row + 1
    };

    let above = &plane[above_row * stride..above_row * stride + w];
    let mid = &plane[row * stride..row * stride + w];
    let below = &plane[below_row * stride..below_row * stride + w];

    sink.process(BayerRow16::<BITS>::new(
      above, mid, below, row, pattern, demosaic, m,
    ))?;
  }
  Ok(())
}

/// LE-only back-compat wrapper over [`bayer16_to_endian`] preserving
/// the pre-endian `bayer16_to::<BITS, S>` walker signature. Forwards to
/// the byte-order-generic helper with `BE = false`.
///
/// Rust forbids defaults on function-position const-generic parameters,
/// so an explicit-turbofish caller written before the endian extension
/// (`bayer16_to::<12, MySink>(...)`) would otherwise fail to compile.
/// Keeping this two-generic wrapper preserves source compatibility for
/// those call sites; BE-aware callers should use [`bayer16_to_endian`]
/// directly. Mirrors the `y216_to` → `y216_to_endian` pairing.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn bayer16_to<const BITS: u32, S: BayerSink16<BITS, false>>(
  src: &BayerFrame16<'_, BITS, false>,
  pattern: BayerPattern,
  demosaic: BayerDemosaic,
  wb: WhiteBalance,
  ccm: ColorCorrectionMatrix,
  sink: &mut S,
) -> Result<(), S::Error> {
  bayer16_to_endian::<BITS, false, S>(src, pattern, demosaic, wb, ccm, sink)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn variants_construct_and_compare() {
    assert_eq!(BayerPattern::Bggr, BayerPattern::Bggr);
    assert_ne!(BayerPattern::Bggr, BayerPattern::Rggb);
  }

  #[test]
  fn is_variant_helpers_work() {
    assert!(BayerPattern::Bggr.is_bggr());
    assert!(!BayerPattern::Bggr.is_rggb());
  }

  #[cfg(feature = "std")]
  #[test]
  fn copy_and_hash() {
    use std::{
      collections::hash_map::DefaultHasher,
      hash::{Hash, Hasher},
    };
    let p = BayerPattern::Grbg;
    let _copy = p; // doesn't move
    let mut h = DefaultHasher::new();
    p.hash(&mut h);
    let _ = h.finish();
  }

  #[cfg(feature = "std")]
  #[test]
  fn as_str_matches_display() {
    use std::format;
    for v in [
      BayerPattern::Bggr,
      BayerPattern::Rggb,
      BayerPattern::Grbg,
      BayerPattern::Gbrg,
    ] {
      assert_eq!(v.as_str(), format!("{v}"));
    }
    assert_eq!(BayerPattern::Bggr.as_str(), "bggr");
  }
}
