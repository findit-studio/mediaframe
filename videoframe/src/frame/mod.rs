//! Frame primitives + the typed source-format `*Frame<'a, BE>` borrow types.
//!
//! ## Always-available primitives
//!
//! - [`Dimensions`] — a `(width, height)` pair in pixels.
//! - [`Rect`] — an axis-aligned integer rectangle (used for visible-region
//!   crops on `VideoFrame`).
//! - [`Rotation`] — display rotation (0 / 90 / 180 / 270).
//! - [`SampleAspectRatio`] — pixel aspect ratio (SAR).
//! - [`Plane<B>`] — one plane of pixel data, generic over the buffer type.
//! - [`VideoFrame<P, B>`] — runtime-tagged frame (no timestamp).
//! - [`TimestampedFrame<F>`] — orthogonal time-carrying wrapper.
//!
//! ## Typed `*Frame<'a, BE>` borrow types (feature-gated)
//!
//! Each pixel-format family is gated behind its own feature flag so
//! consumers compile only the formats they need. Enable an individual
//! family (e.g. `yuv-planar`) or the `frame` umbrella to opt in.
//!
//! | Feature           | Formats                                              |
//! |-------------------|------------------------------------------------------|
//! | `yuv-planar`      | Yuv420p / 422p / 444p / 440p / 411p / 410p + 9-16bit |
//! | `yuv-semi-planar` | NV12 / 16 / 21 / 24 / 42, P010 / 210 / 410 families  |
//! | `yuva`            | YUVA planar 8-bit + high-bit                         |
//! | `yuv-packed`      | YUYV422, UYVY422, YVYU422, UYYVYY411                 |
//! | `yuv-444-packed`  | V410, XV30, XV36, AYUV64, VUYA, VUYX, V30X           |
//! | `y2xx`            | Y210 / Y212 / Y216                                   |
//! | `v210`            | V210                                                 |
//! | `rgb`             | Rgb24/Bgr24/Rgba/Bgra + 16-bit family                |
//! | `rgb-float`       | Rgbf32 / Rgbf16                                      |
//! | `rgb-legacy`      | Rgb444/555/565 + Bgr counterparts                    |
//! | `gbr`             | Gbrp / Gbrap + 9-16bit + float                       |
//! | `gray`            | Gray8-16, Grayf32, Ya8/16                            |
//! | `bayer`           | Bayer 8-16bit, 4 patterns                            |
//! | `xyz`             | Xyz12                                                |
//! | `mono`            | Monoblack / Monowhite / Pal8                         |
//! | `frame`           | umbrella — enables every sub-feature above           |

// === Primitives (always available) ===

// ---- Shared error payload structs (used by per-family `*FrameError` enums) ----
//
// Variant names carry the per-plane / per-axis semantics
// (`InsufficientYStride`, `InsufficientUPlane`, …); the payload carries the
// shape-only data (the offending number + the reference number).
// Each payload has:
//   - private fields,
//   - a `pub const fn new(...)` constructor,
//   - one `pub const fn field(&self) -> T` getter per field,
//   - `#[inline]` on all methods.
// thiserror `#[error("...", .0.field())]` routes Display lookups
// through the getters so the original messages are preserved
// verbatim.

/// `width × height` carried by zero-dimension errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) or height ({height}) is zero")]
pub struct ZeroDimension {
  width: u32,
  height: u32,
}

impl ZeroDimension {
  /// Constructs a `ZeroDimension` payload.
  #[inline]
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
  /// Returns the supplied width.
  #[inline]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Returns the supplied height.
  #[inline]
  pub const fn height(&self) -> u32 {
    self.height
  }
}

/// `width × height` carried by dimension-overflow errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("dimensions {width} × {height} overflow")]
pub struct DimensionOverflow {
  width: u32,
  height: u32,
}

impl DimensionOverflow {
  /// Constructs a `DimensionOverflow` payload.
  #[inline]
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
  /// Returns the supplied width.
  #[inline]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Returns the supplied height.
  #[inline]
  pub const fn height(&self) -> u32 {
    self.height
  }
}

/// Plane stride is smaller than what the declared geometry requires.
/// The variant name (e.g. `InsufficientYStride` vs `InsufficientUvStride`)
/// tells the caller which plane and what unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("stride ({stride}) is smaller than minimum ({min})")]
pub struct InsufficientStride {
  stride: u32,
  min: u32,
}

impl InsufficientStride {
  /// Constructs a `InsufficientStride` payload.
  #[inline]
  pub const fn new(stride: u32, min: u32) -> Self {
    Self { stride, min }
  }
  /// Returns the caller-supplied stride.
  #[inline]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the required minimum.
  #[inline]
  pub const fn min(&self) -> u32 {
    self.min
  }
}

/// Plane buffer is shorter than the declared geometry requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("plane has {actual} bytes/samples but at least {expected} are required")]
pub struct InsufficientPlane {
  expected: usize,
  actual: usize,
}

impl InsufficientPlane {
  /// Constructs a `InsufficientPlane` payload.
  #[inline]
  pub const fn new(expected: usize, actual: usize) -> Self {
    Self { expected, actual }
  }
  /// Returns the minimum required length.
  #[inline]
  pub const fn expected(&self) -> usize {
    self.expected
  }
  /// Returns the actual length supplied.
  #[inline]
  pub const fn actual(&self) -> usize {
    self.actual
  }
}

/// Declared geometry (`stride × rows`) doesn't fit in `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("declared geometry overflows usize: stride={stride} * rows={rows}")]
pub struct GeometryOverflow {
  stride: u32,
  rows: u32,
}

impl GeometryOverflow {
  /// Constructs a `GeometryOverflow` payload.
  #[inline]
  pub const fn new(stride: u32, rows: u32) -> Self {
    Self { stride, rows }
  }
  /// Returns the stride that overflowed.
  #[inline]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the row count that overflowed.
  #[inline]
  pub const fn rows(&self) -> u32 {
    self.rows
  }
}

/// Width-alignment violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) {required}")]
pub struct WidthAlignment {
  /// Sink's configured width.
  width: usize,
  /// The alignment requirement that was violated.
  required: WidthAlignmentRequirement,
}

impl WidthAlignment {
  /// Constructs a new `WidthAlignment` payload.
  #[inline]
  const fn new(width: usize, required: WidthAlignmentRequirement) -> Self {
    Self { width, required }
  }

  /// Constructs a `WidthAlignment` payload for odd widths.
  #[inline]
  pub const fn odd(width: usize) -> Self {
    Self::new(width, WidthAlignmentRequirement::Even)
  }

  /// Constructs a `WidthAlignment` payload for widths that are not a
  #[inline]
  pub const fn multiple_of_four(width: usize) -> Self {
    Self::new(width, WidthAlignmentRequirement::MultipleOfFour)
  }

  /// Sink's configured width.
  #[inline]
  pub const fn width(&self) -> usize {
    self.width
  }

  /// The alignment requirement that was violated.
  #[inline]
  pub const fn required(&self) -> WidthAlignmentRequirement {
    self.required
  }
}

