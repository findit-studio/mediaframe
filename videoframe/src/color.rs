//! Color metadata: enums for matrix, primaries, transfer, range, and
//! chroma location — all closed-form per ITU-T H.273.

use derive_more::{Display, IsVariant};

/// Base id for **videoframe-domain** colour concepts that have no
/// ITU-T H.273 / FFmpeg `AVCol*` code point.
///
/// videoframe is a *superset domain vocabulary*, not an `AVColorSpace`
/// mirror: it serves FFmpeg **and** future RAW SDK backends (R3D /
/// BRAW / ProRes RAW) whose colour science H.273 does not enumerate.
///
/// - **H.273 / FFmpeg code points** use FFmpeg's own numbers (all
///   `< DOMAIN_EXT_BASE`, xtask-verified against the pinned FFmpeg
///   n8.1 `libavutil/pixfmt.h`).
/// - **videoframe-domain concepts** FFmpeg does not enumerate (e.g.
///   the unified [`ColorMatrix::Bt601`]; future RAW camera colour
///   science) get stable ids with **bit 31 set** (`>= DOMAIN_EXT_BASE`).
///   FFmpeg itself reserves `AVCOL_*_EXT_BASE = 256` for its own
///   extensions, so this clearly-disjoint high base never collides.
///
/// Domain ids are **append-only**, stable, and round-trip losslessly.
/// They are **never produced by the FFmpeg ingest path**:
/// `from_u32` of any FFmpeg / H.273 code returns the H.273 variant,
/// never a domain variant. Per-enum domain offsets (`DOMAIN_EXT_BASE
/// + n`) are append-only and documented at each enum.
pub const DOMAIN_EXT_BASE: u32 = 0x8000_0000;

