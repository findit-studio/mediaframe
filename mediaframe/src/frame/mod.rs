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
//! | `yuv-444-packed`  | V410/XV30/XV36/AYUV64/VUYA/VUYX/V30X/AYUV/UYVA/VYU444 |
//! | `y2xx`            | Y210 / Y212 / Y216                                   |
//! | `v210`            | V210                                                 |
//! | `rgb`             | Rgb24/Bgr24/Rgba/Bgra + 16-bit + Rgb96/Rgba128       |
//! | `rgb-float`       | Rgbf32 / Rgbf16 + Rgbaf32 / Rgbaf16                  |
//! | `rgb-legacy`      | Rgb444/555/565 + Bgr counterparts                    |
//! | `gbr`             | Gbrp / Gbrap + 9-16bit + float                       |
//! | `gray`            | Gray8-16/32, Grayf16/32, Ya8/16, Yaf16/32            |
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::dimensions")
)]
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

  /// The **storage** aspect ratio `width / height`, exact and
  /// unreduced.
  ///
  /// This is the raster's own shape, ignoring pixel geometry. Compose
  /// it with a [`SampleAspectRatio`] through [`Self::display_size`] to
  /// get what a viewer actually sees.
  ///
  /// [`None`] when `height == 0`: the ratio is undefined, and
  /// [`Dimensions::default`] is `0×0`, so that is an ordinary state
  /// rather than an exceptional one. Mirrors [`Rational::try_new`] —
  /// the same invariant (`den > 0`) at the same altitude.
  ///
  /// Like [`Rational`] generally the result is **not** reduced to
  /// lowest terms: `1920×1080` reads back as `1920/1080`, not `16/9`.
  ///
  /// ```
  /// use mediaframe::frame::Dimensions;
  ///
  /// let r = Dimensions::new(1920, 1080).aspect_ratio().unwrap();
  /// assert_eq!(r.num(), 1920);
  /// assert_eq!(r.den().get(), 1080);
  /// assert!(Dimensions::default().aspect_ratio().is_none());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn aspect_ratio(&self) -> Option<Rational> {
    match core::num::NonZeroI64::new(self.height as i64) {
      Some(den) => Rational::try_new(self.width as i64, den),
      None => None,
    }
  }

  /// Applies a [`SampleAspectRatio`] to these **coded** dimensions,
  /// returning the size the picture occupies in *square* pixels.
  ///
  /// A SAR is a pixel's display width over its display height, so it
  /// scales the horizontal axis and leaves the vertical one alone —
  /// FFmpeg's `scale=iw*sar:ih` / `setsar` convention, and the meaning
  /// of ISO/IEC 14496-12 `pasp`. Height comes back unchanged, so the
  /// derivation never discards detail.
  ///
  /// # Rounding
  ///
  /// `width × num / den` is rounded **half away from zero**, matching
  /// FFmpeg's `av_rescale` — `av_rescale_rnd` with `AV_ROUND_NEAR_INF`,
  /// which computes `(a * b + c / 2) / c`. The intermediate product is
  /// taken in `i128`, so no representable input can overflow it.
  ///
  /// [`None`] in exactly two cases:
  ///
  /// - `sar.num() == 0`. That is FFmpeg's spelling of an *unknown*
  ///   pixel aspect (see [`Rational::is_zero`]), and there is no
  ///   display size to derive from it. [`SampleAspectRatio`]'s own
  ///   contract asks callers to normalise it to the `1:1` default
  ///   before construction — do that first if the square-pixel reading
  ///   is what you want.
  /// - the derived width exceeds [`u32::MAX`], which [`Dimensions`]
  ///   cannot hold.
  ///
  /// ```
  /// use core::num::NonZeroI64;
  ///
  /// use mediaframe::frame::{Dimensions, SampleAspectRatio};
  ///
  /// // ITU-R BT.601 NTSC 16:9: 720×480 at SAR 40:33 displays as
  /// // 873×480 — (720 × 40 + 16) / 33 = 873.2… → 873.
  /// let coded = Dimensions::new(720, 480);
  /// let sar = SampleAspectRatio::new(40, NonZeroI64::new(33).unwrap());
  /// assert_eq!(coded.display_size(sar), Some(Dimensions::new(873, 480)));
  ///
  /// // Square pixels are the identity.
  /// assert_eq!(coded.display_size(SampleAspectRatio::default()), Some(coded));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn display_size(&self, sar: SampleAspectRatio) -> Option<Self> {
    let num = sar.num() as i128;
    if num == 0 {
      return None;
    }
    let den = sar.den().get() as i128;
    // `av_rescale`'s AV_ROUND_NEAR_INF: `(a * b + c / 2) / c`. Both
    // operands are non-negative here, so that is round-half-up.
    let scaled = (self.width as i128 * num + den / 2) / den;
    if scaled > u32::MAX as i128 {
      return None;
    }
    Some(Self::new(scaled as u32, self.height))
  }

  /// Whether `rect` lies entirely inside the raster these dimensions
  /// describe.
  ///
  /// The predicate is the whole definition —
  /// `rect.x + rect.width <= self.width`, and the same on the vertical
  /// axis — evaluated with checked addition, so an origin plus extent
  /// that overflows `u32` is simply not contained rather than wrapping
  /// into a false positive.
  ///
  /// Three boundary cases fall out of that one formula rather than
  /// being special-cased:
  ///
  /// - **Flush is inside.** A rect ending exactly on the edge
  ///   (`x + width == self.width`) is contained: the raster's columns
  ///   are `0..width`, and such a rect touches column `width - 1` last.
  /// - **One pixel over is outside.**
  /// - **An empty rect is contained wherever its origin is.** A zero
  ///   width and/or height reduces the test to `x <= self.width &&
  ///   y <= self.height`, so [`Rect::default`] is inside every
  ///   `Dimensions` — including [`Dimensions::default`]. An empty
  ///   rectangle covers no pixel, so it has no pixel that could fall
  ///   outside.
  ///
  /// Nothing in this crate enforces the relation: [`VideoFrame`]'s
  /// visible-rect builders and setters assign whatever they are given,
  /// deliberately, because a descriptor is usually assembled field by
  /// field and the coded size may not be set yet. This is the check to
  /// run once the pair is complete.
  ///
  /// ```
  /// use mediaframe::frame::{Dimensions, Rect};
  ///
  /// let coded = Dimensions::new(1920, 1080);
  /// // 480 + 1440 == 1920: flush with the right edge, so inside.
  /// assert!(coded.contains(&Rect::new(480, 0, 1440, 1080)));
  /// // 481 + 1440 == 1921: one column past it.
  /// assert!(!coded.contains(&Rect::new(481, 0, 1440, 1080)));
  /// // An empty rect covers no pixel, so it is inside.
  /// assert!(coded.contains(&Rect::default()));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn contains(&self, rect: &Rect) -> bool {
    match (
      rect.x().checked_add(rect.width()),
      rect.y().checked_add(rect.height()),
    ) {
      (Some(right), Some(bottom)) => right <= self.width && bottom <= self.height,
      _ => false,
    }
  }
}

