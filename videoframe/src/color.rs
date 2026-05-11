//! Color metadata: enums for matrix, primaries, transfer, range, and
//! chroma location — all closed-form per ITU-T H.273.

use derive_more::{Display, IsVariant};

/// Color matrix coefficients per ITU-T H.273 MatrixCoefficients
/// (Table 4) / ISO/IEC 23001-8.
///
/// Read from `AVFrame.colorspace` / `VideoColorSpace.matrix` /
/// `kCVImageBufferYCbCrMatrixKey`.
///
/// For `AVCOL_SPC_UNSPECIFIED` (value `2`), FFmpeg's convention is
/// `Bt709` for sources with `height >= 720` and `Bt601` otherwise —
/// the caller applies that rule when building `ColorInfo`. The
/// `Default` for this enum is `Bt709` (matches FFmpeg's
/// height-≥-720 default).
///
/// Copied verbatim from `colconv::ColorMatrix` (`#[default]`
/// attribute on `Bt709` is the only addition to enable
/// `ColorInfo::default()`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorMatrix {
  /// ITU-R BT.601 (SDTV); also the correct choice for SMPTE170M /
  /// BT470BG (identical coefficients).
  Bt601,
  /// ITU-R BT.709 (HDTV).
  #[default]
  Bt709,
  /// ITU-R BT.2020 non-constant-luminance (UHDTV / HDR10).
  Bt2020Ncl,
  /// SMPTE 240M (legacy 1990s HDTV).
  Smpte240m,
  /// FCC CFR 47 §73.682 (legacy NTSC, very close to BT.601 numerically).
  Fcc,
  /// YCgCo per ITU-T H.273 MatrixCoefficients = 8.
  YCgCo,
}

impl ColorMatrix {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_SPC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Bt601 => "bt601",
      Self::Bt709 => "bt709",
      Self::Bt2020Ncl => "bt2020nc",
      Self::Smpte240m => "smpte240m",
      Self::Fcc => "fcc",
      Self::YCgCo => "ycgco",
    }
  }
}

/// Color primaries per ITU-T H.273 ColourPrimaries (Table 2) /
/// ISO/IEC 23001-8.
///
/// Read from `AVFrame.color_primaries` / `VideoColorSpace.primaries` /
/// `kCVImageBufferColorPrimariesKey`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorPrimaries {
  /// ITU-R BT.709 (HDTV).
  Bt709,
  /// Unspecified — caller infers from height.
  #[default]
  Unspecified,
  /// ITU-R BT.470 System M (legacy NTSC).
  Bt470M,
  /// ITU-R BT.470 System BG (PAL/SECAM).
  Bt470Bg,
  /// SMPTE 170M (NTSC SD; same primaries as BT.601).
  Smpte170M,
  /// SMPTE 240M (legacy 1990s HDTV).
  Smpte240M,
  /// Generic film (ITU-T H.273).
  Film,
  /// ITU-R BT.2020 (UHDTV / HDR10).
  Bt2020,
  /// SMPTE ST 428-1 (XYZ).
  SmpteSt428,
  /// SMPTE RP 431-2 (DCI-P3).
  SmpteRp431,
  /// SMPTE EG 432-1 (Display P3).
  SmpteEg432,
  /// EBU Tech. 3213-E (legacy).
  Ebu3213E,
}

impl ColorPrimaries {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_PRI_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Bt709 => "bt709",
      Self::Unspecified => "unspecified",
      Self::Bt470M => "bt470m",
      Self::Bt470Bg => "bt470bg",
      Self::Smpte170M => "smpte170m",
      Self::Smpte240M => "smpte240m",
      Self::Film => "film",
      Self::Bt2020 => "bt2020",
      Self::SmpteSt428 => "smpte428",
      Self::SmpteRp431 => "smpte431",
      Self::SmpteEg432 => "smpte432",
      Self::Ebu3213E => "ebu3213",
    }
  }
}

