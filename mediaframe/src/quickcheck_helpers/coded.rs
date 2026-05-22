//! Cluster B — closed FFmpeg-coded enums w/ lossless `from_u32`, colour /
//! pixel-format / frame geometry / disposition structs.
//!
//! Coded enums: `T::from_u32(u32::arbitrary(g))` — covers `Unknown(u32)` /
//! `Reserved(_)` arms.
//!
//! Structs: build via public `new(...)` with each field via
//! `<FieldT>::arbitrary(g)`. Watch `NonZeroU32` for `Rational` denom.
//!
//! Owned types: 13 coded enums + 11 structs (colour×6, frame×5).

use ::quickcheck::{Arbitrary, Gen};

/// Emits a `pub(crate) fn snake(g: &mut Gen) -> Ty` that decodes an arbitrary
/// `u32` through the type's lossless `from_u32` — total coverage of
/// `Unknown(u32)` / `Reserved(_)` arms in one line per type.
///
/// Used for LARGE coded enums (matrix / primaries / transfer / pixel format /
/// disposition bitflags) where a uniform-`u32` decode hits named code points
/// often enough that the long tail of `Unknown` codes is the dominant
/// contributor and a `choose`-biased weighting would only slow coverage.
macro_rules! arb_via_code {
  ($($fn:ident => $ty:path),* $(,)?) => { $(
    #[inline]
    pub(crate) fn $fn(g: &mut Gen) -> $ty {
      <$ty>::from_u32(u32::arbitrary(g))
    }
  )* };
}

/// Strictly-closed coded enum (no `Unknown` arm) — pick uniformly from named.
///
/// `arb_via_code!` is unsuitable for tiny enums like `BitRateMode` (3 named
/// codes, no `Unknown`): `u32::arbitrary(g)` collapses to the `_ =>` default
/// arm of `from_u32` (~all 4 G values), so e.g. `Vbr` / `Abr` are never
/// exercised. `choose` over a `const NAMED: &[Ty]` slice fixes this.
macro_rules! qc_via_named_variants {
  ($($fn:ident => $ty:path, [$($variant:ident),+ $(,)?]);* $(;)?) => { $(
    #[inline]
    pub(crate) fn $fn(g: &mut Gen) -> $ty {
      const NAMED: &[$ty] = &[$(<$ty>::$variant),+];
      *g.choose(NAMED).expect("non-empty NAMED slice")
    }
  )* };
}

/// Closed coded enum with `Unknown(u32)` arm — 50/50 between named variants
/// (via `choose`) and an arbitrary `u32` (via `from_u32`, which may land on a
/// named code or fall through to `Unknown`).
///
/// Without the bias, small enums like `Rotation` (4 named + `Unknown`)
/// virtually never sample a named variant from a uniform `u32` decode. The
/// 50/50 split keeps `Unknown` reachable while guaranteeing named-arm coverage.
macro_rules! qc_via_code_weighted {
  ($($fn:ident => $ty:path, [$($variant:ident),+ $(,)?]);* $(;)?) => { $(
    #[inline]
    pub(crate) fn $fn(g: &mut Gen) -> $ty {
      if bool::arbitrary(g) {
        const NAMED: &[$ty] = &[$(<$ty>::$variant),+];
        *g.choose(NAMED).expect("non-empty NAMED slice")
      } else {
        <$ty>::from_u32(u32::arbitrary(g))
      }
    }
  )* };
}

/// Closed coded enum with `Unknown(u32)` arm whose named codes cluster in a
/// low-integer range (FFmpeg `AV*` colour / pixel-format enums). Listing
/// every named variant would be unwieldy (`PixelFormat` has 270), so this
/// 50/50-splits an in-`0..=max_named` code (`u32 % (max+1)` — most land on
/// named variants; gaps fall on `Unknown`) against a full-range `u32` (broad
/// `Unknown` exercise). `arb_via_code!` alone almost never reaches the named
/// range for these — uniform `u32` lands in `0..=947` ~1-in-4.5-million.
macro_rules! qc_via_code_weighted_range {
  ($($fn:ident => $ty:path, max_named = $max:expr);* $(;)?) => { $(
    #[inline]
    pub(crate) fn $fn(g: &mut Gen) -> $ty {
      let code = if bool::arbitrary(g) {
        // `$max < u32::MAX` for every covered type, so `+ 1` is safe.
        u32::arbitrary(g) % ($max + 1)
      } else {
        u32::arbitrary(g)
      };
      <$ty>::from_u32(code)
    }
  )* };
}

// ─── coded enums (13) ────────────────────────────────────────────────────────

// Bitflags: uniform `u32` produces reasonable flag combinations directly —
// every bit pattern is meaningful, so raw-`u32` decode is correct here.
arb_via_code! {
  track_disposition => crate::disposition::TrackDisposition,
}