/// Color matrix coefficients per ITU-T H.273 MatrixCoefficients
/// (Table 4) / ISO/IEC 23001-8.
///
/// Read from `AVFrame.colorspace` / `VideoColorSpace.matrix` /
/// `kCVImageBufferYCbCrMatrixKey`.
///
/// This type's stored `Default` is [`Self::Unspecified`] (FFmpeg
/// `AVCOL_SPC_UNSPECIFIED`, code `2`). For `AVCOL_SPC_UNSPECIFIED`,
/// FFmpeg's convention picks BT.709 for sources with `height >= 720`
/// and BT.601 otherwise — that is a **consumer-side resolution** of
/// `Unspecified` applied at read time, *not* a stored value (the
/// `Bt601` reference there denotes the [`Self::Bt601`] domain
/// variant below).
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVColorSpace` code points** (ITU-T H.273 MatrixCoefficients);
/// FFmpeg is the source of truth (the downstream consumer reads these
/// via a `buffa` `extern_path`). [`Self::Bt601`] is a
/// **videoframe-domain** id (no H.273 code; see [`DOMAIN_EXT_BASE`]).
/// [`Self::Unknown`] carries any unrecognised code through unchanged
/// so the round-trip is lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorMatrix {
  /// Unknown / unrecognised `AVColorSpace` code. The wrapped `u32`
  /// is the original value passed to [`Self::from_u32`] — preserved
  /// so round-tripping unknown codes is lossless.
  Unknown(u32),
  /// GBR (sRGB / ST 428-1); FFmpeg `AVCOL_SPC_RGB` (code `0`).
  Rgb,
  /// **videoframe-domain** unified ITU-R BT.601 YCbCr matrix
  /// (Kr=0.299, Kb=0.114). H.273 has **no single BT.601 code**: it
  /// splits into [`Self::Bt470Bg`] (625-line) and [`Self::Smpte170M`]
  /// (525-line), which carry the *identical* coefficients. The FFmpeg
  /// ingest path therefore yields those two, **never** `Bt601`;
  /// RAW / SDK backends and explicit domain tagging use `Bt601`. Its
  /// id is in the domain-extension band (see [`DOMAIN_EXT_BASE`]),
  /// never an FFmpeg code.
  Bt601,
  /// ITU-R BT.709 (HDTV).
  Bt709,
  /// Unspecified — caller infers (FFmpeg's `height >= 720` →
  /// BT.709, else BT.601 rule is applied downstream).
  Unspecified,
  /// FCC CFR 47 §73.682 (legacy NTSC, very close to BT.601 numerically).
  Fcc,
  /// ITU-R BT.470 System BG / BT.601 625 (SDTV; identical
  /// coefficients to SMPTE170M).
  Bt470Bg,
  /// SMPTE 170M / BT.601 525 (SDTV).
  Smpte170M,
  /// SMPTE 240M (legacy 1990s HDTV).
  Smpte240m,
  /// YCgCo per ITU-T H.273 MatrixCoefficients = 8.
  YCgCo,
  /// ITU-R BT.2020 non-constant-luminance (UHDTV / HDR10).
  Bt2020Ncl,
  /// ITU-R BT.2020 constant-luminance.
  Bt2020Cl,
  /// SMPTE 2085 (Y'D'zD'x).
  Smpte2085,
  /// Chromaticity-derived non-constant luminance.
  ChromaDerivedNcl,
  /// Chromaticity-derived constant luminance.
  ChromaDerivedCl,
  /// ITU-R BT.2100-0 ICtCp.
  Ictcp,
  /// SMPTE ST 2128 IPT-C2.
  IptC2,
  /// YCgCo-R, even bit addition.
  YCgCoRe,
  /// YCgCo-R, odd bit addition.
  YCgCoRo,
}

impl Default for ColorMatrix {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl ColorMatrix {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_SPC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::Rgb => "rgb",
      Self::Bt601 => "bt601",
      Self::Bt709 => "bt709",
      Self::Unspecified => "unspecified",
      Self::Fcc => "fcc",
      Self::Bt470Bg => "bt470bg",
      Self::Smpte170M => "smpte170m",
      Self::Smpte240m => "smpte240m",
      Self::YCgCo => "ycgco",
      Self::Bt2020Ncl => "bt2020nc",
      Self::Bt2020Cl => "bt2020c",
      Self::Smpte2085 => "smpte2085",
      Self::ChromaDerivedNcl => "chroma-derived-nc",
      Self::ChromaDerivedCl => "chroma-derived-c",
      Self::Ictcp => "ictcp",
      Self::IptC2 => "ipt-c2",
      Self::YCgCoRe => "ycgco-re",
      Self::YCgCoRo => "ycgco-ro",
    }
  }

  /// Stable wire id — the **FFmpeg `AVColorSpace` code point**
  /// (ITU-T H.273 MatrixCoefficients) for the H.273 variants, or a
  /// **videoframe-domain** id `>= DOMAIN_EXT_BASE` for concepts
  /// H.273 does not enumerate ([`Self::Bt601`] is the first, at
  /// offset `0`). [`Self::Unknown`] carries its original `u32`
  /// through unchanged so `from_u32(to_u32(x)) == x` for every `x`.
  /// Note `Rgb` is code `0` (non-default, so the `buffa` codec
  /// encodes it explicitly).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Rgb => 0,
      // domain ext offsets (append-only): 0 = Bt601
      Self::Bt601 => DOMAIN_EXT_BASE,
      Self::Bt709 => 1,
      Self::Unspecified => 2,
      Self::Fcc => 4,
      Self::Bt470Bg => 5,
      Self::Smpte170M => 6,
      Self::Smpte240m => 7,
      Self::YCgCo => 8,
      Self::Bt2020Ncl => 9,
      Self::Bt2020Cl => 10,
      Self::Smpte2085 => 11,
      Self::ChromaDerivedNcl => 12,
      Self::ChromaDerivedCl => 13,
      Self::Ictcp => 14,
      Self::IptC2 => 15,
      Self::YCgCoRe => 16,
      Self::YCgCoRo => 17,
    }
  }

  /// Decodes from the code produced by [`Self::to_u32`]. FFmpeg
  /// `AVColorSpace` codes map to their H.273 variant — in particular
  /// `5`/`6` decode to [`Self::Bt470Bg`]/[`Self::Smpte170M`],
  /// **never** [`Self::Bt601`] (the FFmpeg ingest path never yields a
  /// domain variant). [`DOMAIN_EXT_BASE`] (offset `0`) decodes to the
  /// videoframe-domain [`Self::Bt601`]. Any other unrecognised code
  /// (including reserved code `3`, or an unassigned `>=
  /// DOMAIN_EXT_BASE` id) maps to [`Self::Unknown`] carrying the
  /// original value, so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Rgb,
      1 => Self::Bt709,
      2 => Self::Unspecified,
      4 => Self::Fcc,
      5 => Self::Bt470Bg,
      6 => Self::Smpte170M,
      7 => Self::Smpte240m,
      8 => Self::YCgCo,
      9 => Self::Bt2020Ncl,
      10 => Self::Bt2020Cl,
      11 => Self::Smpte2085,
      12 => Self::ChromaDerivedNcl,
      13 => Self::ChromaDerivedCl,
      14 => Self::Ictcp,
      15 => Self::IptC2,
      16 => Self::YCgCoRe,
      17 => Self::YCgCoRo,
      // videoframe-domain ids (append-only): DOMAIN_EXT_BASE + 0 =
      // Bt601. Never reached by the FFmpeg ingest path above.
      DOMAIN_EXT_BASE => Self::Bt601,
      _ => Self::Unknown(v),
    }
  }
}