impl core::fmt::Display for Dimensions {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}x{}", self.width, self.height)
  }
}

/// The error [`Dimensions`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed; the rejected input is deliberately not retained.
/// `#[non_exhaustive]` keeps it constructible only here, so it can grow
/// structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a WIDTHxHEIGHT dimension pair")]
#[non_exhaustive]
pub struct ParseDimensionsError;

impl core::str::FromStr for Dimensions {
  type Err = ParseDimensionsError;

  /// Parses the `WIDTHxHEIGHT` form [`Display`](core::fmt::Display)
  /// renders (`"1920x1080"`).
  ///
  /// # Errors
  ///
  /// Returns [`ParseDimensionsError`] unless the input is exactly two
  /// `u32` values separated by a single `x`.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (w, h) = s.split_once('x').ok_or(ParseDimensionsError)?;
    let width = w.parse().map_err(|_| ParseDimensionsError)?;
    let height = h.parse().map_err(|_| ParseDimensionsError)?;
    Ok(Self::new(width, height))
  }
}

/// An axis-aligned integer rectangle.
///
/// Used for `VideoFrame::visible_rect` (FFmpeg crop /
/// WebCodecs `visibleRect` / ProRes RAW `CleanAperture`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::rect")
)]
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

  /// The rectangle's own aspect ratio `width / height`, exact and
  /// unreduced — [`Dimensions::aspect_ratio`] over the visible
  /// subregion rather than the coded raster.
  ///
  /// The origin does not enter into it. [`None`] when `height == 0`,
  /// for the same reason as [`Dimensions::aspect_ratio`].
  ///
  /// ```
  /// use mediaframe::frame::Rect;
  ///
  /// // A 1440×1080 crop out of a 1920×1080 frame is 4:3 of itself.
  /// let r = Rect::new(240, 0, 1440, 1080).aspect_ratio().unwrap();
  /// assert_eq!((r.num(), r.den().get()), (1440, 1080));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn aspect_ratio(&self) -> Option<Rational> {
    match core::num::NonZeroI64::new(self.height as i64) {
      Some(den) => Rational::try_new(self.width as i64, den),
      None => None,
    }
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
/// corrupt wire value is **rejected** by [`Self::from_u32`] rather than
/// silently collapsed to a valid rotation; a *name* this build does not
/// enumerate is carried verbatim as [`Self::Other`], the crate-wide
/// extension idiom.
///
/// The angle is the **clockwise** rotation to apply for display
/// (matching WebCodecs' `rotation`); callers normalising FFmpeg's
/// counter-clockwise convention negate accordingly. [`Self::D0`] is
/// the default (no rotation / square presentation).
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::rotation")
)]
pub enum Rotation {
  /// No rotation.
  #[default]
  D0,
  /// 90° clockwise.
  D90,
  /// 180°.
  D180,
  /// 270° clockwise (= 90° counter-clockwise).
  D270,
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Rotation {
  /// Degree string for this rotation (`"0"` / `"90"` / `"180"` /
  /// `"270"`); [`Self::Other`] renders the name it carries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::D0 => "0",
      Self::D90 => "90",
      Self::D180 => "180",
      Self::D270 => "270",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id: `0`/`1`/`2`/`3` for
  /// `D0`/`D90`/`D180`/`D270`. Stable and append-only.
  ///
  /// [`None`] for [`Self::Other`]: it names a rotation this build does
  /// not enumerate, and there is no id to invent for it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::D0 => 0,
      Self::D90 => 1,
      Self::D180 => 2,
      Self::D270 => 3,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. [`None`] for an unrecognised value — never a
  /// silent collapse to a default rotation.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::D0,
      1 => Self::D90,
      2 => Self::D180,
      3 => Self::D270,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(Rotation, "rotation", [D0, D90, D180, D270], alloc_escape: Other);

/// The error [`Rotation`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a rotation")]
#[non_exhaustive]
pub struct ParseRotationError;

impl core::str::FromStr for Rotation {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseRotationError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseRotationError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"0" => Self::D0,
      b"90" => Self::D90,
      b"180" => Self::D180,
      b"270" => Self::D270,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseRotationError),
    })
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
/// `den` is a [`core::num::NonZeroI64`] so a SAR can never have a
/// zero denominator, and the numerator cannot be negative; the manual
/// [`Default`] is `1:1` (square), mirroring `mediatime::Timebase`'s
/// non-proto-zero default.
///
/// Represented as a newtype over [`Rational`] — the single source of
/// truth for "exact ratio with a non-zero denominator". The fields
/// are private; the entire public method API (and the `buffa` wire
/// format) is unchanged, delegating to the inner `Rational`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::sample_aspect_ratio")
)]
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
  ///
  /// # Panics
  ///
  /// Panics if `num < 0` or `den < 0`, as [`Rational::new`] does. For
  /// a fallible path, build the ratio with [`Rational::try_new`] and
  /// wrap it — `SampleAspectRatio` is `From<Rational>`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(num: i64, den: core::num::NonZeroI64) -> Self {
    Self(Rational::new(num, den))
  }

  /// Returns the numerator (display-width units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn num(&self) -> i64 {
    self.0.num()
  }

  /// Returns the (non-zero) denominator (display-height units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn den(&self) -> core::num::NonZeroI64 {
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
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Rational::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_num(mut self, num: i64) -> Self {
    self.0 = self.0.with_num(num);
    self
  }

  /// Sets the denominator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Rational::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_den(mut self, den: core::num::NonZeroI64) -> Self {
    self.0 = self.0.with_den(den);
    self
  }

  /// Sets the numerator in place.
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Rational::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_num(&mut self, num: i64) -> &mut Self {
    self.0.set_num(num);
    self
  }

  /// Sets the denominator in place.
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Rational::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_den(&mut self, den: core::num::NonZeroI64) -> &mut Self {
    self.0.set_den(den);
    self
  }
}

