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
///
/// The explicit `#[repr(u32)]` discriminants form a stable,
/// append-only wire mapping (the `#[default]` `Bt709` is `0`); see
/// [`Self::to_u32`] / [`Self::from_u32`]. These do **not** match the
/// ITU-T H.273 numeric codes — they are videoframe-local ids for the
/// `buffa` encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[repr(u32)]
pub enum ColorMatrix {
  /// ITU-R BT.709 (HDTV).
  #[default]
  Bt709 = 0,
  /// ITU-R BT.601 (SDTV); also the correct choice for SMPTE170M /
  /// BT470BG (identical coefficients).
  Bt601 = 1,
  /// ITU-R BT.2020 non-constant-luminance (UHDTV / HDR10).
  Bt2020Ncl = 2,
  /// SMPTE 240M (legacy 1990s HDTV).
  Smpte240m = 3,
  /// FCC CFR 47 §73.682 (legacy NTSC, very close to BT.601 numerically).
  Fcc = 4,
  /// YCgCo per ITU-T H.273 MatrixCoefficients = 8.
  YCgCo = 5,
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

  /// Stable wire id (the explicit `#[repr(u32)]` discriminant);
  /// `Bt709` (the default) is `0`. Additive helper for the `buffa`
  /// wire encoding — the mapping is stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Bt709 => 0,
      Self::Bt601 => 1,
      Self::Bt2020Ncl => 2,
      Self::Smpte240m => 3,
      Self::Fcc => 4,
      Self::YCgCo => 5,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values map to the default
  /// [`Self::Bt709`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Bt601,
      2 => Self::Bt2020Ncl,
      3 => Self::Smpte240m,
      4 => Self::Fcc,
      5 => Self::YCgCo,
      _ => Self::Bt709,
    }
  }
}

/// Color primaries per ITU-T H.273 ColourPrimaries (Table 2) /
/// ISO/IEC 23001-8.
///
/// Read from `AVFrame.color_primaries` / `VideoColorSpace.primaries` /
/// `kCVImageBufferColorPrimariesKey`.
///
/// The explicit `#[repr(u32)]` discriminants form a stable,
/// append-only wire mapping (the `#[default]` `Unspecified` is `0`);
/// see [`Self::to_u32`] / [`Self::from_u32`]. These are
/// videoframe-local ids for the `buffa` encoding, not the ITU-T
/// H.273 ColourPrimaries codes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[repr(u32)]
pub enum ColorPrimaries {
  /// Unspecified — caller infers from height.
  #[default]
  Unspecified = 0,
  /// ITU-R BT.709 (HDTV).
  Bt709 = 1,
  /// ITU-R BT.470 System M (legacy NTSC).
  Bt470M = 2,
  /// ITU-R BT.470 System BG (PAL/SECAM).
  Bt470Bg = 3,
  /// SMPTE 170M (NTSC SD; same primaries as BT.601).
  Smpte170M = 4,
  /// SMPTE 240M (legacy 1990s HDTV).
  Smpte240M = 5,
  /// Generic film (ITU-T H.273).
  Film = 6,
  /// ITU-R BT.2020 (UHDTV / HDR10).
  Bt2020 = 7,
  /// SMPTE ST 428-1 (XYZ).
  SmpteSt428 = 8,
  /// SMPTE RP 431-2 (DCI-P3).
  SmpteRp431 = 9,
  /// SMPTE EG 432-1 (Display P3).
  SmpteEg432 = 10,
  /// EBU Tech. 3213-E (legacy).
  Ebu3213E = 11,
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