/// Discriminates which width-alignment rule was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Display)]
#[non_exhaustive]
pub enum WidthAlignmentRequirement {
  /// Width must be even — 4:2:0 / 4:2:2 chroma-pair stride.
  #[display("is odd")]
  Even,
  /// Width must be a multiple of 4. Fired by planar 4:1:0
  /// ([`Yuv410p`](crate::source::Yuv410p)) and packed 4:1:1
  /// ([`Uyyvyy411`](crate::source::Uyyvyy411)). Note: planar 4:1:1
  /// ([`Yuv411p`](crate::source::Yuv411p)) accepts non-4-aligned
  /// widths via `width.div_ceil(4)` for the chroma row and is NOT
  /// covered by this discriminant.
  #[display("is not a multiple of 4")]
  MultipleOfFour,
}

/// Frame `width` value carried by per-row width-overflow errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) overflow")]
pub struct WidthOverflow {
  width: u32,
}

impl WidthOverflow {
  /// Constructs a `WidthOverflow` payload.
  #[inline]
  pub const fn new(width: u32) -> Self {
    Self { width }
  }
  /// Returns the supplied width.
  #[inline]
  pub const fn width(&self) -> u32 {
    self.width
  }
}

/// `BITS` const-generic value carried by unsupported-bits errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("unsupported BITS ({bits})")]
pub struct UnsupportedBits {
  bits: u32,
}

impl UnsupportedBits {
  /// Constructs an `UnsupportedBits` payload.
  #[inline]
  pub const fn new(bits: u32) -> Self {
    Self { bits }
  }
  /// Returns the supplied `BITS` value.
  #[inline]
  pub const fn bits(&self) -> u32 {
    self.bits
  }
}

/// A `(width, height)` pair in pixels.
///
/// Lives alongside the rest of the frame primitives because the same
/// pair shows up everywhere a video stream is described — the coded
/// dimensions of a `VideoFrame`, the `coded_*` parameters a backend
/// adapter takes when opening a decoder, the per-plane layout helpers
/// in a WebCodecs adapter, etc. Passing it as a single struct rather
/// than two separate `u32` arguments removes a long-running footgun
/// (silent argument swap) and gives a natural place to hang helpers
/// like [`Self::is_zero`] or `Display`.
///
/// `u32` width / height matches WebCodecs' `coded_width` /
/// `coded_height` typing in `web_sys` and FFmpeg's
/// `AVCodecContext::width` / `height`. 65535×65535 (the smaller `u16`
/// packing some adjacent crates use) covers every realistic
/// resolution; the `u32` choice here keeps the public API plug-
/// compatible with both adapter typings.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dimensions {
  width: u32,
  height: u32,
}

impl Dimensions {
  /// Constructs a `Dimensions` with the specified width and height
  /// in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }

  /// Returns the width in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Sets the width (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }

  /// Sets the width in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_width(&mut self, width: u32) -> &mut Self {
    self.width = width;
    self
  }

  /// Sets the height (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  /// Sets the height in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_height(&mut self, height: u32) -> &mut Self {
    self.height = height;
    self
  }

  /// Returns `true` when both width and height are zero — typically
  /// the default-constructed / unset state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_zero(&self) -> bool {
    self.width == 0 && self.height == 0
  }
}

impl core::fmt::Display for Dimensions {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}x{}", self.width, self.height)
  }
}

/// An axis-aligned integer rectangle.
///
/// Used for `VideoFrame::visible_rect` (FFmpeg crop /
/// WebCodecs `visibleRect` / ProRes RAW `CleanAperture`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
  x: u32,
  y: u32,
  width: u32,
  height: u32,
}

impl Rect {
  /// Constructs a `Rect` at `(x, y)` with the given size.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }

  /// Returns the X coordinate of the top-left corner.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn x(&self) -> u32 {
    self.x
  }

  /// Returns the Y coordinate of the top-left corner.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> u32 {
    self.y
  }

  /// Returns the width.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Sets the X coordinate (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_x(mut self, x: u32) -> Self {
    self.x = x;
    self
  }
  /// Sets the Y coordinate (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_y(mut self, y: u32) -> Self {
    self.y = y;
    self
  }
  /// Sets the width (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_width(mut self, w: u32) -> Self {
    self.width = w;
    self
  }
  /// Sets the height (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_height(mut self, h: u32) -> Self {
    self.height = h;
    self
  }

  /// Sets the X coordinate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_x(&mut self, x: u32) -> &mut Self {
    self.x = x;
    self
  }
  /// Sets the Y coordinate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_y(&mut self, y: u32) -> &mut Self {
    self.y = y;
    self
  }
  /// Sets the width in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_width(&mut self, w: u32) -> &mut Self {
    self.width = w;
    self
  }
  /// Sets the height in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_height(&mut self, h: u32) -> &mut Self {
    self.height = h;
    self
  }
}

/// Display rotation applied to the decoded picture before presentation.
///
/// Read from the FFmpeg display matrix side data
/// (`AV_FRAME_DATA_DISPLAYMATRIX` → `av_display_rotation_get`, which
/// returns a counter-clockwise angle in degrees) and from the
/// WebCodecs `VideoFrame` rotation attribute. Only the four
/// axis-aligned multiples of 90° are representable — every container
/// rotation tag in practice is one of these. Any other / future /
/// corrupt wire value is preserved verbatim as [`Self::Unknown`]
/// rather than silently collapsed to a valid rotation (mirrors the
/// lossless `Unknown(u32)` convention of the colour enums).
///
/// The angle is the **clockwise** rotation to apply for display
/// (matching WebCodecs' `rotation`); callers normalising FFmpeg's
/// counter-clockwise convention negate accordingly. [`Self::D0`] is
/// the default (no rotation / square presentation).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum Rotation {
  /// Unknown / unrecognised rotation wire value. The wrapped `u32`
  /// is the original value passed to [`Self::from_u32`] — preserved
  /// so the round-trip is lossless (no silent collapse to `D0`).
  Unknown(u32),
  /// No rotation.
  #[default]
  D0,
  /// 90° clockwise.
  D90,
  /// 180°.
  D180,
  /// 270° clockwise (= 90° counter-clockwise).
  D270,
}

impl Rotation {
  /// Degree string for this rotation (`"0"` / `"90"` / `"180"` /
  /// `"270"`); [`Self::Unknown`] renders as `"unknown"`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::D0 => "0",
      Self::D90 => "90",
      Self::D180 => "180",
      Self::D270 => "270",
    }
  }

  /// Stable `u32` wire id: `0`/`1`/`2`/`3` for
  /// `D0`/`D90`/`D180`/`D270`; [`Self::Unknown`] carries its
  /// original value through unchanged so `from_u32(to_u32(x)) == x`
  /// for every unrecognised `x`. Stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::D0 => 0,
      Self::D90 => 1,
      Self::D180 => 2,
      Self::D270 => 3,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values are preserved as
  /// [`Self::Unknown`] (lossless) rather than mapped to a default.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::D0,
      1 => Self::D90,
      2 => Self::D180,
      3 => Self::D270,
      _ => Self::Unknown(v),
    }
  }
}