impl core::fmt::Display for SampleAspectRatio {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}:{}", self.0.num(), self.0.den())
  }
}

impl core::str::FromStr for SampleAspectRatio {
  type Err = ParseSampleAspectRatioError;

  /// Parses the `NUM:DEN` form [`Display`](core::fmt::Display) renders
  /// (`"40:33"`). The separator is a colon, not the `/` [`Rational`]
  /// uses — an aspect ratio is conventionally written `a:b`, and the
  /// two spellings stay distinguishable.
  ///
  /// # Errors
  ///
  /// Returns [`ParseSampleAspectRatioError`] when the input is not two
  /// `i64` values separated by a single `:`, or when the pair violates
  /// [`Rational::try_new`]'s invariant (`num >= 0`, `den > 0`).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_ratio(s, ':')
      .map(Self)
      .map_err(|kind| ParseSampleAspectRatioError { kind })
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

/// `NonZeroI64` for 1 — [`Rational`]'s default denominator, and the
/// clamp target when a malformed denominator arrives on the wire.
///
/// Spelled out rather than reached for as `NonZeroI64::MIN`, which is
/// `i64::MIN` — a value [`Rational::new`] rejects. (The pre-`i64`
/// `NonZeroU32::MIN` *was* `1`, so the swap is a silent trap.)
pub(crate) const DEN_ONE: core::num::NonZeroI64 = match core::num::NonZeroI64::new(1) {
  Some(v) => v,
  None => unreachable!(),
};

/// A generic exact ratio `num / den` — a non-negative numerator over a
/// strictly positive denominator.
///
/// The reusable rational primitive the rest of the frame layer builds
/// on (e.g. [`FrameRate`]). `den` is a [`core::num::NonZeroI64`] so a
/// ratio can never have a zero denominator, and [`Self::new`] rejects
/// the negative half of both fields; the manual [`Default`] is `1/1`
/// (the multiplicative identity), mirroring [`SampleAspectRatio`]'s
/// non-proto-zero default and `mediatime::Timebase`'s convention.
///
/// This is the format-agnostic numerator/denominator pair; semantic
/// wrappers ([`SampleAspectRatio`] for pixel aspect, [`FrameRate`] for
/// frames-per-second) carry the domain meaning. A `0` numerator is a
/// valid representable state (e.g. an "unknown" FFmpeg `AVRational`
/// `0/1`) — see [`Self::is_zero`].
///
/// # Why `i64` and not `mediatime::Timebase`'s `i32`
///
/// `mediatime::Timebase` is signed 32-bit because it must round-trip
/// into an FFmpeg `AVRational` (`{int num; int den;}`) and because it
/// is an *arithmetic operand*: it multiplies against `i64` PTS values,
/// and the rescale overflow proofs need `num < 2^32` to keep the
/// product inside `i128`.
///
/// Neither applies here. `mediaframe` is a pure receiver — nothing in
/// it is handed back to a decoder SDK — and a `Rational` never
/// multiplies against a PTS, so it carries no width proof. What does
/// apply is the storage end (`sqlx` has no `Type<Postgres>` for `u32`,
/// so a `u32` widens to `i64` to be stored regardless) and the ingest
/// end (R3D metadata returns `unsigned int`, and ISO BMFF `pasp` is
/// `unsigned int(32)`, both of which `i32` would have to *reject*).
/// `i64` is a superset of every source and deletes a conversion with
/// its error path at each boundary.
///
/// Values are stored exactly as declared — the constructor does **not**
/// reduce to lowest terms, so a stream declaring `2/4` reads back as
/// `2/4`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::rational")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
  #[cfg_attr(feature = "serde", serde(deserialize_with = "de_num"))]
  num: i64,
  #[cfg_attr(feature = "serde", serde(deserialize_with = "de_den"))]
  den: core::num::NonZeroI64,
}

