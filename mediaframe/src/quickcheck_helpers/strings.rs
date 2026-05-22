//! Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//!
//! One `pub(crate) fn name(g: &mut Gen) -> T` per type, referenced from each
//! type's container-level `#[quickcheck(arbitrary = "crate::quickcheck_helpers::strings::name")]`.
//!
//! Pattern: 50/50 picks a curated slug (round-tripped through `FromStr`,
//! `Infallible`) or builds `T::Other(SmolStr::from(<arbitrary String>))`.
//!
//! Owned types:
//!   - codec::{VideoCodec, AudioCodec, SubtitleCodec}
//!   - container::Format
//!   - subtitle::Format
//!   - audio::ChannelLayout, audio::SampleFormat, audio::ContainerFormat

/// Internal: emit one `pub(crate) fn $name(g) -> $ty` whose body is the
/// 50/50 curated-slug-vs-`Other` branch shared by every cluster-A helper.
///
/// `FromStr` is `Infallible` for every covered type, so round-tripping a
/// known slug is sound; `g.choose(&non_empty_slice)` returns `Some`, so the
/// `unwrap` is sound.
macro_rules! qc_open_string_enum {
  ($name:ident, $ty:ty, [$($slug:literal),+ $(,)?]) => {
    pub(crate) fn $name(g: &mut ::quickcheck::Gen) -> $ty {
      const SAMPLES: &[&str] = &[$($slug),+];
      if super::coin(g) {
        <$ty as ::core::str::FromStr>::from_str(g.choose(SAMPLES).unwrap()).unwrap()
      } else {
        <$ty>::Other(::smol_str::SmolStr::from(super::arb_string(g)))
      }
    }
  };
}

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
  container_format,
  crate::container::Format,
  ["mp4", "mkv", "webm", "mov", "avi", "mpegts"]
);

qc_open_string_enum!(
  subtitle_format,
  crate::subtitle::Format,
  ["srt", "webvtt", "ass", "ssa", "mov_text", "ttml"]
);

qc_open_string_enum!(
  channel_layout,
  crate::audio::ChannelLayout,
  ["mono", "stereo", "5.1", "7.1", "quad", "5.0"]
);

/// `SampleFormat` has 3 arms (`Unknown(u32)` / named / `Other(SmolStr)`)
/// rather than 2, so it can't share the curated-slug-vs-`Other` macro path
/// (which would only ever produce named or `Other`, never `Unknown`). 3-way
/// pick over a sentinel slice (`Gen` has no `int_in_range`).
pub(crate) fn sample_format(g: &mut ::quickcheck::Gen) -> crate::audio::SampleFormat {
  use ::quickcheck::Arbitrary;
  // All 12 named slugs — a 6-slug subset (Codex round-2 finding) left
  // the planar / double / 64-bit variants reachable only by the rare
  // numeric arm drawing their exact `0..=11` code.
  const SLUGS: &[&str] = &[
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
  ];
  match *g.choose(&[0u8, 1, 2]).expect("non-empty arm-tag slice") {
    0 => <crate::audio::SampleFormat as ::core::str::FromStr>::from_str(
      g.choose(SLUGS).expect("non-empty SLUGS"),
    )
    .unwrap(),
    1 => crate::audio::SampleFormat::from_u32(u32::arbitrary(g)),
    _ => crate::audio::SampleFormat::Other(::smol_str::SmolStr::from(super::arb_string(g))),
  }
}

qc_open_string_enum!(
  audio_container_format,
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus"]
);