/// Pixel (sample) aspect ratio — the ratio of a pixel's display
/// width to its display height.
///
/// Read from `AVStream.sample_aspect_ratio` /
/// `AVFrame.sample_aspect_ratio` (an FFmpeg `AVRational`) and from
/// the WebCodecs display-size derivation. A `0:1` numerator in
/// FFmpeg means "unknown"; callers normalise that to the `1:1`
/// default (square pixels) before constructing this type.
///
/// `den` is a [`core::num::NonZeroU32`] so a SAR can never have a
/// zero denominator; the manual [`Default`] is `1:1` (square),
/// mirroring `mediatime::Timebase`'s non-proto-zero default.
///
/// Represented as a newtype over [`Rational`] — the single source of
/// truth for "exact ratio with a non-zero denominator". The fields
/// are private; the entire public method API (and the `buffa` wire
/// format) is unchanged, delegating to the inner `Rational`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleAspectRatio(Rational);

impl Default for SampleAspectRatio {
  /// `1:1` — square pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self(Rational::default())
  }
}

impl SampleAspectRatio {
  /// Constructs a `SampleAspectRatio` from an explicit
  /// numerator / (non-zero) denominator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(num: u32, den: core::num::NonZeroU32) -> Self {
    Self(Rational::new(num, den))
  }

  /// Returns the numerator (display-width units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn num(&self) -> u32 {
    self.0.num()
  }

  /// Returns the (non-zero) denominator (display-height units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn den(&self) -> core::num::NonZeroU32 {
    self.0.den()
  }

  /// `true` when the pixels are square (`num == den`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_square(&self) -> bool {
    self.0.num() == self.0.den().get()
  }

  /// Returns this SAR as a generic [`Rational`] — the underlying
  /// representation. Purely additive interop; `SampleAspectRatio`'s
  /// public method API is unchanged.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rational(&self) -> Rational {
    self.0
  }

  /// Alias of [`Self::rational`] — views this SAR as a generic
  /// [`Rational`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_rational(&self) -> Rational {
    self.rational()
  }

  /// Sets the numerator (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_num(mut self, num: u32) -> Self {
    self.0 = self.0.with_num(num);
    self
  }

  /// Sets the denominator (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_den(mut self, den: core::num::NonZeroU32) -> Self {
    self.0 = self.0.with_den(den);
    self
  }

  /// Sets the numerator in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_num(&mut self, num: u32) -> &mut Self {
    self.0.set_num(num);
    self
  }

  /// Sets the denominator in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_den(&mut self, den: core::num::NonZeroU32) -> &mut Self {
    self.0.set_den(den);
    self
  }
}

impl core::fmt::Display for SampleAspectRatio {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}:{}", self.0.num(), self.0.den())
  }
}

impl From<SampleAspectRatio> for Rational {
  /// Unwraps the inner [`Rational`] — `SampleAspectRatio` is a newtype
  /// over `Rational`. Additive interop; `SampleAspectRatio`'s own
  /// public method API is unchanged.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(sar: SampleAspectRatio) -> Self {
    sar.0
  }
}

impl From<Rational> for SampleAspectRatio {
  /// Wraps a generic [`Rational`] as a pixel/sample aspect ratio.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(rate: Rational) -> Self {
    Self(rate)
  }
}

/// A generic exact ratio `num / den`.
///
/// The reusable rational primitive the rest of the frame layer builds
/// on (e.g. [`FrameRate`]). `den` is a [`core::num::NonZeroU32`] so a
/// ratio can never have a zero denominator; the manual [`Default`] is
/// `1/1` (the multiplicative identity), mirroring
/// [`SampleAspectRatio`]'s non-proto-zero default and
/// `mediatime::Timebase`'s convention.
///
/// This is the format-agnostic numerator/denominator pair; semantic
/// wrappers ([`SampleAspectRatio`] for pixel aspect, [`FrameRate`] for
/// frames-per-second) carry the domain meaning. A `0` numerator is a
/// valid representable state (e.g. an "unknown" FFmpeg `AVRational`
/// `0/1`) — see [`Self::is_zero`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
  num: u32,
  den: core::num::NonZeroU32,
}

impl Default for Rational {
  /// `1/1` — the multiplicative identity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      num: 1,
      den: core::num::NonZeroU32::MIN,
    }
  }
}

impl Rational {
  /// Constructs a `Rational` from an explicit
  /// numerator / (non-zero) denominator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(num: u32, den: core::num::NonZeroU32) -> Self {
    Self { num, den }
  }

  /// Returns the numerator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn num(&self) -> u32 {
    self.num
  }

  /// Returns the (non-zero) denominator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn den(&self) -> core::num::NonZeroU32 {
    self.den
  }

  /// `true` when the numerator is `0` (the ratio is exactly zero —
  /// e.g. an "unknown" `0/1` FFmpeg `AVRational`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_zero(&self) -> bool {
    self.num == 0
  }

  /// Sets the numerator (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_num(mut self, num: u32) -> Self {
    self.num = num;
    self
  }

  /// Sets the denominator (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_den(mut self, den: core::num::NonZeroU32) -> Self {
    self.den = den;
    self
  }

  /// Sets the numerator in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_num(&mut self, num: u32) -> &mut Self {
    self.num = num;
    self
  }

  /// Sets the denominator in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_den(&mut self, den: core::num::NonZeroU32) -> &mut Self {
    self.den = den;
    self
  }
}

impl core::fmt::Display for Rational {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}/{}", self.num, self.den)
  }
}

/// The frame rate of a video stream as an exact [`Rational`]
/// (frames per second) plus a variable-frame-rate marker.
///
/// `rate` is the nominal frames-per-second ratio (e.g. `30000/1001`
/// for NTSC, `25/1` for PAL). `is_vfr` records that the stream is
/// variable-frame-rate, in which case `rate` is the average / nominal
/// rate only and per-frame timing must be taken from the timestamps.
///
/// This is deliberately **not** [`mediatime::Timebase`]: a frame rate
/// is *not* a presentation-timestamp timebase. They are reciprocal-ish
/// but distinct concepts (a 30000/1001 fps stream is commonly carried
/// on a 1/90000 or 1/1000 PTS timebase) — `mediatime` documents that
/// distinction and intentionally models only the PTS timebase, so the
/// frame-rate concept lives here as its own type.
///
/// The [`Default`] is `{ rate: Rational::default() (1/1),
/// is_vfr: false }`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
  rate: Rational,
  is_vfr: bool,
}

impl FrameRate {
  /// Constructs a `FrameRate` from an exact frames-per-second
  /// [`Rational`] and a variable-frame-rate flag.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rate: Rational, is_vfr: bool) -> Self {
    Self { rate, is_vfr }
  }

  /// Returns the nominal frames-per-second ratio.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rate(&self) -> Rational {
    self.rate
  }

  /// `true` when the stream is variable-frame-rate (the [`Self::rate`]
  /// is then an average / nominal value only).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_vfr(&self) -> bool {
    self.is_vfr
  }

  /// Sets the rate (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_rate(mut self, rate: Rational) -> Self {
    self.rate = rate;
    self
  }

  /// Sets the VFR flag (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_is_vfr(mut self, is_vfr: bool) -> Self {
    self.is_vfr = is_vfr;
    self
  }

  /// Sets the rate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_rate(&mut self, rate: Rational) -> &mut Self {
    self.rate = rate;
    self
  }

  /// Sets the VFR flag in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_is_vfr(&mut self, is_vfr: bool) -> &mut Self {
    self.is_vfr = is_vfr;
    self
  }
}