  /// Stable wire id (the explicit `#[repr(u32)]` discriminant);
  /// `Unspecified` (the default) is `0`. Additive helper for the
  /// `buffa` wire encoding — stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unspecified => 0,
      Self::Bt709 => 1,
      Self::Bt470M => 2,
      Self::Bt470Bg => 3,
      Self::Smpte170M => 4,
      Self::Smpte240M => 5,
      Self::Film => 6,
      Self::Bt2020 => 7,
      Self::SmpteSt428 => 8,
      Self::SmpteRp431 => 9,
      Self::SmpteEg432 => 10,
      Self::Ebu3213E => 11,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values map to the default
  /// [`Self::Unspecified`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Bt709,
      2 => Self::Bt470M,
      3 => Self::Bt470Bg,
      4 => Self::Smpte170M,
      5 => Self::Smpte240M,
      6 => Self::Film,
      7 => Self::Bt2020,
      8 => Self::SmpteSt428,
      9 => Self::SmpteRp431,
      10 => Self::SmpteEg432,
      11 => Self::Ebu3213E,
      _ => Self::Unspecified,
    }
  }
}

/// Transfer characteristics per ITU-T H.273 (Table 3).
///
/// Read from `AVFrame.color_trc` / `VideoColorSpace.transfer` /
/// `kCVImageBufferTransferFunctionKey`.
///
/// The explicit `#[repr(u32)]` discriminants form a stable,
/// append-only wire mapping (the `#[default]` `Unspecified` is `0`);
/// see [`Self::to_u32`] / [`Self::from_u32`]. These are
/// videoframe-local ids for the `buffa` encoding, not the ITU-T
/// H.273 TransferCharacteristics codes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[repr(u32)]
pub enum ColorTransfer {
  /// Unspecified.
  #[default]
  Unspecified = 0,
  /// ITU-R BT.709.
  Bt709 = 1,
  /// BT.470 System M (gamma 2.2).
  Bt470M = 2,
  /// BT.470 System BG (gamma 2.8).
  Bt470Bg = 3,
  /// SMPTE 170M (BT.601).
  Smpte170M = 4,
  /// SMPTE 240M.
  Smpte240M = 5,
  /// Linear transfer.
  Linear = 6,
  /// Log 100:1.
  Log100 = 7,
  /// Log 316.22:1.
  Log316 = 8,
  /// IEC 61966-2-4 (xvYCC).
  Iec6196624 = 9,
  /// ITU-R BT.1361 ECG.
  Bt1361Ecg = 10,
  /// IEC 61966-2-1 (sRGB).
  Iec6196621 = 11,
  /// ITU-R BT.2020 10-bit.
  Bt2020_10Bit = 12,
  /// ITU-R BT.2020 12-bit.
  Bt2020_12Bit = 13,
  /// SMPTE ST 2084 — Perceptual Quantizer (HDR10).
  SmpteSt2084Pq = 14,
  /// SMPTE ST 428.
  SmpteSt428 = 15,
  /// ARIB STD-B67 — Hybrid Log-Gamma.
  AribStdB67Hlg = 16,
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

  /// Stable wire id (the explicit `#[repr(u32)]` discriminant);
  /// `Unspecified` (the default) is `0`. Additive helper for the
  /// `buffa` wire encoding — stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unspecified => 0,
      Self::Bt709 => 1,
      Self::Bt470M => 2,
      Self::Bt470Bg => 3,
      Self::Smpte170M => 4,
      Self::Smpte240M => 5,
      Self::Linear => 6,
      Self::Log100 => 7,
      Self::Log316 => 8,
      Self::Iec6196624 => 9,
      Self::Bt1361Ecg => 10,
      Self::Iec6196621 => 11,
      Self::Bt2020_10Bit => 12,
      Self::Bt2020_12Bit => 13,
      Self::SmpteSt2084Pq => 14,
      Self::SmpteSt428 => 15,
      Self::AribStdB67Hlg => 16,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values map to the default
  /// [`Self::Unspecified`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Bt709,
      2 => Self::Bt470M,
      3 => Self::Bt470Bg,
      4 => Self::Smpte170M,
      5 => Self::Smpte240M,
      6 => Self::Linear,
      7 => Self::Log100,
      8 => Self::Log316,
      9 => Self::Iec6196624,
      10 => Self::Bt1361Ecg,
      11 => Self::Iec6196621,
      12 => Self::Bt2020_10Bit,
      13 => Self::Bt2020_12Bit,
      14 => Self::SmpteSt2084Pq,
      15 => Self::SmpteSt428,
      16 => Self::AribStdB67Hlg,
      _ => Self::Unspecified,
    }
  }
}