/// Color primaries per ITU-T H.273 ColourPrimaries (Table 2) /
/// ISO/IEC 23001-8.
///
/// Read from `AVFrame.color_primaries` / `VideoColorSpace.primaries` /
/// `kCVImageBufferColorPrimariesKey`.
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVColorPrimaries` code points** (ITU-T H.273 ColourPrimaries);
/// FFmpeg is the source of truth (the downstream consumer reads these
/// via a `buffa` `extern_path`). `Default` is [`Self::Unspecified`]
/// (FFmpeg `AVCOL_PRI_UNSPECIFIED`, code `2`); [`Self::Unknown`]
/// carries any unrecognised code (incl. reserved `0`/`3`) through
/// unchanged so the round-trip is lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorPrimaries {
  /// Unknown / unrecognised `AVColorPrimaries` code (incl. the
  /// reserved `0`/`3`). The wrapped `u32` is the original value
  /// passed to [`Self::from_u32`] — preserved so round-tripping
  /// unknown codes is lossless.
  Unknown(u32),
  /// ITU-R BT.709 (HDTV).
  Bt709,
  /// Unspecified — caller infers from height.
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
  /// EBU Tech. 3213-E (legacy) / JEDEC P22.
  Ebu3213E,
}

impl Default for ColorPrimaries {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl ColorPrimaries {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_PRI_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
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

  /// Stable wire id — the **FFmpeg `AVColorPrimaries` code point**
  /// (ITU-T H.273 ColourPrimaries). [`Self::Unknown`] carries its
  /// original `u32` through unchanged so `from_u32(to_u32(x)) == x`
  /// for every `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Bt709 => 1,
      Self::Unspecified => 2,
      Self::Bt470M => 4,
      Self::Bt470Bg => 5,
      Self::Smpte170M => 6,
      Self::Smpte240M => 7,
      Self::Film => 8,
      Self::Bt2020 => 9,
      Self::SmpteSt428 => 10,
      Self::SmpteRp431 => 11,
      Self::SmpteEg432 => 12,
      Self::Ebu3213E => 22,
    }
  }

  /// Decodes from the FFmpeg `AVColorPrimaries` code produced by
  /// [`Self::to_u32`]. Unrecognised codes (including reserved `0`
  /// and `3`) map to [`Self::Unknown`] carrying the original value,
  /// so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Bt709,
      2 => Self::Unspecified,
      4 => Self::Bt470M,
      5 => Self::Bt470Bg,
      6 => Self::Smpte170M,
      7 => Self::Smpte240M,
      8 => Self::Film,
      9 => Self::Bt2020,
      10 => Self::SmpteSt428,
      11 => Self::SmpteRp431,
      12 => Self::SmpteEg432,
      22 => Self::Ebu3213E,
      _ => Self::Unknown(v),
    }
  }
}

