// Cluster B — closed FFmpeg-coded enums w/ lossless `from_u32`, colour /
// pixel-format / frame geometry / disposition structs, frame coded enums.

use super::{arb_via_code, arb_via_code_weighted, arb_via_named_variants};

// Large coded enums: uniform `u32` exercises plenty of named variants
// (typical FFmpeg `AV*` numeric ranges are dense and tight). The
// `Unknown(_)` / `Reserved(_)` arm catches the rest losslessly. Bitflags
// (`TrackDisposition`) likewise produce reasonable flag combinations from
// uniform `u32`.
arb_via_code!(
  crate::color::Matrix,
  crate::color::Primaries,
  crate::color::Transfer,
  crate::pixel_format::PixelFormat,
  crate::disposition::TrackDisposition,
);

// Closed coded enums WITH an `Unknown(u32)` arm and < 10 named variants:
// 50/50 weighted between named picks and arbitrary u32 (Codex round-1
// finding — uniform u32 alone almost never lands on the small named
// numeric range).
arb_via_code_weighted!(crate::color::DynamicRange, [Unspecified, Limited, Full]);
arb_via_code_weighted!(
  crate::color::ChromaLocation,
  [Unspecified, Left, Center, TopLeft, Top, BottomLeft, Bottom]
);
arb_via_code_weighted!(crate::color::DcpTargetGamut, [DciP3, Rec709, Rec2020]);
arb_via_code_weighted!(crate::frame::Rotation, [D0, D90, D180, D270]);
arb_via_code_weighted!(crate::frame::FieldOrder, [Progressive, Tt, Bb, Tb, Bt]);
arb_via_code_weighted!(
  crate::frame::StereoMode,
  [
    Mono,
    SideBySide,
    TopBottom,
    FrameSequence,
    Checkerboard,
    SideBySideQuincunx,
    Lines,
    Columns,
  ]
);

// Strictly closed coded enums (NO `Unknown(u32)` arm — unrecognised
// codes collapse to the default on `from_u32`). Uniform u32 would skew
// to the default; pick uniformly from the named variants instead.
arb_via_named_variants!(crate::audio::BitRateMode, [Cbr, Vbr, Abr]);
arb_via_named_variants!(crate::subtitle::TrackOrigin, [Embedded, Sidecar, External]);

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