/// Interlacing / field order of a video stream.
///
/// Mirrors FFmpeg `AVFieldOrder`
/// (`AVCodecContext::field_order` / `AVFrame` derived state) with the
/// exact numeric code points: `AV_FIELD_UNKNOWN = 0`,
/// `AV_FIELD_PROGRESSIVE = 1`, `AV_FIELD_TT = 2`,
/// `AV_FIELD_BB = 3`, `AV_FIELD_TB = 4`, `AV_FIELD_BT = 5`. Any
/// other / future / corrupt wire value is preserved verbatim as
/// [`Self::Unknown`] rather than collapsed (mirrors the lossless
/// `Unknown(u32)` convention of [`Rotation`] / the colour enums).
///
/// FFmpeg's own `AV_FIELD_UNKNOWN` sentinel is code `0`, so the
/// [`Default`] is `Unknown(0)` — the same default-is-`Unknown(0)`
/// precedent as [`PixelFormat`](crate::pixel_format::PixelFormat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum FieldOrder {
  /// Unknown / unrecognised field-order wire value. The wrapped
  /// `u32` is the original value passed to [`Self::from_u32`] —
  /// preserved so the round-trip is lossless. Also the [`Default`]
  /// (`Unknown(0)`), since FFmpeg's `AV_FIELD_UNKNOWN` is code `0`.
  Unknown(u32),
  /// Progressive (not interlaced) — `AV_FIELD_PROGRESSIVE`.
  Progressive,
  /// Top coded first, top displayed first — `AV_FIELD_TT`.
  Tt,
  /// Bottom coded first, bottom displayed first — `AV_FIELD_BB`.
  Bb,
  /// Top coded first, bottom displayed first — `AV_FIELD_TB`.
  Tb,
  /// Bottom coded first, top displayed first — `AV_FIELD_BT`.
  Bt,
}

impl Default for FieldOrder {
  /// `Unknown(0)` — FFmpeg's `AV_FIELD_UNKNOWN` is code `0`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Unknown(0)
  }
}

impl FieldOrder {
  /// Lowercase slug for this field order (`"progressive"` / `"tt"` /
  /// `"bb"` / `"tb"` / `"bt"`); [`Self::Unknown`] renders as
  /// `"unknown"`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::Progressive => "progressive",
      Self::Tt => "tt",
      Self::Bb => "bb",
      Self::Tb => "tb",
      Self::Bt => "bt",
    }
  }

  /// Stable `u32` wire id = the FFmpeg `AVFieldOrder` code
  /// (`Unknown`→its carried value, `Progressive`=1, `Tt`=2, `Bb`=3,
  /// `Tb`=4, `Bt`=5). [`Self::Unknown`] carries its original value
  /// through unchanged so `from_u32(to_u32(x)) == x` for every
  /// unrecognised `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Progressive => 1,
      Self::Tt => 2,
      Self::Bb => 3,
      Self::Tb => 4,
      Self::Bt => 5,
    }
  }

  /// Decodes from the FFmpeg `AVFieldOrder` code produced by
  /// [`Self::to_u32`]. The canonical `AV_FIELD_UNKNOWN` code `0`
  /// (and any other unrecognised id) maps to [`Self::Unknown`]
  /// carrying the original value, so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Progressive,
      2 => Self::Tt,
      3 => Self::Bb,
      4 => Self::Tb,
      5 => Self::Bt,
      _ => Self::Unknown(v),
    }
  }
}

/// Stereoscopic-3D packing mode of a video stream.
///
/// Mirrors FFmpeg `AVStereo3DType` (the `AV_FRAME_DATA_STEREO3D`
/// side-data `type`) with the exact numeric code points:
/// `AV_STEREO3D_2D = 0` (named [`Self::Mono`]),
/// `AV_STEREO3D_SIDEBYSIDE = 1`, `AV_STEREO3D_TOPBOTTOM = 2`,
/// `AV_STEREO3D_FRAMESEQUENCE = 3`, `AV_STEREO3D_CHECKERBOARD = 4`,
/// `AV_STEREO3D_SIDEBYSIDE_QUINCUNX = 5`, `AV_STEREO3D_LINES = 6`,
/// `AV_STEREO3D_COLUMNS = 7`. Any other / future / corrupt wire
/// value is preserved verbatim as [`Self::Unknown`] (lossless
/// `Unknown(u32)` convention shared with [`Rotation`] / the colour
/// enums).
///
/// The [`Default`] is [`Self::Mono`] — a *real* code (value `0`,
/// FFmpeg `AV_STEREO3D_2D`, plain monoscopic video), so the default
/// is a named variant rather than `Unknown(0)` (the colour-enum
/// named-default precedent, e.g. `DcpTargetGamut::DciP3`), distinct
/// from [`FieldOrder`] whose `0` *is* FFmpeg's UNKNOWN sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum StereoMode {
  /// Unknown / unrecognised wire value. The wrapped `u32` is the
  /// original value passed to [`Self::from_u32`] — preserved so the
  /// round-trip is lossless.
  Unknown(u32),
  /// Plain monoscopic (non-stereo) video — `AV_STEREO3D_2D` (code
  /// `0`). The [`Default`].
  Mono,
  /// Side-by-side — `AV_STEREO3D_SIDEBYSIDE`.
  SideBySide,
  /// Top-bottom — `AV_STEREO3D_TOPBOTTOM`.
  TopBottom,
  /// Frame-sequential — `AV_STEREO3D_FRAMESEQUENCE`.
  FrameSequence,
  /// Checkerboard — `AV_STEREO3D_CHECKERBOARD`.
  Checkerboard,
  /// Side-by-side quincunx — `AV_STEREO3D_SIDEBYSIDE_QUINCUNX`.
  SideBySideQuincunx,
  /// Interleaved by rows — `AV_STEREO3D_LINES`.
  Lines,
  /// Interleaved by columns — `AV_STEREO3D_COLUMNS`.
  Columns,
}

impl Default for StereoMode {
  /// [`Self::Mono`] — FFmpeg `AV_STEREO3D_2D` (code `0`), plain
  /// monoscopic video. A named variant (not `Unknown(0)`), the
  /// colour-enum named-default precedent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Mono
  }
}

impl StereoMode {
  /// Lowercase slug for this stereo mode; [`Self::Unknown`] renders
  /// as `"unknown"`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::Mono => "mono",
      Self::SideBySide => "side-by-side",
      Self::TopBottom => "top-bottom",
      Self::FrameSequence => "frame-sequence",
      Self::Checkerboard => "checkerboard",
      Self::SideBySideQuincunx => "side-by-side-quincunx",
      Self::Lines => "lines",
      Self::Columns => "columns",
    }
  }

  /// Stable `u32` wire id = the FFmpeg `AVStereo3DType` code
  /// (`Mono`=0, `SideBySide`=1, `TopBottom`=2, `FrameSequence`=3,
  /// `Checkerboard`=4, `SideBySideQuincunx`=5, `Lines`=6,
  /// `Columns`=7). [`Self::Unknown`] carries its original value
  /// through unchanged so `from_u32(to_u32(x)) == x` for every
  /// unrecognised `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Mono => 0,
      Self::SideBySide => 1,
      Self::TopBottom => 2,
      Self::FrameSequence => 3,
      Self::Checkerboard => 4,
      Self::SideBySideQuincunx => 5,
      Self::Lines => 6,
      Self::Columns => 7,
    }
  }

  /// Decodes from the FFmpeg `AVStereo3DType` code produced by
  /// [`Self::to_u32`]. The canonical codes map to their named
  /// variants (so a decoded value always round-trips); any other id
  /// maps to [`Self::Unknown`] carrying the original value, so the
  /// round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Mono,
      1 => Self::SideBySide,
      2 => Self::TopBottom,
      3 => Self::FrameSequence,
      4 => Self::Checkerboard,
      5 => Self::SideBySideQuincunx,
      6 => Self::Lines,
      7 => Self::Columns,
      _ => Self::Unknown(v),
    }
  }
}