// Large coded enums with an `Unknown(u32)` arm whose named codes cluster in
// a low-integer range. Uniform `u32` essentially never reaches the named
// range (Codex round-2 finding); `qc_via_code_weighted_range!` 50/50-splits
// an in-range pick against a broad `Unknown` exercise. `max_named` = the
// highest code emitted by each type's `to_u32`:
//   - Matrix      — Self::YCgCoRo      => 17
//   - Primaries   — Self::Ebu3213E     => 22
//   - Transfer    — Self::AribStdB67Hlg => 18
//   - PixelFormat — 270 named codes spanning 0..=947 (FFmpeg AVPixelFormat)
qc_via_code_weighted_range! {
  matrix       => crate::color::Matrix,            max_named = 17;
  primaries    => crate::color::Primaries,         max_named = 22;
  transfer     => crate::color::Transfer,          max_named = 18;
  pixel_format => crate::pixel_format::PixelFormat, max_named = 947;
}

// Strictly-closed (no `Unknown` arm) — pick uniformly from named variants.
qc_via_named_variants! {
  bit_rate_mode => crate::audio::BitRateMode,        [Cbr, Vbr, Abr];
  track_origin  => crate::subtitle::TrackOrigin,     [Embedded, Sidecar, External];
}

// Closed + `Unknown(u32)`, < 10 named variants — 50/50 named-vs-`from_u32`.
qc_via_code_weighted! {
  rotation         => crate::frame::Rotation,        [D0, D90, D180, D270];
  field_order      => crate::frame::FieldOrder,      [Progressive, Tt, Bb, Tb, Bt];
  stereo_mode      => crate::frame::StereoMode,
    [Mono, SideBySide, TopBottom, FrameSequence, Checkerboard, SideBySideQuincunx, Lines, Columns];
  dynamic_range    => crate::color::DynamicRange,    [Unspecified, Limited, Full];
  chroma_location  => crate::color::ChromaLocation,
    [Unspecified, Left, Center, TopLeft, Top, BottomLeft, Bottom];
  dcp_target_gamut => crate::color::DcpTargetGamut,  [DciP3, Rec709, Rec2020];
}

// ─── colour structs ──────────────────────────────────────────────────────────

#[inline]
pub(crate) fn info(g: &mut Gen) -> crate::color::Info {
  crate::color::Info::new(
    primaries(g),
    transfer(g),
    matrix(g),
    dynamic_range(g),
    chroma_location(g),
  )
}

#[inline]
pub(crate) fn content_light_level(g: &mut Gen) -> crate::color::ContentLightLevel {
  crate::color::ContentLightLevel::new(u32::arbitrary(g), u32::arbitrary(g))
}

#[inline]
pub(crate) fn chroma_coord(g: &mut Gen) -> crate::color::ChromaCoord {
  crate::color::ChromaCoord::new(u32::arbitrary(g), u32::arbitrary(g))
}

#[inline]
pub(crate) fn mastering_display(g: &mut Gen) -> crate::color::MasteringDisplay {
  let primaries = [chroma_coord(g), chroma_coord(g), chroma_coord(g)];
  let white_point = chroma_coord(g);
  crate::color::MasteringDisplay::new(primaries, white_point, u32::arbitrary(g), u32::arbitrary(g))
}

#[inline]
pub(crate) fn hdr_static_metadata(g: &mut Gen) -> crate::color::HdrStaticMetadata {
  let md = if bool::arbitrary(g) {
    Some(mastering_display(g))
  } else {
    None
  };
  let cll = if bool::arbitrary(g) {
    Some(content_light_level(g))
  } else {
    None
  };
  crate::color::HdrStaticMetadata::new(md, cll)
}

#[inline]
pub(crate) fn dolby_vision_config(g: &mut Gen) -> crate::color::DolbyVisionConfig {
  crate::color::DolbyVisionConfig::new(
    u8::arbitrary(g),
    u8::arbitrary(g),
    bool::arbitrary(g),
    bool::arbitrary(g),
    u8::arbitrary(g),
  )
}

// ─── frame structs ───────────────────────────────────────────────────────────

#[inline]
pub(crate) fn dimensions(g: &mut Gen) -> crate::frame::Dimensions {
  crate::frame::Dimensions::new(u32::arbitrary(g), u32::arbitrary(g))
}

#[inline]
pub(crate) fn rect(g: &mut Gen) -> crate::frame::Rect {
  crate::frame::Rect::new(
    u32::arbitrary(g),
    u32::arbitrary(g),
    u32::arbitrary(g),
    u32::arbitrary(g),
  )
}

#[inline]
pub(crate) fn rational(g: &mut Gen) -> crate::frame::Rational {
  // `NonZeroU32` impls `quickcheck::Arbitrary` in quickcheck 1.x.
  crate::frame::Rational::new(u32::arbitrary(g), ::core::num::NonZeroU32::arbitrary(g))
}

#[inline]
pub(crate) fn sample_aspect_ratio(g: &mut Gen) -> crate::frame::SampleAspectRatio {
  crate::frame::SampleAspectRatio::new(u32::arbitrary(g), ::core::num::NonZeroU32::arbitrary(g))
}

#[inline]
pub(crate) fn frame_rate(g: &mut Gen) -> crate::frame::FrameRate {
  crate::frame::FrameRate::new(rational(g), bool::arbitrary(g))
}