/// Transfer characteristics per ITU-T H.273 (Table 3).
///
/// Read from `AVFrame.color_trc` / `VideoColorSpace.transfer` /
/// `kCVImageBufferTransferFunctionKey`.
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVColorTransferCharacteristic` code points** (ITU-T H.273
/// TransferCharacteristics); FFmpeg is the source of truth (the
/// downstream consumer reads these via a `buffa` `extern_path`).
/// `Default` is [`Self::Unspecified`] (FFmpeg
/// `AVCOL_TRC_UNSPECIFIED`, code `2`); [`Self::Unknown`] carries any
/// unrecognised code (incl. reserved `0`/`3`) through unchanged so
/// the round-trip is lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorTransfer {
  /// Unknown / unrecognised `AVColorTransferCharacteristic` code
  /// (incl. the reserved `0`/`3`). The wrapped `u32` is the original
  /// value passed to [`Self::from_u32`] — preserved so round-tripping
  /// unknown codes is lossless.
  Unknown(u32),
  /// ITU-R BT.709.
  Bt709,
  /// Unspecified.
  Unspecified,
  /// BT.470 System M (gamma 2.2); FFmpeg `AVCOL_TRC_GAMMA22`.
  Gamma22,
  /// BT.470 System BG (gamma 2.8); FFmpeg `AVCOL_TRC_GAMMA28`.
  Gamma28,
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

impl Default for ColorTransfer {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl ColorTransfer {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_TRC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::Bt709 => "bt709",
      Self::Unspecified => "unspecified",
      Self::Gamma22 => "gamma22",
      Self::Gamma28 => "gamma28",
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

  /// Stable wire id — the **FFmpeg
  /// `AVColorTransferCharacteristic` code point** (ITU-T H.273
  /// TransferCharacteristics). [`Self::Unknown`] carries its original
  /// `u32` through unchanged so `from_u32(to_u32(x)) == x` for every
  /// `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Bt709 => 1,
      Self::Unspecified => 2,
      Self::Gamma22 => 4,
      Self::Gamma28 => 5,
      Self::Smpte170M => 6,
      Self::Smpte240M => 7,
      Self::Linear => 8,
      Self::Log100 => 9,
      Self::Log316 => 10,
      Self::Iec6196624 => 11,
      Self::Bt1361Ecg => 12,
      Self::Iec6196621 => 13,
      Self::Bt2020_10Bit => 14,
      Self::Bt2020_12Bit => 15,
      Self::SmpteSt2084Pq => 16,
      Self::SmpteSt428 => 17,
      Self::AribStdB67Hlg => 18,
    }
  }

  /// Decodes from the FFmpeg `AVColorTransferCharacteristic` code
  /// produced by [`Self::to_u32`]. Unrecognised codes (including
  /// reserved `0` and `3`) map to [`Self::Unknown`] carrying the
  /// original value, so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Bt709,
      2 => Self::Unspecified,
      4 => Self::Gamma22,
      5 => Self::Gamma28,
      6 => Self::Smpte170M,
      7 => Self::Smpte240M,
      8 => Self::Linear,
      9 => Self::Log100,
      10 => Self::Log316,
      11 => Self::Iec6196624,
      12 => Self::Bt1361Ecg,
      13 => Self::Iec6196621,
      14 => Self::Bt2020_10Bit,
      15 => Self::Bt2020_12Bit,
      16 => Self::SmpteSt2084Pq,
      17 => Self::SmpteSt428,
      18 => Self::AribStdB67Hlg,
      _ => Self::Unknown(v),
    }
  }
}

/// Sample range — limited (TV / studio swing) vs. full (PC).
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVColorRange` code points** (`UNSPECIFIED`=0, `MPEG`=1,
/// `JPEG`=2); FFmpeg is the source of truth. `Default` is
/// [`Self::Unspecified`] (FFmpeg `AVCOL_RANGE_UNSPECIFIED`, code
/// `0`); [`Self::Unknown`] carries any unrecognised code through
/// unchanged so the round-trip is lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ColorRange {
  /// Unknown / unrecognised `AVColorRange` code. The wrapped `u32`
  /// is the original value passed to [`Self::from_u32`] — preserved
  /// so round-tripping unknown codes is lossless.
  Unknown(u32),
  /// Unspecified — caller assumes Limited.
  Unspecified,
  /// Limited / studio swing (8-bit luma 16..235, chroma 16..240);
  /// FFmpeg `AVCOL_RANGE_MPEG`.
  Limited,
  /// Full / PC swing (8-bit 0..255); FFmpeg `AVCOL_RANGE_JPEG`.
  Full,
}

impl Default for ColorRange {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl ColorRange {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_RANGE_*` slug; `tv` / `pc`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::Unspecified => "unspecified",
      Self::Limited => "tv",
      Self::Full => "pc",
    }
  }

  /// Stable wire id — the **FFmpeg `AVColorRange` code point**.
  /// [`Self::Unknown`] carries its original `u32` through unchanged
  /// so `from_u32(to_u32(x)) == x` for every `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Unspecified => 0,
      Self::Limited => 1,
      Self::Full => 2,
    }
  }

  /// Decodes from the FFmpeg `AVColorRange` code produced by
  /// [`Self::to_u32`]. Unrecognised codes map to [`Self::Unknown`]
  /// carrying the original value, so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Unspecified,
      1 => Self::Limited,
      2 => Self::Full,
      _ => Self::Unknown(v),
    }
  }
}