/// Sample range — limited (TV / studio swing) vs. full (PC).
///
/// The explicit `#[repr(u32)]` discriminants form a stable,
/// append-only wire mapping (the `#[default]` `Unspecified` is `0`);
/// see [`Self::to_u32`] / [`Self::from_u32`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[repr(u32)]
pub enum ColorRange {
  /// Unspecified — caller assumes Limited.
  #[default]
  Unspecified = 0,
  /// Limited / studio swing (8-bit luma 16..235, chroma 16..240).
  Limited = 1,
  /// Full / PC swing (8-bit 0..255).
  Full = 2,
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

  /// Stable wire id (the explicit `#[repr(u32)]` discriminant);
  /// `Unspecified` (the default) is `0`. Additive helper for the
  /// `buffa` wire encoding — stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unspecified => 0,
      Self::Limited => 1,
      Self::Full => 2,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values map to the default
  /// [`Self::Unspecified`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Limited,
      2 => Self::Full,
      _ => Self::Unspecified,
    }
  }
}

/// Chroma sample location (for subsampled YUV formats).
///
/// Aligns with H.265 SPS chroma_loc / FFmpeg `AVChromaLocation`.
///
/// The explicit `#[repr(u32)]` discriminants form a stable,
/// append-only wire mapping (the `#[default]` `Unspecified` is `0`);
/// see [`Self::to_u32`] / [`Self::from_u32`]. These are
/// videoframe-local ids for the `buffa` encoding, not the
/// `AVChromaLocation` numeric codes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[repr(u32)]
pub enum ChromaLocation {
  /// Unspecified.
  #[default]
  Unspecified = 0,
  /// MPEG-2 / H.264 default (chroma at the left of two luma samples).
  Left = 1,
  /// MPEG-1 / JPEG (chroma centered between four luma samples).
  Center = 2,
  /// DV PAL — top-left.
  TopLeft = 3,
  /// Top.
  Top = 4,
  /// Bottom-left.
  BottomLeft = 5,
  /// Bottom.
  Bottom = 6,
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

  /// Stable wire id (the explicit `#[repr(u32)]` discriminant);
  /// `Unspecified` (the default) is `0`. Additive helper for the
  /// `buffa` wire encoding — stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unspecified => 0,
      Self::Left => 1,
      Self::Center => 2,
      Self::TopLeft => 3,
      Self::Top => 4,
      Self::BottomLeft => 5,
      Self::Bottom => 6,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values map to the default
  /// [`Self::Unspecified`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Left,
      2 => Self::Center,
      3 => Self::TopLeft,
      4 => Self::Top,
      5 => Self::BottomLeft,
      6 => Self::Bottom,
      _ => Self::Unspecified,
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
///
/// The explicit `#[repr(u32)]` discriminants form a stable,
/// append-only wire mapping (the `#[default]` `DciP3` is `0`); see
/// [`Self::to_u32`] / [`Self::from_u32`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, IsVariant)]
#[non_exhaustive]
#[repr(u32)]
pub enum DcpTargetGamut {
  /// **DCI-P3 (theatrical, DCI white)** — the SMPTE ST 428-1 / RP
  /// 431-2 §5.1 D-Cinema decode target. White point is **DCI white**
  /// `(0.314, 0.351)` (~6300 K), *not* D65. Default for `xyz12_to`
  /// when callers do not opt into a re-target. **Distinct from
  /// Display-P3** (which re-uses the P3 primaries with a D65 white
  /// point and is the Apple / web `display-p3` colour space) — for
  /// sRGB / web preview select [`Self::Rec709`] instead.
  #[default]
  DciP3 = 0,
  /// **Rec.709 / sRGB** (D65) — for sRGB-target deliverables and web
  /// preview.
  Rec709 = 1,
  /// **Rec.2020** (D65) — for HDR theatrical / archival.
  Rec2020 = 2,
}

impl DcpTargetGamut {
  /// Returns the default DCP mastering gamut (`DciP3`). Intended for
  /// `Default`-style fallthrough when callers do not override the
  /// gamut explicitly.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn default_dcp() -> Self {
    Self::DciP3
  }

