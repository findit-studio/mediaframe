//! Audio sample-format vocabulary (`AudioFormat`, FFmpeg
//! `AVSampleFormat`) and audio-only container-format vocabulary
//! (`AudioContainerFormat`, audio file extensions).

use core::str::FromStr;

use derive_more::{Display, IsVariant};
use smol_str::SmolStr;

/// Audio sample format — FFmpeg `AVSampleFormat`.
///
/// One named variant per FFmpeg n8.1 sample format (the standard 12
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum AudioFormat {
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
  /// Unknown / unrecognised FFmpeg `AV_SAMPLE_FMT_*` code. The
  /// wrapped `u32` is the original value passed to
  /// [`Self::from_u32`] — preserved so the round-trip is lossless.
  Unknown(u32),
  /// A format slug not enumerated above — carries the slug verbatim
  /// (the [`Self::from_str`] lossless escape).
  Other(SmolStr),
}

impl Default for AudioFormat {
  /// `AV_SAMPLE_FMT_NONE` is `-1` in FFmpeg; we use [`Self::Unknown`]
  /// at code `u32::MAX` as the sentinel (no real code overlaps).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Unknown(u32::MAX)
  }
}

impl AudioFormat {
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
      Self::Unknown(_) => "unknown",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable wire id — the FFmpeg `AV_SAMPLE_FMT_*` enum index for
  /// the named variants. [`Self::Unknown`] carries its original
  /// `u32` through unchanged so `from_u32(to_u32(x)) == x` for every
  /// `x`. [`Self::Other`] (slug-bearing escape) encodes as
  /// `u32::MAX` — it carries no FFmpeg numeric id, so it
  /// canonicalises through the wire to [`Self::Unknown(u32::MAX)`]
  /// (the slug is preserved only on the string-codec path).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn to_u32(&self) -> u32 {
    match self {
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
      Self::Unknown(v) => *v,
      Self::Other(_) => u32::MAX,
    }
  }

  /// Decodes from the FFmpeg `AV_SAMPLE_FMT_*` code produced by
  /// [`Self::to_u32`]. Unrecognised codes round-trip as
  /// [`Self::Unknown`] (lossless).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
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
      _ => Self::Unknown(v),
    }
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

impl FromStr for AudioFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical FFmpeg sample-format slug; unknown
  /// values land in [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "u8" => Self::U8,
      "s16" => Self::S16,
      "s32" => Self::S32,
      "flt" => Self::Flt,
      "dbl" => Self::Dbl,
      "u8p" => Self::U8p,
      "s16p" => Self::S16p,
      "s32p" => Self::S32p,
      "fltp" => Self::Fltp,
      "dblp" => Self::Dblp,
      "s64" => Self::S64,
      "s64p" => Self::S64p,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}

// ---------------------------------------------------------------------------

/// Audio-only file / container format vocabulary.
///
/// Top-level multimedia containers (`mp4`/`mkv`/`mov`/`webm`/…)
/// live on [`crate::container::ContainerFormat`]; this enum
/// enumerates the **audio-only** containers (one audio stream, no
/// video). Closed-ish vocabulary — not FFmpeg-coded, so there is no
/// `to_u32`/`from_u32`; the `Other(SmolStr)` arm preserves unknown
/// slugs losslessly.
///
/// `as_str` returns the file-extension-style slug (`"mp3"`, `"aac"`,
/// `"flac"`, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum AudioContainerFormat {
  /// MPEG-1/2 Audio Layer III (`.mp3`).
  Mp3,
  /// Raw AAC ADTS / ADIF stream (`.aac`).
  Aac,
  /// Free Lossless Audio Codec (`.flac`).
  Flac,
  /// Ogg Vorbis / generic Ogg container (`.ogg`).
  Ogg,
  /// Opus in Ogg or raw (`.opus`).
  Opus,
  /// RIFF WAVE (`.wav`).
  Wav,
  /// Audio Interchange File Format (`.aiff` / `.aif`).
  Aiff,
  /// Apple Lossless (ALAC) — usually carried inside `.m4a`,
  /// occasionally `.caf`; this variant is the bare-codec spelling.
  Alac,
  /// Windows Media Audio (`.wma`).
  Wma,
  /// Monkey's Audio (`.ape`).
  Ape,
  /// WavPack (`.wv`).
  Wv,
  /// Matroska Audio (`.mka`).
  Mka,
  /// MPEG-4 audio-only (`.m4a`) — AAC / ALAC in an MP4 box layout.
  M4a,
  /// Apple Core Audio Format (`.caf`).
  Caf,
  /// A container not enumerated above — carries the
  /// extension-style slug verbatim. Lossless escape.
  Other(SmolStr),
}

