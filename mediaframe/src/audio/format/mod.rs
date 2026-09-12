//! Audio sample-format vocabulary (`SampleFormat`, FFmpeg
//! `AVSampleFormat`) and audio-only container-format vocabulary
//! (`ContainerFormat`, audio file extensions).

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;

/// Audio sample format — FFmpeg `AVSampleFormat`.
///
/// One named variant per FFmpeg n9.0 sample format (the standard 12
/// — `u8`/`s16`/`s32`/`s64` × packed/planar plus `flt`/`dbl` ×
/// packed/planar), with the planar variants suffixed `p` per FFmpeg
/// convention.
///
/// `to_u32` / `from_u32` use the FFmpeg `AV_SAMPLE_FMT_*` enum
/// indices (`U8 = 0`, `S16 = 1`, …, `S64P = 11`); unrecognised
/// codes round-trip via [`Self::Unknown`]. Slugs that don't match
/// any named variant round-trip via [`Self::Other`].
///
/// `#[non_exhaustive]` keeps future additions non-breaking.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::sample_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum SampleFormat {
  /// `AV_SAMPLE_FMT_U8` (code `0`) — unsigned 8-bit, packed.
  U8,
  /// `AV_SAMPLE_FMT_S16` (code `1`) — signed 16-bit, packed.
  S16,
  /// `AV_SAMPLE_FMT_S32` (code `2`) — signed 32-bit, packed.
  S32,
  /// `AV_SAMPLE_FMT_FLT` (code `3`) — 32-bit float, packed.
  Flt,
  /// `AV_SAMPLE_FMT_DBL` (code `4`) — 64-bit float, packed.
  Dbl,
  /// `AV_SAMPLE_FMT_U8P` (code `5`) — unsigned 8-bit, planar.
  U8p,
  /// `AV_SAMPLE_FMT_S16P` (code `6`) — signed 16-bit, planar.
  S16p,
  /// `AV_SAMPLE_FMT_S32P` (code `7`) — signed 32-bit, planar.
  S32p,
  /// `AV_SAMPLE_FMT_FLTP` (code `8`) — 32-bit float, planar.
  Fltp,
  /// `AV_SAMPLE_FMT_DBLP` (code `9`) — 64-bit float, planar.
  Dblp,
  /// `AV_SAMPLE_FMT_S64` (code `10`) — signed 64-bit, packed.
  S64,
  /// `AV_SAMPLE_FMT_S64P` (code `11`) — signed 64-bit, planar.
  S64p,
  /// A format slug not enumerated above — carries the slug verbatim
  /// (the [`Self::from_str`] lossless escape).
  Other(Utf8Bytes),
}

impl Default for SampleFormat {
  /// `Other("")` — the wire-zero / "absent" sentinel, matching
  /// [`ContainerFormat`]. FFmpeg's `AV_SAMPLE_FMT_NONE` is `-1`, outside
  /// the `u32` code space, so it has no numeric spelling here; the empty
  /// slug is the one that round-trips.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Other(Utf8Bytes::new())
  }
}

impl SampleFormat {
  /// FFmpeg-canonical slug (`"u8"`, `"s16"`, `"flt"`, `"u8p"`, …).
  pub fn as_str(&self) -> &str {
    match self {
      Self::U8 => "u8",
      Self::S16 => "s16",
      Self::S32 => "s32",
      Self::Flt => "flt",
      Self::Dbl => "dbl",
      Self::U8p => "u8p",
      Self::S16p => "s16p",
      Self::S32p => "s32p",
      Self::Fltp => "fltp",
      Self::Dblp => "dblp",
      Self::S64 => "s64",
      Self::S64p => "s64p",
      Self::Other(s) => s.as_str(),
    }
  }

