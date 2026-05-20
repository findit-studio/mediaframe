#!/usr/bin/env python3
"""Generate mediaframe::codec from xtask/vendor/ffmpeg-codecs.txt.

Convention:
- Variant ident = each `[._]`-separated segment with the first char
  uppercased and the rest lowercased, concatenated; if the result
  starts with a digit, prefix `_`.
- Examples: `h264` → `H264`, `pcm_s16le` → `PcmS16le`,
  `dvb_subtitle` → `DvbSubtitle`, `acelp.kelvin` → `AcelpKelvin`,
  `4gv` → `_4Gv`, `8svx_exp` → `_8SvxExp`.

The output is one big file written to mediaframe/src/codec.rs.
"""
import re
import sys
import pathlib

VENDOR = pathlib.Path("xtask/vendor/ffmpeg-codecs.txt")
OUT = pathlib.Path("mediaframe/src/codec.rs")

BITMAP_SUBTITLES = {
    "dvb_subtitle",
    "dvd_subtitle",
    "hdmv_pgs_subtitle",
    "xsub",
}


def to_ident(name: str) -> str:
    parts = re.split(r"[._]+", name)
    out = "".join(p[:1].upper() + p[1:].lower() for p in parts if p)
    if out and out[0].isdigit():
        out = "_" + out
    return out


def load_codecs():
    by_type = {"video": [], "audio": [], "subtitle": []}
    for line in VENDOR.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        ty, name = parts[0], parts[1]
        if ty in by_type:
            by_type[ty].append(name)
    for v in by_type.values():
        v.sort()
    return by_type


def render_enum(enum_name: str, doc_kind: str, names: list[str], extra_impl: str = "") -> str:
    # Build variant table.
    variants = []
    seen = set()
    for n in names:
        ident = to_ident(n)
        if ident in seen:
            print(f"WARN: ident collision {ident!r} from {n!r}", file=sys.stderr)
        seen.add(ident)
        variants.append((ident, n))

    enum_body = "\n".join(
        f"    /// `\"{n}\"` (FFmpeg short name)." for ident, n in variants
        for _ in [0]
    )
    # Easier: build variant lines directly.
    variant_lines = []
    for ident, n in variants:
        variant_lines.append(f"    /// FFmpeg `\"{n}\"`.")
        variant_lines.append(f"    {ident},")
    variant_block = "\n".join(variant_lines)

    as_str_arms = "\n".join(
        f"            Self::{ident} => \"{n}\"," for ident, n in variants
    )
    from_str_arms = "\n".join(
        f"            \"{n}\" => Self::{ident}," for ident, n in variants
    )

    return f"""\
/// {doc_kind} codec family.
///
/// Generated from FFmpeg n8.1 `libavcodec/codec_desc.c` — every codec in
/// `xtask/vendor/ffmpeg-codecs.txt` of media type `{doc_kind.lower()}` is
/// a named variant here. The [`Self::Other`] arm is the lossless escape
/// for anything not enumerated (e.g. a codec added in a future FFmpeg
/// release before this enum is regenerated).
///
/// `#[non_exhaustive]` keeps future additions non-breaking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{{}}", self.as_str())]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum {enum_name} {{
{variant_block}
    /// A codec not enumerated above — carries the FFmpeg short name
    /// verbatim.
    Other(SmolStr),
}}

impl {enum_name} {{
    /// Canonical FFmpeg short name (`-codecs` column 2).
    pub fn as_str(&self) -> &str {{
        match self {{
{as_str_arms}
            Self::Other(s) => s.as_str(),
        }}
    }}
{extra_impl}}}

impl FromStr for {enum_name} {{
    type Err = core::convert::Infallible;

    /// Recognise an FFmpeg codec short name; unknown values land in
    /// [`Self::Other`] (infallible, lossless).
    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        Ok(match s {{
{from_str_arms}
            other => Self::Other(SmolStr::new(other)),
        }})
    }}
}}
"""


def render_subtitle_extra_impl(names: list[str]) -> str:
    # is_image_based: matches the FFmpeg AV_CODEC_PROP_BITMAP_SUB flag.
    # Hard-coded based on FFmpeg n8.1 codec_desc.c — see codec_desc-bitmap
    # check at xtask if the set ever needs widening.
    matches = [to_ident(n) for n in sorted(BITMAP_SUBTITLES) if n in names]
    pattern = " | ".join(f"Self::{i}" for i in matches)
    return f"""
    /// True iff this is a **bitmap** (image-based) subtitle codec,
    /// requiring an OCR pipeline stage to extract searchable text.
    /// Matches FFmpeg's `AV_CODEC_PROP_BITMAP_SUB` flag — checked
    /// against `xtask/vendor/ffmpeg-codecs.txt` via the bitmap set
    /// in the generator script.
    pub fn is_image_based(&self) -> bool {{
        matches!(self, {pattern})
    }}
"""