/// One plane of pixel data.
///
/// Generic over the buffer type `B` so the same `Plane` shape works
/// for owned (`Vec<u8>`, `bytes::Bytes`), borrowed (`&'a [u8]`), or
/// custom backend-supplied buffers. The bound `B: AsRef<[u8]>` lives
/// at the use site (`VideoFrame<P, B: AsRef<[u8]>, …>`); `Plane` itself
/// is unbounded so it can be used in const contexts.
///
/// `stride` is bytes per row for video planes, or total plane size
/// in bytes for audio planar formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Plane<B> {
  data: B,
  stride: u32,
}

impl<B> Plane<B> {
  /// Constructs a `Plane` from a buffer and a stride.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(data: B, stride: u32) -> Self {
    Self { data, stride }
  }

  /// Returns the stride in bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Borrows the underlying buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data(&self) -> &B {
    &self.data
  }

  /// Mutably borrows the underlying buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn data_mut(&mut self) -> &mut B {
    &mut self.data
  }

  /// Consumes the plane and returns the underlying buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_data(self) -> B {
    self.data
  }

  /// Sets the stride (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_stride(mut self, stride: u32) -> Self {
    self.stride = stride;
    self
  }

  /// Sets the stride in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_stride(&mut self, stride: u32) -> &mut Self {
    self.stride = stride;
    self
  }
}

/// A runtime-tagged video frame.
///
/// Generic parameters:
/// - `P` — pixel-format identifier. Typically [`crate::pixel_format::PixelFormat`]
///   in mediadecode-style runtime-tagged pipelines, but `P` is left unbounded
///   so backends can substitute a richer type (e.g. an FFmpeg
///   `AVPixelFormat` newtype that round-trips to `PixelFormat`).
/// - `B` — plane data buffer type. Each populated `Plane<B>` carries one
///   plane's bytes; `B: AsRef<[u8]>` at the consumer (e.g. `&'a [u8]`,
///   `Vec<u8>`, `bytes::Bytes`, refcounted FFmpeg buffer).
///
/// `dimensions` is the **coded** width / height; [`Self::visible_rect`]
/// (when present) is the displayable subregion (FFmpeg crop /
/// WebCodecs `visibleRect` / ProRes RAW `CleanAperture`).
///
/// `plane_count` is the number of populated entries in `planes`. Four
/// slots cover every realistic format: NV12 = 2, YUV420P = 3, YUVA /
/// packed-with-alpha = 4, packed RGB / Bayer CFA = 1.
///
/// **No timestamp.** PTS / duration ride on the orthogonal
/// [`TimestampedFrame<F>`] wrapper so the pixel-data layer stays
/// independent of the timekeeping layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoFrame<P, B> {
  dimensions: Dimensions,
  visible_rect: Option<Rect>,
  pixel_format: P,
  plane_count: u8,
  planes: [Plane<B>; 4],
  color: crate::color::ColorInfo,
}

impl<P, B> VideoFrame<P, B> {
  /// Constructs a `VideoFrame`. `visible_rect` defaults to `None`,
  /// color to `ColorInfo::UNSPECIFIED`.
  ///
  /// # Panics
  ///
  /// Panics if `plane_count > 4`. The fixed-size `planes` array has
  /// four slots; passing a larger `plane_count` would later trip
  /// slice indexing inside [`Self::planes`] far from the
  /// construction site. Asserting here fails fast.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    dimensions: Dimensions,
    pixel_format: P,
    planes: [Plane<B>; 4],
    plane_count: u8,
  ) -> Self {
    assert!(
      plane_count as usize <= 4,
      "VideoFrame::new: plane_count exceeds the fixed 4-plane array",
    );
    Self {
      dimensions,
      visible_rect: None,
      pixel_format,
      plane_count,
      planes,
      color: crate::color::ColorInfo::UNSPECIFIED,
    }
  }

  /// Returns the coded dimensions.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn dimensions(&self) -> Dimensions {
    self.dimensions
  }

  /// Returns the coded width (shortcut for `dimensions().width()`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.dimensions.width()
  }

  /// Returns the coded height (shortcut for `dimensions().height()`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.dimensions.height()
  }

  /// Returns the visible / clean-aperture rectangle, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn visible_rect(&self) -> Option<Rect> {
    self.visible_rect
  }

  /// Returns a reference to the pixel-format identifier.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pixel_format(&self) -> &P {
    &self.pixel_format
  }

  /// Returns the populated plane count.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn plane_count(&self) -> u8 {
    self.plane_count
  }

  /// Returns the populated planes as a slice.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn planes(&self) -> &[Plane<B>] {
    &self.planes[..self.plane_count as usize]
  }

  /// Returns one plane by index, or `None` if out of range.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn plane(&self, i: usize) -> Option<&Plane<B>> {
    if i < self.plane_count as usize {
      self.planes.get(i)
    } else {
      None
    }
  }

  /// Returns the color metadata.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn color(&self) -> crate::color::ColorInfo {
    self.color
  }

  /// Sets the visible rect (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_visible_rect(mut self, v: Option<Rect>) -> Self {
    self.visible_rect = v;
    self
  }

  /// Sets the color metadata (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_color(mut self, v: crate::color::ColorInfo) -> Self {
    self.color = v;
    self
  }

  /// Sets the visible rect in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_visible_rect(&mut self, v: Option<Rect>) -> &mut Self {
    self.visible_rect = v;
    self
  }

  /// Sets the color metadata in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_color(&mut self, v: crate::color::ColorInfo) -> &mut Self {
    self.color = v;
    self
  }
}

/// Wraps any inner `F` with optional PTS + duration timestamps.
///
/// This is the orthogonal time-carrying layer. The inner `F` stays
/// pure pixel data — `VideoFrame<P, B>` for runtime-tagged decoder
/// output, or a colconv-typed `Yuv420pFrame<'a, BE>` borrow type for
/// zero-copy conversion pipelines. Composition rather than inheritance
/// keeps the videoframe data layer independent of any timekeeping
/// convention.
///
/// Timestamps use [`mediatime::Timestamp`], a rational-time type from
/// the `mediatime` crate (no_std, zero deps, exact arithmetic). Both
/// PTS and duration are `Option` because backends do not always know
/// them.
///
/// `duration` is deliberately the **same** `mediatime::Timestamp`
/// (timebase ticks) as `pts`, mirroring FFmpeg's `AVFrame.duration`
/// — an `int64` in the stream `time_base`, *not* a wall-clock value.
/// It is intentionally **not** a `core::time::Duration`: that would
/// lose exact rational-timebase precision and diverge from the
/// FFmpeg / `mediatime` model this crate faithfully mirrors. (Codex
/// adversarial-review F2 — reviewed and intentionally kept as-is.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimestampedFrame<F> {
  pts: Option<mediatime::Timestamp>,
  // Timebase ticks, like FFmpeg `AVFrame.duration` — see type doc.
  duration: Option<mediatime::Timestamp>,
  frame: F,
}

