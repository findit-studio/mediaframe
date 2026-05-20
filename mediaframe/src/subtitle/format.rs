//! [`SubtitleFormat`] — file/container vocabulary for subtitle streams.
//!
//! Distinct from [`crate::codec::SubtitleCodec`]: `SubtitleCodec` is the
//! FFmpeg *codec* family enumerated by `libavcodec` (`srt` / `webvtt` /
//! `ass` / …), whereas `SubtitleFormat` is the *file form* / *demuxer
//! tag* — what you get from `ffprobe`'s `format_name` for a sidecar
//! subtitle file, or the matroska `S_TEXT/UTF8`-style stream tag of an
//! embedded track. The two correlate (a `SubtitleCodec::Srt` track is
//! usually carried in a `SubtitleFormat::Srt` file) but the modelling
//! split lets callers describe `.srt` content carried inside an `.mkv`
//! without lying about either axis.

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_str::SmolStr;

/// Subtitle file / track *format* — the demuxer-tag axis of a subtitle
/// stream (`"srt"` / `"webvtt"` / `"ass"` / image-based `"pgs"` / …).
///
/// `#[non_exhaustive]` keeps future additions non-breaking; the
/// [`Self::Other`] arm is the lossless escape for formats not yet
/// enumerated here. `as_str` returns the FFmpeg-canonical slug; the
/// total [`FromStr`] impl is its inverse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum SubtitleFormat {
  /// SubRip — the canonical text-based subtitle format
  /// (`.srt`, FFmpeg slug `"srt"`).
  Srt,
  /// WebVTT — Web Video Text Tracks (`.vtt`, FFmpeg slug `"webvtt"`).
  WebVtt,
  /// Advanced SubStation Alpha (`.ass`, FFmpeg slug `"ass"`).
  Ass,
  /// SubStation Alpha — the v4 predecessor to [`Self::Ass`]
  /// (`.ssa`, FFmpeg slug `"ssa"`).
  Ssa,
  /// MicroDVD (`.sub`, FFmpeg slug `"microdvd"`).
  Sub,
  /// MPlayer2 (`.mpl`, FFmpeg slug `"mpl2"`).
  Mpl2,
  /// LRC — synchronised lyrics, also used for karaoke subtitles
  /// (`.lrc`, FFmpeg slug `"lrc"`).
  Lrc,
  /// SAMI — Synchronized Accessible Media Interchange
  /// (`.smi`, FFmpeg slug `"sami"`).
  Smi,
  /// EBU STL — European Broadcasting Union Subtitle Tape Format
  /// (`.stl`, FFmpeg slug `"stl"`).
  Stl,
  /// YouTube SubViewer (`.sbv`, FFmpeg slug `"subviewer"`).
  Sbv,
  /// W3C Timed Text Markup Language (`.ttml` / `.xml`, FFmpeg slug
  /// `"ttml"`).
  Ttml,
  /// 3GPP / MP4 timed text — `tx3g` boxes in MP4 / MOV containers
  /// (FFmpeg slug `"mov_text"`).
  MovText,
  /// DVD bitmap subtitles — SPU streams from a DVD-Video VOB
  /// (FFmpeg slug `"dvd_subtitle"`). Image-based.
  DvdSub,
  /// Blu-Ray / HDMV PGS bitmap subtitles
  /// (FFmpeg slug `"hdmv_pgs_subtitle"`). Image-based. Alias for
  /// [`Self::HdmvPgs`].
  PgsSub,
  /// Blu-Ray / HDMV PGS bitmap subtitles — same wire format as
  /// [`Self::PgsSub`] under the FFmpeg-canonical demuxer name
  /// (`"hdmv_pgs_subtitle"`). Image-based.
  HdmvPgs,
  /// DVB bitmap subtitles — broadcast-TV image subtitles
  /// (FFmpeg slug `"dvb_subtitle"`). Image-based.
  DvbSub,
  /// DivX bitmap subtitles (FFmpeg slug `"xsub"`). Image-based.
  XSub,
  /// A format not enumerated above — carries the FFmpeg-style short
  /// name verbatim.
  Other(SmolStr),
}

