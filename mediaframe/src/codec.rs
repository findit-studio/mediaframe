//! Stream-descriptor **codec** vocabulary for video, audio, and subtitle
//! tracks — the post-`0.1.0` "broadened charter" batch (locked in the
//! downstream mediaschema `schema/mediaframe-candidates.md`).
//!
//! Each enum is `#[non_exhaustive]` with an `Other(SmolStr)` escape so an
//! unrecognised codec round-trips losslessly through the domain.
//! **Codec-family only** — profile/level live in separate fields per the
//! locked schema convention (e.g. `AudioTrack.profile: SmolStr`).
//!
//! Convention contrast with the colour enums: the H.273 colour numbers are
//! a stable wire-numbered vocabulary, so those enums use the
//! `Unknown(u32)` + `to_u32`/`from_u32` pattern. Codec IDs are *not* a
//! stable numeric vocabulary (FFmpeg's `AVCodecID` is huge and churns
//! across versions), so codecs use a string-tagged `Other(SmolStr)` arm
//! instead.

use core::str::FromStr;

use derive_more::{Display, IsVariant};
use smol_str::SmolStr;

// ===========================================================================
// VideoCodec
// ===========================================================================

/// Video codec family.
///
/// Named variants cover the common cases in real-world media; everything
/// else round-trips through [`Self::Other`] (the lowercase short string
/// FFmpeg / containers use — `"h264"`, `"hevc"`, `"av1"`, …).
///
/// `#[non_exhaustive]` so consumers must handle future variants without a
/// breaking-change build break.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum VideoCodec {
    /// H.264 / AVC (`"h264"`).
    H264,
    /// H.265 / HEVC (`"hevc"`).
    Hevc,
    /// AOMedia Video 1 (`"av1"`).
    Av1,
    /// VP9 (`"vp9"`).
    Vp9,
    /// VP8 (`"vp8"`).
    Vp8,
    /// MPEG-2 Part 2 (`"mpeg2video"`).
    Mpeg2,
    /// MPEG-4 Part 2 (legacy DivX/Xvid, `"mpeg4"`).
    Mpeg4,
    /// Apple ProRes (`"prores"`).
    ProRes,
    /// DNxHD / DNxHR (`"dnxhd"`).
    DnxHd,
    /// AVC Intra (`"avc_intra"`).
    AvcIntra,
    /// JPEG 2000 (`"jpeg2000"`).
    Jpeg2000,
    /// Motion JPEG (`"mjpeg"`).
    Mjpeg,
    /// Theora (`"theora"`).
    Theora,
    /// FFV1 (lossless intermediate, `"ffv1"`).
    Ffv1,
    /// Anything else — carries the codec's short string verbatim.
    Other(SmolStr),
}

impl VideoCodec {
    /// Canonical short string (lowercase; matches the FFmpeg / container
    /// short name where applicable).
    pub fn as_str(&self) -> &str {
        match self {
            Self::H264 => "h264",
            Self::Hevc => "hevc",
            Self::Av1 => "av1",
            Self::Vp9 => "vp9",
            Self::Vp8 => "vp8",
            Self::Mpeg2 => "mpeg2video",
            Self::Mpeg4 => "mpeg4",
            Self::ProRes => "prores",
            Self::DnxHd => "dnxhd",
            Self::AvcIntra => "avc_intra",
            Self::Jpeg2000 => "jpeg2000",
            Self::Mjpeg => "mjpeg",
            Self::Theora => "theora",
            Self::Ffv1 => "ffv1",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl FromStr for VideoCodec {
    type Err = core::convert::Infallible;

    /// Recognise a codec from its short string; unknown values land in
    /// [`Self::Other`] (lossless — never fails).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "h264" => Self::H264,
            "hevc" | "h265" => Self::Hevc,
            "av1" => Self::Av1,
            "vp9" => Self::Vp9,
            "vp8" => Self::Vp8,
            "mpeg2video" | "mpeg2" => Self::Mpeg2,
            "mpeg4" => Self::Mpeg4,
            "prores" => Self::ProRes,
            "dnxhd" | "dnxhr" => Self::DnxHd,
            "avc_intra" => Self::AvcIntra,
            "jpeg2000" => Self::Jpeg2000,
            "mjpeg" => Self::Mjpeg,
            "theora" => Self::Theora,
            "ffv1" => Self::Ffv1,
            other => Self::Other(SmolStr::new(other)),
        })
    }
}

