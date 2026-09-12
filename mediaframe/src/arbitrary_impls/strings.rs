// Cluster A — open string enums w/ `Other(Utf8Bytes)` and total `FromStr`.
//
// Every type covered here has:
//   - an `Other(Utf8Bytes)` lossless-escape arm, and
//   - an `impl FromStr` whose `Err = core::convert::Infallible`,
// so the shared `arb_open_string_enum!` macro applies directly. The 50/50
// branch in the macro flips between a curated slug (round-tripped through
// `FromStr` to exercise the named arms) and `Other(Utf8Bytes::from(<arbitrary
// String>))` (exercises the lossless escape — including empty strings,
// pre-known slugs, and arbitrary bytes — for fuzz coverage).
//
// Slug picks: ~6 canonical FFmpeg / file-extension slugs per type, drawn
// from each type's own `as_str()` match. Common-case picks (not edge
// cases) — the goal is "this is a real value a real file would carry",
// since the `Other` branch already covers everything else.

super::arb_open_string_enum!(
  crate::codec::VideoCodec,
  ["h264", "hevc", "av1", "vp9", "mpeg4", "prores"]
);

super::arb_open_string_enum!(
  crate::codec::AudioCodec,
  ["aac", "mp3", "opus", "flac", "ac3", "alac"]
);

super::arb_open_string_enum!(
  crate::codec::SubtitleCodec,
  ["srt", "ass", "ssa", "webvtt", "mov_text", "dvb_subtitle"]
);

super::arb_open_string_enum!(
  crate::codec::DataCodec,
  ["klv", "timed_id3", "scte_35", "bin_data", "ttf", "otf"]
);

// The full roster seeded on purpose — `AttachmentCodec` only has three
// named variants (see `ATTACHMENT_CODECS` in `xtask/src/main.rs`), so
// there is no "representative sample" distinct from "all of them".
super::arb_open_string_enum!(crate::codec::AttachmentCodec, ["ttf", "otf", "bin_data"]);

// `m2ts`/`3g2` are seeded on purpose, not just `mpegts`/… — R5 promoted
// `M2ts`/`Threeg2` to their own variants (structurally distinct from
// `MpegTs`/`Threegp`, not aliases), and without their own canonical
// slugs here they'd be reachable only through the unconstrained-string
// branch's negligible odds. See `reachability_r5_r6_promoted_variants_are_generated`
// in `arbitrary_impls::tests`.
super::arb_open_string_enum!(
  crate::container::Format,
  ["mp4", "mkv", "webm", "mov", "avi", "mpegts", "m2ts", "3g2"]
);

// `heic` is seeded on purpose, not just `heif` — R6 promoted `Heic` to
// its own variant (HEVC-brand, distinct from `Heif`'s generic `mif1`
// brand), same reachability concern as `m2ts`/`3g2`/`aifc` above.
super::arb_open_string_enum!(
  crate::image::Format,
  [
    "jpeg", "png", "heif", "heic", "avif", "tiff", "dng", "cr2", "nef", "arw"
  ]
);

super::arb_open_string_enum!(
  crate::subtitle::Format,
  ["srt", "webvtt", "ass", "ssa", "mov_text", "ttml"]
);

super::arb_open_string_enum!(
  crate::subtitle::TrackOrigin,
  ["embedded", "sidecar", "external", "derived"]
);

// Both spellings of the crossed pairs are seeded on purpose — see the
// twin list in `quickcheck_helpers::strings`. The unqualified `"5.0"` /
// `"5.1"` / `"7.1(wide)"` are FFmpeg's names for the **back** layouts,
// so the short spelling alone leaves `Ch5_0` / `Ch5_1` /
// `Ch7_1Wide` unreachable.
super::arb_open_string_enum!(
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

// `aifc` is seeded on purpose, not just the others — R5 promoted `Aifc`
// to its own variant (distinct `FORM` type from `Aiff`), same
// reachability concern as `m2ts`/`3g2`/`heic` above.
super::arb_open_string_enum!(
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus", "aifc"]
);

super::arb_open_string_enum!(
  crate::audio::SampleFormat,
  [
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p"
  ]
);
