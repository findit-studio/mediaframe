//! Cluster A — open string enums w/ `Other(Utf8Bytes)` and total `FromStr`.
//!
//! One `pub(crate) fn name(g: &mut Gen) -> T` per type, referenced from each
//! type's container-level `#[quickcheck(arbitrary = "crate::quickcheck_helpers::strings::name")]`.
//!
//! Pattern: 50/50 picks a curated slug or an arbitrary string — **both
//! routed through `FromStr`**, so every generated value is canonical.
//!
//! Owned types:
//!   - codec::{VideoCodec, AudioCodec, SubtitleCodec, DataCodec, AttachmentCodec}
//!   - container::Format
//!   - image::Format
//!   - subtitle::Format
//!   - audio::ChannelLayout, audio::SampleFormat, audio::ContainerFormat

qc_open_string_enum!(
  video_codec,
  crate::codec::VideoCodec,
  ["h264", "hevc", "av1", "vp9", "mpeg4", "prores"]
);

qc_open_string_enum!(
  audio_codec,
  crate::codec::AudioCodec,
  ["aac", "mp3", "opus", "flac", "ac3", "alac"]
);

qc_open_string_enum!(
  subtitle_codec,
  crate::codec::SubtitleCodec,
  ["srt", "ass", "ssa", "webvtt", "mov_text", "dvb_subtitle"]
);

qc_open_string_enum!(
  data_codec,
  crate::codec::DataCodec,
  ["klv", "timed_id3", "scte_35", "bin_data", "ttf", "otf"]
);

// The full roster seeded on purpose — `AttachmentCodec` only has three
// named variants (see `ATTACHMENT_CODECS` in `xtask/src/main.rs`), so
// there is no "representative sample" distinct from "all of them".
qc_open_string_enum!(
  attachment_codec,
  crate::codec::AttachmentCodec,
  ["ttf", "otf", "bin_data"]
);

// `m2ts`/`3g2` seeded on purpose — R5 promoted `M2ts`/`Threeg2` to their
// own variants; see the matching note in `arbitrary_impls::strings`.
qc_open_string_enum!(
  container_format,
  crate::container::Format,
  ["mp4", "mkv", "webm", "mov", "avi", "mpegts", "m2ts", "3g2"]
);

// `heic` seeded on purpose — R6 promoted `Heic` to its own variant.
qc_open_string_enum!(
  image_format,
  crate::image::Format,
  [
    "jpeg", "png", "heif", "heic", "avif", "tiff", "dng", "cr2", "nef", "arw"
  ]
);

qc_open_string_enum!(
  subtitle_format,
  crate::subtitle::Format,
  ["srt", "webvtt", "ass", "ssa", "mov_text", "ttml"]
);

qc_open_string_enum!(
  track_origin,
  crate::subtitle::TrackOrigin,
  ["embedded", "sidecar", "external", "derived"]
);

// Both spellings of the crossed pairs are seeded on purpose. FFmpeg
// gives the unqualified `"5.0"` / `"5.1"` / `"7.1(wide)"` to the
// **back**-speaker layouts and qualifies the side ones, so seeding only
// the short spelling reaches `Ch5_0Back` / `Ch5_1Back` /
// `Ch7_1WideBack` and leaves their side twins unreachable — exactly
// the half whose slug moved in 0.4.0.
qc_open_string_enum!(
  channel_layout,
  crate::audio::ChannelLayout,
  [
    "mono",
    "stereo",
    "5.1",
    "5.1(side)",
    "7.1",
    "quad",
    "5.0",
    "5.0(side)",
    "7.1(wide)",
    "7.1(wide-side)"
  ]
);

qc_open_string_enum!(
  sample_format,
  crate::audio::SampleFormat,
  [
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p"
  ]
);

// `aifc` seeded on purpose — R5 promoted `Aifc` to its own variant.
qc_open_string_enum!(
  audio_container_format,
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus", "aifc"]
);