  /// Stable wire id (the explicit `#[repr(u32)]` discriminant);
  /// `DciP3` (the default) is `0`. Additive helper for the `buffa`
  /// wire encoding — stable and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::DciP3 => 0,
      Self::Rec709 => 1,
      Self::Rec2020 => 2,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unrecognised values map to the default
  /// [`Self::DciP3`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Rec709,
      2 => Self::Rec2020,
      _ => Self::DciP3,
    }
  }
}

/// Content light level metadata per CTA-861.3 (HDR10).
///
/// Read from FFmpeg `AVContentLightMetadata`
/// (`AV_FRAME_DATA_CONTENT_LIGHT_LEVEL` side data on a decoded frame,
/// or `AV_PKT_DATA_CONTENT_LIGHT_LEVEL` on a packet / stream). Both
/// values are in candelas per square metre (cd/m², "nits"). Not
/// exposed by WebCodecs — it carries no static HDR metadata.
///
/// This is clip / stream level (and frame-level when carried as
/// frame side data); the per-frame [`ColorInfo`] enums are
/// unchanged.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentLightLevel {
  max_cll: u32,
  max_fall: u32,
}

impl ContentLightLevel {
  /// Constructs a `ContentLightLevel` from MaxCLL / MaxFALL
  /// (cd/m²).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(max_cll: u32, max_fall: u32) -> Self {
    Self { max_cll, max_fall }
  }

  /// Maximum Content Light Level (`MaxCLL`, cd/m²).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn max_cll(&self) -> u32 {
    self.max_cll
  }

  /// Maximum Frame-Average Light Level (`MaxFALL`, cd/m²).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn max_fall(&self) -> u32 {
    self.max_fall
  }

  /// Sets `MaxCLL` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_max_cll(mut self, v: u32) -> Self {
    self.max_cll = v;
    self
  }

  /// Sets `MaxFALL` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_max_fall(mut self, v: u32) -> Self {
    self.max_fall = v;
    self
  }

  /// Sets `MaxCLL` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_max_cll(&mut self, v: u32) -> &mut Self {
    self.max_cll = v;
    self
  }

  /// Sets `MaxFALL` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_max_fall(&mut self, v: u32) -> &mut Self {
    self.max_fall = v;
    self
  }
}

/// A CIE 1931 `xy` chromaticity coordinate in SMPTE ST 2086
/// fixed-point units.
///
/// Both `x` and `y` are stored in **0.00002 increments** (i.e. the
/// floating value is `raw / 50000.0`), matching the spec-integer
/// encoding of FFmpeg's `AVMasteringDisplayMetadata` (which uses
/// `AVRational`s of `n/50000`). Storing the raw `u16` ST 2086 units
/// keeps the type lossless and `Copy`/`const`-friendly.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChromaCoord {
  x: u16,
  y: u16,
}