/// Transfer characteristics per ITU-T H.273 (Table 3).
///
/// Read from `AVFrame.color_trc` / `VideoColorSpace.transfer` /
/// `kCVImageBufferTransferFunctionKey`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorTransfer {
  /// ITU-R BT.709.
  Bt709,
  /// Unspecified.
  #[default]
  Unspecified,
  /// BT.470 System M (gamma 2.2).
  Bt470M,
  /// BT.470 System BG (gamma 2.8).
  Bt470Bg,
  /// SMPTE 170M (BT.601).
  Smpte170M,
  /// SMPTE 240M.
  Smpte240M,
  /// Linear transfer.
  Linear,
  /// Log 100:1.
  Log100,
  /// Log 316.22:1.
  Log316,
  /// IEC 61966-2-4 (xvYCC).
  Iec6196624,
  /// ITU-R BT.1361 ECG.
  Bt1361Ecg,
  /// IEC 61966-2-1 (sRGB).
  Iec6196621,
  /// ITU-R BT.2020 10-bit.
  Bt2020_10Bit,
  /// ITU-R BT.2020 12-bit.
  Bt2020_12Bit,
  /// SMPTE ST 2084 — Perceptual Quantizer (HDR10).
  SmpteSt2084Pq,
  /// SMPTE ST 428.
  SmpteSt428,
  /// ARIB STD-B67 — Hybrid Log-Gamma.
  AribStdB67Hlg,
}

impl ColorTransfer {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_TRC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Bt709 => "bt709",
      Self::Unspecified => "unspecified",
      Self::Bt470M => "gamma22",
      Self::Bt470Bg => "gamma28",
      Self::Smpte170M => "smpte170m",
      Self::Smpte240M => "smpte240m",
      Self::Linear => "linear",
      Self::Log100 => "log100",
      Self::Log316 => "log316",
      Self::Iec6196624 => "iec61966-2-4",
      Self::Bt1361Ecg => "bt1361e",
      Self::Iec6196621 => "iec61966-2-1",
      Self::Bt2020_10Bit => "bt2020-10",
      Self::Bt2020_12Bit => "bt2020-12",
      Self::SmpteSt2084Pq => "smpte2084",
      Self::SmpteSt428 => "smpte428",
      Self::AribStdB67Hlg => "arib-std-b67",
    }
  }
}

/// Sample range — limited (TV / studio swing) vs. full (PC).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorRange {
  /// Unspecified — caller assumes Limited.
  #[default]
  Unspecified,
  /// Limited / studio swing (8-bit luma 16..235, chroma 16..240).
  Limited,
  /// Full / PC swing (8-bit 0..255).
  Full,
}