// ===========================================================================
// AudioCodec
// ===========================================================================

/// Audio codec family.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum AudioCodec {
    /// AAC (`"aac"`).
    Aac,
    /// MP3 (`"mp3"`).
    Mp3,
    /// FLAC (`"flac"`).
    Flac,
    /// Opus (`"opus"`).
    Opus,
    /// Vorbis (`"vorbis"`).
    Vorbis,
    /// Dolby AC-3 (`"ac3"`).
    Ac3,
    /// Dolby Digital Plus / E-AC-3 (`"eac3"`).
    Eac3,
    /// DTS (`"dts"`).
    Dts,
    /// Dolby TrueHD (`"truehd"`).
    TrueHd,
    /// PCM 16-bit little-endian (`"pcm_s16le"`).
    PcmS16Le,
    /// PCM 24-bit little-endian (`"pcm_s24le"`).
    PcmS24Le,
    /// PCM 32-bit little-endian (`"pcm_s32le"`).
    PcmS32Le,
    /// PCM 32-bit float little-endian (`"pcm_f32le"`).
    PcmF32Le,
    /// ALAC (Apple Lossless, `"alac"`).
    Alac,
    /// WMA Pro / WMA v2 (`"wmapro"`/`"wmav2"` — collapsed; use `Other` if
    /// the variant matters).
    Wma,
    /// AMR Narrow Band (`"amr_nb"`).
    AmrNb,
    /// AMR Wide Band (`"amr_wb"`).
    AmrWb,
    /// Anything else — carries the codec's short string verbatim.
    Other(SmolStr),
}

impl AudioCodec {
    /// Canonical short string (lowercase; matches the FFmpeg / container
    /// short name where applicable).
    pub fn as_str(&self) -> &str {
        match self {
            Self::Aac => "aac",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Opus => "opus",
            Self::Vorbis => "vorbis",
            Self::Ac3 => "ac3",
            Self::Eac3 => "eac3",
            Self::Dts => "dts",
            Self::TrueHd => "truehd",
            Self::PcmS16Le => "pcm_s16le",
            Self::PcmS24Le => "pcm_s24le",
            Self::PcmS32Le => "pcm_s32le",
            Self::PcmF32Le => "pcm_f32le",
            Self::Alac => "alac",
            Self::Wma => "wma",
            Self::AmrNb => "amr_nb",
            Self::AmrWb => "amr_wb",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl FromStr for AudioCodec {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "aac" => Self::Aac,
            "mp3" => Self::Mp3,
            "flac" => Self::Flac,
            "opus" => Self::Opus,
            "vorbis" => Self::Vorbis,
            "ac3" => Self::Ac3,
            "eac3" => Self::Eac3,
            "dts" => Self::Dts,
            "truehd" => Self::TrueHd,
            "pcm_s16le" => Self::PcmS16Le,
            "pcm_s24le" => Self::PcmS24Le,
            "pcm_s32le" => Self::PcmS32Le,
            "pcm_f32le" => Self::PcmF32Le,
            "alac" => Self::Alac,
            "wma" | "wmav2" | "wmapro" => Self::Wma,
            "amr_nb" => Self::AmrNb,
            "amr_wb" => Self::AmrWb,
            other => Self::Other(SmolStr::new(other)),
        })
    }
}

// ===========================================================================
// SubtitleCodec
// ===========================================================================

