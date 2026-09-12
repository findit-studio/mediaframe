//! Color metadata: enums for matrix, primaries, transfer, range, and
//! chroma location — all closed-form per ITU-T H.273.

use derive_more::{Display, IsVariant};
#[cfg(any(feature = "std", feature = "alloc"))]
use smol_bytes::Utf8Bytes;

/// Base id for **mediaframe-domain** colour concepts that have no
/// ITU-T H.273 / FFmpeg `AVCol*` code point.
///
/// mediaframe is a *superset domain vocabulary*, not an `AVColorSpace`
/// mirror: it serves FFmpeg **and** future RAW SDK backends (R3D /
/// BRAW / ProRes RAW) whose colour science H.273 does not enumerate.
///
/// - **H.273 / FFmpeg code points** use FFmpeg's own numbers (all
///   `< DOMAIN_EXT_BASE`, xtask-verified against the pinned FFmpeg
///   n9.0 `libavutil/pixfmt.h`).
/// - **mediaframe-domain concepts** FFmpeg does not enumerate (e.g.
///   the unified [`Matrix::Bt601`]; future RAW camera colour
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
/// **mediaframe-domain** id (no H.273 code; see [`DOMAIN_EXT_BASE`]).
/// [`Self::Other`] carries any name this build does not enumerate, so
/// the *text* round-trip is lossless; the numeric helpers speak only
/// FFmpeg's code space and return [`None`] outside it.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::matrix")
)]
pub enum Matrix {
  /// GBR (sRGB / ST 428-1); FFmpeg `AVCOL_SPC_RGB` (code `0`).
  Rgb,
  /// **mediaframe-domain** unified ITU-R BT.601 YCbCr matrix
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for Matrix {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl Matrix {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_SPC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
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
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// The **FFmpeg `AVColorSpace` code point**
  /// (ITU-T H.273 MatrixCoefficients) for the H.273 variants, or a
  /// **mediaframe-domain** id `>= DOMAIN_EXT_BASE` for concepts
  /// H.273 does not enumerate ([`Self::Bt601`] is the first, at
  /// offset `0`).
  ///
  /// [`None`] for [`Self::Other`]: it names something FFmpeg has no
  /// code for, and inventing one would lose the name.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
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
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the code produced by [`Self::to_u32`]. FFmpeg
  /// `AVColorSpace` codes map to their H.273 variant — in particular
  /// `5`/`6` decode to [`Self::Bt470Bg`]/[`Self::Smpte170M`],
  /// **never** [`Self::Bt601`] (the FFmpeg ingest path never yields a
  /// domain variant). [`DOMAIN_EXT_BASE`] (offset `0`) decodes to the
  /// mediaframe-domain [`Self::Bt601`]. Any other unrecognised code
  /// (including reserved code `3`, or an unassigned `>=
  /// DOMAIN_EXT_BASE` id) yields [`None`] — a number is FFmpeg's
  /// spelling, not a name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
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
      // mediaframe-domain ids (append-only): DOMAIN_EXT_BASE + 0 =
      // Bt601. Never reached by the FFmpeg ingest path above.
      DOMAIN_EXT_BASE => Self::Bt601,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  Matrix,
  "colour matrix",
  [
    Rgb, Bt601, Bt709, Unspecified, Fcc, Bt470Bg, Smpte170M, Smpte240m, YCgCo,
    Bt2020Ncl, Bt2020Cl, Smpte2085, ChromaDerivedNcl, ChromaDerivedCl, Ictcp,
    IptC2, YCgCoRe, YCgCoRo
  ],
  alloc_escape: Other
);

/// The error [`Matrix`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a colour-matrix name")]
#[non_exhaustive]
pub struct ParseMatrixError;

impl core::str::FromStr for Matrix {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseMatrixError;

  /// Reads a colour-matrix name: the canonical slug [`Self::as_str`]
  /// renders, **or** FFmpeg's spelling of the same code point where the
  /// two differ.
  ///
  /// Emission is injective and canonical — [`Self::as_str`],
  /// [`Display`](core::fmt::Display) and serde render one slug per
  /// variant and never a synonym — so `parse(display(x)) == x` holds for
  /// every named variant. Parse is the wider side: it also takes
  /// `av_color_space_name`'s spellings `gbr` ([`Self::Rgb`]) and
  /// `unknown` ([`Self::Unspecified`]), so a value copied off `ffprobe`
  /// keeps its H.273 code instead of riding [`Self::Other`], which has
  /// no [`Self::to_u32`].
  ///
  /// # Errors
  ///
  /// Returns [`ParseMatrixError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"rgb" => Self::Rgb,
      b"bt601" => Self::Bt601,
      b"bt709" => Self::Bt709,
      b"unspecified" => Self::Unspecified,
      b"fcc" => Self::Fcc,
      b"bt470bg" => Self::Bt470Bg,
      b"smpte170m" => Self::Smpte170M,
      b"smpte240m" => Self::Smpte240m,
      b"ycgco" => Self::YCgCo,
      b"bt2020nc" => Self::Bt2020Ncl,
      b"bt2020c" => Self::Bt2020Cl,
      b"smpte2085" => Self::Smpte2085,
      b"chroma-derived-nc" => Self::ChromaDerivedNcl,
      b"chroma-derived-c" => Self::ChromaDerivedCl,
      b"ictcp" => Self::Ictcp,
      b"ipt-c2" => Self::IptC2,
      b"ycgco-re" => Self::YCgCoRe,
      b"ycgco-ro" => Self::YCgCoRo,

      // FFmpeg's `av_color_space_name` spellings. Accepted, never
      // emitted — keep in step with `MATRIX_FFMPEG_SYNONYMS`.
      b"gbr" => Self::Rgb,
      b"unknown" => Self::Unspecified,

      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseMatrixError),
    })
  }
}

/// FFmpeg's spellings for [`Matrix`] code points whose canonical
/// mediaframe slug differs, as `(ffmpeg_name, canonical_slug)`.
///
/// The crate-wide rule: emit one canonical slug per variant, parse that
/// slug **plus** the documented FFmpeg name. `cfg(test)` because the one
/// consumer is the nail test proving no synonym shadows a canonical
/// slug.
#[cfg(test)]
const MATRIX_FFMPEG_SYNONYMS: &[(&str, &str)] = &[("gbr", "rgb"), ("unknown", "unspecified")];