/// Chroma sample location (for subsampled YUV formats).
///
/// Aligns with H.265 SPS chroma_loc / FFmpeg `AVChromaLocation`.
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVChromaLocation` code points**; FFmpeg is the source of truth.
/// `Default` is [`Self::Unspecified`] (FFmpeg
/// `AVCHROMA_LOC_UNSPECIFIED`, code `0`); [`Self::Unknown`] carries
/// any unrecognised code through unchanged so the round-trip is
/// lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ChromaLocation {
  /// Unknown / unrecognised `AVChromaLocation` code. The wrapped
  /// `u32` is the original value passed to [`Self::from_u32`] —
  /// preserved so round-tripping unknown codes is lossless.
  Unknown(u32),
  /// Unspecified.
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

impl Default for ChromaLocation {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl ChromaLocation {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCHROMA_LOC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unknown(_) => "unknown",
      Self::Unspecified => "unspecified",
      Self::Left => "left",
      Self::Center => "center",
      Self::TopLeft => "topleft",
      Self::Top => "top",
      Self::BottomLeft => "bottomleft",
      Self::Bottom => "bottom",
    }
  }

  /// Stable wire id — the **FFmpeg `AVChromaLocation` code point**.
  /// [`Self::Unknown`] carries its original `u32` through unchanged
  /// so `from_u32(to_u32(x)) == x` for every `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::Unspecified => 0,
      Self::Left => 1,
      Self::Center => 2,
      Self::TopLeft => 3,
      Self::Top => 4,
      Self::BottomLeft => 5,
      Self::Bottom => 6,
    }
  }

  /// Decodes from the FFmpeg `AVChromaLocation` code produced by
  /// [`Self::to_u32`]. Unrecognised codes map to [`Self::Unknown`]
  /// carrying the original value, so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Unspecified,
      1 => Self::Left,
      2 => Self::Center,
      3 => Self::TopLeft,
      4 => Self::Top,
      5 => Self::BottomLeft,
      6 => Self::Bottom,
      _ => Self::Unknown(v),
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
  /// Every field — including `matrix` — stores the FFmpeg
  /// `UNSPECIFIED` code; this is exactly `#[derive(Default)]` since
  /// each enum's `Default` is now its `Unspecified` variant. The
  /// FFmpeg BT.709-vs-BT.601-by-height fallback for an unspecified
  /// matrix is a **consumer** concern applied at read time, not
  /// stored here.
  pub const UNSPECIFIED: Self = Self {
    primaries: ColorPrimaries::Unspecified,
    transfer: ColorTransfer::Unspecified,
    matrix: ColorMatrix::Unspecified,
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
/// This enum has **no FFmpeg analog** (it selects a videoframe XYZ →
/// RGB matrix); it keeps its own videoframe-local wire numbering
/// (`DciP3`=0, `Rec709`=1, `Rec2020`=2) rather than an FFmpeg code.
/// `Default` is [`Self::DciP3`]; [`Self::Unknown`] carries any
/// unrecognised id through unchanged so the round-trip is lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
#[non_exhaustive]
pub enum DcpTargetGamut {
  /// Unknown / unrecognised wire id. The wrapped `u32` is the
  /// original value passed to [`Self::from_u32`] — preserved so
  /// round-tripping unknown ids is lossless.
  Unknown(u32),
  /// **DCI-P3 (theatrical, DCI white)** — the SMPTE ST 428-1 / RP
  /// 431-2 §5.1 D-Cinema decode target. White point is **DCI white**
  /// `(0.314, 0.351)` (~6300 K), *not* D65. Default for `xyz12_to`
  /// when callers do not opt into a re-target. **Distinct from
  /// Display-P3** (which re-uses the P3 primaries with a D65 white
  /// point and is the Apple / web `display-p3` colour space) — for
  /// sRGB / web preview select [`Self::Rec709`] instead.
  DciP3,
  /// **Rec.709 / sRGB** (D65) — for sRGB-target deliverables and web
  /// preview.
  Rec709,
  /// **Rec.2020** (D65) — for HDR theatrical / archival.
  Rec2020,
}

impl Default for DcpTargetGamut {
  #[inline]
  fn default() -> Self {
    Self::DciP3
  }
}

impl DcpTargetGamut {
  /// Returns the default DCP mastering gamut (`DciP3`). Intended for
  /// `Default`-style fallthrough when callers do not override the
  /// gamut explicitly.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn default_dcp() -> Self {
    Self::DciP3
  }

  /// Stable videoframe-local wire id (no FFmpeg analog); `DciP3`
  /// (the default) is `0`. [`Self::Unknown`] carries its original
  /// `u32` through unchanged so `from_u32(to_u32(x)) == x` for every
  /// `x`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Unknown(v) => *v,
      Self::DciP3 => 0,
      Self::Rec709 => 1,
      Self::Rec2020 => 2,
    }
  }

  /// Decodes from the videoframe-local wire id produced by
  /// [`Self::to_u32`]. Unrecognised ids map to [`Self::Unknown`]
  /// carrying the original value, so the round-trip is lossless.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::DciP3,
      1 => Self::Rec709,
      2 => Self::Rec2020,
      _ => Self::Unknown(v),
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
/// Both `x` and `y` are in **0.00002 increments** (the floating
/// value is `raw / 50000.0`), matching the spec-integer encoding of
/// FFmpeg's `AVMasteringDisplayMetadata` (`AVRational`s of
/// `n/50000`). In-range ST 2086 values fit a `u16` (≤ 50000), but
/// the buffa wire field is `uint32`; storage is **`u32` so any
/// out-of-range / future / corrupt producer value round-trips
/// losslessly** rather than being silently saturated (Codex
/// adversarial-review F3). Validity is a separate concern from
/// preservation — see [`HdrStaticMetadata`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChromaCoord {
  x: u32,
  y: u32,
}