impl SubtitleFormat {
  /// Canonical FFmpeg-style short name for this format (matches the
  /// demuxer / codec slug FFmpeg uses for the corresponding file
  /// form). [`Self::Other`] returns the wrapped slug verbatim.
  pub fn as_str(&self) -> &str {
    match self {
      Self::Srt => "srt",
      Self::WebVtt => "webvtt",
      Self::Ass => "ass",
      Self::Ssa => "ssa",
      Self::Sub => "microdvd",
      Self::Mpl2 => "mpl2",
      Self::Lrc => "lrc",
      Self::Smi => "sami",
      Self::Stl => "stl",
      Self::Sbv => "subviewer",
      Self::Ttml => "ttml",
      Self::MovText => "mov_text",
      Self::DvdSub => "dvd_subtitle",
      Self::PgsSub => "hdmv_pgs_subtitle",
      Self::HdmvPgs => "hdmv_pgs_subtitle",
      Self::DvbSub => "dvb_subtitle",
      Self::XSub => "xsub",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Is this format **image-based** (rendered subtitles carried as
  /// bitmaps), as opposed to text-based?
  ///
  /// Required by mediaschema's `MediaErrorFlags::REQUIRES_OCR`
  /// derivation: bitmap subtitle tracks need an OCR pipeline stage
  /// to extract searchable text.
  ///
  /// - `Some(true)`: known image-based format ([`Self::DvdSub`],
  ///   [`Self::PgsSub`], [`Self::HdmvPgs`], [`Self::DvbSub`],
  ///   [`Self::XSub`]).
  /// - `Some(false)`: known text-based format (everything else
  ///   enumerated here).
  /// - `None`: [`Self::Other`] — the format is not in the
  ///   enumerated set, so this method cannot classify it.
  pub const fn is_image_based(&self) -> Option<bool> {
    match self {
      Self::DvdSub | Self::PgsSub | Self::HdmvPgs | Self::DvbSub | Self::XSub => Some(true),
      Self::Srt
      | Self::WebVtt
      | Self::Ass
      | Self::Ssa
      | Self::Sub
      | Self::Mpl2
      | Self::Lrc
      | Self::Smi
      | Self::Stl
      | Self::Sbv
      | Self::Ttml
      | Self::MovText => Some(false),
      Self::Other(_) => None,
    }
  }
}

impl FromStr for SubtitleFormat {
  type Err = core::convert::Infallible;

  /// Recognise an FFmpeg-style short name; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "srt" => Self::Srt,
      "webvtt" => Self::WebVtt,
      "ass" => Self::Ass,
      "ssa" => Self::Ssa,
      "microdvd" => Self::Sub,
      "mpl2" => Self::Mpl2,
      "lrc" => Self::Lrc,
      "sami" => Self::Smi,
      "stl" => Self::Stl,
      "subviewer" => Self::Sbv,
      "ttml" => Self::Ttml,
      "mov_text" => Self::MovText,
      "dvd_subtitle" => Self::DvdSub,
      "hdmv_pgs_subtitle" => Self::PgsSub,
      "dvb_subtitle" => Self::DvbSub,
      "xsub" => Self::XSub,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::string::ToString;

  /// Every named variant's slug round-trips through `as_str` →
  /// `FromStr`. [`SubtitleFormat::HdmvPgs`] shares its slug with
  /// [`SubtitleFormat::PgsSub`] so the round-trip canonicalises to
  /// `PgsSub`; that pair is verified separately.
  const NAMED_SLUGS: &[(&str, SubtitleFormat)] = &[
    ("srt", SubtitleFormat::Srt),
    ("webvtt", SubtitleFormat::WebVtt),
    ("ass", SubtitleFormat::Ass),
    ("ssa", SubtitleFormat::Ssa),
    ("microdvd", SubtitleFormat::Sub),
    ("mpl2", SubtitleFormat::Mpl2),
    ("lrc", SubtitleFormat::Lrc),
    ("sami", SubtitleFormat::Smi),
    ("stl", SubtitleFormat::Stl),
    ("subviewer", SubtitleFormat::Sbv),
    ("ttml", SubtitleFormat::Ttml),
    ("mov_text", SubtitleFormat::MovText),
    ("dvd_subtitle", SubtitleFormat::DvdSub),
    ("hdmv_pgs_subtitle", SubtitleFormat::PgsSub),
    ("dvb_subtitle", SubtitleFormat::DvbSub),
    ("xsub", SubtitleFormat::XSub),
  ];

  #[test]
  fn as_str_round_trips_for_every_named_variant() {
    for (slug, variant) in NAMED_SLUGS {
      assert_eq!(variant.as_str(), *slug, "as_str mismatch for {variant:?}");
      let parsed: SubtitleFormat = slug.parse().unwrap();
      assert_eq!(&parsed, variant, "FromStr mismatch for {slug:?}");
    }
  }

  #[test]
  fn hdmv_pgs_slug_canonicalises_to_pgs_sub() {
    // `HdmvPgs` and `PgsSub` share the FFmpeg `"hdmv_pgs_subtitle"`
    // slug. Both render to it; parsing the slug picks the first
    // arm — `PgsSub`. `HdmvPgs` is kept as an alias for callers
    // that prefer the FFmpeg-canonical name.
    assert_eq!(SubtitleFormat::HdmvPgs.as_str(), "hdmv_pgs_subtitle");
    assert_eq!(SubtitleFormat::PgsSub.as_str(), "hdmv_pgs_subtitle");
    let parsed: SubtitleFormat = "hdmv_pgs_subtitle".parse().unwrap();
    assert_eq!(parsed, SubtitleFormat::PgsSub);
  }

  #[test]
  fn from_str_is_total_for_unknown_slug() {
    let parsed: SubtitleFormat = "definitely_not_a_real_subtitle_format_xyz".parse().unwrap();
    assert!(matches!(parsed, SubtitleFormat::Other(_)));
    assert_eq!(parsed.as_str(), "definitely_not_a_real_subtitle_format_xyz");
  }

  #[test]
  fn is_image_based_classifies_known_variants() {
    // Image-based.
    assert_eq!(SubtitleFormat::DvdSub.is_image_based(), Some(true));
    assert_eq!(SubtitleFormat::PgsSub.is_image_based(), Some(true));
    assert_eq!(SubtitleFormat::HdmvPgs.is_image_based(), Some(true));
    assert_eq!(SubtitleFormat::DvbSub.is_image_based(), Some(true));
    assert_eq!(SubtitleFormat::XSub.is_image_based(), Some(true));
    // Text-based.
    assert_eq!(SubtitleFormat::Srt.is_image_based(), Some(false));
    assert_eq!(SubtitleFormat::WebVtt.is_image_based(), Some(false));
    assert_eq!(SubtitleFormat::Ass.is_image_based(), Some(false));
    assert_eq!(SubtitleFormat::MovText.is_image_based(), Some(false));
    // Unknown.
    assert_eq!(
      SubtitleFormat::Other(SmolStr::new("weird")).is_image_based(),
      None,
    );
  }

  #[test]
  fn display_matches_as_str() {
    for (_slug, variant) in NAMED_SLUGS {
      assert_eq!(variant.to_string(), variant.as_str());
    }
    assert_eq!(
      SubtitleFormat::Other(SmolStr::new("custom_fmt")).to_string(),
      "custom_fmt",
    );
  }

  #[test]
  fn is_variant_predicates() {
    assert!(SubtitleFormat::Srt.is_srt());
    assert!(!SubtitleFormat::Srt.is_web_vtt());
    assert!(SubtitleFormat::Other(SmolStr::new("x")).is_other());
  }
}