impl<F> TimestampedFrame<F> {
  /// Constructs a `TimestampedFrame`. PTS and duration default to
  /// `None`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(frame: F) -> Self {
    Self {
      pts: None,
      duration: None,
      frame,
    }
  }

  /// Returns the presentation timestamp, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pts(&self) -> Option<mediatime::Timestamp> {
    self.pts
  }

  /// Returns the duration, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn duration(&self) -> Option<mediatime::Timestamp> {
    self.duration
  }

  /// Borrows the inner frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn frame(&self) -> &F {
    &self.frame
  }

  /// Mutably borrows the inner frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn frame_mut(&mut self) -> &mut F {
    &mut self.frame
  }

  /// Consumes the wrapper and returns the inner frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_frame(self) -> F {
    self.frame
  }

  /// Sets the PTS (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_pts(mut self, v: Option<mediatime::Timestamp>) -> Self {
    self.pts = v;
    self
  }

  /// Sets the duration (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_duration(mut self, v: Option<mediatime::Timestamp>) -> Self {
    self.duration = v;
    self
  }

  /// Sets the PTS in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_pts(&mut self, v: Option<mediatime::Timestamp>) -> &mut Self {
    self.pts = v;
    self
  }

  /// Sets the duration in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_duration(&mut self, v: Option<mediatime::Timestamp>) -> &mut Self {
    self.duration = v;
    self
  }
}

// === Per-family Frame modules (feature-gated) ===

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod planar_8bit;
#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod subsampled_high_bit_planar;
use derive_more::{Display, IsVariant};
#[cfg(feature = "yuv-planar")]
pub use planar_8bit::*;
#[cfg(feature = "yuv-planar")]
pub use subsampled_high_bit_planar::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod semi_planar_8bit;
#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod subsampled_high_bit_pn;
#[cfg(feature = "yuv-semi-planar")]
pub use semi_planar_8bit::*;
#[cfg(feature = "yuv-semi-planar")]
pub use subsampled_high_bit_pn::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva;
#[cfg(feature = "yuva")]
pub use yuva::*;

#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod packed_yuv_4_1_1;
#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod packed_yuv_8bit;
#[cfg(feature = "yuv-packed")]
pub use packed_yuv_4_1_1::*;
#[cfg(feature = "yuv-packed")]
pub use packed_yuv_8bit::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod packed_yuv_4_4_4;
#[cfg(feature = "yuv-444-packed")]
pub use packed_yuv_4_4_4::*;

#[cfg(feature = "y2xx")]
#[cfg_attr(docsrs, doc(cfg(feature = "y2xx")))]
mod y2xx;
#[cfg(feature = "y2xx")]
pub use y2xx::*;

#[cfg(feature = "v210")]
#[cfg_attr(docsrs, doc(cfg(feature = "v210")))]
mod v210;
#[cfg(feature = "v210")]
pub use v210::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_10bit;
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_16bit;
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_8bit;
#[cfg(feature = "rgb")]
pub use packed_rgb_8bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_10bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_16bit::*;

#[cfg(feature = "rgb-float")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-float")))]
mod packed_rgb_f16;
#[cfg(feature = "rgb-float")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-float")))]
mod packed_rgb_float;
#[cfg(feature = "rgb-float")]
pub use packed_rgb_f16::*;
#[cfg(feature = "rgb-float")]
pub use packed_rgb_float::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod legacy_rgb;
#[cfg(feature = "rgb-legacy")]
pub use legacy_rgb::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod planar_gbr_8bit;
#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod planar_gbr_float;
#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod planar_gbr_high_bit;
#[cfg(feature = "gbr")]
pub use planar_gbr_8bit::*;
#[cfg(feature = "gbr")]
pub use planar_gbr_float::*;
#[cfg(feature = "gbr")]
pub use planar_gbr_high_bit::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray;
#[cfg(feature = "gray")]
pub use gray::*;

#[cfg(feature = "bayer")]
#[cfg_attr(docsrs, doc(cfg(feature = "bayer")))]
mod bayer;
#[cfg(feature = "bayer")]
pub use bayer::*;

#[cfg(feature = "xyz")]
#[cfg_attr(docsrs, doc(cfg(feature = "xyz")))]
mod xyz12;
#[cfg(feature = "xyz")]
pub use xyz12::*;

#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod mono1bit;
#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod pal8;
#[cfg(feature = "mono")]
pub use mono1bit::*;
#[cfg(feature = "mono")]
pub use pal8::*;

// === Tests ===

#[cfg(test)]
mod tests_primitives {
  use super::*;

  #[test]
  fn dimensions_construction_and_accessors() {
    let d = Dimensions::new(1920, 1080);
    assert_eq!(d.width(), 1920);
    assert_eq!(d.height(), 1080);
    assert!(!d.is_zero());
    assert!(Dimensions::default().is_zero());
  }

  #[test]
  fn dimensions_builder() {
    let d = Dimensions::new(0, 0).with_width(640).with_height(480);
    assert_eq!(d.width(), 640);
    assert_eq!(d.height(), 480);
  }

  #[cfg(feature = "std")]
  #[test]
  fn dimensions_display() {
    assert_eq!(std::format!("{}", Dimensions::new(1920, 1080)), "1920x1080");
  }

  #[test]
  fn rect_construction_and_accessors() {
    let r = Rect::new(10, 20, 1280, 720);
    assert_eq!(r.x(), 10);
    assert_eq!(r.y(), 20);
    assert_eq!(r.width(), 1280);
    assert_eq!(r.height(), 720);
  }

  #[test]
  fn rect_builder_chains() {
    let r = Rect::default()
      .with_x(8)
      .with_y(8)
      .with_width(640)
      .with_height(360);
    assert_eq!((r.x(), r.y(), r.width(), r.height()), (8, 8, 640, 360));
  }

  #[test]
  fn rotation_defaults_and_as_str() {
    assert!(matches!(Rotation::default(), Rotation::D0));
    assert_eq!(Rotation::D0.as_str(), "0");
    assert_eq!(Rotation::D90.as_str(), "90");
    assert_eq!(Rotation::D180.as_str(), "180");
    assert_eq!(Rotation::D270.as_str(), "270");
    assert!(Rotation::D90.is_d_90());
  }

  #[test]
  fn rotation_u32_round_trip_and_unknown() {
    for r in [
      Rotation::D0,
      Rotation::D90,
      Rotation::D180,
      Rotation::D270,
      Rotation::Unknown(99),
      Rotation::Unknown(4242),
    ] {
      assert_eq!(Rotation::from_u32(r.to_u32()), r);
    }
    assert_eq!(Rotation::from_u32(0), Rotation::D0);
    assert_eq!(Rotation::from_u32(3), Rotation::D270);
    // Unrecognised → preserved losslessly (no silent collapse to D0).
    assert_eq!(Rotation::from_u32(99), Rotation::Unknown(99));
    assert_eq!(Rotation::from_u32(99).to_u32(), 99);
  }

