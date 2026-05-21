//! Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//!
//! One `pub(crate) fn name(g: &mut Gen) -> T` per type, referenced from each
//! type's container-level `#[quickcheck(with = "crate::quickcheck_helpers::strings::name")]`.
//!
//! Pattern: 50/50 picks a curated slug (round-tripped through `FromStr`,
//! `Infallible`) or builds `T::Other(SmolStr::from(<arbitrary String>))`.
//!
//! Owned types:
//!   - codec::{VideoCodec, AudioCodec, SubtitleCodec}
//!   - container::Format
//!   - subtitle::Format
//!   - audio::ChannelLayout, audio::SampleFormat, audio::ContainerFormat
