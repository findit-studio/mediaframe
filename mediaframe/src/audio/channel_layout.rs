//! Audio channel layout vocabulary — the common named layouts plus
//! an `Other(SmolStr)` lossless escape for anything outside the
//! closed set.
//!
//! The named variants cover the `AV_CH_LAYOUT_*` shapes FFmpeg n8.1
//! exposes; layouts not enumerated here (custom orderings, ambisonic
//! variants beyond first-order, etc.) round-trip through
//! [`ChannelLayout::Other`] carrying the FFmpeg-canonical slug
//! verbatim.

use core::str::FromStr;

use derive_more::{Display, IsVariant};
use smol_str::SmolStr;

/// Audio channel layout — the common named layouts plus an
/// `Other(SmolStr)` lossless escape.
///
/// Read from FFmpeg `AV_CH_LAYOUT_*` constants (`AVChannelLayout`'s
/// canonical name) / WebCodecs `AudioData.channelLayout`. Named
/// variants are the universally-understood shapes; layouts FFmpeg
/// can describe but this enum doesn't enumerate (e.g.
/// `"hexadecagonal"`, `"22.2"`, ambisonic groupings beyond
/// `Ambisonic1`/`2`/`3`) round-trip through [`Self::Other`] carrying
/// the FFmpeg-canonical slug verbatim — never silently collapsed.
///
/// `#[non_exhaustive]` keeps future additions non-breaking. `Display`
/// renders via [`Self::as_str`].
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::channel_layout")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum ChannelLayout {
  /// Single channel: `"mono"` (FFmpeg `AV_CH_LAYOUT_MONO`).
  Mono,
  /// L+R: `"stereo"` (FFmpeg `AV_CH_LAYOUT_STEREO`).
  Stereo,
  /// L+R+LFE: `"2.1"` (FFmpeg `AV_CH_LAYOUT_2POINT1`).
  N2Point1,
  /// L+R+C: `"3.0"` (FFmpeg `AV_CH_LAYOUT_SURROUND`).
  N3Point0,
  /// L+R+BC (back-center surround): `"3.0(back)"` (FFmpeg
  /// `AV_CH_LAYOUT_2_1`).
  N3Point0Back,
  /// L+R+C+LFE: `"3.1"` (FFmpeg `AV_CH_LAYOUT_3POINT1`).
  N3Point1,
  /// L+R+SL+SR or L+R+BL+BR: `"quad"` (FFmpeg `AV_CH_LAYOUT_QUAD`).
  Quad,
  /// L+R+C+SL+SR: `"5.0"` (FFmpeg `AV_CH_LAYOUT_5POINT0`).
  N5Point0,
  /// L+R+C+BL+BR variant (back instead of side): `"5.0(side)"` —
  /// here named for the FFmpeg-side `AV_CH_LAYOUT_5POINT0_BACK`.
  N5Point0Back,
  /// L+R+C+LFE+SL+SR: `"5.1"` (FFmpeg `AV_CH_LAYOUT_5POINT1`).
  N5Point1,
  /// L+R+C+LFE+BL+BR variant: `"5.1(side)"` — corresponds to FFmpeg
  /// `AV_CH_LAYOUT_5POINT1_BACK` (which FFmpeg labels the
  /// historically-backwards-named layout).
  N5Point1Back,
  /// 6.0: `"6.0"` (FFmpeg `AV_CH_LAYOUT_6POINT0`).
  N6Point0,
  /// 6.1: `"6.1"` (FFmpeg `AV_CH_LAYOUT_6POINT1`).
  N6Point1,
  /// 7.0: `"7.0"` (FFmpeg `AV_CH_LAYOUT_7POINT0`).
  N7Point0,
  /// 7.1: `"7.1"` (FFmpeg `AV_CH_LAYOUT_7POINT1`).
  N7Point1,
  /// Hexagonal (6 channels in a hexagon, no LFE): `"hexagonal"`
  /// (FFmpeg `AV_CH_LAYOUT_HEXAGONAL`).
  Hexagonal,
  /// Octagonal (8 channels around): `"octagonal"` (FFmpeg
  /// `AV_CH_LAYOUT_OCTAGONAL`).
  Octagonal,
  /// First-order Ambisonic B-format (WXYZ, 4 channels): `"ambisonic1"`.
  Ambisonic1,
  /// Second-order Ambisonic (9 channels): `"ambisonic2"`.
  Ambisonic2,
  /// Third-order Ambisonic (16 channels): `"ambisonic3"`.
  Ambisonic3,
  /// A layout not enumerated above — carries the FFmpeg-canonical
  /// name verbatim (e.g. `"22.2"`, `"hexadecagonal"`, a custom
  /// layout description). Lossless escape.
  Other(SmolStr),
}

