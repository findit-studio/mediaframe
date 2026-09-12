//! Multimedia container-format vocabulary — top-level (video +
//! audio) containers.
//!
//! Audio-only containers (`mp3`, `flac`, `wav`, …) live on
//! [`crate::audio::ContainerFormat`]; this enum enumerates the
//! multimedia containers that carry one-or-more streams of *any*
//! kind (video, audio, subtitle, data).

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;

/// Top-level multimedia container format.
///
/// Closed-ish vocabulary covering the containers a typical
/// media-ingest pipeline encounters — not FFmpeg-coded, so there is
/// no `to_u32`/`from_u32`; the `Other(Utf8Bytes)` arm preserves
/// unknown slugs losslessly.
///
/// `as_str` returns the canonical extension-style slug (`"mov"`,
/// `"mp4"`, `"mkv"`, `"webm"`, …); [`Self::as_extension`] returns the
/// same spelling for most variants but diverges for `MpegTs` (`"ts"`)
/// and `Ogg` (`"ogv"`). A handful of variants also have genuine
/// alternate on-disk spellings (`.qt`, `.ogx`, `.mpg4`, `.3gpp`) —
/// [`Self::extensions`] lists every one, canonical first, and
/// [`FromStr`](core::str::FromStr) accepts them all, ignore-case.
///
/// **`M2ts` and `Threeg2` are separate variants, not aliases of
/// `MpegTs`/`Threegp`** — an R5 correction. `.m2ts`/`.mts`/`.m2t` and
/// `.3g2`/`.3gp2` were briefly (R1/R3) folded into `MpegTs.extensions()`
/// / `Threegp.extensions()`; both landings failed this crate's own
/// identical-bytes test for an alias (Codex R5), because both name a
/// structurally different container from the one they were attached to
/// — see [`Self::M2ts`] and [`Self::Threeg2`]'s own docs for the byte
/// -level distinction each carries.
///
/// **Variant naming note:** the `.3gp` / `.3g2` containers' variants are
/// named [`Self::Threegp`] / [`Self::Threeg2`] — Rust identifiers cannot
/// start with a digit, and `_3gp` would render as `"3gp"` under
/// `derive_more::Display`'s snake-casing but is unidiomatic. The
/// `as_str()` / `FromStr` surface still returns / matches the canonical
/// `"3gp"` / `"3g2"` slugs. `M2ts` mirrors `MpegTs`'s own capitalisation
/// convention (a digit mid-identifier gets no case break on either
/// side: `M2ts`, not `M2Ts` or `M2TS`).
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::container_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum Format {
  /// QuickTime File Format (`.mov`; `.qt` is a recognised alias —
  /// ExifTool's own file-type table aliases `QT` directly to `MOV`).
  Mov,
  /// ISO Base Media / MPEG-4 Part 14 (`.mp4`; `.mpg4` is a recognised
  /// alias — IANA's own `video/mp4` registration text: "mp4 and mpg4
  /// are both declared"). The auto-derived predicate name would be
  /// `is_mp_4` (digit-snake-case); the hand-written [`Self::is_mp4`]
  /// uses the cleaner name.
  ///
  /// **Deliberately excluded**: `.m4v` / `.m4b` / `.m4p` — three
  /// independent sources agree these are distinct products sharing
  /// `.mp4`'s ISOBMFF machinery, not spellings of it: ExifTool keeps
  /// them as their own file types (video / audiobook / protected);
  /// ffmpeg gives them their own dedicated `ipod` encoder profile,
  /// separate from the plain `mp4` muxer; IANA's registration text
  /// names only `mp4`/`mpg4`, nothing else. The distinguishing test
  /// applied everywhere in this module: an alias is a spelling of the
  /// *same bytes* (`.mpg4` and `.mp4` name the identical ISO/IEC
  /// 14496-14 structure); `.m4v`/`.m4b` denote a structurally
  /// different profile that merely shares the parser.
  #[is_variant(ignore)]
  Mp4,
  /// Matroska (`.mkv`). No alias — ExifTool groups `.webm` under the
  /// same internal "MKV" module family, but mediaframe keeps
  /// [`Self::Webm`] a distinct variant (a real, separate product
  /// profile, not a spelling of `.mkv`), the same reasoning that
  /// keeps `Mov` and `Mp4` apart.
  Mkv,
  /// WebM — Matroska subset for VP8/9 + Vorbis/Opus (`.webm`).
  Webm,
  /// Audio-Video Interleave (`.avi`).
  Avi,
  /// Flash Video (`.flv`).
  Flv,
  /// MPEG-2 Transport Stream — raw, **unprefixed** 188-byte-aligned TS
  /// packets, no wrapper (`.ts`; `.m2t` is a recognised alias of *this*
  /// variant, not [`Self::M2ts`] — see below). FFmpeg slug: `"mpegts"`.
  ///
  /// **`.m2t` lives here, not on `M2ts`** — an R7 correction (Codex R7
  /// HIGH finding). R5 moved `.m2t` to `M2ts` alongside `.m2ts`/`.mts`
  /// on ExifTool's *static* alias table alone (`M2T`/`MTS`/`TS` all
  /// alias to the `M2TS` file-type *name*) — the same shape of mistake
  /// `.apl` made, applied to itself one round later: a real citation,
  /// unchecked against actual byte layout. ExifTool's own **content
  /// detector** (`M2TS.pm`'s `ProcessM2TS`, verified directly against
  /// the real Perl source, not the static table) measures packet stride
  /// by sync-byte spacing and picks the FileType from that, not the
  /// extension: `$et->SetFileType($tcLen ? 'M2TS' : 'M2T')` — a file
  /// with **unprefixed** 188-byte packets (`$tcLen == 0`) is labelled
  /// `M2T`, not `M2TS`. `.m2t` names the same 188-byte world this
  /// variant already covers; `.m2ts`/`.mts` name the 192-byte BDAV
  /// world `M2ts` covers. `extensions()` is `["ts", "m2t"]`.
  MpegTs,
  /// MPEG-2 Transport Stream, BDAV/M2TS-wrapped — Blu-ray Disc /
  /// AVCHD's 192-byte packet framing: a 4-byte `TP_extra_header`
  /// (carries an arrival-timecode + copy-permission field) prepended to
  /// every standard 188-byte MPEG-TS packet (`.m2ts`; `.mts` is a
  /// recognised alias of *this* variant — `.m2t` is not, see
  /// [`Self::MpegTs`]'s own doc).
  ///
  /// **Not [`Self::MpegTs`]** — an R5 correction (Codex R5 HIGH finding,
  /// per the user's 甲 ruling): R1/R3 folded `.m2ts`/`.mts`/`.m2t` into
  /// `MpegTs.extensions()` on ExifTool's alias table alone, without
  /// checking that ExifTool's own target of those aliases (`M2TS`) is a
  /// distinct on-disk packet layout from plain `.ts`, not a spelling of
  /// it — the exact "shared source citation, unchecked byte identity"
  /// mistake `.apl` (`audio::ContainerFormat::Ape`) made and failed on
  /// one round earlier. ExifTool's own table is what actually names the
  /// split: `M2T`/`MTS`/`TS` all alias to `M2TS` (`crate::container` R1
  /// census), never the reverse — mediaframe's R1 pass misread that as
  /// "these three are `MpegTs`'s aliases" when ExifTool was naming a
  /// *fourth*, separate file type all along. ffmpeg shares one
  /// `mpegts` demuxer/muxer implementation for both packet strides
  /// (auto-detecting which framing a stream uses — its muxer's own
  /// `Common extensions` field lists `ts,m2t,m2ts,mts` together), the
  /// same "one implementation, multiple real formats" shape already
  /// established for `Mov`/`Mp4`/`Cr3`/`Heif`/`Avif` sharing ffmpeg's
  /// `mov` demuxer — shared tooling is never itself the identical-bytes
  /// proof; the 192-vs-188-byte packet structure is.
  ///
  /// **`.m2t` moved to [`Self::MpegTs`]** — an R7 correction (Codex R7
  /// HIGH finding, applying this same identical-bytes discipline to R5's
  /// *own* work): ExifTool's static alias table (what R5 used) says
  /// `M2T` aliases to the `M2TS` file-type *name*, but ExifTool's actual
  /// content detector (`M2TS.pm`) labels unprefixed 188-byte content
  /// `M2T` and reserves `M2TS` for the 192-byte prefixed form — see
  /// `MpegTs`'s own doc for the exact source line. `.m2t` names the
  /// 188-byte world, not this one. The auto-derived
  /// predicate name would be digit-snake-case (`is_m_2ts` or similar);
  /// the hand-written [`Self::is_m2ts`] uses the cleaner name.
  #[is_variant(ignore)]
  M2ts,
  /// Ogg container (`.ogv` / `.ogx` — video-bearing Ogg). Audio-only
  /// `.ogg` is [`crate::audio::ContainerFormat::Ogg`] instead. `.ogx`
  /// is Xiph.org's own generic/multiplexed Ogg extension (RFC 5334)
  /// — ExifTool does not track it (no `OGX` entry in its file-type
  /// table at all), so this one alias's provenance is the format's
  /// own spec rather than the ExifTool census.
  Ogg,
  /// Advanced Systems Format (`.asf`).
  Asf,
  /// RealMedia (`.rm`). No alias: ExifTool keeps `.rmvb` / `.ra` /
  /// `.ram` / `.rpm` as their own distinct file types under a shared
  /// "Real" family, not spelling variants of `.rm` — growing this
  /// family with real variant support is future scope, not a text-face
  /// alias.
  Rm,
  /// Windows Media Video (`.wmv`) — an ASF subprofile, exposed
  /// separately because callers often differentiate it from generic
  /// `.asf`.
  Wmv,
  /// Material Exchange Format (`.mxf`) — broadcast-mastering
  /// container.
  Mxf,
  /// General Exchange Format (`.gxf`) — SMPTE 360M.
  Gxf,
  /// 3GPP — the GSM/UMTS-lineage 3rd Generation Partnership Project
  /// multimedia container (`.3gp`; `.3gpp` is a recognised long-form
  /// alias of *this* variant only). Variant name is `Threegp` because
  /// Rust identifiers cannot start with a digit.
  ///
  /// **No longer includes `.3g2`/`.3gp2`** (R5 correction — see
  /// [`Self::Threeg2`]'s own doc for why those moved to their own
  /// variant). What remains: ExifTool aliases the long-form (`3GPP` →
  /// `3GP`); IANA's `video/3gpp` registration independently confirms
  /// `.3gpp` itself (`"3gp and 3gpp are both declared ...; 3gp is
  /// preferred"`) — two independent citable sources for this one alias,
  /// both naming the *same* 3GPP-brand ISOBMFF file, not ExifTool alone.
  /// (R2 had excluded `.3gpp` as "unrealistic on-disk spellings" — a
  /// subjective call, not a source-grounded one; R3's sweep reversed
  /// that for consistency with every other alias in this module, all
  /// landed on citable-source grounds rather than a realism judgment —
  /// that reversal stands; only the R3 *addition* of `.3g2`/`.3gp2` to
  /// this variant specifically is what R5 corrects.)
  Threegp,
  /// 3GPP2 — the CDMA2000-lineage sibling standard (a genuinely
  /// *different* standards body from 3GPP: 3GPP2 is the CDMA/ANSI-41
  /// carrier-lineage counterpart to 3GPP's GSM/UMTS lineage) (`.3g2`;
  /// `.3gp2` is a recognised long-form alias of *this* variant).
  ///
  /// **Not [`Self::Threegp`]** — an R5 correction (Codex R5 HIGH
  /// finding, per the user's 甲 ruling). `.3g2` files are ISOBMFF
  /// (`ftyp`-boxed) the same family way `.3gp` files are, but the
  /// `ftyp` box's own `major_brand` field is what actually names the
  /// format on disk, and it differs by standard: 3GPP-family brands
  /// (`3gp4`/`3gp5`/`3gp6`/…) for [`Self::Threegp`] vs the 3GPP2-family
  /// brand (`3g2a`) for this variant — a real, structurally-encoded
  /// distinction inside the container itself, not a filename
  /// convention. ffmpeg independently confirms the split at the tooling
  /// level: `3gp` and `3g2` are two separately-named, dedicated muxers
  /// (`ffmpeg -muxers`: `3gp  3GP (3GPP file format)` /
  /// `3g2  3GP2 (3GPP2 file format)`), unlike the single shared `mov`
  /// demuxer that `Mov`/`Mp4`/`Cr3`/`Heif`/`Avif` all ride — the
  /// strongest ffmpeg-side signal in this module that two extensions
  /// name genuinely different formats rather than one implementation's
  /// convenience grouping. ExifTool's own table aliases the long-form
  /// the same way it does for `Threegp` (`3GP2` → `3G2`, never to
  /// `3GP`). The auto-derived predicate name would be digit-snake-case;
  /// the hand-written [`Self::is_threeg2`] uses the cleaner name.
  #[is_variant(ignore)]
  Threeg2,
  /// A container not enumerated above — carries the
  /// extension-style slug verbatim. Lossless escape.
  Other(Utf8Bytes),
}