  #[test]
  fn sample_aspect_ratio_default_is_square() {
    let s = SampleAspectRatio::default();
    assert_eq!(s.num(), 1);
    assert_eq!(s.den().get(), 1);
    assert!(s.is_square());
  }

  #[test]
  fn sample_aspect_ratio_construction_and_builders() {
    let nz = |n: u32| core::num::NonZeroU32::new(n).unwrap();
    let s = SampleAspectRatio::new(40, nz(33));
    assert_eq!(s.num(), 40);
    assert_eq!(s.den().get(), 33);
    assert!(!s.is_square());
    let s2 = SampleAspectRatio::default().with_num(16).with_den(nz(9));
    assert_eq!((s2.num(), s2.den().get()), (16, 9));
    let mut s3 = SampleAspectRatio::default();
    s3.set_num(4).set_den(nz(3));
    assert_eq!((s3.num(), s3.den().get()), (4, 3));
  }

  #[cfg(feature = "std")]
  #[test]
  fn sample_aspect_ratio_display() {
    let nz = core::num::NonZeroU32::new(11).unwrap();
    assert_eq!(std::format!("{}", SampleAspectRatio::new(10, nz)), "10:11");
  }

  #[test]
  fn plane_holds_owned_buffer() {
    let p: Plane<[u8; 4]> = Plane::new([1, 2, 3, 4], 4);
    assert_eq!(p.stride(), 4);
    assert_eq!(p.data(), &[1, 2, 3, 4]);
    let raw = p.into_data();
    assert_eq!(raw, [1, 2, 3, 4]);
  }

  #[test]
  fn plane_holds_borrowed_buffer() {
    let backing = [10u8, 20, 30, 40];
    let p: Plane<&[u8]> = Plane::new(&backing[..], 2);
    assert_eq!(p.stride(), 2);
    assert_eq!(*p.data(), &[10, 20, 30, 40][..]);
  }

  #[test]
  fn plane_with_stride_builder() {
    let p = Plane::new([0u8; 2], 0).with_stride(64);
    assert_eq!(p.stride(), 64);
  }

  // ---------- VideoFrame -------------------------------------------------

  use crate::{color::ColorInfo, pixel_format::PixelFormat};

  #[test]
  fn video_frame_construction_defaults() {
    let planes: [Plane<&[u8]>; 4] = [
      Plane::new(&[][..], 16),
      Plane::new(&[][..], 8),
      Plane::new(&[][..], 8),
      Plane::new(&[][..], 0),
    ];
    let vf = VideoFrame::new(Dimensions::new(16, 16), PixelFormat::Yuv420p, planes, 3);
    assert_eq!(vf.dimensions(), Dimensions::new(16, 16));
    assert_eq!(vf.width(), 16);
    assert_eq!(vf.height(), 16);
    assert_eq!(*vf.pixel_format(), PixelFormat::Yuv420p);
    assert_eq!(vf.plane_count(), 3);
    assert!(vf.visible_rect().is_none());
    assert_eq!(vf.color(), ColorInfo::UNSPECIFIED);
  }

  #[test]
  fn video_frame_planes_slice_uses_plane_count() {
    let planes: [Plane<u32>; 4] = [
      Plane::new(1, 0),
      Plane::new(2, 0),
      Plane::new(3, 0),
      Plane::new(4, 0),
    ];
    let vf = VideoFrame::new(Dimensions::new(2, 2), PixelFormat::Yuv420p, planes, 2);
    assert_eq!(vf.planes().len(), 2);
    assert_eq!(*vf.plane(0).unwrap().data(), 1);
    assert_eq!(*vf.plane(1).unwrap().data(), 2);
    assert!(vf.plane(2).is_none());
    assert!(vf.plane(7).is_none());
  }

  #[test]
  #[should_panic(expected = "plane_count exceeds the fixed 4-plane array")]
  fn video_frame_new_panics_on_plane_count_over_4() {
    let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
    let _ = VideoFrame::new(Dimensions::new(1, 1), PixelFormat::Yuv420p, planes, 5);
  }

  #[test]
  fn video_frame_with_visible_rect_and_color_chain() {
    let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
    let vf = VideoFrame::new(Dimensions::new(8, 8), PixelFormat::Yuv420p, planes, 3)
      .with_visible_rect(Some(Rect::new(0, 0, 6, 6)));
    assert_eq!(vf.visible_rect(), Some(Rect::new(0, 0, 6, 6)));
  }

  // ---------- TimestampedFrame ------------------------------------------

  #[test]
  fn timestamped_frame_construction_defaults() {
    let tf: TimestampedFrame<&'static str> = TimestampedFrame::new("payload");
    assert!(tf.pts().is_none());
    assert!(tf.duration().is_none());
    assert_eq!(*tf.frame(), "payload");
  }

  #[test]
  fn timestamped_frame_into_frame_consumes() {
    let tf = TimestampedFrame::new(42u32);
    let raw = tf.into_frame();
    assert_eq!(raw, 42);
  }

  #[test]
  fn timestamped_frame_pts_builder() {
    let tb = mediatime::Timebase::new(1, core::num::NonZeroU32::new(1000).unwrap());
    let ts = mediatime::Timestamp::new(1000, tb);
    let tf = TimestampedFrame::new(0u8)
      .with_pts(Some(ts))
      .with_duration(Some(ts));
    assert_eq!(tf.pts(), Some(ts));
    assert_eq!(tf.duration(), Some(ts));
  }

  #[test]
  fn timestamped_frame_wraps_video_frame() {
    let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
    let vf = VideoFrame::new(Dimensions::new(4, 4), PixelFormat::Yuv420p, planes, 3);
    let tf = TimestampedFrame::new(vf);
    assert_eq!(tf.frame().dimensions(), Dimensions::new(4, 4));
  }

  // ---------- Rational --------------------------------------------------

  #[test]
  fn rational_default_is_one_over_one() {
    let r = Rational::default();
    assert_eq!(r.num(), 1);
    assert_eq!(r.den().get(), 1);
    assert!(!r.is_zero());
  }

  #[test]
  fn rational_construction_builders_and_is_zero() {
    let nz = |n: u32| core::num::NonZeroU32::new(n).unwrap();
    let r = Rational::new(30000, nz(1001));
    assert_eq!(r.num(), 30000);
    assert_eq!(r.den().get(), 1001);
    assert!(!r.is_zero());
    let z = Rational::new(0, nz(1));
    assert!(z.is_zero());
    let r2 = Rational::default().with_num(24).with_den(nz(1));
    assert_eq!((r2.num(), r2.den().get()), (24, 1));
    let mut r3 = Rational::default();
    r3.set_num(16).set_den(nz(9));
    assert_eq!((r3.num(), r3.den().get()), (16, 9));
  }

  #[cfg(feature = "std")]
  #[test]
  fn rational_display() {
    let nz = core::num::NonZeroU32::new(1001).unwrap();
    assert_eq!(std::format!("{}", Rational::new(30000, nz)), "30000/1001");
  }