impl Default for Rational {
  /// `1/1` — the multiplicative identity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      num: 1,
      den: DEN_ONE,
    }
  }
}

impl Rational {
  /// Constructs a `Rational` from an explicit
  /// numerator / (non-zero) denominator.
  ///
  /// # Panics
  ///
  /// - Panics if `num < 0` (a negative ratio is meaningless for the
  ///   aspect / rate domains this primitive serves).
  /// - Panics if `den <= 0` (`NonZeroI64` rules out zero; this rules
  ///   out the negative denominators an `AVRational` would tolerate).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(num: i64, den: core::num::NonZeroI64) -> Self {
    assert!(num >= 0, "rational numerator must not be negative");
    assert!(den.get() > 0, "rational denominator must be positive");

    Self { num, den }
  }

  /// Fallible variant of [`Self::new`]: returns `None` instead of
  /// panicking when `num < 0` or `den < 0`. Accepts `num == 0` (the
  /// "unknown" `0/1` `AVRational`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(num: i64, den: core::num::NonZeroI64) -> Option<Self> {
    if num >= 0 && den.get() > 0 {
      Some(Self { num, den })
    } else {
      None
    }
  }

  /// Returns the numerator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn num(&self) -> i64 {
    self.num
  }

  /// Returns the (non-zero) denominator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn den(&self) -> core::num::NonZeroI64 {
    self.den
  }

  /// `true` when the numerator is `0` (the ratio is exactly zero —
  /// e.g. an "unknown" `0/1` FFmpeg `AVRational`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_zero(&self) -> bool {
    self.num == 0
  }

  /// Sets the numerator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Self::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_num(mut self, num: i64) -> Self {
    self.set_num(num);
    self
  }

  /// Sets the denominator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Self::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_den(mut self, den: core::num::NonZeroI64) -> Self {
    self.set_den(den);
    self
  }

  /// Sets the numerator in place.
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Self::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_num(&mut self, num: i64) -> &mut Self {
    // Routed through the constructor so the sign invariants have
    // exactly one enforcement site. Direct field assignment would let
    // a mutator mint the negative values `new` refuses.
    *self = Self::new(num, self.den);
    self
  }

  /// Sets the denominator in place.
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Self::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_den(&mut self, den: core::num::NonZeroI64) -> &mut Self {
    *self = Self::new(self.num, den);
    self
  }
}

