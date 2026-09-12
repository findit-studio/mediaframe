//! Cluster B — the FFmpeg-coded name vocabularies, colour / pixel-format /
//! frame geometry / disposition structs.
//!
//! Name vocabularies: a curated slug or an arbitrary string, both through
//! `FromStr`, so every generated value is canonical.
//!
//! Structs: build via public `new(...)` with each field via
//! `<FieldT>::arbitrary(g)`. Watch `Rational`'s `NonZeroI64` denom —
//! quickcheck has no `Arbitrary` for the signed `NonZero` family, and
//! the constructor rejects negatives; see [`parts`].
//!
//! Owned types: 13 coded enums + 11 structs (colour×6, frame×5).

use ::quickcheck::{Arbitrary, Gen};

/// Emits a `pub(crate) fn snake(g: &mut Gen) -> Ty` that decodes an arbitrary
/// `u32` through the type's `from_u32`. Only `TrackDisposition` uses it: it
/// is a bit set, so every `u32` is a meaningful value.
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

// ─── coded enums (13) ────────────────────────────────────────────────────────

// Bitflags: uniform `u32` produces reasonable flag combinations directly —
// every bit pattern is meaningful, so raw-`u32` decode is correct here.
arb_via_code! {
  track_disposition => crate::disposition::TrackDisposition,
}

// The colour / frame / pixel-format vocabularies are open string enums now:
// `Other(Utf8Bytes)` is their escape, so they generate the way the codec
// family does. The numeric generators these replaced existed to reach
// `Unknown(u32)`, which no longer exists. `Matrix::Bt601` is in the curated
// slugs deliberately — it is the one mediaframe-domain variant, and the
// numeric generator reached it roughly one draw in 8.6 billion.
qc_open_string_enum!(
  matrix,
  crate::color::Matrix,
  [
    "bt709",
    "bt601",
    "bt470bg",
    "smpte170m",
    "bt2020nc",
    "unspecified"
  ]
);
qc_open_string_enum!(
  primaries,
  crate::color::Primaries,
  [
    "bt709",
    "bt470bg",
    "smpte170m",
    "bt2020",
    "smpte431",
    "unspecified"
  ]
);
qc_open_string_enum!(
  transfer,
  crate::color::Transfer,
  [
    "bt709",
    "smpte170m",
    "iec61966-2-1",
    "smpte2084",
    "arib-std-b67",
    "unspecified"
  ]
);
qc_open_string_enum!(
  dynamic_range,
  crate::color::DynamicRange,
  ["tv", "pc", "unspecified"]
);
qc_open_string_enum!(
  chroma_location,
  crate::color::ChromaLocation,
  [
    "left",
    "center",
    "topleft",
    "top",
    "bottomleft",
    "unspecified"
  ]
);
qc_open_string_enum!(
  dcp_target_gamut,
  crate::color::DcpTargetGamut,
  ["dci-p3", "rec709", "rec2020"]
);
qc_open_string_enum!(
  pixel_format,
  crate::pixel_format::PixelFormat,
  ["yuv420p", "yuv422p10le", "nv12", "rgb24", "rgba", "p010le"]
);
qc_open_string_enum!(rotation, crate::frame::Rotation, ["0", "90", "180", "270"]);
qc_open_string_enum!(
  field_order,
  crate::frame::FieldOrder,
  ["unknown", "progressive", "tt", "bb", "tb", "bt"]
);
qc_open_string_enum!(
  stereo_mode,
  crate::frame::StereoMode,
  [
    "mono",
    "side-by-side",
    "top-bottom",
    "frame-sequence",
    "checkerboard",
    "lines"
  ]
);

// Strictly-closed (no `Unknown` arm) — pick uniformly from named variants.
qc_via_named_variants! {
  bit_rate_mode => crate::audio::BitRateMode,        [Cbr, Vbr, Abr];
  channel_order => crate::audio::ChannelOrder,       [Unspecified, Native, Custom, Ambisonic];
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

/// Draws a `Rational` numerator / denominator pair inside the range
/// [`crate::frame::Rational::new`] accepts.
///
/// quickcheck 1.x implements `Arbitrary` only for the *unsigned*
/// `NonZero` family, so `NonZeroI64` has none to swap in; and a signed
/// draw would anyway be half negative, which the constructor asserts
/// against — a generator must not be able to panic the type it
/// generates. Both halves are therefore folded out of an unsigned draw
/// into `0..=i64::MAX` (`1..=i64::MAX` for the denominator), legal by
/// construction rather than by rejection.
#[inline]
fn parts(g: &mut Gen) -> (i64, ::core::num::NonZeroI64) {
  #[allow(clippy::cast_possible_wrap)] // masked to 63 bits: always non-negative
  let num = (u64::arbitrary(g) >> 1) as i64;
  #[allow(clippy::cast_possible_wrap)]
  let den = ((u64::arbitrary(g) >> 1) as i64).max(1);
  let den = ::core::num::NonZeroI64::new(den).unwrap_or(crate::frame::DEN_ONE);
  (num, den)
}

#[inline]
pub(crate) fn rational(g: &mut Gen) -> crate::frame::Rational {
  let (num, den) = parts(g);
  crate::frame::Rational::new(num, den)
}

#[inline]
pub(crate) fn sample_aspect_ratio(g: &mut Gen) -> crate::frame::SampleAspectRatio {
  let (num, den) = parts(g);
  crate::frame::SampleAspectRatio::new(num, den)
}

#[inline]
pub(crate) fn frame_rate(g: &mut Gen) -> crate::frame::FrameRate {
  crate::frame::FrameRate::new(rational(g), bool::arbitrary(g))
}

// ─── bayer / RAW development ─────────────────────────────────────────────────
//
// Mirrors the `arbitrary` cluster: closed enums pick uniformly from the
// named variants, and the two float structs generate through `try_new` so
// every value is valid by construction.

#[cfg(feature = "bayer")]
qc_via_named_variants! {
  bayer_pattern  => crate::frame::BayerPattern,  [Rggb, Bggr, Grbg, Gbrg];
  bayer_demosaic => crate::frame::BayerDemosaic, [Bilinear];
  wb_channel     => crate::frame::WbChannel,     [R, G, B];
}

/// Gains are finite and non-negative by the type's invariant; `0.0` is
/// legal (zeroes the channel) so the range is closed at the bottom.
#[cfg(feature = "bayer")]
pub(crate) fn white_balance(g: &mut Gen) -> crate::frame::WhiteBalance {
  let gain = |g: &mut Gen| (u32::arbitrary(g) % 8_001) as f32 / 1_000.0;
  let (r, gr, b) = (gain(g), gain(g), gain(g));
  crate::frame::WhiteBalance::try_new(r, gr, b).expect("gains are finite and non-negative")
}

/// Real CCMs are O(1-5) and regularly negative (they subtract crosstalk);
/// stay well inside `MAX_COEFFICIENT_ABS`.
#[cfg(feature = "bayer")]
pub(crate) fn color_correction_matrix(g: &mut Gen) -> crate::frame::ColorCorrectionMatrix {
  let mut m = [[0.0f32; 3]; 3];
  for row in &mut m {
    for cell in row {
      *cell = (i32::arbitrary(g).rem_euclid(16_001) - 8_000) as f32 / 1_000.0;
    }
  }
  crate::frame::ColorCorrectionMatrix::try_new(m).expect("coefficients are finite and bounded")
}
