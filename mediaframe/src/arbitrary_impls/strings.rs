// Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//
// Every type covered here has:
//   - an `Other(SmolStr)` lossless-escape arm, and
//   - an `impl FromStr` whose `Err = core::convert::Infallible`,
// so the shared `arb_open_string_enum!` macro applies directly. The 50/50
// branch in the macro flips between a curated slug (round-tripped through
// `FromStr` to exercise the named arms) and `Other(SmolStr::from(<arbitrary
// String>))` (exercises the lossless escape — including empty strings,
// pre-known slugs, and arbitrary bytes — for fuzz coverage).
//
// Slug picks: ~6 canonical FFmpeg / file-extension slugs per type, drawn
// from each type's own `as_str()` match. Common-case picks (not edge
// cases) — the goal is "this is a real value a real file would carry",
// since the `Other` branch already covers everything else.
//
// `audio::SampleFormat` carries BOTH `Unknown(u32)` and `Other(SmolStr)`,
// so the open-string-enum macro (which only exercises slugs + `Other`)
// leaves `Unknown(_)` unreachable. It gets a bespoke 3-way generator
// further down (Codex round-1 finding).

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
  crate::container::Format,
  ["mp4", "mkv", "webm", "mov", "avi", "mpegts"]
);

super::arb_open_string_enum!(
  crate::subtitle::Format,
  ["srt", "webvtt", "ass", "ssa", "mov_text", "ttml"]
);

super::arb_open_string_enum!(
  crate::audio::ChannelLayout,
  ["mono", "stereo", "5.1", "7.1", "quad", "5.0"]
);

super::arb_open_string_enum!(
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus"]
);

// Bespoke 3-way for `SampleFormat`: it has BOTH `Unknown(u32)` AND
// `Other(SmolStr)`. The shared open-string-enum macro only exercises
// curated slugs + `Other`; the `Unknown(_)` numeric-escape arm is
// otherwise unreachable. Dispatch evenly across (named slug / `Unknown(u32)`
// via `from_u32` / `Other` via an arbitrary string).
//
// EVERY branch produces a CANONICAL value (Codex round-4 finding): all
// string construction goes through `FromStr`, never `Other(_)` directly.
// `from_str` maps a named slug to the named variant and only a *non-named*
// slug to `Other` — so we can never emit a malformed `Other("s16")` that
// serde would canonicalise to `S16` on the round trip. `from_u32` is
// likewise canonical (a named code → the named variant). An arbitrary
// string is virtually never one of the 12 named slugs, so the `Other` arm
// stays well-covered.
impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::SampleFormat {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    use ::core::str::FromStr;
    // All 12 named slugs — a 6-slug subset (Codex round-2 finding) left
    // the planar / double / 64-bit variants reachable only by the rare
    // numeric branch drawing their exact `0..=11` code.
    const SLUGS: &[&str] = &[
      "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
    ];
    match u.int_in_range(0..=2u8)? {
      // Named — curated slug through `FromStr` (`Infallible`).
      0 => Ok(crate::audio::SampleFormat::from_str(u.choose(SLUGS)?).unwrap()),
      // `Unknown(u32)` (or a named variant if the code is canonical).
      1 => Ok(crate::audio::SampleFormat::from_u32(
        <u32 as ::arbitrary::Arbitrary>::arbitrary(u)?,
      )),
      // `Other` — arbitrary string through `FromStr`, so a string that
      // happens to equal a named slug canonicalises instead of becoming
      // a non-round-trippable `Other(named_slug)`.
      _ => {
        let s = <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?;
        Ok(crate::audio::SampleFormat::from_str(&s).unwrap())
      }
    }
  }
}
