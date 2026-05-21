// Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//
// Owned types:
//   - codec::{VideoCodec, AudioCodec, SubtitleCodec}
//   - container::Format
//   - subtitle::Format
//   - audio::ChannelLayout, audio::SampleFormat, audio::ContainerFormat
//     (the open string-enum ones — verify each before adding; if a type is
//     unit-only + coded, leave it for cluster B `arb_via_code!`).
//
// Use `super::arb_open_string_enum!` with a curated 6-slug sample list pulled
// from each type's own `as_str()` match.