  /// The FFmpeg `AV_SAMPLE_FMT_*` enum index for the named variants —
  /// a boundary helper for FFmpeg interop, not a wire form.
  ///
  /// [`Self::Other`] returns [`None`]: it names a format FFmpeg has no
  /// code for, and inventing one would lose the name. The slug from
  /// [`Self::as_str`] is the spelling that always survives.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::U8 => 0,
      Self::S16 => 1,
      Self::S32 => 2,
      Self::Flt => 3,
      Self::Dbl => 4,
      Self::U8p => 5,
      Self::S16p => 6,
      Self::S32p => 7,
      Self::Fltp => 8,
      Self::Dblp => 9,
      Self::S64 => 10,
      Self::S64p => 11,
      Self::Other(_) => return None,
    })
  }

  /// Decodes an FFmpeg `AV_SAMPLE_FMT_*` code, or [`None`] if this build
  /// names no format for it. The numeric space is FFmpeg's, so an
  /// unrecognised code carries no name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::U8,
      1 => Self::S16,
      2 => Self::S32,
      3 => Self::Flt,
      4 => Self::Dbl,
      5 => Self::U8p,
      6 => Self::S16p,
      7 => Self::S32p,
      8 => Self::Fltp,
      9 => Self::Dblp,
      10 => Self::S64,
      11 => Self::S64p,
      _ => return None,
    })
  }

  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`]'s own match table,
  /// walked through [`Self::from_str`] rather than duplicated here — so a
  /// canonical spelling returns that **named** variant, never a second
  /// value for a meaning this vocabulary already has one for. Only a
  /// genuine stranger reaches [`Self::Other`], carrying the caller's
  /// spelling verbatim: the escape is a lossless passthrough for a name
  /// this build does not know, not a fold target.
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::from_str(slug.as_ref()).unwrap()
  }

  /// `true` for the planar layout variants (`*p`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_planar(&self) -> bool {
    matches!(
      self,
      Self::U8p | Self::S16p | Self::S32p | Self::Fltp | Self::Dblp | Self::S64p
    )
  }
}

roster!(
  SampleFormat,
  "sample format",
  [
    U8, S16, S32, Flt, Dbl, U8p, S16p, S32p, Fltp, Dblp, S64, S64p
  ],
  escape: Other
);

impl FromStr for SampleFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical FFmpeg sample-format slug; unknown
  /// values land in [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"u8" => Self::U8,
      b"s16" => Self::S16,
      b"s32" => Self::S32,
      b"flt" => Self::Flt,
      b"dbl" => Self::Dbl,
      b"u8p" => Self::U8p,
      b"s16p" => Self::S16p,
      b"s32p" => Self::S32p,
      b"fltp" => Self::Fltp,
      b"dblp" => Self::Dblp,
      b"s64" => Self::S64,
      b"s64p" => Self::S64p,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

// ---------------------------------------------------------------------------

/// Audio-only file / container format vocabulary.
///
/// Top-level multimedia containers (`mp4`/`mkv`/`mov`/`webm`/…)
/// live on [`crate::container::Format`]; this enum
/// enumerates the **audio-only** containers (one audio stream, no
/// video). Closed-ish vocabulary — not FFmpeg-coded, so there is no
/// `to_u32`/`from_u32`; the `Other(Utf8Bytes)` arm preserves unknown
/// slugs losslessly.
///
/// `as_str` returns the file-extension-style slug (`"mp3"`, `"aac"`,
/// `"flac"`, …); a handful of variants also have genuine alternate
/// on-disk spellings (`.aif`, `.wvp`, `.oga`/`.spx`, `.adts`, `.mac`) —
/// [`Self::extensions`] lists every one, canonical first, and
/// [`FromStr`](core::str::FromStr) accepts them all, ignore-case.
///
/// **`Aifc` is a separate variant, not an `Aiff` alias** — an R5
/// correction. `.aifc` briefly (R2) lived in `Aiff.extensions()`; that
/// landing failed this crate's own identical-bytes test, the same class
/// of mistake `.apl` made one round later on `Ape` — see [`Self::Aifc`]
/// and [`Self::Aiff`]'s own docs for the byte-level distinction.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::audio_container_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum ContainerFormat {
  /// MPEG-1/2 Audio Layer III (`.mp3`). The auto-derived predicate
  /// name would be `is_mp_3` (digit-snake-case); the hand-written
  /// [`Self::is_mp3`] uses the cleaner name.
  #[is_variant(ignore)]
  Mp3,
  /// Raw AAC ADTS / ADIF stream (`.aac`; `.adts` is a recognised
  /// alias). IANA's `audio/aac` registration lists `.adts` alongside
  /// `.aac`; ffmpeg's own ADTS muxer/demuxer (`adts`) independently
  /// confirms `.adts` as a real on-disk spelling for the same raw
  /// bitstream `.aac` names — see the module doc's R3 provenance
  /// table.
  Aac,
  /// Free Lossless Audio Codec (`.flac`).
  Flac,
  /// Ogg Vorbis / generic Ogg container (`.ogg`; `.oga` / `.spx` are
  /// recognised aliases). RFC 5334 §10.3 registers all three under
  /// `audio/ogg`: `.ogg` is the legacy Vorbis-only spelling (kept
  /// canonical here — it predates and remains this crate's existing
  /// choice), `.oga` is the Skeleton-aware general-audio spelling, and
  /// `.spx` is the legacy Speex-only spelling. mediaframe does not
  /// split `Ogg` by codec (Vorbis / Opus / Speex / FLAC-in-Ogg all
  /// live under this one variant, per its own "generic Ogg container"
  /// framing above), so `.spx` is treated the same way as `.oga`: an
  /// alternate spelling of the same container, not a second format.
  Ogg,
  /// Opus in Ogg or raw (`.opus`).
  Opus,
  /// RIFF WAVE (`.wav`).
  Wav,
  /// Audio Interchange File Format — uncompressed PCM (`.aiff`;
  /// `.aif` is a recognised alias, the truncated legacy spelling of the
  /// identical form).
  ///
  /// **No longer includes `.aifc`** (R5 correction — see [`Self::Aifc`]'s
  /// own doc for why). What remains is a true alias: ExifTool keeps
  /// `AIF` as a pure `Lookup::Alias` of `AIFF` (byte-identical, spelling
  /// only), unlike `AIFC` which ExifTool keeps as its own file-type
  /// entry — that distinction was already correctly recorded in this
  /// crate's own R2 census; the wrong call was folding `AIFC` in anyway.
  Aiff,
  /// AIFF-Compressed — Apple's IFF-based container for *compressed*
  /// audio codecs riding the same chunked-container family as
  /// [`Self::Aiff`], but not the same on-disk bytes (`.aifc`).
  ///
  /// **Not [`Self::Aiff`]** — an R5 correction (Codex R5 HIGH finding,
  /// per the user's 甲 ruling). Both are `FORM <size> <formType>
  /// <chunks…>` IFF containers, but the 4-byte `formType` at the fixed
  /// header offset literally differs — `AIFF` for plain AIFF, `AIFC`
  /// for this variant, per Apple's own AIFF-C specification
  /// ("Audio Interchange File Format AIFF-C", Apple Computer, 1991).
  /// AIFC additionally *requires* an `FVER` (Format Version) chunk plain
  /// AIFF never carries, and its `COMM` (Common) chunk is a strict
  /// superset of AIFF's — two extra fields, a 4-byte `compressionType`
  /// code and a Pascal-string `compressionName`, that plain AIFF's
  /// `COMM` chunk has no room for. A plain-AIFF reader hits the
  /// `formType`-and-`COMM` mismatch immediately; this is not a filename
  /// convention, it is a different required byte layout from the first
  /// twelve bytes on. ffmpeg's own dedicated `aiff` demuxer/muxer reads
  /// and writes both forms (compressed COMM-chunk codecs included) under
  /// one implementation, the same "shared tooling, still a different
  /// format" shape `.apl` and `.m2ts`/`.3g2` already established
  /// elsewhere in this sweep — shared tooling is a hint to check, never
  /// itself the identical-bytes proof.
  Aifc,
  /// Apple Lossless (ALAC) — usually carried inside `.m4a`,
  /// occasionally `.caf`; this variant is the bare-codec spelling.
  /// **`.caf` is deliberately not a recognised alias extension of this
  /// variant** — [`Self::Caf`] already names that container in its own
  /// right, and treating `.caf` as also-Alac would claim one on-disk
  /// spelling names two different formats depending on which variant
  /// asked, unlike the genuine same-format aliases elsewhere in this
  /// enum.
  Alac,
  /// Windows Media Audio (`.wma`).
  Wma,
  /// Monkey's Audio (`.ape`; `.mac` is a recognised alias — genuinely
  /// the same bitstream, not just a shared demuxer, and this one is
  /// verified rather than taken on ffmpeg's extension list alone).
  ///
  /// **`.mac` is Monkey's Audio's own original extension.** Per the
  /// project's own official version history (monkeysaudio.com,
  /// `versionhistory.html`): v2.40 beta already shipped "a verify mode
  /// to verify `.mac` files"; **v3.00**: "now uses the extension `.APE`
  /// instead of `.MAC`" — a pure rename, not a format change; v3.40
  /// beta then added backward-compatible playback support for "the old
  /// `.MAC` extension" specifically *because* old `.mac` files remained
  /// byte-identical Monkey's Audio content. The on-disk format's own
  /// magic signature never changed across the rename — `"MAC "` (4
  /// bytes, space included) — which is exactly what ExifTool's file-type
  /// detector still matches for `APE` today (`head.starts_with(b"MAC ")`,
  /// no separate rule for either extension).
  ///
  /// **Empirically re-verified**, not just documented: a synthetic
  /// 332-byte file was built from the real Monkey's Audio v3.98+
  /// `APE_DESCRIPTOR` + `APE_HEADER` layout (`"MAC "` + version 3990 +
  /// the documented header fields) and probed with `ffprobe` under three
  /// extensions (`.mac`, `.ape`, an extension ffmpeg has no APE
  /// association for at all) and both auto-detected and forced
  /// (`-f ape`) — all five runs produced the *identical* `ape`-demuxer
  /// diagnostic (`"No frames in the file!"`, i.e. the header parsed
  /// successfully as valid APE structure; the only failure is the
  /// deliberately-omitted real audio frame data). Extension played no
  /// role in any of the five outcomes — the signature alone decided it,
  /// for `.mac` exactly as it does for `.ape`. This is the same
  /// ffprobe-through-the-demuxer method that *rejected* `.apl` (below);
  /// here it accepts `.mac` outright.
  ///
  /// **`.apl` is deliberately excluded** (R4 correction — landed in R3,
  /// then reverted): ffmpeg's `ape` demuxer common-extensions field also
  /// lists `apl`, but an APE Link file is Monkey's Audio's own per-track
  /// *sidecar* — split points derived from a CUE sheet against a
  /// companion `.ape` image — not the compressed bitstream itself.
  /// ffprobe verified this directly: forced through the `ape` demuxer,
  /// APL content is rejected (no MAC bitstream signature to probe for).
  /// ffmpeg's shared demuxer registration was a hint to go check, not
  /// itself the proof — same "different bytes, shared infrastructure"
  /// class this module already excludes `.m4v` and `.mp1`/`.mp2` under.
  Ape,
  /// WavPack (`.wv`; `.wvp` is a recognised alias — ExifTool's own
  /// file-type table aliases `WVP` directly to `WV`).
  Wv,
  /// Matroska Audio (`.mka`).
  Mka,
  /// MPEG-4 audio-only (`.m4a`) — AAC / ALAC in an MP4 box layout.
  /// The auto-derived predicate name would be `is_m_4_a`
  /// (digit-snake-case); the hand-written [`Self::is_m4a`] uses the
  /// cleaner name.
  #[is_variant(ignore)]
  M4a,
  /// Apple Core Audio Format (`.caf`).
  Caf,
  /// A container not enumerated above — carries the
  /// extension-style slug verbatim. Lossless escape.
  Other(Utf8Bytes),
}

