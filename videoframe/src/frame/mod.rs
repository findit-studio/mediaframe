//! Frame primitives + the typed source-format `*Frame<'a, BE>` borrow types.
//!
//! ## Always-available primitives
//!
//! - [`Dimensions`] — a `(width, height)` pair in pixels.
//! - [`Rect`] — an axis-aligned integer rectangle (used for visible-region
//!   crops on `VideoFrame`).
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

/// Frame `width` value carried by odd-width errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) is odd")]
pub struct OddWidth {
  width: u32,
}

impl OddWidth {
  /// Constructs an `OddWidth` payload.
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

/// Frame `width` value carried by width-not-a-multiple-of-4 errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) is not a multiple of 4")]
pub struct WidthNotMultipleOf4 {
  width: u32,
}

impl WidthNotMultipleOf4 {
  /// Constructs a `WidthNotMultipleOf4` payload.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimestampedFrame<F> {
  pts: Option<mediatime::Timestamp>,
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
}

// === Frame-family tests (feature-gated) ===

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests;