/// The **closed** set of colour matrices a conversion kernel has
/// coefficients for — the `Copy` selector the row walkers carry.
///
/// [`Matrix`] is the *descriptor* vocabulary: open, `#[non_exhaustive]`,
/// and able to name a matrix nobody tabulates coefficients for
/// ([`Matrix::Other`], plus the H.273 names no conversion kernel in this
/// ecosystem implements). A kernel cannot pick coefficients from a name,
/// so carrying a `Matrix` into a row makes an unconvertible row
/// *representable*, and every consumer then needs a silent fallback arm
/// to survive one. That fallback is where a wrong picture comes from: a
/// caller asking for ICtCp gets BT.709 pixels and no diagnostic.
///
/// `KernelMatrix` is the other half of the pair:
///
/// - **closed** — deliberately *not* `#[non_exhaustive]`, so a
///   downstream kernel's `match` is exhaustive and the compiler proves
///   it handles every coefficient set. Adding a variant here is a
///   breaking change, which is correct: a new coefficient set is
///   precisely the event a kernel must be told about.
/// - **`Copy`** — no heap payload, which is what keeps every generated
///   `*Row<'a>` type `Copy`.
///
/// Convert at the door with [`TryFrom<&Matrix>`](Matrix). That
/// conversion is the *one* place an unsupported matrix is refused, and
/// past it the wrong value is unrepresentable:
///
/// ```
/// // Positive control for the two `compile_fail` blocks below — the
/// // path resolves, so their failure is the missing variant and not a
/// // typo.
/// let _ = mediaframe::color::KernelMatrix::Bt709;
/// ```
///
/// ```compile_fail
/// // No `Other`: a row carrying an unnamed matrix cannot be built.
/// let _ = mediaframe::color::KernelMatrix::Other;
/// ```
///
/// ```compile_fail
/// // Nor is a *named* matrix without tabulated coefficients spellable.
/// let _ = mediaframe::color::KernelMatrix::Ictcp;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelMatrix {
  /// Unified ITU-R BT.601 (`Kr=0.299`, `Kb=0.114`) — see
  /// [`Matrix::Bt601`].
  Bt601,
  /// ITU-R BT.709 (HDTV) — see [`Matrix::Bt709`].
  Bt709,
  /// Unspecified — see [`Matrix::Unspecified`].
  ///
  /// Kept as an explicit arm rather than refused at the door, because
  /// refusing it would be a behaviour change: `Unspecified` is
  /// [`Matrix`]'s own `Default`, it reaches the kernels today, and they
  /// resolve it themselves (FFmpeg's convention is BT.709 for
  /// `height >= 720` and BT.601 below; the kernel this crate feeds
  /// currently resolves it to BT.709 unconditionally). The resolution
  /// stays the consumer's, exactly as it is today — this arm only makes
  /// it *nameable* instead of arriving through a fallback.
  Unspecified,
  /// FCC CFR 47 §73.682 — see [`Matrix::Fcc`]. Numerically a close
  /// approximation of BT.601 and tabulated on the same arm.
  Fcc,
  /// ITU-R BT.470 System BG / BT.601 625 — see [`Matrix::Bt470Bg`].
  /// Coefficients identical to [`Self::Bt601`].
  Bt470Bg,
  /// SMPTE 170M / BT.601 525 — see [`Matrix::Smpte170M`]. Coefficients
  /// identical to [`Self::Bt601`].
  Smpte170M,
  /// SMPTE 240M — see [`Matrix::Smpte240m`].
  Smpte240m,
  /// YCgCo (H.273 `MatrixCoefficients = 8`) — see [`Matrix::YCgCo`].
  YCgCo,
  /// ITU-R BT.2020 non-constant-luminance — see [`Matrix::Bt2020Ncl`].
  Bt2020Ncl,
  /// Chromaticity-derived non-constant luminance — see
  /// [`Matrix::ChromaDerivedNcl`]. The only affine member: its
  /// coefficients are derived from the signalled [`Primaries`] rather
  /// than tabulated.
  ChromaDerivedNcl,
}

/// The error the [`Matrix`] → [`KernelMatrix`] conversion returns: this
/// matrix names no coefficient set a conversion kernel can use.
///
/// Opaque and sealed, like the crate's parse errors — the rejected
/// matrix is deliberately not retained (this type is available at the
/// no-alloc tier, where there is nowhere to put an owned copy, and the
/// caller still holds the [`Matrix`] it passed).
/// `#[non_exhaustive]` keeps it constructible only here, so it can grow
/// structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("this colour matrix has no conversion-kernel coefficients")]
#[non_exhaustive]
pub struct UnsupportedKernelMatrixError;

impl TryFrom<&Matrix> for KernelMatrix {
  type Error = UnsupportedKernelMatrixError;

  /// The open → closed exchange, taken at the kernel door.
  ///
  /// # Errors
  ///
  /// Returns [`UnsupportedKernelMatrixError`] for every [`Matrix`] no
  /// kernel tabulates coefficients for: [`Matrix::Rgb`] (the GBR
  /// identity — not a YCbCr matrix at all), the constant-luminance and
  /// perceptual matrices ([`Matrix::Bt2020Cl`], [`Matrix::Smpte2085`],
  /// [`Matrix::ChromaDerivedCl`], [`Matrix::Ictcp`], [`Matrix::IptC2`],
  /// [`Matrix::YCgCoRe`], [`Matrix::YCgCoRo`]), and [`Matrix::Other`].
  fn try_from(m: &Matrix) -> Result<Self, Self::Error> {
    // Spelled arm by arm rather than with a wildcard: `Matrix` is
    // `#[non_exhaustive]` only for *other* crates, so this match is
    // exhaustiveness-checked here and a new colour matrix cannot be
    // added without someone deciding whether a kernel can convert it.
    Ok(match m {
      Matrix::Bt601 => Self::Bt601,
      Matrix::Bt709 => Self::Bt709,
      Matrix::Unspecified => Self::Unspecified,
      Matrix::Fcc => Self::Fcc,
      Matrix::Bt470Bg => Self::Bt470Bg,
      Matrix::Smpte170M => Self::Smpte170M,
      Matrix::Smpte240m => Self::Smpte240m,
      Matrix::YCgCo => Self::YCgCo,
      Matrix::Bt2020Ncl => Self::Bt2020Ncl,
      Matrix::ChromaDerivedNcl => Self::ChromaDerivedNcl,
      Matrix::Rgb
      | Matrix::Bt2020Cl
      | Matrix::Smpte2085
      | Matrix::ChromaDerivedCl
      | Matrix::Ictcp
      | Matrix::IptC2
      | Matrix::YCgCoRe
      | Matrix::YCgCoRo => return Err(UnsupportedKernelMatrixError),
      #[cfg(any(feature = "std", feature = "alloc"))]
      Matrix::Other(_) => return Err(UnsupportedKernelMatrixError),
    })
  }
}