/// Subtitle codec / format family.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum SubtitleCodec {
    /// SubRip Text (`"srt"`).
    Srt,
    /// Advanced SubStation Alpha (`"ass"`).
    Ass,
    /// SubStation Alpha (`"ssa"`).
    Ssa,
    /// Web Video Text Tracks (`"webvtt"`).
    WebVtt,
    /// MP4 timed text / 3GPP timed text (`"mov_text"`).
    MovText,
    /// DVB subtitles (bitmap, `"dvbsub"`).
    DvbSub,
    /// Presentation Graphic Stream — Blu-ray (bitmap, `"hdmv_pgs"`).
    Pgs,
    /// DVD subtitles (bitmap, `"dvd_subtitle"`).
    DvdSub,
    /// CEA-608 closed captions (`"eia_608"`).
    Cea608,
    /// CEA-708 closed captions (`"eia_708"`).
    Cea708,
    /// TTML (`"ttml"`).
    Ttml,
    /// MicroDVD (`"microdvd"`).
    MicroDvd,
    /// Anything else — carries the codec's short string verbatim.
    Other(SmolStr),
}

impl SubtitleCodec {
    /// Canonical short string (lowercase; matches the FFmpeg / container
    /// short name where applicable).
    pub fn as_str(&self) -> &str {
        match self {
            Self::Srt => "srt",
            Self::Ass => "ass",
            Self::Ssa => "ssa",
            Self::WebVtt => "webvtt",
            Self::MovText => "mov_text",
            Self::DvbSub => "dvbsub",
            Self::Pgs => "hdmv_pgs",
            Self::DvdSub => "dvd_subtitle",
            Self::Cea608 => "eia_608",
            Self::Cea708 => "eia_708",
            Self::Ttml => "ttml",
            Self::MicroDvd => "microdvd",
            Self::Other(s) => s.as_str(),
        }
    }

    /// True iff the codec is a **bitmap** (image-based) subtitle format —
    /// these require an OCR pipeline stage (per locked
    /// `subtitle_track.md` r3 / `subtitle_cues.md` r3).
    pub fn is_image_based(&self) -> bool {
        matches!(self, Self::DvbSub | Self::Pgs | Self::DvdSub)
    }
}