impl Default for ContainerFormat {
  /// `Other("")` — the wire-zero / "absent" sentinel. Audio
  /// containers vary by source; there is no universally-defensible
  /// default. Callers picking a meaningful fallback should be
  /// explicit.
  #[inline]
  fn default() -> Self {
    Self::Other(Utf8Bytes::new())
  }
}

impl ContainerFormat {
  /// True iff this is [`Self::Mp3`]. Hand-written to override the
  /// auto-derived `is_mp_3` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_mp3(&self) -> bool {
    matches!(self, Self::Mp3)
  }

  /// True iff this is [`Self::M4a`]. Hand-written to override the
  /// auto-derived `is_m_4_a` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_m4a(&self) -> bool {
    matches!(self, Self::M4a)
  }

  /// File-extension-style slug (`"mp3"`, `"aac"`, `"flac"`, …).
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mp3 => "mp3",
      Self::Aac => "aac",
      Self::Flac => "flac",
      Self::Ogg => "ogg",
      Self::Opus => "opus",
      Self::Wav => "wav",
      Self::Aiff => "aiff",
      Self::Aifc => "aifc",
      Self::Alac => "alac",
      Self::Wma => "wma",
      Self::Ape => "ape",
      Self::Wv => "wv",
      Self::Mka => "mka",
      Self::M4a => "m4a",
      Self::Caf => "caf",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Primary file-on-disk extension (without the leading dot —
  /// `"mp3"`, `"flac"`, `"m4a"`, …). For most audio containers the
  /// extension matches the FFmpeg slug from [`Self::as_str`]; the
  /// exception is `Alac`, which has no standalone extension (the
  /// codec rides inside `.m4a`), so this method returns `"m4a"`.
  ///
  /// Returns `""` for [`Self::Other`] — the open variant carries an
  /// FFmpeg slug, not an extension, so the mapping is unknown.
  /// Returns `&'static str` (not `&str`) so the value is compile-time
  /// stable and the method is `const`.
  #[inline(always)]
  pub const fn as_extension(&self) -> &'static str {
    match self {
      Self::Mp3 => "mp3",
      Self::Aac => "aac",
      Self::Flac => "flac",
      Self::Ogg => "ogg",
      Self::Opus => "opus",
      Self::Wav => "wav",
      Self::Aiff => "aiff",
      Self::Aifc => "aifc",
      Self::Alac => "m4a",
      Self::Wma => "wma",
      Self::Ape => "ape",
      Self::Wv => "wv",
      Self::Mka => "mka",
      Self::M4a => "m4a",
      Self::Caf => "caf",
      Self::Other(_) => "",
    }
  }

  /// Every recognised on-disk spelling for this format, canonical first
  /// (== [`Self::as_extension`]) and aliases after. [`FromStr`] accepts
  /// every entry, ignore-case — a caller collecting "every spelling this
  /// format might be saved under" should iterate this rather than call
  /// [`Self::as_extension`] alone.
  ///
  /// Most variants carry exactly one spelling; see each variant's own doc
  /// for where a listed alias comes from — ExifTool's own file-type
  /// aliases for most, RFC 5334 §10.3 for `Ogg`'s `.oga`/`.spx`. `Alac`
  /// deliberately does **not** list `.caf` — see its own doc.
  ///
  /// Returns `&[]` for [`Self::Other`] — the open variant carries an
  /// FFmpeg slug, not a known extension set.
  #[inline]
  pub const fn extensions(&self) -> &'static [&'static str] {
    match self {
      Self::Mp3 => &["mp3"],
      Self::Aac => &["aac", "adts"],
      Self::Flac => &["flac"],
      Self::Ogg => &["ogg", "oga", "spx"],
      Self::Opus => &["opus"],
      Self::Wav => &["wav"],
      Self::Aiff => &["aiff", "aif"],
      Self::Aifc => &["aifc"],
      Self::Alac => &["m4a"],
      Self::Wma => &["wma"],
      Self::Ape => &["ape", "mac"],
      Self::Wv => &["wv", "wvp"],
      Self::Mka => &["mka"],
      Self::M4a => &["m4a"],
      Self::Caf => &["caf"],
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
  ContainerFormat,
  "audio container format",
  [
    Mp3, Aac, Flac, Ogg, Opus, Wav, Aiff, Aifc, Alac, Wma, Ape, Wv, Mka,
    M4a, Caf
  ],
  escape: Other
);

impl FromStr for ContainerFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical extension-style slug **or any alias
  /// extension** from [`Self::extensions`]; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"mp3" => Self::Mp3,
      b"aac" | b"adts" => Self::Aac,
      b"flac" => Self::Flac,
      b"ogg" | b"oga" | b"spx" => Self::Ogg,
      b"opus" => Self::Opus,
      b"wav" => Self::Wav,
      b"aiff" | b"aif" => Self::Aiff,
      b"aifc" => Self::Aifc,
      b"alac" => Self::Alac,
      b"wma" => Self::Wma,
      b"ape" | b"mac" => Self::Ape,
      b"wv" | b"wvp" => Self::Wv,
      b"mka" => Self::Mka,
      b"m4a" => Self::M4a,
      b"caf" => Self::Caf,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

#[cfg(test)]
mod tests;