impl From<KernelMatrix> for Matrix {
  /// Widens back to the descriptor vocabulary. Total and injective —
  /// every [`KernelMatrix`] names exactly one [`Matrix`], so
  /// `KernelMatrix::try_from(&Matrix::from(k)) == Ok(k)`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(k: KernelMatrix) -> Self {
    match k {
      KernelMatrix::Bt601 => Self::Bt601,
      KernelMatrix::Bt709 => Self::Bt709,
      KernelMatrix::Unspecified => Self::Unspecified,
      KernelMatrix::Fcc => Self::Fcc,
      KernelMatrix::Bt470Bg => Self::Bt470Bg,
      KernelMatrix::Smpte170M => Self::Smpte170M,
      KernelMatrix::Smpte240m => Self::Smpte240m,
      KernelMatrix::YCgCo => Self::YCgCo,
      KernelMatrix::Bt2020Ncl => Self::Bt2020Ncl,
      KernelMatrix::ChromaDerivedNcl => Self::ChromaDerivedNcl,
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
/// (FFmpeg `AVCOL_PRI_UNSPECIFIED`, code `2`); [`Self::Other`] carries
/// any name this build does not enumerate, so the *text* round-trip is
/// lossless.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::primaries")
)]
pub enum Primaries {
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for Primaries {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl Primaries {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_PRI_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
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
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// The **FFmpeg `AVColorPrimaries` code point**
  /// (ITU-T H.273 ColourPrimaries).
  ///
  /// [`None`] for [`Self::Other`]: it names something FFmpeg has no
  /// code for, and inventing one would lose the name.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
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
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVColorPrimaries` code produced by
  /// [`Self::to_u32`].
  ///
  /// [`None`] for a code this build names nothing for — a number is
  /// FFmpeg's spelling, not a name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
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
      _ => return None,
    })
  }

  // CIE 1931 xy white points in [`ChromaCoord`] SMPTE ST 2086 units
  // (0.00002 increments; floating value = `raw / 50000.0`), matching
  // FFmpeg `csp.c` `WP_*`. `WHITE_E` is the equal-energy point
  // (exactly 1/3, 1/3); `50000 / 3` rounds to `16667`.
  const WHITE_D65: ChromaCoord = ChromaCoord::new(15635, 16450);
  const WHITE_C: ChromaCoord = ChromaCoord::new(15500, 15800);
  const WHITE_DCI: ChromaCoord = ChromaCoord::new(15700, 17550);
  const WHITE_E: ChromaCoord = ChromaCoord::new(16667, 16667);

  /// CIE 1931 `xy` chromaticities of the **R, G, B** primaries (index
  /// `0` = red, `1` = green, `2` = blue, matching FFmpeg's
  /// `display_primaries` layout) defined by this colour-primaries
  /// standard, per ITU-T H.273 ColourPrimaries / FFmpeg
  /// `av_csp_primaries_desc` (`libavutil/csp.c`).
  ///
  /// Coordinates are in [`ChromaCoord`]'s SMPTE ST 2086 fixed-point
  /// units (0.00002 increments; floating value = `raw / 50000.0`), so
  /// BT.709 red `(0.640, 0.330)` is `(32000, 16500)`.
  ///
  /// Returns [`None`] for [`Self::Unspecified`] and [`Self::Other`],
  /// which carry no defined primaries.
  ///
  /// [`Self::SmpteSt428`] reports FFmpeg's tabulated D-Cinema primaries
  /// (white point E), **not** the CIE XYZ identity that ITU-T H.273
  /// Table 2 lists for ST 428-1 — FFmpeg's `av_csp_primaries_desc` is
  /// the authority here.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn chromaticities(&self) -> Option<[ChromaCoord; 3]> {
    match self {
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => None,
      Self::Unspecified => None,
      Self::Bt709 => Some([
        ChromaCoord::new(32000, 16500),
        ChromaCoord::new(15000, 30000),
        ChromaCoord::new(7500, 3000),
      ]),
      Self::Bt470M => Some([
        ChromaCoord::new(33500, 16500),
        ChromaCoord::new(10500, 35500),
        ChromaCoord::new(7000, 4000),
      ]),
      Self::Bt470Bg => Some([
        ChromaCoord::new(32000, 16500),
        ChromaCoord::new(14500, 30000),
        ChromaCoord::new(7500, 3000),
      ]),
      // SMPTE 170M and 240M share identical primaries (D65).
      Self::Smpte170M | Self::Smpte240M => Some([
        ChromaCoord::new(31500, 17000),
        ChromaCoord::new(15500, 29750),
        ChromaCoord::new(7750, 3500),
      ]),
      Self::Film => Some([
        ChromaCoord::new(34050, 15950),
        ChromaCoord::new(12150, 34600),
        ChromaCoord::new(7250, 2450),
      ]),
      Self::Bt2020 => Some([
        ChromaCoord::new(35400, 14600),
        ChromaCoord::new(8500, 39850),
        ChromaCoord::new(6550, 2300),
      ]),
      Self::SmpteSt428 => Some([
        ChromaCoord::new(36750, 13250),
        ChromaCoord::new(13700, 35900),
        ChromaCoord::new(8350, 450),
      ]),
      // DCI-P3 (RP 431-2) and Display-P3 (EG 432-1) share the P3
      // primaries; they differ only in white point (DCI vs D65).
      Self::SmpteRp431 | Self::SmpteEg432 => Some([
        ChromaCoord::new(34000, 16000),
        ChromaCoord::new(13250, 34500),
        ChromaCoord::new(7500, 3000),
      ]),
      Self::Ebu3213E => Some([
        ChromaCoord::new(31500, 17000),
        ChromaCoord::new(14750, 30250),
        ChromaCoord::new(7750, 3850),
      ]),
    }
  }

  /// CIE 1931 `xy` reference white point defined by this
  /// colour-primaries standard, per ITU-T H.273 / FFmpeg
  /// `av_csp_primaries_desc` (`libavutil/csp.c`).
  ///
  /// Most standards use D65 `(0.3127, 0.3290)`; the exceptions are
  /// [`Self::Bt470M`] / [`Self::Film`] (CIE C), [`Self::SmpteRp431`]
  /// (DCI white `(0.314, 0.351)`), and [`Self::SmpteSt428`]
  /// (equal-energy E `(1/3, 1/3)`). Coordinates use the same
  /// [`ChromaCoord`] ST 2086 units as [`Self::chromaticities`].
  ///
  /// Returns [`None`] for [`Self::Unspecified`] and [`Self::Other`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn white_point(&self) -> Option<ChromaCoord> {
    match self {
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => None,
      Self::Unspecified => None,
      Self::Bt709
      | Self::Bt470Bg
      | Self::Smpte170M
      | Self::Smpte240M
      | Self::Bt2020
      | Self::SmpteEg432
      | Self::Ebu3213E => Some(Self::WHITE_D65),
      Self::Bt470M | Self::Film => Some(Self::WHITE_C),
      Self::SmpteRp431 => Some(Self::WHITE_DCI),
      Self::SmpteSt428 => Some(Self::WHITE_E),
    }
  }

  /// Whether these primaries encode color directly in **CIE 1931 XYZ**
  /// rather than an RGB gamut — i.e. the channels *are* X, Y, Z.
  ///
  /// True only for [`Self::SmpteSt428`] (SMPTE ST 428-1, Digital Cinema),
  /// whose colorimetric primaries are the XYZ axes — chromaticities
  /// `(1, 0)`, `(0, 1)`, `(0, 0)` — not a set of physical RGB primaries.
  ///
  /// This is the colorimetric *interpretation*, distinct from what
  /// [`Self::chromaticities`] returns: that method reports FFmpeg's
  /// tabulated D-Cinema RGB primaries for `SmpteSt428` (mirroring
  /// `av_csp_primaries_desc`, the authority for that method), whereas a
  /// consumer deriving an XYZ↔RGB relationship should treat ST 428-1 as
  /// the XYZ identity. Use this predicate to branch on that distinction
  /// (e.g. skip building an RGB-primaries-derived matrix for XYZ data).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_cie_xyz(&self) -> bool {
    matches!(self, Self::SmpteSt428)
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  Primaries,
  "set of colour primaries",
  [
    Bt709, Unspecified, Bt470M, Bt470Bg, Smpte170M, Smpte240M, Film, Bt2020,
    SmpteSt428, SmpteRp431, SmpteEg432, Ebu3213E
  ],
  alloc_escape: Other
);

/// The error [`Primaries`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a colour-primaries name")]
#[non_exhaustive]
pub struct ParsePrimariesError;

impl core::str::FromStr for Primaries {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParsePrimariesError;

  /// Reads a primaries name: the canonical slug [`Self::as_str`]
  /// renders, **or** FFmpeg's spelling of the same code point where the
  /// two differ.
  ///
  /// Emission is injective and canonical — never a synonym — so
  /// `parse(display(x)) == x` holds for every named variant. Parse also
  /// takes `av_color_primaries_name`'s `unknown`
  /// ([`Self::Unspecified`]); every other name in that table already
  /// matches this vocabulary.
  ///
  /// # Errors
  ///
  /// Returns [`ParsePrimariesError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"bt709" => Self::Bt709,
      b"unspecified" => Self::Unspecified,
      b"bt470m" => Self::Bt470M,
      b"bt470bg" => Self::Bt470Bg,
      b"smpte170m" => Self::Smpte170M,
      b"smpte240m" => Self::Smpte240M,
      b"film" => Self::Film,
      b"bt2020" => Self::Bt2020,
      b"smpte428" => Self::SmpteSt428,
      b"smpte431" => Self::SmpteRp431,
      b"smpte432" => Self::SmpteEg432,
      b"ebu3213" => Self::Ebu3213E,

      // FFmpeg's `av_color_primaries_name` spelling. Accepted, never
      // emitted — keep in step with `PRIMARIES_FFMPEG_SYNONYMS`.
      b"unknown" => Self::Unspecified,

      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParsePrimariesError),
    })
  }
}