impl ColorRange {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_RANGE_*` slug; `tv` / `pc`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unspecified => "unspecified",
      Self::Limited => "tv",
      Self::Full => "pc",
    }
  }
}

/// Chroma sample location (for subsampled YUV formats).
///
/// Aligns with H.265 SPS chroma_loc / FFmpeg `AVChromaLocation`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ChromaLocation {
  /// Unspecified.
  #[default]
  Unspecified,
  /// MPEG-2 / H.264 default (chroma at the left of two luma samples).
  Left,
  /// MPEG-1 / JPEG (chroma centered between four luma samples).
  Center,
  /// DV PAL — top-left.
  TopLeft,
  /// Top.
  Top,
  /// Bottom-left.
  BottomLeft,
  /// Bottom.
  Bottom,
}

impl ChromaLocation {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCHROMA_LOC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unspecified => "unspecified",
      Self::Left => "left",
      Self::Center => "center",
      Self::TopLeft => "topleft",
      Self::Top => "top",
      Self::BottomLeft => "bottomleft",
      Self::Bottom => "bottom",
    }
  }
}

/// Bundled color metadata that rides on every video frame.
///
/// Every backend except R3D and BRAW exposes color metadata natively;
/// RAW backends populate from clip-level color science and leave
/// `Unspecified` if absent. `ColorInfo::UNSPECIFIED` is the sensible
/// default for RAW backends that don't carry per-frame color data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorInfo {
  primaries: ColorPrimaries,
  transfer: ColorTransfer,
  matrix: ColorMatrix,
  range: ColorRange,
  chroma_location: ChromaLocation,
}

impl ColorInfo {
  /// All-`Unspecified` color info (for `Default` / RAW-backend use).
  /// Matrix defaults to `Bt709` (matches FFmpeg's height-≥-720
  /// fallback for `AVCOL_SPC_UNSPECIFIED`).
  pub const UNSPECIFIED: Self = Self {
    primaries: ColorPrimaries::Unspecified,
    transfer: ColorTransfer::Unspecified,
    matrix: ColorMatrix::Bt709,
    range: ColorRange::Unspecified,
    chroma_location: ChromaLocation::Unspecified,
  };

  /// Constructs a `ColorInfo` from explicit components.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    primaries: ColorPrimaries,
    transfer: ColorTransfer,
    matrix: ColorMatrix,
    range: ColorRange,
    chroma_location: ChromaLocation,
  ) -> Self {
    Self {
      primaries,
      transfer,
      matrix,
      range,
      chroma_location,
    }
  }

  /// Returns the color primaries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn primaries(&self) -> ColorPrimaries {
    self.primaries
  }

  /// Returns the transfer characteristics.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn transfer(&self) -> ColorTransfer {
    self.transfer
  }

  /// Returns the YUV→RGB matrix coefficients.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn matrix(&self) -> ColorMatrix {
    self.matrix
  }

  /// Returns the sample range (limited / full).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn range(&self) -> ColorRange {
    self.range
  }

  /// Returns the chroma sample location.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn chroma_location(&self) -> ChromaLocation {
    self.chroma_location
  }

  /// Sets the primaries (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_primaries(mut self, v: ColorPrimaries) -> Self {
    self.primaries = v;
    self
  }

  /// Sets the transfer (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_transfer(mut self, v: ColorTransfer) -> Self {
    self.transfer = v;
    self
  }

  /// Sets the matrix (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_matrix(mut self, v: ColorMatrix) -> Self {
    self.matrix = v;
    self
  }

  /// Sets the range (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_range(mut self, v: ColorRange) -> Self {
    self.range = v;
    self
  }

  /// Sets the chroma location (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_chroma_location(mut self, v: ChromaLocation) -> Self {
    self.chroma_location = v;
    self
  }

  /// Sets the primaries in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_primaries(&mut self, v: ColorPrimaries) -> &mut Self {
    self.primaries = v;
    self
  }

  /// Sets the transfer in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_transfer(&mut self, v: ColorTransfer) -> &mut Self {
    self.transfer = v;
    self
  }

  /// Sets the matrix in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_matrix(&mut self, v: ColorMatrix) -> &mut Self {
    self.matrix = v;
    self
  }

  /// Sets the range in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_range(&mut self, v: ColorRange) -> &mut Self {
    self.range = v;
    self
  }

  /// Sets the chroma location in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_chroma_location(&mut self, v: ChromaLocation) -> &mut Self {
    self.chroma_location = v;
    self
  }
}

/// Target RGB gamut for the XYZ → RGB matrix step in the
/// [`source::Xyz12`](crate::source::Xyz12) source pipeline (`xyz12_to`).
///
/// The Digital Cinema Package (`AV_PIX_FMT_XYZ12LE`) source carries
/// CIE XYZ samples that need a 3×3 matrix conversion to a target RGB
/// space before any OETF / integer narrow. The default [`Self::DciP3`]
/// target is the **theatrical SMPTE ST 428-1 / RP 431-2** decode using
/// the **DCI white** point `(0.314, 0.351)` — *not* D65; downstream
/// re-targets to Rec.709 (sRGB / web preview) or Rec.2020 (HDR /
/// archival) are supported by runtime-selecting a different matrix at
/// the walker call site.
///
/// White points by variant: `DciP3` = DCI white (~6300 K),
/// `Rec709` = D65, `Rec2020` = D65. See `xyz12_constants.rs` for the
/// exact 27 f32 matrix constants per gamut, derived from each
/// standard's chromaticity coordinates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, IsVariant)]
#[non_exhaustive]
pub enum DcpTargetGamut {
  /// **DCI-P3 (theatrical, DCI white)** — the SMPTE ST 428-1 / RP
  /// 431-2 §5.1 D-Cinema decode target. White point is **DCI white**
  /// `(0.314, 0.351)` (~6300 K), *not* D65. Default for `xyz12_to`
  /// when callers do not opt into a re-target. **Distinct from
  /// Display-P3** (which re-uses the P3 primaries with a D65 white
  /// point and is the Apple / web `display-p3` colour space) — for
  /// sRGB / web preview select [`Self::Rec709`] instead.
  #[default]
  DciP3,
  /// **Rec.709 / sRGB** (D65) — for sRGB-target deliverables and web
  /// preview.
  Rec709,
  /// **Rec.2020** (D65) — for HDR theatrical / archival.
  Rec2020,
}

impl DcpTargetGamut {
  /// Returns the default DCP mastering gamut (`DciP3`). Intended for
  /// `Default`-style fallthrough when callers do not override the
  /// gamut explicitly.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn default_dcp() -> Self {
    Self::DciP3
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_match_spec() {
    assert!(matches!(ColorMatrix::default(), ColorMatrix::Bt709));
    assert!(matches!(
      ColorPrimaries::default(),
      ColorPrimaries::Unspecified
    ));
    assert!(matches!(
      ColorTransfer::default(),
      ColorTransfer::Unspecified
    ));
    assert!(matches!(ColorRange::default(), ColorRange::Unspecified));
    assert!(matches!(
      ChromaLocation::default(),
      ChromaLocation::Unspecified
    ));
  }

  #[test]
  fn is_variant_helpers_compile_for_each_enum() {
    assert!(ColorMatrix::Bt709.is_bt_709());
    assert!(ColorPrimaries::Bt2020.is_bt_2020());
    assert!(ColorTransfer::SmpteSt2084Pq.is_smpte_st_2084_pq());
    assert!(ColorRange::Full.is_full());
    assert!(ChromaLocation::Center.is_center());
  }

  #[test]
  fn copy_and_eq() {
    let m1 = ColorMatrix::Bt709;
    let m2 = m1; // Copy
    assert_eq!(m1, m2);
  }

  #[test]
  fn color_info_default_is_unspecified_with_bt709_matrix() {
    let ci = ColorInfo::default();
    assert_eq!(ci, ColorInfo::UNSPECIFIED);
    assert!(ci.primaries().is_unspecified());
    assert!(ci.matrix().is_bt_709());
  }

  #[test]
  fn color_info_builders_chain() {
    let ci = ColorInfo::UNSPECIFIED
      .with_primaries(ColorPrimaries::Bt2020)
      .with_transfer(ColorTransfer::SmpteSt2084Pq)
      .with_matrix(ColorMatrix::Bt2020Ncl)
      .with_range(ColorRange::Limited)
      .with_chroma_location(ChromaLocation::Left);
    assert!(ci.primaries().is_bt_2020());
    assert!(ci.transfer().is_smpte_st_2084_pq());
    assert!(ci.matrix().is_bt_2020_ncl());
    assert!(ci.range().is_limited());
    assert!(ci.chroma_location().is_left());
  }

  #[test]
  fn color_info_setters_chain() {
    let mut ci = ColorInfo::UNSPECIFIED;
    ci.set_primaries(ColorPrimaries::Bt709)
      .set_transfer(ColorTransfer::Bt709)
      .set_matrix(ColorMatrix::Bt709)
      .set_range(ColorRange::Limited)
      .set_chroma_location(ChromaLocation::Left);
    assert!(ci.primaries().is_bt_709());
    assert!(ci.range().is_limited());
  }

  #[test]
  fn color_info_const_construction() {
    const CI: ColorInfo = ColorInfo::new(
      ColorPrimaries::Bt709,
      ColorTransfer::Bt709,
      ColorMatrix::Bt709,
      ColorRange::Limited,
      ChromaLocation::Left,
    );
    assert!(CI.matrix().is_bt_709());
  }

  #[cfg(feature = "std")]
  #[test]
  fn as_str_matches_display() {
    use std::format;
    // Spot-check: every variant's Display goes through `as_str()`.
    for (s, d) in [
      (
        ColorMatrix::Bt601.as_str(),
        format!("{}", ColorMatrix::Bt601),
      ),
      (
        ColorMatrix::Bt2020Ncl.as_str(),
        format!("{}", ColorMatrix::Bt2020Ncl),
      ),
      (
        ColorMatrix::YCgCo.as_str(),
        format!("{}", ColorMatrix::YCgCo),
      ),
    ] {
      assert_eq!(s, d, "ColorMatrix as_str/Display mismatch");
    }
    assert_eq!(ColorPrimaries::SmpteSt428.as_str(), "smpte428");
    assert_eq!(ColorTransfer::SmpteSt2084Pq.as_str(), "smpte2084");
    assert_eq!(ColorTransfer::Bt2020_10Bit.as_str(), "bt2020-10");
    assert_eq!(ColorRange::Limited.as_str(), "tv");
    assert_eq!(ColorRange::Full.as_str(), "pc");
    assert_eq!(ChromaLocation::TopLeft.as_str(), "topleft");
  }
}