  // ---------- SampleAspectRatio ↔ Rational interop ----------------------

  #[test]
  fn sample_aspect_ratio_rational_interop() {
    let nz = |n: u32| core::num::NonZeroU32::new(n).unwrap();
    let sar = SampleAspectRatio::new(40, nz(33));
    let via_method: Rational = sar.as_rational();
    let via_from: Rational = Rational::from(sar);
    let via_into: Rational = sar.into();
    assert_eq!(via_method, Rational::new(40, nz(33)));
    assert_eq!(via_method, via_from);
    assert_eq!(via_from, via_into);
    // Default 1:1 SAR maps to the 1/1 Rational default.
    assert_eq!(
      SampleAspectRatio::default().as_rational(),
      Rational::default()
    );
  }

  #[test]
  fn sample_aspect_ratio_rational_round_trip_both_ways() {
    let nz = |n: u32| core::num::NonZeroU32::new(n).unwrap();
    // SAR -> Rational -> SAR
    let sar = SampleAspectRatio::new(40, nz(33));
    let r: Rational = sar.into();
    let back: SampleAspectRatio = r.into();
    assert_eq!(back, sar);
    assert_eq!(sar.rational(), r);
    assert_eq!(sar.rational(), sar.as_rational());
    // Rational -> SAR -> Rational
    let r2 = Rational::new(16, nz(9));
    let s2 = SampleAspectRatio::from(r2);
    assert_eq!((s2.num(), s2.den().get()), (16, 9));
    assert_eq!(Rational::from(s2), r2);
  }

  #[test]
  fn sample_aspect_ratio_default_is_one_to_one() {
    let d = SampleAspectRatio::default();
    assert_eq!((d.num(), d.den().get()), (1, 1));
    assert!(d.is_square());
    assert_eq!(d, SampleAspectRatio::new(1, core::num::NonZeroU32::MIN));
  }

  #[test]
  fn sample_aspect_ratio_eq_and_hash_parity() {
    use core::hash::{Hash, Hasher};
    let nz = |n: u32| core::num::NonZeroU32::new(n).unwrap();
    let a = SampleAspectRatio::new(40, nz(33));
    let b = SampleAspectRatio::default().with_num(40).with_den(nz(33));
    assert_eq!(a, b);

    fn h(s: &SampleAspectRatio) -> u64 {
      // `no_std`-friendly deterministic hasher (FNV-1a).
      struct Fnv(u64);
      impl Hasher for Fnv {
        fn finish(&self) -> u64 {
          self.0
        }
        fn write(&mut self, bytes: &[u8]) {
          for &x in bytes {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x0100_0000_01b3);
          }
        }
      }
      let mut hasher = Fnv(0xcbf2_9ce4_8422_2325);
      s.hash(&mut hasher);
      hasher.finish()
    }
    assert_eq!(h(&a), h(&b));
  }

  // ---------- FrameRate -------------------------------------------------

  #[test]
  fn frame_rate_default_is_one_over_one_cfr() {
    let fr = FrameRate::default();
    assert_eq!(fr.rate(), Rational::default());
    assert!(!fr.is_vfr());
  }

  #[test]
  fn frame_rate_construction_and_builders() {
    let nz = |n: u32| core::num::NonZeroU32::new(n).unwrap();
    let ntsc = Rational::new(30000, nz(1001));
    let fr = FrameRate::new(ntsc, false);
    assert_eq!(fr.rate(), ntsc);
    assert!(!fr.is_vfr());
    let vfr = FrameRate::default().with_rate(ntsc).with_is_vfr(true);
    assert_eq!(vfr.rate(), ntsc);
    assert!(vfr.is_vfr());
    let mut fr3 = FrameRate::default();
    fr3.set_rate(Rational::new(25, nz(1))).set_is_vfr(true);
    assert_eq!(fr3.rate(), Rational::new(25, nz(1)));
    assert!(fr3.is_vfr());
  }

  // ---------- FieldOrder ------------------------------------------------

  #[test]
  fn field_order_default_is_unknown_zero_and_as_str() {
    assert_eq!(FieldOrder::default(), FieldOrder::Unknown(0));
    assert_eq!(FieldOrder::Unknown(0).as_str(), "unknown");
    assert_eq!(FieldOrder::Progressive.as_str(), "progressive");
    assert_eq!(FieldOrder::Tt.as_str(), "tt");
    assert_eq!(FieldOrder::Bb.as_str(), "bb");
    assert_eq!(FieldOrder::Tb.as_str(), "tb");
    assert_eq!(FieldOrder::Bt.as_str(), "bt");
    assert!(FieldOrder::Progressive.is_progressive());
  }

  #[test]
  fn field_order_u32_round_trip_and_unknown() {
    for f in [
      FieldOrder::Progressive,
      FieldOrder::Tt,
      FieldOrder::Bb,
      FieldOrder::Tb,
      FieldOrder::Bt,
      FieldOrder::Unknown(0),
      FieldOrder::Unknown(99),
      FieldOrder::Unknown(4242),
    ] {
      assert_eq!(FieldOrder::from_u32(f.to_u32()), f);
    }
    assert_eq!(FieldOrder::from_u32(1), FieldOrder::Progressive);
    assert_eq!(FieldOrder::from_u32(5), FieldOrder::Bt);
    // FFmpeg's own UNKNOWN sentinel (0) decodes to Unknown(0).
    assert_eq!(FieldOrder::from_u32(0), FieldOrder::Unknown(0));
    assert_eq!(FieldOrder::from_u32(99), FieldOrder::Unknown(99));
    assert_eq!(FieldOrder::from_u32(99).to_u32(), 99);
  }

  // ---------- StereoMode ------------------------------------------------

  #[test]
  fn stereo_mode_default_is_mono_and_as_str() {
    assert_eq!(StereoMode::default(), StereoMode::Mono);
    assert_eq!(StereoMode::Mono.as_str(), "mono");
    assert_eq!(StereoMode::SideBySide.as_str(), "side-by-side");
    assert_eq!(StereoMode::Columns.as_str(), "columns");
    assert_eq!(StereoMode::Unknown(0).as_str(), "unknown");
    assert!(StereoMode::Mono.is_mono());
  }

  #[test]
  fn stereo_mode_u32_round_trip_and_unknown() {
    for s in [
      StereoMode::Mono,
      StereoMode::SideBySide,
      StereoMode::TopBottom,
      StereoMode::FrameSequence,
      StereoMode::Checkerboard,
      StereoMode::SideBySideQuincunx,
      StereoMode::Lines,
      StereoMode::Columns,
      StereoMode::Unknown(99),
      StereoMode::Unknown(4242),
    ] {
      assert_eq!(StereoMode::from_u32(s.to_u32()), s);
    }
    assert_eq!(StereoMode::from_u32(0), StereoMode::Mono);
    assert_eq!(StereoMode::from_u32(7), StereoMode::Columns);
    // Unrecognised → preserved losslessly.
    assert_eq!(StereoMode::from_u32(99), StereoMode::Unknown(99));
    assert_eq!(StereoMode::from_u32(99).to_u32(), 99);
  }
}

// === Frame-family tests (feature-gated) ===

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests;