/// FFmpeg's spelling for the one [`Primaries`] code point whose
/// canonical mediaframe slug differs, as `(ffmpeg_name,
/// canonical_slug)`. See [`MATRIX_FFMPEG_SYNONYMS`] for the rule.
#[cfg(test)]
const PRIMARIES_FFMPEG_SYNONYMS: &[(&str, &str)] = &[("unknown", "unspecified")];

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
/// `AVCOL_TRC_UNSPECIFIED`, code `2`); [`Self::Other`] carries any
/// name this build does not enumerate, so the *text* round-trip is
/// lossless.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::transfer")
)]
pub enum Transfer {
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for Transfer {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl Transfer {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_TRC_*` slug).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
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
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// The **FFmpeg
  /// `AVColorTransferCharacteristic` code point** (ITU-T H.273
  /// TransferCharacteristics).
  ///
  /// [`None`] for [`Self::Other`]: it names something FFmpeg has no
  /// code for, and inventing one would lose the name.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
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
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVColorTransferCharacteristic` code
  /// produced by [`Self::to_u32`].
  ///
  /// [`None`] for a code this build names nothing for — a number is
  /// FFmpeg's spelling, not a name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
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
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  Transfer,
  "transfer characteristic",
  [
    Bt709, Unspecified, Gamma22, Gamma28, Smpte170M, Smpte240M, Linear,
    Log100, Log316, Iec6196624, Bt1361Ecg, Iec6196621, Bt2020_10Bit,
    Bt2020_12Bit, SmpteSt2084Pq, SmpteSt428, AribStdB67Hlg
  ],
  alloc_escape: Other
);

/// The error [`Transfer`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a transfer-characteristics name")]
#[non_exhaustive]
pub struct ParseTransferError;

impl core::str::FromStr for Transfer {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseTransferError;

  /// Reads a transfer-characteristics name: the canonical slug
  /// [`Self::as_str`] renders, **or** FFmpeg's spelling of the same code
  /// point where the two differ.
  ///
  /// Emission is injective and canonical — never a synonym — so
  /// `parse(display(x)) == x` holds for every named variant. Parse also
  /// takes `av_color_transfer_name`'s three divergent spellings:
  /// `unknown` ([`Self::Unspecified`]), and `bt470m` / `bt470bg`, which
  /// FFmpeg uses for the curves this vocabulary spells `gamma22`
  /// ([`Self::Gamma22`]) and `gamma28` ([`Self::Gamma28`]). Neither
  /// `bt470m` nor `bt470bg` names any [`Transfer`] variant of its own —
  /// those spellings belong to [`Primaries`], a different type.
  ///
  /// # Errors
  ///
  /// Returns [`ParseTransferError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"bt709" => Self::Bt709,
      b"unspecified" => Self::Unspecified,
      b"gamma22" => Self::Gamma22,
      b"gamma28" => Self::Gamma28,
      b"smpte170m" => Self::Smpte170M,
      b"smpte240m" => Self::Smpte240M,
      b"linear" => Self::Linear,
      b"log100" => Self::Log100,
      b"log316" => Self::Log316,
      b"iec61966-2-4" => Self::Iec6196624,
      b"bt1361e" => Self::Bt1361Ecg,
      b"iec61966-2-1" => Self::Iec6196621,
      b"bt2020-10" => Self::Bt2020_10Bit,
      b"bt2020-12" => Self::Bt2020_12Bit,
      b"smpte2084" => Self::SmpteSt2084Pq,
      b"smpte428" => Self::SmpteSt428,
      b"arib-std-b67" => Self::AribStdB67Hlg,

      // FFmpeg's `av_color_transfer_name` spellings. Accepted, never
      // emitted — keep in step with `TRANSFER_FFMPEG_SYNONYMS`.
      b"unknown" => Self::Unspecified,
      b"bt470m" => Self::Gamma22,
      b"bt470bg" => Self::Gamma28,

      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseTransferError),
    })
  }
}