impl Default for ChannelLayout {
  /// `Other("")` — the wire-zero / "absent" sentinel. There is no
  /// universally-defensible default channel layout (mono vs stereo
  /// is context-dependent); the empty-string `Other` mirrors the
  /// `buffa`-compatible "absent" state. Callers picking a meaningful
  /// fallback should be explicit (`ChannelLayout::Stereo` is the
  /// common one).
  #[inline]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl ChannelLayout {
  /// FFmpeg-canonical layout slug (e.g. `"mono"`, `"stereo"`,
  /// `"5.1"`, `"7.1"`). [`Self::Other`] returns the wrapped string
  /// verbatim.
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mono => "mono",
      Self::Stereo => "stereo",
      Self::N2Point1 => "2.1",
      Self::N3Point0 => "3.0",
      Self::N3Point0Back => "3.0(back)",
      Self::N3Point1 => "3.1",
      Self::Quad => "quad",
      Self::N5Point0 => "5.0",
      Self::N5Point0Back => "5.0(side)",
      Self::N5Point1 => "5.1",
      Self::N5Point1Back => "5.1(side)",
      Self::N6Point0 => "6.0",
      Self::N6Point1 => "6.1",
      Self::N7Point0 => "7.0",
      Self::N7Point1 => "7.1",
      Self::Hexagonal => "hexagonal",
      Self::Octagonal => "octagonal",
      Self::Ambisonic1 => "ambisonic1",
      Self::Ambisonic2 => "ambisonic2",
      Self::Ambisonic3 => "ambisonic3",
      Self::Other(s) => s.as_str(),
    }
  }
}

impl FromStr for ChannelLayout {
  type Err = core::convert::Infallible;
  /// Recognise a canonical layout slug; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "mono" => Self::Mono,
      "stereo" => Self::Stereo,
      "2.1" => Self::N2Point1,
      "3.0" => Self::N3Point0,
      "3.0(back)" => Self::N3Point0Back,
      "3.1" => Self::N3Point1,
      "quad" => Self::Quad,
      "5.0" => Self::N5Point0,
      "5.0(side)" => Self::N5Point0Back,
      "5.1" => Self::N5Point1,
      "5.1(side)" => Self::N5Point1Back,
      "6.0" => Self::N6Point0,
      "6.1" => Self::N6Point1,
      "7.0" => Self::N7Point0,
      "7.1" => Self::N7Point1,
      "hexagonal" => Self::Hexagonal,
      "octagonal" => Self::Octagonal,
      "ambisonic1" => Self::Ambisonic1,
      "ambisonic2" => Self::Ambisonic2,
      "ambisonic3" => Self::Ambisonic3,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::string::ToString;

  #[test]
  fn every_named_variant_round_trips() {
    for slug in [
      "mono",
      "stereo",
      "2.1",
      "3.0",
      "3.0(back)",
      "3.1",
      "quad",
      "5.0",
      "5.0(side)",
      "5.1",
      "5.1(side)",
      "6.0",
      "6.1",
      "7.0",
      "7.1",
      "hexagonal",
      "octagonal",
      "ambisonic1",
      "ambisonic2",
      "ambisonic3",
    ] {
      let v: ChannelLayout = slug.parse().unwrap();
      assert!(!v.is_other(), "`{slug}` should be a named variant");
      assert_eq!(v.as_str(), slug, "round-trip mismatch for `{slug}`");
    }
  }

  #[test]
  fn unknown_layout_lands_in_other() {
    let v: ChannelLayout = "22.2".parse().unwrap();
    assert!(v.is_other());
    assert_eq!(v.as_str(), "22.2");
    assert_eq!(v.to_string(), "22.2");
  }

  #[test]
  fn display_matches_as_str() {
    assert_eq!(ChannelLayout::Stereo.to_string(), "stereo");
    assert_eq!(ChannelLayout::N5Point1.to_string(), "5.1");
    assert_eq!(
      ChannelLayout::Other(SmolStr::new("custom_layout")).to_string(),
      "custom_layout"
    );
  }

  #[test]
  fn is_variant_predicates() {
    assert!(ChannelLayout::Mono.is_mono());
    assert!(ChannelLayout::Stereo.is_stereo());
    assert!(ChannelLayout::N5Point1.is_n_5_point_1());
    assert!(ChannelLayout::Other(SmolStr::new("x")).is_other());
  }
}
