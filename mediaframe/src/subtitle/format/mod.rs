//! [`Format`] — file/container vocabulary for subtitle streams.
//!
//! Distinct from [`crate::codec::SubtitleCodec`]: `SubtitleCodec` is the
//! FFmpeg *codec* family enumerated by `libavcodec` (`srt` / `webvtt` /
//! `ass` / …), whereas `Format` is the *file form* / *demuxer
//! tag* — what you get from `ffprobe`'s `format_name` for a sidecar
//! subtitle file, or the matroska `S_TEXT/UTF8`-style stream tag of an
//! embedded track. The two correlate (a `SubtitleCodec::Srt` track is
//! usually carried in a `Format::Srt` file) but the modelling
//! split lets callers describe `.srt` content carried inside an `.mkv`
//! without lying about either axis.

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;

/// Subtitle file / track *format* — the demuxer-tag axis of a subtitle
/// stream (`"srt"` / `"webvtt"` / `"ass"` / image-based `"pgs"` / …).
///
/// `#[non_exhaustive]` keeps future additions non-breaking; the
/// [`Self::Other`] arm is the lossless escape for formats not yet
/// enumerated here. `as_str` returns the FFmpeg-canonical slug; the
/// total [`FromStr`] impl is its inverse.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::subtitle_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum Format {
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
  /// MPlayer2 (`.mpl`, FFmpeg slug `"mpl2"`). The auto-derived
  /// predicate name would be `is_mpl_2` (digit-snake-case); the
  /// hand-written [`Self::is_mpl2`] uses the cleaner name.
  #[is_variant(ignore)]
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
  /// Blu-Ray / HDMV PGS bitmap subtitles, under the FFmpeg-canonical
  /// demuxer name (`"hdmv_pgs_subtitle"`). Image-based.
  HdmvPgs,
  /// DVB bitmap subtitles — broadcast-TV image subtitles
  /// (FFmpeg slug `"dvb_subtitle"`). Image-based.
  DvbSub,
  /// DivX bitmap subtitles (FFmpeg slug `"xsub"`). Image-based.
  XSub,
  /// A format not enumerated above — carries the FFmpeg-style short
  /// name verbatim.
  Other(Utf8Bytes),
}

impl Default for Format {
  /// `Other("")` — the "unspecified format" sentinel, matching the
  /// convention used by every other string-bearing format enum in
  /// the crate (`audio::ContainerFormat`, `container::Format`,
  /// `audio::ChannelLayout`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Other(Utf8Bytes::new())
  }
}

impl Format {
  /// True iff this is [`Self::Mpl2`]. Hand-written to override the
  /// auto-derived `is_mpl_2` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_mpl2(&self) -> bool {
    matches!(self, Self::Mpl2)
  }

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
      Self::HdmvPgs => "hdmv_pgs_subtitle",
      Self::DvbSub => "dvb_subtitle",
      Self::XSub => "xsub",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Primary file-on-disk extension (without the leading dot —
  /// `"srt"`, `"vtt"`, `"ass"`, …). Distinct from the FFmpeg slug
  /// returned by [`Self::as_str`] for the formats whose codec name
  /// diverges from the extension (`WebVtt` slug `"webvtt"` vs
  /// extension `"vtt"`; `Sub` slug `"microdvd"` vs extension
  /// `"sub"`; `Smi` slug `"sami"` vs extension `"smi"`; `Sbv` slug
  /// `"subviewer"` vs extension `"sbv"`).
  ///
  /// **Image-based / container-embedded formats return `""`** —
  /// they have no standalone file extension because they ride
  /// inside a video container ([`Self::MovText`] in MP4 / MOV;
  /// [`Self::DvbSub`] in MPEG-TS broadcast; [`Self::XSub`] in
  /// DivX), come as multi-file pairs ([`Self::DvdSub`] writes
  /// paired `.idx` and `.sub` files), or are demuxed straight from
  /// disc images ([`Self::HdmvPgs`] becomes
  /// `.sup` only after extraction). Callers that have already
  /// extracted these to disk and know the conventional extension
  /// should hardcode it on their side; this method returns the
  /// **default-on-disk** presence, which for bitmap-subtitle
  /// variants is "none".
  ///
  /// Returns `""` for [`Self::Other`] — the open variant carries an
  /// FFmpeg slug, not an extension, so the mapping is unknown.
  /// Returns `&'static str` (not `&str`) so the value is compile-time
  /// stable and the method is `const`.
  #[inline(always)]
  pub const fn as_extension(&self) -> &'static str {
    match self {
      Self::Srt => "srt",
      Self::WebVtt => "vtt",
      Self::Ass => "ass",
      Self::Ssa => "ssa",
      Self::Sub => "sub",
      Self::Mpl2 => "mpl",
      Self::Lrc => "lrc",
      Self::Smi => "smi",
      Self::Stl => "stl",
      Self::Sbv => "sbv",
      Self::Ttml => "ttml",
      Self::MovText => "",
      Self::DvdSub => "",
      Self::HdmvPgs => "",
      Self::DvbSub => "",
      Self::XSub => "",
      Self::Other(_) => "",
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
  ///   [`Self::HdmvPgs`], [`Self::DvbSub`],
  ///   [`Self::XSub`]).
  /// - `Some(false)`: known text-based format (everything else
  ///   enumerated here).
  /// - `None`: [`Self::Other`] — the format is not in the
  ///   enumerated set, so this method cannot classify it.
  pub const fn is_image_based(&self) -> Option<bool> {
    match self {
      Self::DvdSub | Self::HdmvPgs | Self::DvbSub | Self::XSub => Some(true),
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
}

roster!(
  Format,
  "subtitle format",
  [
    Srt, WebVtt, Ass, Ssa, Sub, Mpl2, Lrc, Smi, Stl, Sbv, Ttml, MovText,
    DvdSub, HdmvPgs, DvbSub, XSub
  ],
  escape: Other
);

impl FromStr for Format {
  type Err = core::convert::Infallible;

  /// Recognise an FFmpeg-style short name; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"srt" => Self::Srt,
      b"webvtt" => Self::WebVtt,
      b"ass" => Self::Ass,
      b"ssa" => Self::Ssa,
      b"microdvd" => Self::Sub,
      b"mpl2" => Self::Mpl2,
      b"lrc" => Self::Lrc,
      b"sami" => Self::Smi,
      b"stl" => Self::Stl,
      b"subviewer" => Self::Sbv,
      b"ttml" => Self::Ttml,
      b"mov_text" => Self::MovText,
      b"dvd_subtitle" => Self::DvdSub,
      b"hdmv_pgs_subtitle" => Self::HdmvPgs,
      b"dvb_subtitle" => Self::DvbSub,
      b"xsub" => Self::XSub,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

#[cfg(test)]
mod tests;