/// Field validators for [`Rational`]'s derived `Deserialize`.
///
/// The derive assigns fields directly, bypassing [`Rational::new`].
/// While the fields were `u32`/`NonZeroU32` their types made every
/// invariant violation unrepresentable; `i64`/`NonZeroI64` no longer
/// do, so deserialization would otherwise be a second construction
/// path able to mint a `Rational` the constructor rejects. The two
/// invariants are independent per field, so a `deserialize_with` on
/// each is enough — no intermediate representation and no allocation.
/// [`SampleAspectRatio`] and [`FrameRate`] derive through `Rational`,
/// so guarding it guards them.
#[cfg(feature = "serde")]
mod de {
  use core::num::NonZeroI64;

  use serde::{Deserialize, Deserializer, de::Error};

  pub(super) fn de_num<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    let v = i64::deserialize(d)?;
    if v < 0 {
      return Err(D::Error::custom("rational numerator must not be negative"));
    }
    Ok(v)
  }

  pub(super) fn de_den<'de, D: Deserializer<'de>>(d: D) -> Result<NonZeroI64, D::Error> {
    let v = NonZeroI64::deserialize(d)?;
    if v.get() < 0 {
      return Err(D::Error::custom("rational denominator must be positive"));
    }
    Ok(v)
  }
}

#[cfg(feature = "serde")]
use de::{de_den, de_num};

impl core::fmt::Display for Rational {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}/{}", self.num, self.den)
  }
}

/// Splits `NUM<sep>DEN` and runs the pair through [`Rational::try_new`], so
/// parsing cannot become a second construction path that mints a ratio the
/// constructor would reject.
/// Why a ratio spelling was rejected.
///
/// Public because the two cases are genuinely different: a caller
/// forwarding user input wants to say "that is not a ratio" for the first
/// and "that ratio is not representable" for the second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RatioParseKind {
  /// The input does not have the shape at all — no separator, a
  /// non-numeric component, or trailing text.
  Malformed,
  /// The input parsed, but the pair violates [`Rational::try_new`]'s
  /// invariant (`num >= 0`, `den > 0`).
  OutOfRange,
}

impl core::fmt::Display for RatioParseKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      Self::Malformed => "malformed",
      Self::OutOfRange => "value out of range",
    })
  }
}

/// The error [`Rational`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Carries [`RatioParseKind`] — unlike the name vocabularies, a ratio can
/// fail for two reasons a caller reports differently. The rejected input
/// itself is not retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a NUM/DEN rational: {kind}")]
pub struct ParseRationalError {
  kind: RatioParseKind,
}

impl ParseRationalError {
  /// Why the input was rejected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn kind(&self) -> RatioParseKind {
    self.kind
  }
}

/// The error [`SampleAspectRatio`]'s [`FromStr`](core::str::FromStr)
/// returns. Same two cases as [`ParseRationalError`], over the `NUM:DEN`
/// spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a NUM:DEN sample aspect ratio: {kind}")]
pub struct ParseSampleAspectRatioError {
  kind: RatioParseKind,
}

