// Cluster B — closed FFmpeg-coded enums w/ lossless `from_u32`, colour /
// pixel-format / frame geometry / disposition structs, frame coded enums.

use super::arb_via_code;

arb_via_code!(
  crate::color::Matrix,
  crate::color::Primaries,
  crate::color::Transfer,
  crate::color::DynamicRange,
  crate::color::ChromaLocation,
  crate::color::DcpTargetGamut,
  crate::pixel_format::PixelFormat,
  crate::frame::Rotation,
  crate::frame::FieldOrder,
  crate::frame::StereoMode,
  crate::subtitle::TrackOrigin,
  crate::audio::BitRateMode,
  crate::disposition::TrackDisposition,
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

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::Rational {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      u32::arbitrary(u)?,
      core::num::NonZeroU32::arbitrary(u)?,
    ))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::frame::SampleAspectRatio {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    Ok(Self::new(
      u32::arbitrary(u)?,
      core::num::NonZeroU32::arbitrary(u)?,
    ))
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
