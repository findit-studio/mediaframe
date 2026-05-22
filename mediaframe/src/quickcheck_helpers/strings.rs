//! Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//!
//! One `pub(crate) fn name(g: &mut Gen) -> T` per type, referenced from each
//! type's container-level `#[quickcheck(arbitrary = "crate::quickcheck_helpers::strings::name")]`.
//!
//! Pattern: 50/50 picks a curated slug or an arbitrary string — **both
//! routed through `FromStr`**, so every generated value is canonical.
//!
//! Owned types:
//!   - codec::{VideoCodec, AudioCodec, SubtitleCodec}
//!   - container::Format
//!   - subtitle::Format
//!   - audio::ChannelLayout, audio::SampleFormat, audio::ContainerFormat

/// Internal: emit one `pub(crate) fn $name(g) -> $ty` — 50/50 a curated slug
/// or an arbitrary string, both through `FromStr`.
///
/// `FromStr` is the canonicalising constructor (`Infallible` for every
/// covered type): a named slug yields the named variant, only a non-named
/// slug yields `Other`. Routing the arbitrary-string branch through it too
/// (rather than `Other(SmolStr::from(s))` directly) guarantees a string
/// equal to a named slug becomes that named variant — never a malformed
/// `Other("h264")` that serde would canonicalise to `H264` on the round
/// trip (Codex round-4 finding). An arbitrary string is virtually never a
/// named slug, so the `Other` arm stays well-covered.
macro_rules! qc_open_string_enum {
  ($name:ident, $ty:ty, [$($slug:literal),+ $(,)?]) => {
    pub(crate) fn $name(g: &mut ::quickcheck::Gen) -> $ty {
      const SAMPLES: &[&str] = &[$($slug),+];
      let s = if super::coin(g) {
        ::std::string::String::from(*g.choose(SAMPLES).unwrap())
      } else {
        super::arb_string(g)
      };
      <$ty as ::core::str::FromStr>::from_str(&s).unwrap()
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
///
/// Every branch produces a CANONICAL value (Codex round-4 finding): string
/// construction goes through `FromStr`, never `Other(_)` directly — a string
/// equal to a named slug becomes that named variant, so we never emit a
/// malformed `Other("s16")` that serde would canonicalise to `S16`.
/// `from_u32` is likewise canonical. An arbitrary string is virtually never
/// one of the 12 named slugs, so the `Other` arm stays well-covered.
pub(crate) fn sample_format(g: &mut ::quickcheck::Gen) -> crate::audio::SampleFormat {
  use ::core::str::FromStr;
  use ::quickcheck::Arbitrary;
  // All 12 named slugs — a 6-slug subset (Codex round-2 finding) left
  // the planar / double / 64-bit variants reachable only by the rare
  // numeric arm drawing their exact `0..=11` code.
  const SLUGS: &[&str] = &[
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
  ];
  match *g.choose(&[0u8, 1, 2]).expect("non-empty arm-tag slice") {
    // Named — curated slug through `FromStr`.
    0 => crate::audio::SampleFormat::from_str(g.choose(SLUGS).expect("non-empty SLUGS")).unwrap(),
    // `Unknown(u32)` (or a named variant for a canonical code).
    1 => crate::audio::SampleFormat::from_u32(u32::arbitrary(g)),
    // `Other` — arbitrary string through `FromStr`, so a string equal to a
    // named slug canonicalises instead of becoming a non-round-trippable
    // `Other(named_slug)`.
    _ => crate::audio::SampleFormat::from_str(&super::arb_string(g)).unwrap(),
  }
}

qc_open_string_enum!(
  audio_container_format,
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus"]
);