impl Default for Format {
  /// `Other("")` — the wire-zero / "absent" sentinel. Containers
  /// vary by source; there is no universally-defensible default.
  /// Callers picking a meaningful fallback should be explicit
  /// (`Format::Mp4` is the common one).
  #[inline]
  fn default() -> Self {
    Self::Other(Utf8Bytes::new())
  }
}

impl Format {
  /// True iff this is [`Self::Mp4`]. Hand-written to override the
  /// auto-derived `is_mp_4` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_mp4(&self) -> bool {
    matches!(self, Self::Mp4)
  }

  /// True iff this is [`Self::M2ts`]. Hand-written to override the
  /// auto-derived digit-snake-case predicate name.
  #[inline(always)]
  pub const fn is_m2ts(&self) -> bool {
    matches!(self, Self::M2ts)
  }

  /// True iff this is [`Self::Threeg2`]. Hand-written to override the
  /// auto-derived digit-snake-case predicate name.
  #[inline(always)]
  pub const fn is_threeg2(&self) -> bool {
    matches!(self, Self::Threeg2)
  }

  /// Canonical extension-style slug (`"mov"`, `"mp4"`, `"mkv"`,
  /// `"webm"`, `"3gp"`, …).
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mov => "mov",
      Self::Mp4 => "mp4",
      Self::Mkv => "mkv",
      Self::Webm => "webm",
      Self::Avi => "avi",
      Self::Flv => "flv",
      Self::MpegTs => "mpegts",
      Self::M2ts => "m2ts",
      Self::Ogg => "ogg",
      Self::Asf => "asf",
      Self::Rm => "rm",
      Self::Wmv => "wmv",
      Self::Mxf => "mxf",
      Self::Gxf => "gxf",
      Self::Threegp => "3gp",
      Self::Threeg2 => "3g2",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Primary file-on-disk extension (without the leading dot —
  /// `"mov"`, `"mp4"`, `"ts"`, `"ogv"`, `"3gp"`, …). Distinct from
  /// the FFmpeg slug returned by [`Self::as_str`]: `MpegTs` returns
  /// `"ts"` here (vs `"mpegts"`); `Ogg` returns `"ogv"` for the
  /// video-bearing form (vs the generic Ogg slug).
  ///
  /// Returns `""` for [`Self::Other`] — the open variant carries an
  /// FFmpeg slug, not an extension, so the mapping is unknown.
  /// Returns `&'static str` (not `&str`) so the value is compile-time
  /// stable and the method is `const`.
  #[inline(always)]
  pub const fn as_extension(&self) -> &'static str {
    match self {
      Self::Mov => "mov",
      Self::Mp4 => "mp4",
      Self::Mkv => "mkv",
      Self::Webm => "webm",
      Self::Avi => "avi",
      Self::Flv => "flv",
      Self::MpegTs => "ts",
      Self::M2ts => "m2ts",
      Self::Ogg => "ogv",
      Self::Asf => "asf",
      Self::Rm => "rm",
      Self::Wmv => "wmv",
      Self::Mxf => "mxf",
      Self::Gxf => "gxf",
      Self::Threegp => "3gp",
      Self::Threeg2 => "3g2",
      Self::Other(_) => "",
    }
  }

  /// Every recognised on-disk spelling for this format, canonical first
  /// (== [`Self::as_extension`]) and aliases after. [`FromStr`] accepts
  /// every entry, ignore-case — a caller collecting "every spelling this
  /// format might be saved under" (the way findit's walk collects
  /// container extensions into one set) should iterate this rather than
  /// call [`Self::as_extension`] alone.
  ///
  /// Most variants carry exactly one spelling; see each variant's own doc
  /// for where a listed alias comes from (ExifTool's own file-type
  /// aliases in most cases; `Ogg`'s `.ogx` is the one exception, sourced
  /// from Xiph.org's own spec since ExifTool does not track it).
  ///
  /// Returns `&[]` for [`Self::Other`] — the open variant carries an
  /// FFmpeg slug, not a known extension set.
  #[inline]
  pub const fn extensions(&self) -> &'static [&'static str] {
    match self {
      Self::Mov => &["mov", "qt"],
      Self::Mp4 => &["mp4", "mpg4"],
      Self::Mkv => &["mkv"],
      Self::Webm => &["webm"],
      Self::Avi => &["avi"],
      Self::Flv => &["flv"],
      Self::MpegTs => &["ts", "m2t"],
      Self::M2ts => &["m2ts", "mts"],
      Self::Ogg => &["ogv", "ogx"],
      Self::Asf => &["asf"],
      Self::Rm => &["rm"],
      Self::Wmv => &["wmv"],
      Self::Mxf => &["mxf"],
      Self::Gxf => &["gxf"],
      Self::Threegp => &["3gp", "3gpp"],
      Self::Threeg2 => &["3g2", "3gp2"],
      Self::Other(_) => &[],
    }
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`]'s own match table
  /// (canonical spelling and every documented alias extension), walked
  /// through [`Self::from_str`] rather than duplicated here — so a
  /// recognised spelling returns that **named** variant, never a second
  /// value for a meaning this vocabulary already has one for. Only a
  /// genuine stranger reaches [`Self::Other`], carrying the caller's
  /// spelling verbatim: the escape is a lossless passthrough for a name
  /// this build does not know, not a fold target.
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  Format,
  "container format",
  [
    Mov, Mp4, Mkv, Webm, Avi, Flv, MpegTs, M2ts, Ogg, Asf, Rm, Wmv, Mxf,
    Gxf, Threegp, Threeg2
  ],
  escape: Other
);

impl FromStr for Format {
  type Err = core::convert::Infallible;
  /// Recognise a canonical container slug **or any alias extension**
  /// from [`Self::extensions`]; unknown values land in [`Self::Other`]
  /// (infallible, lossless).
  ///
  /// Before this the parser only accepted each variant's [`Self::as_str`]
  /// slug — for `MpegTs`/`Ogg`/`Threegp` that slug is not even the same
  /// spelling as [`Self::as_extension`] (`"mpegts"` vs `"ts"`, `"ogg"` vs
  /// `"ogv"`), so the primary on-disk extension for those three, and
  /// every alias, was previously unparseable and fell to [`Self::Other`].
  /// The face and the parser must agree; this closes that gap.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"mov" | b"qt" => Self::Mov,
      b"mp4" | b"mpg4" => Self::Mp4,
      b"mkv" => Self::Mkv,
      b"webm" => Self::Webm,
      b"avi" => Self::Avi,
      b"flv" => Self::Flv,
      b"mpegts" | b"ts" | b"m2t" => Self::MpegTs,
      b"m2ts" | b"mts" => Self::M2ts,
      b"ogg" | b"ogv" | b"ogx" => Self::Ogg,
      b"asf" => Self::Asf,
      b"rm" => Self::Rm,
      b"wmv" => Self::Wmv,
      b"mxf" => Self::Mxf,
      b"gxf" => Self::Gxf,
      b"3gp" | b"3gpp" => Self::Threegp,
      b"3g2" | b"3gp2" => Self::Threeg2,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

#[cfg(test)]
mod tests;
