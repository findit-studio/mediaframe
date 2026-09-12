// Cluster B — the FFmpeg-coded name vocabularies, colour / pixel-format /
// frame geometry / disposition structs, frame coded enums.

use super::{arb_via_code, arb_via_named_variants};

// Bitflags: uniform `u32` produces reasonable flag combinations directly,
// and every bit pattern is meaningful — keep `TrackDisposition` on raw u32.
arb_via_code!(crate::disposition::TrackDisposition);

// The colour / frame / pixel-format vocabularies are open string enums now:
// `Other(Utf8Bytes)` is their escape, so they generate the same way the codec
// family does — a curated slug, or an arbitrary string, both routed through
// `FromStr` so every generated value is canonical. The old numeric
// generators (`arb_via_code_weighted*`) existed to reach `Unknown(u32)`,
// which no longer exists.
//
// Curated slugs are drawn from each type's own `as_str()` table, favouring
// values a real file carries; `Matrix::Bt601` is included deliberately —
// it is the one mediaframe-domain variant, and the numeric generator used
// to reach it roughly one draw in 8.6 billion.
super::arb_open_string_enum!(
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
super::arb_open_string_enum!(
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
super::arb_open_string_enum!(
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
super::arb_open_string_enum!(crate::color::DynamicRange, ["tv", "pc", "unspecified"]);
super::arb_open_string_enum!(
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
super::arb_open_string_enum!(
  crate::color::DcpTargetGamut,
  ["dci-p3", "rec709", "rec2020"]
);
super::arb_open_string_enum!(
  crate::pixel_format::PixelFormat,
  ["yuv420p", "yuv422p10le", "nv12", "rgb24", "rgba", "p010le"]
);
super::arb_open_string_enum!(crate::frame::Rotation, ["0", "90", "180", "270"]);
super::arb_open_string_enum!(
  crate::frame::FieldOrder,
  ["unknown", "progressive", "tt", "bb", "tb", "bt"]
);
super::arb_open_string_enum!(
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

// Strictly closed coded enums (no escape arm at all — an unrecognised
// code has nowhere to go). Uniform u32 would skew to the default; pick
// uniformly from the named variants instead.
arb_via_named_variants!(crate::audio::BitRateMode, [Cbr, Vbr, Abr]);
arb_via_named_variants!(
  crate::audio::ChannelOrder,
  [Unspecified, Native, Custom, Ambisonic]
);

// ─── colour structs ──────────────────────────────────────────────────────────

impl<'a> ::arbitrary::Arbitrary<'a> for crate::color::Info {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      crate::color::Primaries::arbitrary(u)?,
      crate::color::Transfer::arbitrary(u)?,
      crate::color::Matrix::arbitrary(u)?,
      crate::color::DynamicRange::arbitrary(u)?,
      crate::color::ChromaLocation::arbitrary(u)?,
    ))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::color::ContentLightLevel {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(u32::arbitrary(u)?, u32::arbitrary(u)?))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::color::ChromaCoord {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(u32::arbitrary(u)?, u32::arbitrary(u)?))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::color::MasteringDisplay {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let primaries = [
      crate::color::ChromaCoord::arbitrary(u)?,
      crate::color::ChromaCoord::arbitrary(u)?,
      crate::color::ChromaCoord::arbitrary(u)?,
    ];
    let white_point = crate::color::ChromaCoord::arbitrary(u)?;
    Ok(Self::new(
      primaries,
      white_point,
      u32::arbitrary(u)?,
      u32::arbitrary(u)?,
    ))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::color::HdrStaticMetadata {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      <Option<crate::color::MasteringDisplay> as ::arbitrary::Arbitrary>::arbitrary(u)?,
      <Option<crate::color::ContentLightLevel> as ::arbitrary::Arbitrary>::arbitrary(u)?,
    ))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::color::DolbyVisionConfig {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      u8::arbitrary(u)?,
      u8::arbitrary(u)?,
      bool::arbitrary(u)?,
      bool::arbitrary(u)?,
      u8::arbitrary(u)?,
    ))
  }
}

// ─── frame structs ───────────────────────────────────────────────────────────

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::Dimensions {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(u32::arbitrary(u)?, u32::arbitrary(u)?))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::Rect {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      u32::arbitrary(u)?,
      u32::arbitrary(u)?,
      u32::arbitrary(u)?,
      u32::arbitrary(u)?,
    ))
  }
}