impl ChromaCoord {
  /// Constructs a `ChromaCoord` from raw ST 2086 units (0.00002
  /// increments; floating value = `raw / 50000.0`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }

  /// Returns the `x` coordinate in ST 2086 units (0.00002
  /// increments).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn x(&self) -> u32 {
    self.x
  }

  /// Returns the `y` coordinate in ST 2086 units (0.00002
  /// increments).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> u32 {
    self.y
  }

  /// Sets `x` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_x(mut self, x: u32) -> Self {
    self.x = x;
    self
  }

  /// Sets `y` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_y(mut self, y: u32) -> Self {
    self.y = y;
    self
  }

  /// Sets `x` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_x(&mut self, x: u32) -> &mut Self {
    self.x = x;
    self
  }

  /// Sets `y` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_y(&mut self, y: u32) -> &mut Self {
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
    // The five FFmpeg colour enums default to their `Unspecified`
    // variant (FFmpeg `UNSPECIFIED` code: 2 for primaries/transfer/
    // matrix, 0 for range/chroma).
    assert!(matches!(ColorMatrix::default(), ColorMatrix::Unspecified));
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
    // `DcpTargetGamut` has no FFmpeg analog; its default is `DciP3`.
    assert!(matches!(DcpTargetGamut::default(), DcpTargetGamut::DciP3));
  }

  #[test]
  fn is_variant_helpers_compile_for_each_enum() {
    assert!(ColorMatrix::Bt709.is_bt_709());
    assert!(ColorMatrix::Rgb.is_rgb());
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
  fn color_info_default_is_all_unspecified() {
    let ci = ColorInfo::default();
    assert_eq!(ci, ColorInfo::UNSPECIFIED);
    assert!(ci.primaries().is_unspecified());
    // Matrix is now stored as `Unspecified` too (the FFmpeg
    // height-fallback is a consumer concern, not stored).
    assert!(ci.matrix().is_unspecified());
    assert!(ci.transfer().is_unspecified());
    assert!(ci.range().is_unspecified());
    assert!(ci.chroma_location().is_unspecified());
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
        ColorMatrix::Bt709.as_str(),
        format!("{}", ColorMatrix::Bt709),
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
    // Pre-existing slugs are byte-stable (no churn).
    assert_eq!(ColorMatrix::Bt2020Ncl.as_str(), "bt2020nc");
    assert_eq!(ColorMatrix::Smpte240m.as_str(), "smpte240m");
    assert_eq!(ColorMatrix::YCgCo.as_str(), "ycgco");
    assert_eq!(ColorPrimaries::SmpteSt428.as_str(), "smpte428");
    assert_eq!(ColorTransfer::SmpteSt2084Pq.as_str(), "smpte2084");
    assert_eq!(ColorTransfer::Bt2020_10Bit.as_str(), "bt2020-10");
    // `Gamma22`/`Gamma28` keep the pre-existing gamma slugs.
    assert_eq!(ColorTransfer::Gamma22.as_str(), "gamma22");
    assert_eq!(ColorTransfer::Gamma28.as_str(), "gamma28");
    assert_eq!(ColorRange::Limited.as_str(), "tv");
    assert_eq!(ColorRange::Full.as_str(), "pc");
    assert_eq!(ChromaLocation::TopLeft.as_str(), "topleft");
  }

  #[test]
  fn enum_u32_uses_ffmpeg_codes_and_round_trips() {
    // `to_u32()` returns the real FFmpeg n8.1 code point for the
    // named variants (spot-checks against libavutil/pixfmt.h).
    assert_eq!(ColorPrimaries::Unspecified.to_u32(), 2);
    assert_eq!(ColorPrimaries::Bt709.to_u32(), 1);
    assert_eq!(ColorPrimaries::Ebu3213E.to_u32(), 22);
    assert_eq!(ColorTransfer::Unspecified.to_u32(), 2);
    assert_eq!(ColorTransfer::SmpteSt2084Pq.to_u32(), 16);
    assert_eq!(ColorTransfer::AribStdB67Hlg.to_u32(), 18);
    assert_eq!(ColorMatrix::Rgb.to_u32(), 0);
    assert_eq!(ColorMatrix::Unspecified.to_u32(), 2);
    assert_eq!(ColorMatrix::Ictcp.to_u32(), 14);
    assert_eq!(ColorRange::Unspecified.to_u32(), 0);
    assert_eq!(ColorRange::Limited.to_u32(), 1);
    assert_eq!(ColorRange::Full.to_u32(), 2);
    assert_eq!(ChromaLocation::Unspecified.to_u32(), 0);

    // `default()` is the `Unspecified` variant for the five FFmpeg
    // enums (NOT necessarily wire id 0).
    assert_eq!(ColorMatrix::default(), ColorMatrix::Unspecified);
    assert_eq!(ColorPrimaries::default(), ColorPrimaries::Unspecified);
    assert_eq!(ColorTransfer::default(), ColorTransfer::Unspecified);
    assert_eq!(ColorRange::default(), ColorRange::Unspecified);
    assert_eq!(ChromaLocation::default(), ChromaLocation::Unspecified);
    assert_eq!(DcpTargetGamut::default(), DcpTargetGamut::DciP3);

    // Round-trip `from_u32(to_u32()) == v` for EVERY named variant.
    for m in [
      ColorMatrix::Rgb,
      ColorMatrix::Bt601,
      ColorMatrix::Bt709,
      ColorMatrix::Unspecified,
      ColorMatrix::Fcc,
      ColorMatrix::Bt470Bg,
      ColorMatrix::Smpte170M,
      ColorMatrix::Smpte240m,
      ColorMatrix::YCgCo,
      ColorMatrix::Bt2020Ncl,
      ColorMatrix::Bt2020Cl,
      ColorMatrix::Smpte2085,
      ColorMatrix::ChromaDerivedNcl,
      ColorMatrix::ChromaDerivedCl,
      ColorMatrix::Ictcp,
      ColorMatrix::IptC2,
      ColorMatrix::YCgCoRe,
      ColorMatrix::YCgCoRo,
    ] {
      assert_eq!(ColorMatrix::from_u32(m.to_u32()), m);
    }
    for p in [
      ColorPrimaries::Bt709,
      ColorPrimaries::Unspecified,
      ColorPrimaries::Bt470M,
      ColorPrimaries::Bt470Bg,
      ColorPrimaries::Smpte170M,
      ColorPrimaries::Smpte240M,
      ColorPrimaries::Film,
      ColorPrimaries::Bt2020,
      ColorPrimaries::SmpteSt428,
      ColorPrimaries::SmpteRp431,
      ColorPrimaries::SmpteEg432,
      ColorPrimaries::Ebu3213E,
    ] {
      assert_eq!(ColorPrimaries::from_u32(p.to_u32()), p);
    }
    for t in [
      ColorTransfer::Bt709,
      ColorTransfer::Unspecified,
      ColorTransfer::Gamma22,
      ColorTransfer::Gamma28,
      ColorTransfer::Smpte170M,
      ColorTransfer::Smpte240M,
      ColorTransfer::Linear,
      ColorTransfer::Log100,
      ColorTransfer::Log316,
      ColorTransfer::Iec6196624,
      ColorTransfer::Bt1361Ecg,
      ColorTransfer::Iec6196621,
      ColorTransfer::Bt2020_10Bit,
      ColorTransfer::Bt2020_12Bit,
      ColorTransfer::SmpteSt2084Pq,
      ColorTransfer::SmpteSt428,
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
      ChromaLocation::Center,
      ChromaLocation::TopLeft,
      ChromaLocation::Top,
      ChromaLocation::BottomLeft,
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

    // Unrecognised codes are now LOSSLESS via `Unknown(n)` (no
    // silent collapse to the default), and round-trip exactly.
    assert_eq!(ColorMatrix::from_u32(9_999), ColorMatrix::Unknown(9_999));
    assert_eq!(ColorMatrix::Unknown(9_999).to_u32(), 9_999);
    // Reserved FFmpeg code 3 is Unknown for every FFmpeg enum.
    assert_eq!(ColorPrimaries::from_u32(3), ColorPrimaries::Unknown(3));
    assert_eq!(ColorPrimaries::from_u32(0), ColorPrimaries::Unknown(0));
    assert_eq!(ColorTransfer::from_u32(3), ColorTransfer::Unknown(3));
    assert_eq!(ColorRange::from_u32(7), ColorRange::Unknown(7));
    assert_eq!(ColorRange::Unknown(7).to_u32(), 7);
    assert_eq!(ChromaLocation::from_u32(42), ChromaLocation::Unknown(42));
    assert_eq!(DcpTargetGamut::from_u32(9_999), DcpTargetGamut::Unknown(9_999));
    assert_eq!(DcpTargetGamut::Unknown(9_999).to_u32(), 9_999);
  }

  #[test]
  fn color_matrix_bt601_is_domain_variant() {
    // Released-API slug restored (the public removal is reverted).
    assert_eq!(ColorMatrix::Bt601.as_str(), "bt601");
    #[cfg(feature = "std")]
    {
      use std::format;
      assert_eq!(format!("{}", ColorMatrix::Bt601), "bt601");
    }

    // `Bt601` lives in the videoframe-domain extension band at
    // offset 0, NOT an FFmpeg code; it round-trips losslessly.
    assert_eq!(ColorMatrix::Bt601.to_u32(), DOMAIN_EXT_BASE);
    assert_eq!(ColorMatrix::Bt601.to_u32(), 0x8000_0000);
    assert_eq!(ColorMatrix::from_u32(0x8000_0000), ColorMatrix::Bt601);
    assert_eq!(
      ColorMatrix::from_u32(ColorMatrix::Bt601.to_u32()),
      ColorMatrix::Bt601
    );

    // Regression: FFmpeg codes 5/6 stay BT.470BG / SMPTE170M and are
    // NEVER decoded as the domain `Bt601` (FFmpeg ingest path never
    // yields a domain variant).
    assert_eq!(ColorMatrix::from_u32(5), ColorMatrix::Bt470Bg);
    assert_eq!(ColorMatrix::from_u32(6), ColorMatrix::Smpte170M);
    assert_ne!(ColorMatrix::from_u32(5), ColorMatrix::Bt601);
    assert_ne!(ColorMatrix::from_u32(6), ColorMatrix::Bt601);

    // `Bt601` is NOT the default (stays `Unspecified`).
    assert_eq!(ColorMatrix::default(), ColorMatrix::Unspecified);
    assert_ne!(ColorMatrix::default(), ColorMatrix::Bt601);

    // An unassigned bit-31 id stays lossless `Unknown` and
    // round-trips (domain band is append-only, not exhaustive).
    assert_eq!(
      ColorMatrix::from_u32(0x8000_00FF),
      ColorMatrix::Unknown(0x8000_00FF)
    );
    assert_eq!(ColorMatrix::Unknown(0x8000_00FF).to_u32(), 0x8000_00FF);

    // `is_variant` helper is generated for the new variant.
    assert!(ColorMatrix::Bt601.is_bt_601());
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