impl Default for AudioContainerFormat {
  /// `Other("")` — the wire-zero / "absent" sentinel. Audio
  /// containers vary by source; there is no universally-defensible
  /// default. Callers picking a meaningful fallback should be
  /// explicit.
  #[inline]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl AudioContainerFormat {
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
}

impl FromStr for AudioContainerFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical extension-style slug; unknown values
  /// land in [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "mp3" => Self::Mp3,
      "aac" => Self::Aac,
      "flac" => Self::Flac,
      "ogg" => Self::Ogg,
      "opus" => Self::Opus,
      "wav" => Self::Wav,
      "aiff" => Self::Aiff,
      "alac" => Self::Alac,
      "wma" => Self::Wma,
      "ape" => Self::Ape,
      "wv" => Self::Wv,
      "mka" => Self::Mka,
      "m4a" => Self::M4a,
      "caf" => Self::Caf,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::string::ToString;

  #[test]
  fn audio_format_u32_round_trips_named_variants() {
    for v in [
      AudioFormat::U8,
      AudioFormat::S16,
      AudioFormat::S32,
      AudioFormat::Flt,
      AudioFormat::Dbl,
      AudioFormat::U8p,
      AudioFormat::S16p,
      AudioFormat::S32p,
      AudioFormat::Fltp,
      AudioFormat::Dblp,
      AudioFormat::S64,
      AudioFormat::S64p,
    ] {
      let back = AudioFormat::from_u32(v.to_u32());
      assert_eq!(back, v, "round-trip mismatch for `{}`", v.as_str());
    }
  }

  #[test]
  fn audio_format_unknown_u32_round_trips() {
    let v = AudioFormat::Unknown(12_345);
    assert_eq!(AudioFormat::from_u32(v.to_u32()), v);
  }

  #[test]
  fn audio_format_from_str_named() {
    for slug in [
      "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
    ] {
      let v: AudioFormat = slug.parse().unwrap();
      assert!(!v.is_other(), "`{slug}` should be a named variant");
      assert_eq!(v.as_str(), slug);
    }
  }

  #[test]
  fn audio_format_unknown_slug_lands_in_other() {
    let v: AudioFormat = "weird_sample_fmt".parse().unwrap();
    assert!(v.is_other());
    assert_eq!(v.as_str(), "weird_sample_fmt");
  }

  #[test]
  fn audio_format_is_planar_predicate() {
    assert!(AudioFormat::U8p.is_planar());
    assert!(AudioFormat::S16p.is_planar());
    assert!(AudioFormat::Fltp.is_planar());
    assert!(!AudioFormat::U8.is_planar());
    assert!(!AudioFormat::Flt.is_planar());
  }

  #[test]
  fn audio_format_display_matches_as_str() {
    assert_eq!(AudioFormat::Flt.to_string(), "flt");
    assert_eq!(AudioFormat::Fltp.to_string(), "fltp");
  }

  #[test]
  fn audio_container_round_trips_named_variants() {
    for slug in [
      "mp3", "aac", "flac", "ogg", "opus", "wav", "aiff", "alac", "wma", "ape", "wv", "mka", "m4a",
      "caf",
    ] {
      let v: AudioContainerFormat = slug.parse().unwrap();
      assert!(!v.is_other(), "`{slug}` should be a named variant");
      assert_eq!(v.as_str(), slug);
    }
  }

  #[test]
  fn audio_container_unknown_lands_in_other() {
    let v: AudioContainerFormat = "weird_audio_container".parse().unwrap();
    assert!(v.is_other());
    assert_eq!(v.as_str(), "weird_audio_container");
  }

  #[test]
  fn audio_container_display_matches_as_str() {
    assert_eq!(AudioContainerFormat::Mp3.to_string(), "mp3");
    assert_eq!(AudioContainerFormat::Flac.to_string(), "flac");
    assert_eq!(
      AudioContainerFormat::Other(SmolStr::new("snd")).to_string(),
      "snd"
    );
  }
}