def main():
    by = load_codecs()

    header = """\
//! Stream-descriptor **codec** vocabulary for video, audio, and subtitle
//! tracks.
//!
//! **Generated** from `xtask/vendor/ffmpeg-codecs.txt` (FFmpeg n8.1
//! `libavcodec/codec_desc.c`). Every codec FFmpeg knows under media types
//! `video` / `audio` / `subtitle` has a named variant here; the
//! [`VideoCodec::Other`] / [`AudioCodec::Other`] / [`SubtitleCodec::Other`]
//! arms remain as a lossless escape for codecs added in a future FFmpeg
//! release before this file is regenerated.
//!
//! Regenerate via `cargo xtask sync` (refreshes the vendored table) plus
//! re-running the generator (`tools/gen_codec.py` if checked in, or the
//! upstream PR's commit message has the script inline). `cargo xtask check`
//! verifies every named variant exists in the vendored table.

use core::str::FromStr;

use derive_more::{Display, IsVariant};
use smol_str::SmolStr;

"""

    out = []
    out.append(header)
    out.append(render_enum("VideoCodec", "Video", by["video"]))
    out.append("")
    out.append(render_enum("AudioCodec", "Audio", by["audio"]))
    out.append("")
    out.append(render_enum("SubtitleCodec", "Subtitle", by["subtitle"],
                           extra_impl=render_subtitle_extra_impl(by["subtitle"])))

    # Tests.
    tests = """
// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The same vendored table xtask uses — single source of truth.
    const VENDOR: &str = include_str!("../../xtask/vendor/ffmpeg-codecs.txt");

    fn vendored_of(media: &'static str) -> impl Iterator<Item = &'static str> {
        VENDOR.lines().filter_map(move |l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let mut it = l.split_whitespace();
            match (it.next(), it.next()) {
                (Some(m), Some(n)) if m == media => Some(n),
                _ => None,
            }
        })
    }

    #[test]
    fn every_video_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("video") {
            let c: VideoCodec = name.parse().unwrap();
            assert!(
                !c.is_other(),
                "video `{name}` should parse to a named variant"
            );
            assert_eq!(c.as_str(), name, "round-trip mismatch for `{name}`");
            n += 1;
        }
        assert!(n > 0, "vendored video list is empty?");
    }

    #[test]
    fn every_audio_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("audio") {
            let c: AudioCodec = name.parse().unwrap();
            assert!(
                !c.is_other(),
                "audio `{name}` should parse to a named variant"
            );
            assert_eq!(c.as_str(), name);
            n += 1;
        }
        assert!(n > 0);
    }

    #[test]
    fn every_subtitle_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("subtitle") {
            let c: SubtitleCodec = name.parse().unwrap();
            assert!(
                !c.is_other(),
                "subtitle `{name}` should parse to a named variant"
            );
            assert_eq!(c.as_str(), name);
            n += 1;
        }
        assert!(n > 0);
    }

    #[test]
    fn unknown_codec_preserves_string_through_other() {
        let v: VideoCodec = "definitely_not_a_real_codec_xyz".parse().unwrap();
        assert!(v.is_other());
        assert_eq!(v.as_str(), "definitely_not_a_real_codec_xyz");
        let a: AudioCodec = "ditto_audio".parse().unwrap();
        assert!(a.is_other());
        assert_eq!(a.as_str(), "ditto_audio");
        let s: SubtitleCodec = "no_such_subtitle".parse().unwrap();
        assert!(s.is_other());
        assert_eq!(s.as_str(), "no_such_subtitle");
    }

    #[test]
    fn subtitle_image_based_set_matches_ffmpeg() {
        // AV_CODEC_PROP_BITMAP_SUB in FFmpeg n8.1 codec_desc.c.
        for n in ["dvb_subtitle", "hdmv_pgs_subtitle", "dvd_subtitle", "xsub"] {
            let c: SubtitleCodec = n.parse().unwrap();
            assert!(c.is_image_based(), "`{n}` should be image-based");
        }
        // Text formats must NOT be image-based.
        for n in ["subrip", "ass", "ssa", "webvtt", "mov_text", "ttml", "microdvd"] {
            let c: SubtitleCodec = n.parse().unwrap();
            assert!(!c.is_image_based(), "`{n}` should NOT be image-based");
        }
    }

    #[test]
    fn display_matches_as_str() {
        // Sample across kinds — `#[display]` must agree with `as_str`.
        assert_eq!(VideoCodec::H264.to_string(), "h264");
        assert_eq!(AudioCodec::Opus.to_string(), "opus");
        assert_eq!(SubtitleCodec::Webvtt.to_string(), "webvtt");
        assert_eq!(
            VideoCodec::Other(SmolStr::new("custom_codec")).to_string(),
            "custom_codec"
        );
    }
}
"""
    out.append(tests)

    OUT.write_text("\n".join(out))
    counts = {k: len(v) for k, v in by.items()}
    print(
        f"Wrote {OUT}: {counts['video']} video + {counts['audio']} audio + "
        f"{counts['subtitle']} subtitle = {sum(counts.values())} named variants."
    )


if __name__ == "__main__":
    main()