/// FFmpeg's spellings for [`Transfer`] code points whose canonical
/// mediaframe slug differs, as `(ffmpeg_name, canonical_slug)`. See
/// [`MATRIX_FFMPEG_SYNONYMS`] for the rule.
#[cfg(test)]
const TRANSFER_FFMPEG_SYNONYMS: &[(&str, &str)] = &[
  ("bt470bg", "gamma28"),
  ("bt470m", "gamma22"),
  ("unknown", "unspecified"),
];

/// Sample range — limited (TV / studio swing) vs. full (PC).
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVColorRange` code points** (`UNSPECIFIED`=0, `MPEG`=1,
/// `JPEG`=2); FFmpeg is the source of truth. `Default` is
/// [`Self::Unspecified`] (FFmpeg `AVCOL_RANGE_UNSPECIFIED`, code
/// `0`); [`Self::Other`] carries any name this build does not
/// enumerate, so the *text* round-trip is lossless.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::dynamic_range")
)]
pub enum DynamicRange {
  /// Unspecified — caller assumes Limited.
  Unspecified,
  /// Limited / studio swing (8-bit luma 16..235, chroma 16..240);
  /// FFmpeg `AVCOL_RANGE_MPEG`.
  Limited,
  /// Full / PC swing (8-bit 0..255); FFmpeg `AVCOL_RANGE_JPEG`.
  Full,
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
}

impl Default for DynamicRange {
  #[inline]
  fn default() -> Self {
    Self::Unspecified
  }
}

impl DynamicRange {
  /// Lowercase FFmpeg-style identifier for this variant
  /// (`AVCOL_RANGE_*` slug; `tv` / `pc`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Unspecified => "unspecified",
      Self::Limited => "tv",
      Self::Full => "pc",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// The **FFmpeg `AVColorRange` code point**.
  ///
  /// [`None`] for [`Self::Other`]: it names something FFmpeg has no
  /// code for, and inventing one would lose the name.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Unspecified => 0,
      Self::Limited => 1,
      Self::Full => 2,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVColorRange` code produced by
  /// [`Self::to_u32`].
  ///
  /// [`None`] for a code this build names nothing for — a number is
  /// FFmpeg's spelling, not a name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::Unspecified,
      1 => Self::Limited,
      2 => Self::Full,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(DynamicRange, "dynamic range", [Unspecified, Limited, Full], alloc_escape: Other);

/// The error [`DynamicRange`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a sample-range name")]
#[non_exhaustive]
pub struct ParseDynamicRangeError;

impl core::str::FromStr for DynamicRange {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseDynamicRangeError;

  /// Reads a sample-range name: the canonical slug [`Self::as_str`]
  /// renders, **or** FFmpeg's spelling of the same code point where the
  /// two differ.
  ///
  /// Emission is injective and canonical — never a synonym — so
  /// `parse(display(x)) == x` holds for every named variant. Parse also
  /// takes `av_color_range_name`'s `unknown` ([`Self::Unspecified`]);
  /// `tv` and `pc` are already FFmpeg's own spellings.
  ///
  /// # Errors
  ///
  /// Returns [`ParseDynamicRangeError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"unspecified" => Self::Unspecified,
      b"tv" => Self::Limited,
      b"pc" => Self::Full,

      // FFmpeg's `av_color_range_name` spelling. Accepted, never
      // emitted — keep in step with `DYNAMIC_RANGE_FFMPEG_SYNONYMS`.
      b"unknown" => Self::Unspecified,

      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseDynamicRangeError),
    })
  }
}

/// FFmpeg's spelling for the one [`DynamicRange`] code point whose
/// canonical mediaframe slug differs, as `(ffmpeg_name,
/// canonical_slug)`. See [`MATRIX_FFMPEG_SYNONYMS`] for the rule.
#[cfg(test)]
const DYNAMIC_RANGE_FFMPEG_SYNONYMS: &[(&str, &str)] = &[("unknown", "unspecified")];