impl FromStr for SubtitleCodec {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "srt" | "subrip" => Self::Srt,
            "ass" => Self::Ass,
            "ssa" => Self::Ssa,
            "webvtt" | "vtt" => Self::WebVtt,
            "mov_text" | "tx3g" => Self::MovText,
            "dvbsub" | "dvb_subtitle" => Self::DvbSub,
            "hdmv_pgs" | "pgs" => Self::Pgs,
            "dvd_subtitle" | "dvdsub" => Self::DvdSub,
            "eia_608" | "cea608" => Self::Cea608,
            "eia_708" | "cea708" => Self::Cea708,
            "ttml" => Self::Ttml,
            "microdvd" => Self::MicroDvd,
            other => Self::Other(SmolStr::new(other)),
        })
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_codec_known_round_trip() {
        for s in [
            "h264",
            "hevc",
            "av1",
            "vp9",
            "vp8",
            "mpeg2video",
            "mpeg4",
            "prores",
            "dnxhd",
            "avc_intra",
            "jpeg2000",
            "mjpeg",
            "theora",
            "ffv1",
        ] {
            let c: VideoCodec = s.parse().unwrap();
            assert!(!c.is_other(), "{s} should parse to a named variant");
            assert_eq!(c.as_str(), s);
        }
    }

    #[test]
    fn video_codec_aliases() {
        // hevc / h265 collapse:
        let h: VideoCodec = "h265".parse().unwrap();
        assert_eq!(h, VideoCodec::Hevc);
        // mpeg2 short form:
        let m: VideoCodec = "mpeg2".parse().unwrap();
        assert_eq!(m, VideoCodec::Mpeg2);
        // dnxhd / dnxhr collapse:
        let d: VideoCodec = "dnxhr".parse().unwrap();
        assert_eq!(d, VideoCodec::DnxHd);
    }

    #[test]
    fn video_codec_other_preserves_string() {
        let weird: VideoCodec = "wmv3".parse().unwrap();
        assert!(weird.is_other());
        assert_eq!(weird.as_str(), "wmv3");
        // Round-trip: the unknown short string survives.
        let again: VideoCodec = weird.as_str().parse().unwrap();
        assert_eq!(weird, again);
    }

    #[test]
    fn audio_codec_known_round_trip() {
        for s in [
            "aac",
            "mp3",
            "flac",
            "opus",
            "vorbis",
            "ac3",
            "eac3",
            "dts",
            "truehd",
            "pcm_s16le",
            "pcm_s24le",
            "pcm_s32le",
            "pcm_f32le",
            "alac",
            "wma",
            "amr_nb",
            "amr_wb",
        ] {
            let c: AudioCodec = s.parse().unwrap();
            assert!(!c.is_other(), "{s} should parse to a named variant");
            assert_eq!(c.as_str(), s);
        }
    }

    #[test]
    fn audio_codec_wma_variants_collapse() {
        let a: AudioCodec = "wmav2".parse().unwrap();
        let b: AudioCodec = "wmapro".parse().unwrap();
        assert_eq!(a, AudioCodec::Wma);
        assert_eq!(b, AudioCodec::Wma);
    }

    #[test]
    fn audio_codec_other_preserves_string() {
        let c: AudioCodec = "musepack".parse().unwrap();
        assert!(c.is_other());
        assert_eq!(c.as_str(), "musepack");
    }

    #[test]
    fn subtitle_codec_known_round_trip() {
        for s in [
            "srt",
            "ass",
            "ssa",
            "webvtt",
            "mov_text",
            "dvbsub",
            "hdmv_pgs",
            "dvd_subtitle",
            "eia_608",
            "eia_708",
            "ttml",
            "microdvd",
        ] {
            let c: SubtitleCodec = s.parse().unwrap();
            assert!(!c.is_other(), "{s} should parse to a named variant");
            assert_eq!(c.as_str(), s);
        }
    }

    #[test]
    fn subtitle_codec_aliases() {
        let v: SubtitleCodec = "vtt".parse().unwrap();
        assert_eq!(v, SubtitleCodec::WebVtt);
        let p: SubtitleCodec = "pgs".parse().unwrap();
        assert_eq!(p, SubtitleCodec::Pgs);
        let d: SubtitleCodec = "dvdsub".parse().unwrap();
        assert_eq!(d, SubtitleCodec::DvdSub);
        let sr: SubtitleCodec = "subrip".parse().unwrap();
        assert_eq!(sr, SubtitleCodec::Srt);
    }

    #[test]
    fn subtitle_codec_is_image_based() {
        // Bitmap formats — OCR required.
        assert!(SubtitleCodec::Pgs.is_image_based());
        assert!(SubtitleCodec::DvbSub.is_image_based());
        assert!(SubtitleCodec::DvdSub.is_image_based());
        // Text formats — no OCR.
        assert!(!SubtitleCodec::Srt.is_image_based());
        assert!(!SubtitleCodec::Ass.is_image_based());
        assert!(!SubtitleCodec::WebVtt.is_image_based());
        assert!(!SubtitleCodec::MovText.is_image_based());
        assert!(!SubtitleCodec::Cea608.is_image_based());
        assert!(!SubtitleCodec::Cea708.is_image_based());
        // Unknown ones are treated as non-bitmap by default — bitmaps are
        // the named variants.
        let weird: SubtitleCodec = "something_new".parse().unwrap();
        assert!(!weird.is_image_based());
    }

    #[test]
    fn display_matches_as_str_across_kinds() {
        // The #[display] attr must agree with as_str().
        assert_eq!(VideoCodec::H264.to_string(), "h264");
        assert_eq!(AudioCodec::Opus.to_string(), "opus");
        assert_eq!(SubtitleCodec::Pgs.to_string(), "hdmv_pgs");
        assert_eq!(
            VideoCodec::Other(SmolStr::new("custom_codec")).to_string(),
            "custom_codec"
        );
    }
}