impl ParseSampleAspectRatioError {
  /// Why the input was rejected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn kind(&self) -> RatioParseKind {
    self.kind
  }
}

fn parse_ratio(s: &str, sep: char) -> Result<Rational, RatioParseKind> {
  let (n, d) = s.split_once(sep).ok_or(RatioParseKind::Malformed)?;
  let num: i64 = n.parse().map_err(|_| RatioParseKind::Malformed)?;
  let den: i64 = d.parse().map_err(|_| RatioParseKind::Malformed)?;
  let den = core::num::NonZeroI64::new(den).ok_or(RatioParseKind::OutOfRange)?;
  Rational::try_new(num, den).ok_or(RatioParseKind::OutOfRange)
}

impl core::str::FromStr for Rational {
  type Err = ParseRationalError;

  /// Parses the `NUM/DEN` form [`Display`](core::fmt::Display) renders
  /// (`"30000/1001"`).
  ///
  /// # Errors
  ///
  /// Returns [`ParseRationalError`] when the input is not two `i64`
  /// values separated by a single `/`, or when the pair violates
  /// [`Rational::try_new`]'s invariant (`num >= 0`, `den > 0`).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_ratio(s, '/').map_err(|kind| ParseRationalError { kind })
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::frame_rate")
)]
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

  /// Marks the stream variable-frame-rate (`is_vfr = true`; consuming
  /// builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_is_vfr(mut self) -> Self {
    self.is_vfr = true;
    self
  }

  /// Assigns the raw VFR flag (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_is_vfr(mut self, is_vfr: bool) -> Self {
    self.is_vfr = is_vfr;
    self
  }

  /// Sets the rate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_rate(&mut self, rate: Rational) -> &mut Self {
    self.rate = rate;
    self
  }

  /// Marks the stream variable-frame-rate (`is_vfr = true`) in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_is_vfr(&mut self) -> &mut Self {
    self.is_vfr = true;
    self
  }

  /// Assigns the raw VFR flag in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_is_vfr(&mut self, is_vfr: bool) -> &mut Self {
    self.is_vfr = is_vfr;
    self
  }

  /// Clears the VFR flag (`is_vfr = false`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_is_vfr(&mut self) -> &mut Self {
    self.is_vfr = false;
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
/// [`None`] from [`Self::from_u32`] rather than collapsed; a *name*
/// this build does not enumerate rides [`Self::Other`].
///
/// FFmpeg's own `AV_FIELD_UNKNOWN` sentinel is code `0`, so the
/// [`Default`] is the named [`Self::Unknown`] — the same
/// FFmpeg-names-its-own-absence precedent as
/// [`PixelFormat::None`](crate::pixel_format::PixelFormat::None).
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::field_order")
)]
pub enum FieldOrder {
  /// Field order not known — FFmpeg's own `AV_FIELD_UNKNOWN` (code `0`),
  /// and the [`Default`].
  ///
  /// A **named** member of the FFmpeg vocabulary, not an escape arm: it
  /// carries no payload, owns the slug `"unknown"`, and round-trips
  /// exactly. "The container did not say" is a field order a stream can
  /// state, and it is the state a freshly-defaulted descriptor is in.
  Unknown,
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for FieldOrder {
  /// [`Self::Unknown`] — FFmpeg's `AV_FIELD_UNKNOWN`, code `0`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Unknown
  }
}