/// Chroma sample location (for subsampled YUV formats).
///
/// Aligns with H.265 SPS chroma_loc / FFmpeg `AVChromaLocation`.
///
/// [`Self::to_u32`] / [`Self::from_u32`] use the **FFmpeg
/// `AVChromaLocation` code points**; FFmpeg is the source of truth.
/// `Default` is [`Self::Unspecified`] (FFmpeg
/// `AVCHROMA_LOC_UNSPECIFIED`, code `0`); [`Self::Other`] carries any
/// name this build does not enumerate, so the *text* round-trip is
/// lossless.
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::chroma_location")
)]
pub enum ChromaLocation {
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
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
  pub fn as_str(&self) -> &str {
    match self {
      Self::Unspecified => "unspecified",
      Self::Left => "left",
      Self::Center => "center",
      Self::TopLeft => "topleft",
      Self::Top => "top",
      Self::BottomLeft => "bottomleft",
      Self::Bottom => "bottom",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// The **FFmpeg `AVChromaLocation` code point**.
  ///
  /// [`None`] for [`Self::Other`]: it names something FFmpeg has no
  /// code for, and inventing one would lose the name.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Unspecified => 0,
      Self::Left => 1,
      Self::Center => 2,
      Self::TopLeft => 3,
      Self::Top => 4,
      Self::BottomLeft => 5,
      Self::Bottom => 6,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVChromaLocation` code produced by
  /// [`Self::to_u32`].
  ///
  /// [`None`] for a code this build names nothing for — a number is
  /// FFmpeg's spelling, not a name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::Unspecified,
      1 => Self::Left,
      2 => Self::Center,
      3 => Self::TopLeft,
      4 => Self::Top,
      5 => Self::BottomLeft,
      6 => Self::Bottom,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  ChromaLocation,
  "chroma siting",
  [
    Unspecified, Left, Center, TopLeft, Top, BottomLeft, Bottom
  ],
  alloc_escape: Other
);

/// The error [`ChromaLocation`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a chroma-location name")]
#[non_exhaustive]
pub struct ParseChromaLocationError;

impl core::str::FromStr for ChromaLocation {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseChromaLocationError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseChromaLocationError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"unspecified" => Self::Unspecified,
      b"left" => Self::Left,
      b"center" => Self::Center,
      b"topleft" => Self::TopLeft,
      b"top" => Self::Top,
      b"bottomleft" => Self::BottomLeft,
      b"bottom" => Self::Bottom,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseChromaLocationError),
    })
  }
}

/// Bundled color metadata that rides on every video frame.
///
/// Every backend except R3D and BRAW exposes color metadata natively;
/// RAW backends populate from clip-level color science and leave
/// `Unspecified` if absent. `Info::UNSPECIFIED` is the sensible
/// default for RAW backends that don't carry per-frame color data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::info")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Info {
  primaries: Primaries,
  transfer: Transfer,
  matrix: Matrix,
  range: DynamicRange,
  chroma_location: ChromaLocation,
}

impl Default for Info {
  /// Delegates to [`Info::UNSPECIFIED`] — the canonical all-`Unspecified`
  /// instance is the single source of truth for the default.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::UNSPECIFIED
  }
}

impl Info {
  /// All-`Unspecified` color info (for `Default` / RAW-backend use).
  /// Every field — including `matrix` — stores the FFmpeg
  /// `UNSPECIFIED` code; `Default` delegates to this const, and it
  /// coincides with each enum's `Default` (its `Unspecified` variant). The
  /// FFmpeg BT.709-vs-BT.601-by-height fallback for an unspecified
  /// matrix is a **consumer** concern applied at read time, not
  /// stored here.
  pub const UNSPECIFIED: Self = Self {
    primaries: Primaries::Unspecified,
    transfer: Transfer::Unspecified,
    matrix: Matrix::Unspecified,
    range: DynamicRange::Unspecified,
    chroma_location: ChromaLocation::Unspecified,
  };

  /// Constructs a `Info` from explicit components.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    primaries: Primaries,
    transfer: Transfer,
    matrix: Matrix,
    range: DynamicRange,
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
  pub fn primaries(&self) -> Primaries {
    self.primaries.clone()
  }

  /// Returns the transfer characteristics.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn transfer(&self) -> Transfer {
    self.transfer.clone()
  }

  /// Returns the YUV→RGB matrix coefficients.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn matrix(&self) -> Matrix {
    self.matrix.clone()
  }

  /// Returns the sample range (limited / full).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn range(&self) -> DynamicRange {
    self.range.clone()
  }

  /// Returns the chroma sample location.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn chroma_location(&self) -> ChromaLocation {
    self.chroma_location.clone()
  }

  /// Sets the primaries (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_primaries(mut self, v: Primaries) -> Self {
    self.primaries = v;
    self
  }

  /// Sets the transfer (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_transfer(mut self, v: Transfer) -> Self {
    self.transfer = v;
    self
  }

  /// Sets the matrix (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_matrix(mut self, v: Matrix) -> Self {
    self.matrix = v;
    self
  }

  /// Sets the range (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_range(mut self, v: DynamicRange) -> Self {
    self.range = v;
    self
  }

  /// Sets the chroma location (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_chroma_location(mut self, v: ChromaLocation) -> Self {
    self.chroma_location = v;
    self
  }

  /// Sets the primaries in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_primaries(&mut self, v: Primaries) -> &mut Self {
    self.primaries = v;
    self
  }

  /// Sets the transfer in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_transfer(&mut self, v: Transfer) -> &mut Self {
    self.transfer = v;
    self
  }

  /// Sets the matrix in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_matrix(&mut self, v: Matrix) -> &mut Self {
    self.matrix = v;
    self
  }

  /// Sets the range in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_range(&mut self, v: DynamicRange) -> &mut Self {
    self.range = v;
    self
  }

  /// Sets the chroma location in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_chroma_location(&mut self, v: ChromaLocation) -> &mut Self {
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
/// This enum has **no FFmpeg analog** (it selects a mediaframe XYZ →
/// RGB matrix); it keeps its own mediaframe-local wire numbering
/// (`DciP3`=0, `Rec709`=1, `Rec2020`=2) rather than an FFmpeg code.
/// `Default` is [`Self::DciP3`]. [`Self::Other`] carries any gamut
/// name this build does not enumerate; [`Self::from_u32`] returns a
/// named variant for a canonical id (`0`/`1`/`2`) and [`None`]
/// otherwise. Construct the escape through [`Self::other`], which runs
/// the ignore-case parse first and only falls to [`Self::Other`] for a
/// genuine stranger, carrying its spelling verbatim; it survives a
/// buffa round-trip — which is correct (the id *is* that gamut), not
/// data loss. The crate-wide extension idiom (Codex adversarial-review F8).
///
/// **Tier.** [`Self::Other`] needs a heap, so it exists only at the
/// `alloc` / `std` tier; at the no-alloc tier this vocabulary is
/// **closed** and an unrecognised slug is rejected rather than
/// collapsed onto a named variant — an error beats a wrong value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, IsVariant, Display)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::dcp_target_gamut")
)]
pub enum DcpTargetGamut {
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
  /// A slug this vocabulary does not enumerate — carried verbatim. The
  /// crate-wide extension idiom: a downstream backend naming a value
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(Utf8Bytes),
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

  /// Lowercase identifier for this variant, in the same style as the
  /// five H.273 colour enums (`"dci-p3"` / `"rec709"` / `"rec2020"`).
  ///
  /// [`Self::Other`] renders the name it carries, so the rendering is
  /// injective across the whole value space and
  /// [`FromStr`](core::str::FromStr) is its exact inverse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::DciP3 => "dci-p3",
      Self::Rec709 => "rec709",
      Self::Rec2020 => "rec2020",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// The mediaframe-local id (no FFmpeg analog); `DciP3` (the default)
  /// is `0`.
  ///
  /// [`None`] for [`Self::Other`]: it names something FFmpeg has no
  /// code for, and inventing one would lose the name.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::DciP3 => 0,
      Self::Rec709 => 1,
      Self::Rec2020 => 2,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the mediaframe-local wire id produced by
  /// [`Self::to_u32`].
  ///
  /// [`None`] for a code this build names nothing for — a number is
  /// FFmpeg's spelling, not a name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::DciP3,
      1 => Self::Rec709,
      2 => Self::Rec2020,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling or a documented alias
  /// returns that **named** variant, never a second value for a meaning
  /// this vocabulary already has one for. Only a genuine stranger reaches
  /// [`Self::Other`], carrying the caller's spelling verbatim: the escape
  /// is a lossless passthrough for a name this build does not know, not a
  /// fold target.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

/// The **closed** set of DCP target gamuts an XYZ → RGB kernel has a
/// matrix for — [`KernelMatrix`]'s twin on the gamut axis.
///
/// [`DcpTargetGamut`] is the open descriptor vocabulary and can name a
/// gamut nobody has a 3×3 matrix or a luma basis for
/// ([`DcpTargetGamut::Other`]). Passing one of those to the XYZ walker
/// used to be a documented **panic**; converting at the door instead
/// makes it a refusal the caller can see coming, and past the door the
/// unconvertible row is unrepresentable.
///
/// Same shape as [`KernelMatrix`]: closed (not `#[non_exhaustive]`, so a
/// kernel's `match` is exhaustive) and `Copy` (so the row type is).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelGamut {
  /// DCI-P3 theatrical, DCI white — see [`DcpTargetGamut::DciP3`].
  DciP3,
  /// Rec.709 / sRGB (D65) — see [`DcpTargetGamut::Rec709`].
  Rec709,
  /// Rec.2020 (D65) — see [`DcpTargetGamut::Rec2020`].
  Rec2020,
}

/// The error the [`DcpTargetGamut`] → [`KernelGamut`] conversion
/// returns: this gamut has no defined XYZ → RGB matrix or luma basis.
///
/// Opaque and sealed on the same reasoning as
/// [`UnsupportedKernelMatrixError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("this target gamut has no conversion-kernel matrix")]
#[non_exhaustive]
pub struct UnsupportedKernelGamutError;

impl TryFrom<&DcpTargetGamut> for KernelGamut {
  type Error = UnsupportedKernelGamutError;

  /// The open → closed exchange, taken at the XYZ kernel door.
  ///
  /// # Errors
  ///
  /// Returns [`UnsupportedKernelGamutError`] for
  /// [`DcpTargetGamut::Other`] — a gamut this build does not name has
  /// no defined luma basis and must not be silently colour-converted as
  /// if it were one of the three that do.
  fn try_from(g: &DcpTargetGamut) -> Result<Self, Self::Error> {
    Ok(match g {
      DcpTargetGamut::DciP3 => Self::DciP3,
      DcpTargetGamut::Rec709 => Self::Rec709,
      DcpTargetGamut::Rec2020 => Self::Rec2020,
      #[cfg(any(feature = "std", feature = "alloc"))]
      DcpTargetGamut::Other(_) => return Err(UnsupportedKernelGamutError),
    })
  }
}

impl From<KernelGamut> for DcpTargetGamut {
  /// Widens back to the descriptor vocabulary. Total and injective.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(k: KernelGamut) -> Self {
    match k {
      KernelGamut::DciP3 => Self::DciP3,
      KernelGamut::Rec709 => Self::Rec709,
      KernelGamut::Rec2020 => Self::Rec2020,
    }
  }
}

/// The error [`DcpTargetGamut`]'s [`FromStr`](core::str::FromStr) returns **at the
/// no-alloc tier**.
///
/// Since 0.5.0 this is no longer `FromStr::Err` at the `alloc` / `std`
/// tier. There the vocabulary is open and the parse is total, so the
/// associated type is [`Infallible`](core::convert::Infallible) and the
/// signature says what the behaviour always was. The type itself is
/// unchanged and still exported: the lean build returns it, and code that
/// names it keeps compiling.
///
/// Opaque and sealed: the input is deliberately not retained (this type is
/// reachable only at the no-alloc tier, where there is nowhere to put an
/// owned copy, and the input is attacker-controlled on the deserialization
/// path). `#[non_exhaustive]` keeps it constructible only here, so it can
/// grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a DCP target-gamut name")]
#[non_exhaustive]
pub struct ParseDcpTargetGamutError;

impl core::str::FromStr for DcpTargetGamut {
  /// [`Infallible`](core::convert::Infallible) wherever the escape arm
  /// exists, which is exactly where the parse is total; the vocabulary's
  /// own refusal at the no-alloc tier, where it is closed. The predicate
  /// is the one that gates [`Self::Other`], so the two cannot drift.
  #[cfg(any(feature = "std", feature = "alloc"))]
  type Err = core::convert::Infallible;
  /// See the `alloc`-tier arm above.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  type Err = ParseDcpTargetGamutError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseDcpTargetGamutError`] only at the no-alloc tier, where the vocabulary is
  /// closed. At the `alloc` / `std` tier this parse is **total** — a slug
  /// this type does not name rides [`Self::Other`], carrying the caller's
  /// spelling verbatim — and `Self::Err` is
  /// [`Infallible`](core::convert::Infallible) there, so the totality is
  /// checkable by the compiler rather than only promised here.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"dci-p3" => Self::DciP3,
      b"rec709" => Self::Rec709,
      b"rec2020" => Self::Rec2020,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(Utf8Bytes::from(s)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseDcpTargetGamutError),
    })
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
/// frame side data); the per-frame [`Info`] enums are
/// unchanged.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::content_light_level")
)]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::chroma_coord")
)]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::mastering_display")
)]
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
/// This is intentionally *separate* from [`Info`]: `Info`
/// stays per-frame closed-form enums only; HDR10 static metadata is
/// clip / stream level and optional, so it lives in its own type.
/// (Dynamic HDR — HDR10+ / Dolby Vision RPU — is out of scope here.)
// golden-rule §9: both fields are `Option` — skip-serialize when `None`
// (never emit `null`); `serde(default)` (whole struct has a meaningful
// all-`None` `Default`) restores an omitted field on deserialize.
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(default)
)]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::hdr_static_metadata")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HdrStaticMetadata {
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  mastering: Option<MasteringDisplay>,
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
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

