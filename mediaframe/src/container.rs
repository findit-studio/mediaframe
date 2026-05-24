//! Multimedia container-format vocabulary — top-level (video +
//! audio) containers.
//!
//! Audio-only containers (`mp3`, `flac`, `wav`, …) live on
//! [`crate::audio::ContainerFormat`]; this enum enumerates the
//! multimedia containers that carry one-or-more streams of *any*
//! kind (video, audio, subtitle, data).

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
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
  /// QuickTime File Format (`.mov`).
  Mov,
  /// ISO Base Media / MPEG-4 Part 14 (`.mp4`). The auto-derived
  /// predicate name would be `is_mp_4` (digit-snake-case); the
  /// hand-written [`Self::is_mp4`] uses the cleaner name.
  #[is_variant(ignore)]
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
  /// `.ogg` is [`crate::audio::ContainerFormat::Ogg`] instead.
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

impl Default for Format {
  /// `Other("")` — the wire-zero / "absent" sentinel. Containers
  /// vary by source; there is no universally-defensible default.
  /// Callers picking a meaningful fallback should be explicit
  /// (`Format::Mp4` is the common one).
  #[inline]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl Format {
  /// True iff this is [`Self::Mp4`]. Hand-written to override the
  /// auto-derived `is_mp_4` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_mp4(&self) -> bool {
    matches!(self, Self::Mp4)
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
      Self::Ogg => "ogv",
      Self::Asf => "asf",
      Self::Rm => "rm",
      Self::Wmv => "wmv",
      Self::Mxf => "mxf",
      Self::Gxf => "gxf",
      Self::Threegp => "3gp",
      Self::Other(_) => "",
    }
  }
}

impl FromStr for Format {
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
      let v: Format = slug.parse().unwrap();
      assert!(!v.is_other(), "`{slug}` should be a named variant");
      assert_eq!(v.as_str(), slug);
    }
  }

  #[test]
  fn unknown_slug_lands_in_other() {
    let v: Format = "weird_container".parse().unwrap();
    assert!(v.is_other());
    assert_eq!(v.as_str(), "weird_container");
    assert_eq!(v.to_string(), "weird_container");
  }

  #[test]
  fn display_matches_as_str() {
    assert_eq!(Format::Mp4.to_string(), "mp4");
    assert_eq!(Format::MpegTs.to_string(), "mpegts");
    assert_eq!(Format::Threegp.to_string(), "3gp");
    assert_eq!(Format::Other(SmolStr::new("custom")).to_string(), "custom");
  }

  #[test]
  fn is_variant_predicates() {
    // Hand-written `is_mp4` (vs the auto-derived `is_mp_4` that the
    // `IsVariant` derive would otherwise produce) — see the
    // `#[is_variant(ignore)]` attribute on `Format::Mp4`.
    assert!(Format::Mp4.is_mp4());
    assert!(!Format::Mkv.is_mp4());
    assert!(Format::Threegp.is_threegp());
    assert!(Format::Other(SmolStr::new("x")).is_other());
  }

  #[test]
  fn unwrap_other_borrowed_view() {
    // `Other(SmolStr)` carries data — golden-rule §2 mandates
    // unwrap/try_unwrap accessors for data-carrying variants.
    let v = Format::Other(SmolStr::new("custom"));
    assert_eq!(v.unwrap_other_ref().as_str(), "custom");
    assert!(v.try_unwrap_other_ref().is_ok());
    let named = Format::Mp4;
    assert!(named.try_unwrap_other_ref().is_err());
  }

  #[test]
  fn as_extension_matches_disk_form() {
    // Most variants: slug == extension.
    assert_eq!(Format::Mov.as_extension(), "mov");
    assert_eq!(Format::Mp4.as_extension(), "mp4");
    assert_eq!(Format::Mkv.as_extension(), "mkv");
    assert_eq!(Format::Webm.as_extension(), "webm");
    assert_eq!(Format::Avi.as_extension(), "avi");
    assert_eq!(Format::Flv.as_extension(), "flv");
    assert_eq!(Format::Threegp.as_extension(), "3gp");
    // Variants where extension differs from FFmpeg slug.
    assert_eq!(Format::MpegTs.as_str(), "mpegts");
    assert_eq!(Format::MpegTs.as_extension(), "ts");
    assert_eq!(Format::Ogg.as_str(), "ogg");
    assert_eq!(Format::Ogg.as_extension(), "ogv");
    // Other has no known extension.
    assert_eq!(Format::Other(SmolStr::new("weird")).as_extension(), "");
  }
}