impl FieldOrder {
  /// Lowercase slug for this field order (`"progressive"` / `"tt"` /
  /// `"bb"` / `"tb"` / `"bt"`); [`Self::Unknown`] is FFmpeg's own named
  /// `"unknown"`, and [`Self::Other`] renders the name it carries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Unknown => "unknown",
      Self::Progressive => "progressive",
      Self::Tt => "tt",
      Self::Bb => "bb",
      Self::Tb => "tb",
      Self::Bt => "bt",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id = the FFmpeg `AVFieldOrder` code
  /// (`Unknown`=0, `Progressive`=1, `Tt`=2, `Bb`=3, `Tb`=4, `Bt`=5).
  ///
  /// [`None`] for [`Self::Other`]: FFmpeg has no code for a name it
  /// does not know.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Unknown => 0,
      Self::Progressive => 1,
      Self::Tt => 2,
      Self::Bb => 3,
      Self::Tb => 4,
      Self::Bt => 5,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVFieldOrder` code produced by
  /// [`Self::to_u32`]. Code `0` is FFmpeg's own `AV_FIELD_UNKNOWN` and
  /// decodes to the named [`Self::Unknown`]; any other unrecognised id
  /// yields [`None`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::Unknown,
      1 => Self::Progressive,
      2 => Self::Tt,
      3 => Self::Bb,
      4 => Self::Tb,
      5 => Self::Bt,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(FieldOrder, "field order", [Unknown, Progressive, Tt, Bb, Tb, Bt], alloc_escape: Other);

/// The error [`FieldOrder`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a field-order name")]
#[non_exhaustive]
pub struct ParseFieldOrderError;

impl core::str::FromStr for FieldOrder {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseFieldOrderError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseFieldOrderError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"unknown" => Self::Unknown,
      b"progressive" => Self::Progressive,
      b"tt" => Self::Tt,
      b"bb" => Self::Bb,
      b"tb" => Self::Tb,
      b"bt" => Self::Bt,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseFieldOrderError),
    })
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
/// value is **rejected** by [`Self::from_u32`]; a *name* this build
/// does not enumerate rides [`Self::Other`], the crate-wide extension
/// idiom shared with [`Rotation`] / the colour enums.
///
/// The [`Default`] is [`Self::Mono`] — a *real* code (value `0`,
/// FFmpeg `AV_STEREO3D_2D`, plain monoscopic video), so the default
/// is a real mode rather than an absence (the colour-enum named-default
/// precedent, e.g. `DcpTargetGamut::DciP3`), distinct from
/// [`FieldOrder`] whose `0` *is* FFmpeg's UNKNOWN sentinel.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::stereo_mode")
)]
pub enum StereoMode {
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for StereoMode {
  /// [`Self::Mono`] — FFmpeg `AV_STEREO3D_2D` (code `0`), plain
  /// monoscopic video — a real mode, not an absence; the colour-enum
  /// named-default precedent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Mono
  }
}

impl StereoMode {
  /// Lowercase slug for this stereo mode; [`Self::Other`] renders the
  /// name it carries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mono => "mono",
      Self::SideBySide => "side-by-side",
      Self::TopBottom => "top-bottom",
      Self::FrameSequence => "frame-sequence",
      Self::Checkerboard => "checkerboard",
      Self::SideBySideQuincunx => "side-by-side-quincunx",
      Self::Lines => "lines",
      Self::Columns => "columns",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id = the FFmpeg `AVStereo3DType` code
  /// (`Mono`=0, `SideBySide`=1, `TopBottom`=2, `FrameSequence`=3,
  /// `Checkerboard`=4, `SideBySideQuincunx`=5, `Lines`=6,
  /// `Columns`=7).
  ///
  /// [`None`] for [`Self::Other`]: FFmpeg has no code for a name it
  /// does not know.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Mono => 0,
      Self::SideBySide => 1,
      Self::TopBottom => 2,
      Self::FrameSequence => 3,
      Self::Checkerboard => 4,
      Self::SideBySideQuincunx => 5,
      Self::Lines => 6,
      Self::Columns => 7,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVStereo3DType` code produced by
  /// [`Self::to_u32`]. The canonical codes map to their named variants;
  /// any other id yields [`None`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::Mono,
      1 => Self::SideBySide,
      2 => Self::TopBottom,
      3 => Self::FrameSequence,
      4 => Self::Checkerboard,
      5 => Self::SideBySideQuincunx,
      6 => Self::Lines,
      7 => Self::Columns,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  StereoMode,
  "stereo mode",
  [
    Mono, SideBySide, TopBottom, FrameSequence, Checkerboard,
    SideBySideQuincunx, Lines, Columns
  ],
  alloc_escape: Other
);

/// The error [`StereoMode`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a stereo-mode name")]
#[non_exhaustive]
pub struct ParseStereoModeError;