impl ChromaCoord {
  /// Constructs a `ChromaCoord` from raw ST 2086 units (0.00002
  /// increments; floating value = `raw / 50000.0`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(x: u16, y: u16) -> Self {
    Self { x, y }
  }

  /// Returns the `x` coordinate in ST 2086 units (0.00002
  /// increments).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn x(&self) -> u16 {
    self.x
  }

  /// Returns the `y` coordinate in ST 2086 units (0.00002
  /// increments).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> u16 {
    self.y
  }

  /// Sets `x` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_x(mut self, x: u16) -> Self {
    self.x = x;
    self
  }

  /// Sets `y` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_y(mut self, y: u16) -> Self {
    self.y = y;
    self
  }

  /// Sets `x` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_x(&mut self, x: u16) -> &mut Self {
    self.x = x;
    self
  }

  /// Sets `y` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_y(&mut self, y: u16) -> &mut Self {
    self.y = y;
    self
  }
}

/// Mastering display color volume per SMPTE ST 2086 (HDR10).
///
/// Spec-integer encoding matching FFmpeg
/// `AVMasteringDisplayMetadata` (`AV_FRAME_DATA_MASTERING_DISPLAY_METADATA`
/// side data on a decoded frame, or
/// `AV_PKT_DATA_MASTERING_DISPLAY_METADATA` on a packet / stream;
/// CoreVideo `kCVImageBufferMasteringDisplayColorVolumeKey`):
///
/// - [`ChromaCoord`] chromaticities are in ST 2086 units of
///   **0.00002** (floating value = `raw / 50000.0`).
/// - `display_primaries` are the **R, G, B** primaries, in that
///   order (index `0` = red, `1` = green, `2` = blue) — matching
///   FFmpeg's `display_primaries[3][2]` layout.
/// - `max_luminance` / `min_luminance` are in units of **0.0001
///   cd/m²** (floating value = `raw / 10000.0`), matching FFmpeg's
///   `n/10000` `AVRational` luminance encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MasteringDisplay {
  display_primaries: [ChromaCoord; 3],
  white_point: ChromaCoord,
  max_luminance: u32,
  min_luminance: u32,
}

impl MasteringDisplay {
  /// Constructs a `MasteringDisplay` from the R/G/B primaries, the
  /// white point, and the max / min luminance (0.0001 cd/m² units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    display_primaries: [ChromaCoord; 3],
    white_point: ChromaCoord,
    max_luminance: u32,
    min_luminance: u32,
  ) -> Self {
    Self {
      display_primaries,
      white_point,
      max_luminance,
      min_luminance,
    }
  }

  /// Returns the R/G/B display primaries (index `0` = red, `1` =
  /// green, `2` = blue).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn display_primaries(&self) -> [ChromaCoord; 3] {
    self.display_primaries
  }

  /// Returns the white point chromaticity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn white_point(&self) -> ChromaCoord {
    self.white_point
  }

  /// Returns the maximum display luminance (0.0001 cd/m² units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn max_luminance(&self) -> u32 {
    self.max_luminance
  }

  /// Returns the minimum display luminance (0.0001 cd/m² units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn min_luminance(&self) -> u32 {
    self.min_luminance
  }

  /// Sets the R/G/B display primaries (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_display_primaries(mut self, v: [ChromaCoord; 3]) -> Self {
    self.display_primaries = v;
    self
  }

  /// Sets the white point (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_white_point(mut self, v: ChromaCoord) -> Self {
    self.white_point = v;
    self
  }

  /// Sets the max luminance (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_max_luminance(mut self, v: u32) -> Self {
    self.max_luminance = v;
    self
  }

  /// Sets the min luminance (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_min_luminance(mut self, v: u32) -> Self {
    self.min_luminance = v;
    self
  }

  /// Sets the R/G/B display primaries in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_display_primaries(&mut self, v: [ChromaCoord; 3]) -> &mut Self {
    self.display_primaries = v;
    self
  }

  /// Sets the white point in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_white_point(&mut self, v: ChromaCoord) -> &mut Self {
    self.white_point = v;
    self
  }

  /// Sets the max luminance in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_max_luminance(&mut self, v: u32) -> &mut Self {
    self.max_luminance = v;
    self
  }

  /// Sets the min luminance in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_min_luminance(&mut self, v: u32) -> &mut Self {
    self.min_luminance = v;
    self
  }
}

