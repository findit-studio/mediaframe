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
macro_rules! arb_via_code {
  ($($fn:ident => $ty:path),* $(,)?) => { $(
    #[inline]
    pub(crate) fn $fn(g: &mut Gen) -> $ty {
      <$ty>::from_u32(u32::arbitrary(g))
    }
  )* };
}

// ─── coded enums (13) ────────────────────────────────────────────────────────

arb_via_code! {
  matrix            => crate::color::Matrix,
  primaries         => crate::color::Primaries,
  transfer          => crate::color::Transfer,
  dynamic_range     => crate::color::DynamicRange,
  chroma_location   => crate::color::ChromaLocation,
  dcp_target_gamut  => crate::color::DcpTargetGamut,
  pixel_format      => crate::pixel_format::PixelFormat,
  rotation          => crate::frame::Rotation,
  field_order       => crate::frame::FieldOrder,
  stereo_mode       => crate::frame::StereoMode,
  track_origin      => crate::subtitle::TrackOrigin,
  bit_rate_mode     => crate::audio::BitRateMode,
  track_disposition => crate::disposition::TrackDisposition,
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