impl core::str::FromStr for StereoMode {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseStereoModeError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseStereoModeError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"mono" => Self::Mono,
      b"side-by-side" => Self::SideBySide,
      b"top-bottom" => Self::TopBottom,
      b"frame-sequence" => Self::FrameSequence,
      b"checkerboard" => Self::Checkerboard,
      b"side-by-side-quincunx" => Self::SideBySideQuincunx,
      b"lines" => Self::Lines,
      b"columns" => Self::Columns,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseStereoModeError),
    })
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
  pub const fn data_ref(&self) -> &B {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoFrame<P, B> {
  dimensions: Dimensions,
  visible_rect: Option<Rect>,
  pixel_format: P,
  plane_count: u8,
  planes: [Plane<B>; 4],
  color: crate::color::Info,
}

impl<P, B> VideoFrame<P, B> {
  /// Constructs a `VideoFrame`. `visible_rect` defaults to `None`,
  /// color to `Info::UNSPECIFIED`.
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
      color: crate::color::Info::UNSPECIFIED,
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
  ///
  /// Not checked against [`Self::dimensions`] on assignment — a
  /// descriptor is usually filled in field by field. Check the pair
  /// with [`Dimensions::contains`] once both are set.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn visible_rect(&self) -> Option<Rect> {
    self.visible_rect
  }

  /// Returns a reference to the pixel-format identifier.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pixel_format_ref(&self) -> &P {
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
  pub fn color(&self) -> crate::color::Info {
    self.color.clone()
  }

  /// Sets the visible rect to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_visible_rect(mut self, v: Rect) -> Self {
    self.visible_rect = Some(v);
    self
  }

  /// Assigns the raw visible-rect wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_visible_rect(mut self, v: Option<Rect>) -> Self {
    self.visible_rect = v;
    self
  }

  /// Sets the color metadata (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_color(mut self, v: crate::color::Info) -> Self {
    self.color = v;
    self
  }

  /// Sets the visible rect to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_visible_rect(&mut self, v: Rect) -> &mut Self {
    self.visible_rect = Some(v);
    self
  }

  /// Assigns the raw visible-rect wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_visible_rect(&mut self, v: Option<Rect>) -> &mut Self {
    self.visible_rect = v;
    self
  }

  /// Clears the visible rect (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_visible_rect(&mut self) -> &mut Self {
    self.visible_rect = None;
    self
  }

  /// Sets the color metadata in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_color(&mut self, v: crate::color::Info) -> &mut Self {
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
/// keeps the mediaframe data layer independent of any timekeeping
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
  pub const fn frame_ref(&self) -> &F {
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

  /// Sets the PTS to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_pts(mut self, v: mediatime::Timestamp) -> Self {
    self.pts = Some(v);
    self
  }

  /// Assigns the raw PTS wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_pts(mut self, v: Option<mediatime::Timestamp>) -> Self {
    self.pts = v;
    self
  }

  /// Sets the duration to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_duration(mut self, v: mediatime::Timestamp) -> Self {
    self.duration = Some(v);
    self
  }

  /// Assigns the raw duration wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_duration(mut self, v: Option<mediatime::Timestamp>) -> Self {
    self.duration = v;
    self
  }

  /// Sets the PTS to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_pts(&mut self, v: mediatime::Timestamp) -> &mut Self {
    self.pts = Some(v);
    self
  }

  /// Assigns the raw PTS wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_pts(&mut self, v: Option<mediatime::Timestamp>) -> &mut Self {
    self.pts = v;
    self
  }

  /// Clears the PTS (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_pts(&mut self) -> &mut Self {
    self.pts = None;
    self
  }

  /// Sets the duration to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_duration(&mut self, v: mediatime::Timestamp) -> &mut Self {
    self.duration = Some(v);
    self
  }

  /// Assigns the raw duration wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_duration(&mut self, v: Option<mediatime::Timestamp>) -> &mut Self {
    self.duration = v;
    self
  }

  /// Clears the duration (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_duration(&mut self) -> &mut Self {
    self.duration = None;
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
#[cfg(any(feature = "std", feature = "alloc"))]
use smol_bytes::Utf8Bytes;
#[cfg(feature = "yuv-planar")]
pub use subsampled_high_bit_planar::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv20;
#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod semi_planar_8bit;
#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod subsampled_high_bit_pn;
#[cfg(feature = "yuv-semi-planar")]
pub use nv20::*;
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
mod packed_rgb_32bit;
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_8bit;
#[cfg(feature = "rgb")]
pub use packed_rgb_8bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_10bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_16bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_32bit::*;

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

// The geometry *projections* keep their own suite: they are about
// derived shape (aspect ratio, display size), not about the primitives'
// construction and accessors that `tests_primitives` covers.
#[cfg(test)]
mod aspect_tests;
#[cfg(test)]
mod contains_tests;

#[cfg(test)]
mod tests_primitives;

// === Frame-family tests (feature-gated) ===

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests;
