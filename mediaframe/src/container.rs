//! Multimedia container-format vocabulary — top-level (video +
//! audio) containers.
//!
//! Audio-only containers (`mp3`, `flac`, `wav`, …) live on
//! [`crate::audio::AudioContainerFormat`]; this enum enumerates the
//! multimedia containers that carry one-or-more streams of *any*
//! kind (video, audio, subtitle, data).

use core::str::FromStr;

use derive_more::{Display, IsVariant};
use smol_str::SmolStr;

/// Top-level multimedia container format.
///
/// Closed-ish vocabulary covering the containers a typical
/// media-ingest pipeline encounters — not FFmpeg-coded, so there is
/// no `to_u32`/`from_u32`; the `Other(SmolStr)` arm preserves
/// unknown slugs losslessly.
///
/// `as_str` returns the canonical extension-style slug (`"mov"`,
/// `"mp4"`, `"mkv"`, `"webm"`, …).
///
/// **Variant naming note:** the `.3gp` container's variant is named
/// [`Self::Threegp`] — Rust identifiers cannot start with a digit,
/// and `_3gp` would render as `"3gp"` under `derive_more::Display`'s
/// snake-casing but is unidiomatic. The `as_str()` / `FromStr`
/// surface still returns / matches the canonical `"3gp"` slug.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ContainerFormat {
  /// QuickTime File Format (`.mov`).
  Mov,
  /// ISO Base Media / MPEG-4 Part 14 (`.mp4`).
  Mp4,
  /// Matroska (`.mkv`).
  Mkv,
  /// WebM — Matroska subset for VP8/9 + Vorbis/Opus (`.webm`).
  Webm,
  /// Audio-Video Interleave (`.avi`).
  Avi,
  /// Flash Video (`.flv`).
  Flv,
  /// MPEG-2 Transport Stream (`.ts`, `.m2ts`). FFmpeg slug:
  /// `"mpegts"`.
  MpegTs,
  /// Ogg container (`.ogv` / `.ogx` — video-bearing Ogg). Audio-only
  /// `.ogg` is [`crate::audio::AudioContainerFormat::Ogg`] instead.
  Ogg,
  /// Advanced Systems Format (`.asf`).
  Asf,
  /// RealMedia (`.rm`).
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
  /// 3GPP / 3GPP2 (`.3gp`, `.3g2`). Variant name is `Threegp`
  /// because Rust identifiers cannot start with a digit.
  Threegp,
  /// A container not enumerated above — carries the
  /// extension-style slug verbatim. Lossless escape.
  Other(SmolStr),
}

impl Default for ContainerFormat {
  /// `Other("")` — the wire-zero / "absent" sentinel. Containers
  /// vary by source; there is no universally-defensible default.
  /// Callers picking a meaningful fallback should be explicit
  /// (`ContainerFormat::Mp4` is the common one).
  #[inline]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl ContainerFormat {
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
      Self::Ogg => "ogg",
      Self::Asf => "asf",
      Self::Rm => "rm",
      Self::Wmv => "wmv",
      Self::Mxf => "mxf",
      Self::Gxf => "gxf",
      Self::Threegp => "3gp",
      Self::Other(s) => s.as_str(),
    }
  }
}

impl FromStr for ContainerFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical container slug; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "mov" => Self::Mov,
      "mp4" => Self::Mp4,
      "mkv" => Self::Mkv,
      "webm" => Self::Webm,
      "avi" => Self::Avi,
      "flv" => Self::Flv,
      "mpegts" => Self::MpegTs,
      "ogg" => Self::Ogg,
      "asf" => Self::Asf,
      "rm" => Self::Rm,
      "wmv" => Self::Wmv,
      "mxf" => Self::Mxf,
      "gxf" => Self::Gxf,
      "3gp" => Self::Threegp,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::string::ToString;

  #[test]
  fn every_named_variant_round_trips() {
    for slug in [
      "mov", "mp4", "mkv", "webm", "avi", "flv", "mpegts", "ogg", "asf", "rm", "wmv", "mxf", "gxf",
      "3gp",
    ] {
      let v: ContainerFormat = slug.parse().unwrap();
      assert!(!v.is_other(), "`{slug}` should be a named variant");
      assert_eq!(v.as_str(), slug);
    }
  }

  #[test]
  fn unknown_slug_lands_in_other() {
    let v: ContainerFormat = "weird_container".parse().unwrap();
    assert!(v.is_other());
    assert_eq!(v.as_str(), "weird_container");
    assert_eq!(v.to_string(), "weird_container");
  }

  #[test]
  fn display_matches_as_str() {
    assert_eq!(ContainerFormat::Mp4.to_string(), "mp4");
    assert_eq!(ContainerFormat::MpegTs.to_string(), "mpegts");
    assert_eq!(ContainerFormat::Threegp.to_string(), "3gp");
    assert_eq!(
      ContainerFormat::Other(SmolStr::new("custom")).to_string(),
      "custom"
    );
  }

  #[test]
  fn is_variant_predicates() {
    assert!(ContainerFormat::Mp4.is_mp_4());
    assert!(ContainerFormat::Threegp.is_threegp());
    assert!(ContainerFormat::Other(SmolStr::new("x")).is_other());
  }
}