/// Clip / stream-level optional HDR10 **static** metadata.
///
/// Bundles the two SMPTE ST 2086 / CTA-861.3 static descriptors that
/// ride alongside a stream rather than on every frame. Both are
/// [`Option`] because a source may carry one, both, or neither
/// (SDR / WebCodecs sources carry neither).
///
/// This is intentionally *separate* from [`ColorInfo`]: `ColorInfo`
/// stays per-frame closed-form enums only; HDR10 static metadata is
/// clip / stream level and optional, so it lives in its own type.
/// (Dynamic HDR — HDR10+ / Dolby Vision RPU — is out of scope here.)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HdrStaticMetadata {
  mastering: Option<MasteringDisplay>,
  content_light: Option<ContentLightLevel>,
}

impl HdrStaticMetadata {
  /// Constructs an `HdrStaticMetadata` from optional mastering
  /// display + content light level descriptors.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    mastering: Option<MasteringDisplay>,
    content_light: Option<ContentLightLevel>,
  ) -> Self {
    Self {
      mastering,
      content_light,
    }
  }

  /// Returns the mastering display color volume, if present.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn mastering(&self) -> Option<MasteringDisplay> {
    self.mastering
  }

  /// Returns the content light level metadata, if present.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn content_light(&self) -> Option<ContentLightLevel> {
    self.content_light
  }

  /// Sets the mastering display (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_mastering(mut self, v: Option<MasteringDisplay>) -> Self {
    self.mastering = v;
    self
  }

  /// Sets the content light level (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_content_light(mut self, v: Option<ContentLightLevel>) -> Self {
    self.content_light = v;
    self
  }

  /// Sets the mastering display in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_mastering(&mut self, v: Option<MasteringDisplay>) -> &mut Self {
    self.mastering = v;
    self
  }

  /// Sets the content light level in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_content_light(&mut self, v: Option<ContentLightLevel>) -> &mut Self {
    self.content_light = v;
    self
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

  #[test]
  fn enum_u32_round_trips_and_defaults_at_zero() {
    // Every default maps to wire id 0.
    assert_eq!(ColorMatrix::default().to_u32(), 0);
    assert_eq!(ColorPrimaries::default().to_u32(), 0);
    assert_eq!(ColorTransfer::default().to_u32(), 0);
    assert_eq!(ColorRange::default().to_u32(), 0);
    assert_eq!(ChromaLocation::default().to_u32(), 0);
    assert_eq!(DcpTargetGamut::default().to_u32(), 0);

    for m in [
      ColorMatrix::Bt709,
      ColorMatrix::Bt601,
      ColorMatrix::Bt2020Ncl,
      ColorMatrix::Smpte240m,
      ColorMatrix::Fcc,
      ColorMatrix::YCgCo,
    ] {
      assert_eq!(ColorMatrix::from_u32(m.to_u32()), m);
    }
    for p in [
      ColorPrimaries::Unspecified,
      ColorPrimaries::Bt709,
      ColorPrimaries::Bt2020,
      ColorPrimaries::SmpteEg432,
      ColorPrimaries::Ebu3213E,
    ] {
      assert_eq!(ColorPrimaries::from_u32(p.to_u32()), p);
    }
    for t in [
      ColorTransfer::Unspecified,
      ColorTransfer::Bt709,
      ColorTransfer::SmpteSt2084Pq,
      ColorTransfer::AribStdB67Hlg,
    ] {
      assert_eq!(ColorTransfer::from_u32(t.to_u32()), t);
    }
    for r in [ColorRange::Unspecified, ColorRange::Limited, ColorRange::Full] {
      assert_eq!(ColorRange::from_u32(r.to_u32()), r);
    }
    for c in [
      ChromaLocation::Unspecified,
      ChromaLocation::Left,
      ChromaLocation::Bottom,
    ] {
      assert_eq!(ChromaLocation::from_u32(c.to_u32()), c);
    }
    for g in [
      DcpTargetGamut::DciP3,
      DcpTargetGamut::Rec709,
      DcpTargetGamut::Rec2020,
    ] {
      assert_eq!(DcpTargetGamut::from_u32(g.to_u32()), g);
    }

    // Unknown values fall back to the default variant.
    assert_eq!(ColorMatrix::from_u32(9_999), ColorMatrix::Bt709);
    assert_eq!(ColorPrimaries::from_u32(9_999), ColorPrimaries::Unspecified);
    assert_eq!(ColorTransfer::from_u32(9_999), ColorTransfer::Unspecified);
    assert_eq!(ColorRange::from_u32(9_999), ColorRange::Unspecified);
    assert_eq!(ChromaLocation::from_u32(9_999), ChromaLocation::Unspecified);
    assert_eq!(DcpTargetGamut::from_u32(9_999), DcpTargetGamut::DciP3);
  }

  #[test]
  fn content_light_level_construct_and_builders() {
    let c = ContentLightLevel::new(1000, 400);
    assert_eq!(c.max_cll(), 1000);
    assert_eq!(c.max_fall(), 400);
    assert_eq!(ContentLightLevel::default(), ContentLightLevel::new(0, 0));
    let c2 = ContentLightLevel::default()
      .with_max_cll(4000)
      .with_max_fall(1000);
    assert_eq!((c2.max_cll(), c2.max_fall()), (4000, 1000));
    let mut c3 = ContentLightLevel::default();
    c3.set_max_cll(600).set_max_fall(200);
    assert_eq!((c3.max_cll(), c3.max_fall()), (600, 200));
  }

  #[test]
  fn chroma_coord_and_mastering_display() {
    let red = ChromaCoord::new(34000, 16000);
    let green = ChromaCoord::new(13250, 34500);
    let blue = ChromaCoord::new(7500, 3000);
    let wp = ChromaCoord::default().with_x(15635).with_y(16450);
    assert_eq!((red.x(), red.y()), (34000, 16000));
    assert_eq!((wp.x(), wp.y()), (15635, 16450));

    const MD: MasteringDisplay = MasteringDisplay::new(
      [
        ChromaCoord::new(34000, 16000),
        ChromaCoord::new(13250, 34500),
        ChromaCoord::new(7500, 3000),
      ],
      ChromaCoord::new(15635, 16450),
      10_000_000,
      50,
    );
    assert_eq!(MD.display_primaries()[1], green);
    assert_eq!(MD.white_point(), ChromaCoord::new(15635, 16450));
    assert_eq!(MD.max_luminance(), 10_000_000);
    assert_eq!(MD.min_luminance(), 50);

    let mut md = MasteringDisplay::default();
    md.set_display_primaries([red, green, blue])
      .set_white_point(wp)
      .set_max_luminance(4_000_0000)
      .set_min_luminance(5);
    assert_eq!(md.display_primaries()[2], blue);
    assert_eq!(md.min_luminance(), 5);
  }

  #[test]
  fn hdr_static_metadata_optionals() {
    let empty = HdrStaticMetadata::default();
    assert!(empty.mastering().is_none());
    assert!(empty.content_light().is_none());

    let cll = ContentLightLevel::new(1000, 400);
    let md = MasteringDisplay::new(
      [ChromaCoord::new(1, 2), ChromaCoord::new(3, 4), ChromaCoord::new(5, 6)],
      ChromaCoord::new(7, 8),
      9,
      10,
    );
    let h = HdrStaticMetadata::new(Some(md), Some(cll));
    assert_eq!(h.mastering(), Some(md));
    assert_eq!(h.content_light(), Some(cll));

    let h2 = HdrStaticMetadata::default()
      .with_content_light(Some(cll))
      .with_mastering(None);
    assert_eq!(h2.content_light(), Some(cll));
    assert!(h2.mastering().is_none());
  }
}