/// Draws a `Rational` numerator / denominator pair inside the range
/// [`crate::frame::Rational::new`] accepts.
///
/// `NonZeroI64::arbitrary` covers the whole non-zero `i64` range, half
/// of which is negative and would trip the constructor's assert — a
/// generator must not be able to panic the type it generates. Folding
/// an unsigned draw into `0..=i64::MAX` (and `1..=i64::MAX` for the
/// denominator) keeps every draw legal by construction rather than by
/// rejection, so no input byte is ever wasted retrying.
fn parts(
  u: &mut ::arbitrary::Unstructured<'_>,
) -> ::arbitrary::Result<(i64, core::num::NonZeroI64)> {
  #[allow(clippy::cast_possible_wrap)] // masked to 63 bits: always non-negative
  let num = (<u64 as ::arbitrary::Arbitrary>::arbitrary(u)? >> 1) as i64;
  #[allow(clippy::cast_possible_wrap)]
  let den = ((<u64 as ::arbitrary::Arbitrary>::arbitrary(u)? >> 1) as i64).max(1);
  let den = core::num::NonZeroI64::new(den).unwrap_or(crate::frame::DEN_ONE);
  Ok((num, den))
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::Rational {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let (num, den) = parts(u)?;
    Ok(Self::new(num, den))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::SampleAspectRatio {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let (num, den) = parts(u)?;
    Ok(Self::new(num, den))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::FrameRate {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      crate::frame::Rational::arbitrary(u)?,
      bool::arbitrary(u)?,
    ))
  }
}

// ─── bayer / RAW development ─────────────────────────────────────────────────
//
// Closed vocabularies: pick uniformly from the named variants. The two
// float structs generate through `try_new`, so every value is valid by
// construction — an `f32` drawn raw is NaN or ±inf often enough that a
// field-wise generator would spend most of its draws on values the
// constructor refuses.

#[cfg(feature = "bayer")]
arb_via_named_variants!(crate::frame::BayerPattern, [Rggb, Bggr, Grbg, Gbrg]);
#[cfg(feature = "bayer")]
arb_via_named_variants!(crate::frame::BayerDemosaic, [Bilinear]);
#[cfg(feature = "bayer")]
arb_via_named_variants!(crate::frame::WbChannel, [R, G, B]);

#[cfg(feature = "bayer")]
impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::WhiteBalance {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Gains are finite and non-negative by the type's invariant; `0.0` is
    // legal (zeroes the channel) so the range is closed at the bottom.
    let gain = |u: &mut ::arbitrary::Unstructured<'a>| -> ::arbitrary::Result<f32> {
      Ok(u.int_in_range(0u32..=8_000)? as f32 / 1_000.0)
    };
    let (r, g, b) = (gain(u)?, gain(u)?, gain(u)?);
    crate::frame::WhiteBalance::try_new(r, g, b).map_err(|_| ::arbitrary::Error::IncorrectFormat)
  }
}

#[cfg(feature = "bayer")]
impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::ColorCorrectionMatrix {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Real CCMs are O(1-5) and regularly negative (they subtract
    // crosstalk); stay well inside `MAX_COEFFICIENT_ABS`.
    let mut m = [[0.0f32; 3]; 3];
    for row in &mut m {
      for cell in row {
        *cell = u.int_in_range(-8_000i32..=8_000)? as f32 / 1_000.0;
      }
    }
    crate::frame::ColorCorrectionMatrix::try_new(m).map_err(|_| ::arbitrary::Error::IncorrectFormat)
  }
}