/// Dolby Vision decoder configuration record.
///
/// Read from FFmpeg `AVDOVIDecoderConfigurationRecord`
/// (`AV_PKT_DATA_DOVI_CONF` packet side data /
/// `AV_FRAME_DATA_DOVI_METADATA`'s configuration). This is the
/// stream-level DoVi *configuration* (which profile / level, whether
/// an RPU and an enhancement layer are present, and the base-layer
/// signal compatibility id) — it is **distinct from** the HDR10
/// static metadata in [`HdrStaticMetadata`] (SMPTE ST 2086 /
/// CTA-861.3) and from the per-frame [`Info`] enums. The DoVi
/// RPU payload itself (dynamic metadata) is out of scope here; only
/// the configuration record is modelled.
///
/// All fields default to `0` (`#[derive(Default)]`), matching an
/// absent / unset configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::dolby_vision_config")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DolbyVisionConfig {
  profile: u8,
  level: u8,
  rpu_present: bool,
  el_present: bool,
  bl_signal_compat_id: u8,
}

impl DolbyVisionConfig {
  /// Constructs a `DolbyVisionConfig` from the FFmpeg
  /// `AVDOVIDecoderConfigurationRecord` fields: Dolby Vision profile
  /// and level, RPU / enhancement-layer presence flags, and the
  /// base-layer signal compatibility id.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    profile: u8,
    level: u8,
    rpu_present: bool,
    el_present: bool,
    bl_signal_compat_id: u8,
  ) -> Self {
    Self {
      profile,
      level,
      rpu_present,
      el_present,
      bl_signal_compat_id,
    }
  }

  /// Returns the Dolby Vision profile.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn profile(&self) -> u8 {
    self.profile
  }

  /// Returns the Dolby Vision level.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn level(&self) -> u8 {
    self.level
  }

  /// `true` when an RPU (Reference Processing Unit) is present.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rpu_present(&self) -> bool {
    self.rpu_present
  }

  /// `true` when an enhancement layer is present.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn el_present(&self) -> bool {
    self.el_present
  }

  /// Returns the base-layer signal compatibility id.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bl_signal_compat_id(&self) -> u8 {
    self.bl_signal_compat_id
  }

  /// Sets the profile (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_profile(mut self, v: u8) -> Self {
    self.profile = v;
    self
  }

  /// Sets the level (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_level(mut self, v: u8) -> Self {
    self.level = v;
    self
  }

  /// Marks the RPU as present (`rpu_present = true`; consuming
  /// builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_rpu_present(mut self) -> Self {
    self.rpu_present = true;
    self
  }

  /// Assigns the raw RPU-present flag (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_rpu_present(mut self, v: bool) -> Self {
    self.rpu_present = v;
    self
  }

  /// Marks the enhancement layer as present (`el_present = true`;
  /// consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_el_present(mut self) -> Self {
    self.el_present = true;
    self
  }

  /// Assigns the raw enhancement-layer-present flag (consuming
  /// builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_el_present(mut self, v: bool) -> Self {
    self.el_present = v;
    self
  }

  /// Sets the base-layer signal compatibility id (consuming
  /// builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_bl_signal_compat_id(mut self, v: u8) -> Self {
    self.bl_signal_compat_id = v;
    self
  }

  /// Sets the profile in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_profile(&mut self, v: u8) -> &mut Self {
    self.profile = v;
    self
  }

  /// Sets the level in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_level(&mut self, v: u8) -> &mut Self {
    self.level = v;
    self
  }

  /// Marks the RPU as present (`rpu_present = true`) in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_rpu_present(&mut self) -> &mut Self {
    self.rpu_present = true;
    self
  }

  /// Assigns the raw RPU-present flag in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_rpu_present(&mut self, v: bool) -> &mut Self {
    self.rpu_present = v;
    self
  }

  /// Clears the RPU-present flag (`rpu_present = false`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_rpu_present(&mut self) -> &mut Self {
    self.rpu_present = false;
    self
  }

  /// Marks the enhancement layer as present (`el_present = true`) in
  /// place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_el_present(&mut self) -> &mut Self {
    self.el_present = true;
    self
  }

  /// Assigns the raw enhancement-layer-present flag in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_el_present(&mut self, v: bool) -> &mut Self {
    self.el_present = v;
    self
  }

  /// Clears the enhancement-layer-present flag (`el_present = false`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_el_present(&mut self) -> &mut Self {
    self.el_present = false;
    self
  }

  /// Sets the base-layer signal compatibility id in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_bl_signal_compat_id(&mut self, v: u8) -> &mut Self {
    self.bl_signal_compat_id = v;
    self
  }
}

// The kernel-selector door keeps its own suite: it is about the
// closed `Kernel*` types, not about the descriptor vocabularies the
// main suite covers.
#[cfg(test)]
mod kernel_tests;

#[cfg(test)]
mod tests;
