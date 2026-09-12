//! Audio channel layout vocabulary — the common named layouts plus
//! an `Other(Utf8Bytes)` lossless escape for anything outside the
//! closed set.
//!
//! The named variants cover **every** entry in FFmpeg n9.0's
//! `channel_layout_map[]` — all forty — plus the three ambisonic
//! groupings FFmpeg models as a channel *order* rather than a map entry.
//! What still rides [`ChannelLayout::Other`] is what the map does not
//! name at all: custom channel orderings, ambisonic groupings beyond
//! third order, and whatever a later FFmpeg adds. The escape carries the
//! FFmpeg-canonical slug verbatim.
//!
//! Channel abbreviations in the variant docs are FFmpeg's own
//! (`ffmpeg -layouts`): `FL`/`FR` front left/right, `FC` front centre,
//! `LFE` low frequency, `BL`/`BR` back left/right, `BC` back centre,
//! `SL`/`SR` side left/right, `FLC`/`FRC` front left/right-of-centre,
//! `WL`/`WR` wide left/right, `TC` top centre, `TFL`/`TFC`/`TFR` top
//! front left/centre/right, `TBL`/`TBC`/`TBR` top back
//! left/centre/right, `TSL`/`TSR` top side left/right,
//! `BFL`/`BFC`/`BFR` bottom front left/centre/right, `LFE2` second low
//! frequency, `DL`/`DR` downmix left/right.
//!
//! **Two namings, and they do not line up.** A variant's *ident* follows
//! the FFmpeg constant (`Ch5_1Back` ⇒ `AV_CH_LAYOUT_5POINT1_BACK`);
//! its *slug* is the name FFmpeg's `channel_layout_map[]` gives that
//! constant — what `av_channel_layout_describe` prints, and therefore
//! what an FFmpeg-sourced string actually says. The two disagree three
//! ways, each of which has already cost this crate or its neighbours a
//! bug:
//!
//! 1. **The unqualified name went to whichever layout had it first.**
//!    FFmpeg hands plain `"5.0"`, `"5.1"` and `"7.1(wide)"` to the
//!    **back**-speaker layouts and qualifies the side ones `"5.0(side)"`,
//!    `"5.1(side)"`, `"7.1(wide-side)"`, so six arms read crossed
//!    against their idents. For the 5.1.2 family it runs the other way:
//!    `"5.1.2"` is the side layout and the back one is qualified
//!    `"5.1.2(back)"`. There is no rule to derive here — only the map.
//! 2. **`_BACK` in a constant is not `(back)` in a slug.**
//!    `5POINT1POINT4_BACK`, `7POINT1POINT4_BACK` and
//!    `9POINT1POINT4_BACK` are named `"5.1.4"`, `"7.1.4"` and `"9.1.4"`:
//!    there the suffix distinguishes top-*back* height channels, not
//!    surround placement, and FFmpeg never minted an unsuffixed sibling
//!    to qualify against.
//! 3. **Two idents follow the slug instead of the constant.**
//!    `AV_CH_LAYOUT_2_1` and `AV_CH_LAYOUT_2_2` are historical spellings
//!    that describe no arrangement anyone would recognise — and `2_1`
//!    read as an ident collides with the real `2POINT1`. They are
//!    [`ChannelLayout::Ch3_0Back`] and [`ChannelLayout::QuadSide`],
//!    after `"3.0(back)"` and `"quad(side)"`.
//!
//! `channel_layout_slugs_match_ffmpegs_map` transcribes the whole map
//! and pins every arm, because nothing else catches a plausible-looking
//! "correction".

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;

/// Audio channel layout — the common named layouts plus an
/// `Other(Utf8Bytes)` lossless escape.
///
/// Read from FFmpeg `AV_CH_LAYOUT_*` constants (`AVChannelLayout`'s
/// canonical name) / WebCodecs `AudioData.channelLayout`. Layouts FFmpeg
/// can describe but this enum doesn't enumerate (a custom channel
/// ordering, an ambisonic grouping beyond `Ambisonic1`/`2`/`3`)
/// round-trip through [`Self::Other`] carrying the FFmpeg-canonical slug
/// verbatim — never silently collapsed.
///
/// `#[non_exhaustive]` keeps future additions non-breaking. `Display`
/// renders via [`Self::as_str`].
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::channel_layout")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum ChannelLayout {
  /// `"mono"` — FC (FFmpeg `AV_CH_LAYOUT_MONO`).
  Mono,
  /// `"stereo"` — FL+FR (FFmpeg `AV_CH_LAYOUT_STEREO`).
  Stereo,
  /// `"downmix"` — DL+DR (FFmpeg `AV_CH_LAYOUT_STEREO_DOWNMIX`).
  ///
  /// The Dolby matrix-encoded stereo pair, carried as its own channel
  /// ids so a downmix is distinguishable from an original `Stereo`
  /// recording. FFmpeg names the constant after the source and the map
  /// entry after the result, hence the crossed spelling.
  StereoDownmix,
  /// `"binaural"` — BIL+BIR (FFmpeg `AV_CH_LAYOUT_BINAURAL`).
  ///
  /// **Not a stereo pair.** Binaural audio is rendered for headphones
  /// with the head-related transfer function already baked in, so each
  /// channel is what one *ear* receives rather than what one *speaker*
  /// emits. Playing it over loudspeakers, or downmixing it to
  /// [`Self::Mono`], destroys the spatial cue it exists to carry —
  /// which is why FFmpeg gives it channel ids of its own instead of
  /// reusing FL/FR.
  Binaural,
  /// `"2.1"` — FL+FR+LFE (FFmpeg `AV_CH_LAYOUT_2POINT1`).
  Ch2_1,
  /// `"3.0"` — FL+FR+FC (FFmpeg `AV_CH_LAYOUT_SURROUND`).
  Ch3_0,
  /// `"3.0(back)"` — FL+FR+BC (FFmpeg `AV_CH_LAYOUT_2_1`).
  ///
  /// Back-centre surround rather than a front centre. The ident follows
  /// the slug here: the constant's `2_1` is a historical spelling, and
  /// as an ident it would collide with the real 2.1 above.
  Ch3_0Back,
  /// `"3.1"` — FL+FR+FC+LFE (FFmpeg `AV_CH_LAYOUT_3POINT1`).
  Ch3_1,
  /// `"3.1.2"` — FL+FR+FC+LFE+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_3POINT1POINT2`).
  Ch3_1_2,
  /// `"4.0"` — FL+FR+FC+BC (FFmpeg `AV_CH_LAYOUT_4POINT0`).
  Ch4_0,
  /// `"4.1"` — FL+FR+FC+LFE+BC (FFmpeg `AV_CH_LAYOUT_4POINT1`).
  Ch4_1,
  /// `"quad"` — FL+FR+BL+BR (FFmpeg `AV_CH_LAYOUT_QUAD`).
  ///
  /// The **side** four-channel layout is [`Self::QuadSide`].
  Quad,
  /// `"quad(side)"` — FL+FR+SL+SR (FFmpeg `AV_CH_LAYOUT_2_2`).
  ///
  /// The ident follows the slug for the same reason
  /// [`Self::Ch3_0Back`]'s does: the constant's `2_2` names no
  /// arrangement a reader would recognise, and reads as a "2.2" that
  /// does not exist.
  QuadSide,
  /// `"5.0(side)"` — FL+FR+FC+SL+SR (FFmpeg `AV_CH_LAYOUT_5POINT0`).
  ///
  /// FFmpeg's plain `"5.0"` is the **back** layout, not this one — see
  /// [`Self::Ch5_0Back`].
  Ch5_0,
  /// `"5.0"` — FL+FR+FC+BL+BR (FFmpeg `AV_CH_LAYOUT_5POINT0_BACK`).
  ///
  /// The unqualified `"5.0"` is FFmpeg's name for the back-speaker
  /// layout; the side-speaker one is [`Self::Ch5_0`], spelled
  /// `"5.0(side)"`.
  Ch5_0Back,
  /// `"5.1(side)"` — FL+FR+FC+LFE+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1`).
  ///
  /// FFmpeg's plain `"5.1"` is the **back** layout — see
  /// [`Self::Ch5_1Back`].
  Ch5_1,
  /// `"5.1"` — FL+FR+FC+LFE+BL+BR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1_BACK`).
  ///
  /// This is what an FFmpeg-sourced `"5.1"` means — the historically
  /// unqualified spelling belongs to the back-speaker layout, and the
  /// side-speaker one is [`Self::Ch5_1`], spelled `"5.1(side)"`.
  Ch5_1Back,
  /// `"5.1.2"` — FL+FR+FC+LFE+SL+SR+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1POINT2`).
  ///
  /// The unqualified name is the **side** layout here — the opposite of
  /// the 5.1 family, where it is the back one. See [`Self::Ch5_1_2Back`].
  Ch5_1_2,
  /// `"5.1.2(back)"` — FL+FR+FC+LFE+BL+BR+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1POINT2_BACK`).
  ///
  /// Here the qualifier runs the opposite way to the 5.1 family: the
  /// unqualified `"5.1.2"` is the *side* layout, [`Self::Ch5_1_2`].
  /// FFmpeg's `AV_CH_LAYOUT_7POINT1_TOP_BACK` is a deprecated alias of
  /// this same layout, not a separate one.
  Ch5_1_2Back,
  /// `"5.1.4"` — FL+FR+FC+LFE+SL+SR+TFL+TFR+TBL+TBR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1POINT4_BACK`).
  ///
  /// The constant's `_BACK` marks the top-*back* height pair, not the
  /// surrounds — those are side (SL/SR) — so the slug carries no
  /// qualifier.
  Ch5_1_4Back,
  /// `"6.0"` — FL+FR+FC+BC+SL+SR (FFmpeg `AV_CH_LAYOUT_6POINT0`).
  Ch6_0,
  /// `"6.0(front)"` — FL+FR+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_6POINT0_FRONT`).
  ///
  /// No front centre: the centre pair is FLC/FRC, off-centre.
  Ch6_0Front,
  /// `"6.1"` — FL+FR+FC+LFE+BC+SL+SR (FFmpeg `AV_CH_LAYOUT_6POINT1`).
  Ch6_1,
  /// `"6.1(back)"` — FL+FR+FC+LFE+BL+BR+BC (FFmpeg
  /// `AV_CH_LAYOUT_6POINT1_BACK`).
  Ch6_1Back,
  /// `"6.1(front)"` — FL+FR+LFE+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_6POINT1_FRONT`).
  ///
  /// No front centre, as with [`Self::Ch6_0Front`].
  Ch6_1Front,
  /// `"7.0"` — FL+FR+FC+BL+BR+SL+SR (FFmpeg `AV_CH_LAYOUT_7POINT0`).
  Ch7_0,
  /// `"7.0(front)"` — FL+FR+FC+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT0_FRONT`).
  Ch7_0Front,
  /// `"7.1"` — FL+FR+FC+LFE+BL+BR+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1`).
  Ch7_1,
  /// `"7.1(wide-side)"` — FL+FR+FC+LFE+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1_WIDE`).
  ///
  /// FFmpeg's plain `"7.1(wide)"` is the **back** layout, not this one —
  /// see [`Self::Ch7_1WideBack`]. The same crossing as the 5.x pairs,
  /// one family further up.
  Ch7_1Wide,
  /// `"7.1(wide)"` — FL+FR+FC+LFE+BL+BR+FLC+FRC (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1_WIDE_BACK`).
  ///
  /// The unqualified `"7.1(wide)"` belongs to the back-speaker layout;
  /// the side-speaker one is [`Self::Ch7_1Wide`], spelled
  /// `"7.1(wide-side)"`.
  Ch7_1WideBack,
  /// `"7.1.2"` — FL+FR+FC+LFE+BL+BR+SL+SR+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1POINT2`).
  Ch7_1_2,
  /// `"7.1.4"` — FL+FR+FC+LFE+BL+BR+SL+SR+TFL+TFR+TBL+TBR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1POINT4_BACK`).
  ///
  /// `_BACK` marks the top-back height pair, not the surrounds — see
  /// [`Self::Ch5_1_4Back`].
  Ch7_1_4Back,
  /// `"7.2.3"` — FL+FR+FC+LFE+BL+BR+SL+SR+TFL+TFR+TBC+LFE2 (FFmpeg
  /// `AV_CH_LAYOUT_7POINT2POINT3`).
  ///
  /// The `.2` is two LFE channels (LFE and LFE2); the `.3` is three
  /// height channels (TFL, TFR, TBC).
  Ch7_2_3,
  /// `"9.1.4"` — FL+FR+FC+LFE+BL+BR+FLC+FRC+SL+SR+TFL+TFR+TBL+TBR
  /// (FFmpeg `AV_CH_LAYOUT_9POINT1POINT4_BACK`).
  ///
  /// `_BACK` marks the top-back height pair, not the surrounds — see
  /// [`Self::Ch5_1_4Back`].
  Ch9_1_4Back,
  /// `"9.1.6"` — FL+FR+FC+LFE+BL+BR+FLC+FRC+SL+SR+TFL+TFR+TBL+TBR+
  /// TSL+TSR (FFmpeg `AV_CH_LAYOUT_9POINT1POINT6`).
  ///
  /// [`Self::Ch9_1_4Back`] plus the top *side* pair. This one is the
  /// exception that proves the previous rule: its constant carries no
  /// `_BACK`, because the height channels it adds are not back ones.
  Ch9_1_6,
  /// `"22.2"` — the 24-channel NHK Super Hi-Vision arrangement:
  /// FL+FR+FC+LFE+BL+BR+FLC+FRC+BC+SL+SR+TC+TFL+TFC+TFR+TBL+TBC+TBR+
  /// LFE2+TSL+TSR+BFC+BFL+BFR (FFmpeg `AV_CH_LAYOUT_22POINT2`).
  Ch22_2,
  /// `"hexagonal"` — FL+FR+FC+BL+BR+BC (FFmpeg
  /// `AV_CH_LAYOUT_HEXAGONAL`). Six channels in a hexagon, no LFE.
  Hexagonal,
  /// `"octagonal"` — FL+FR+FC+BL+BR+BC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_OCTAGONAL`). Eight channels around, no LFE.
  Octagonal,
  /// `"hexadecagonal"` — FL+FR+FC+BL+BR+BC+SL+SR+TFL+TFC+TFR+TBL+TBC+
  /// TBR+WL+WR (FFmpeg `AV_CH_LAYOUT_HEXADECAGONAL`). Sixteen channels,
  /// no LFE.
  Hexadecagonal,
  /// `"cube"` — FL+FR+BL+BR+TFL+TFR+TBL+TBR (FFmpeg
  /// `AV_CH_LAYOUT_CUBE`). Eight channels on the corners of a cube, no
  /// centre and no LFE.
  Cube,
  /// First-order Ambisonic B-format (WXYZ, 4 channels): `"ambisonic1"`.
  Ambisonic1,
  /// Second-order Ambisonic (9 channels): `"ambisonic2"`.
  Ambisonic2,
  /// Third-order Ambisonic (16 channels): `"ambisonic3"`.
  Ambisonic3,
  /// A layout not enumerated above — carries the FFmpeg-canonical
  /// name verbatim (a custom channel ordering, a higher-order ambisonic
  /// grouping, a layout a later FFmpeg adds). Lossless escape.
  ///
  /// Every entry in FFmpeg n9.0's `channel_layout_map[]` is named above,
  /// so nothing this release can classify lands here.
  Other(Utf8Bytes),
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
    Self::Other(Utf8Bytes::new())
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
      Self::StereoDownmix => "downmix",
      Self::Binaural => "binaural",
      Self::Ch2_1 => "2.1",
      Self::Ch3_0 => "3.0",
      Self::Ch3_0Back => "3.0(back)",
      Self::Ch3_1 => "3.1",
      Self::Ch3_1_2 => "3.1.2",
      Self::Ch4_0 => "4.0",
      Self::Ch4_1 => "4.1",
      Self::Quad => "quad",
      Self::QuadSide => "quad(side)",
      // FFmpeg gives the unqualified name to the BACK layout and
      // qualifies the side one — see `channel_layout_map[]`. The idents
      // follow the constants, the slugs follow FFmpeg's names, so these
      // six read crossed and are exactly right.
      Self::Ch5_0 => "5.0(side)",
      Self::Ch5_0Back => "5.0",
      Self::Ch5_1 => "5.1(side)",
      Self::Ch5_1Back => "5.1",
      // ...and here the qualifier runs the other way: the unqualified
      // "5.1.2" is the SIDE layout.
      Self::Ch5_1_2 => "5.1.2",
      Self::Ch5_1_2Back => "5.1.2(back)",
      // `_BACK` in these three constants marks the top-back height pair,
      // not the surrounds, and FFmpeg minted no unsuffixed sibling — so
      // no `(back)` reaches the slug.
      Self::Ch5_1_4Back => "5.1.4",
      Self::Ch6_0 => "6.0",
      Self::Ch6_0Front => "6.0(front)",
      Self::Ch6_1 => "6.1",
      Self::Ch6_1Back => "6.1(back)",
      Self::Ch6_1Front => "6.1(front)",
      Self::Ch7_0 => "7.0",
      Self::Ch7_0Front => "7.0(front)",
      Self::Ch7_1 => "7.1",
      Self::Ch7_1Wide => "7.1(wide-side)",
      Self::Ch7_1WideBack => "7.1(wide)",
      Self::Ch7_1_2 => "7.1.2",
      Self::Ch7_1_4Back => "7.1.4",
      Self::Ch7_2_3 => "7.2.3",
      Self::Ch9_1_4Back => "9.1.4",
      Self::Ch9_1_6 => "9.1.6",
      Self::Ch22_2 => "22.2",
      Self::Hexagonal => "hexagonal",
      Self::Octagonal => "octagonal",
      Self::Hexadecagonal => "hexadecagonal",
      Self::Cube => "cube",
      Self::Ambisonic1 => "ambisonic1",
      Self::Ambisonic2 => "ambisonic2",
      Self::Ambisonic3 => "ambisonic3",
      Self::Other(s) => s.as_str(),
    }
  }
  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`]'s own match table,
  /// walked through [`Self::from_str`] rather than duplicated here — so a
  /// canonical spelling returns that **named** variant, never a second
  /// value for a meaning this vocabulary already has one for. Only a
  /// genuine stranger reaches [`Self::Other`], carrying the caller's
  /// spelling verbatim: the escape is a lossless passthrough for a name
  /// this build does not know, not a fold target.
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  ChannelLayout,
  "channel layout",
  [
    Mono,
    Stereo,
    StereoDownmix,
    Binaural,
    Ch2_1,
    Ch3_0,
    Ch3_0Back,
    Ch3_1,
    Ch3_1_2,
    Ch4_0,
    Ch4_1,
    Quad,
    QuadSide,
    Ch5_0,
    Ch5_0Back,
    Ch5_1,
    Ch5_1Back,
    Ch5_1_2,
    Ch5_1_2Back,
    Ch5_1_4Back,
    Ch6_0,
    Ch6_0Front,
    Ch6_1,
    Ch6_1Back,
    Ch6_1Front,
    Ch7_0,
    Ch7_0Front,
    Ch7_1,
    Ch7_1Wide,
    Ch7_1WideBack,
    Ch7_1_2,
    Ch7_1_4Back,
    Ch7_2_3,
    Ch9_1_4Back,
    Ch9_1_6,
    Ch22_2,
    Hexagonal,
    Octagonal,
    Hexadecagonal,
    Cube,
    Ambisonic1,
    Ambisonic2,
    Ambisonic3
  ],
  escape: Other
);

impl FromStr for ChannelLayout {
  type Err = core::convert::Infallible;
  /// Recognise a canonical layout slug; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"mono" => Self::Mono,
      b"stereo" => Self::Stereo,
      b"downmix" => Self::StereoDownmix,
      b"binaural" => Self::Binaural,
      b"2.1" => Self::Ch2_1,
      b"3.0" => Self::Ch3_0,
      b"3.0(back)" => Self::Ch3_0Back,
      b"3.1" => Self::Ch3_1,
      b"3.1.2" => Self::Ch3_1_2,
      b"4.0" => Self::Ch4_0,
      b"4.1" => Self::Ch4_1,
      b"quad" => Self::Quad,
      b"quad(side)" => Self::QuadSide,
      b"5.0(side)" => Self::Ch5_0,
      b"5.0" => Self::Ch5_0Back,
      b"5.1(side)" => Self::Ch5_1,
      b"5.1" => Self::Ch5_1Back,
      b"5.1.2" => Self::Ch5_1_2,
      b"5.1.2(back)" => Self::Ch5_1_2Back,
      b"5.1.4" => Self::Ch5_1_4Back,
      b"6.0" => Self::Ch6_0,
      b"6.0(front)" => Self::Ch6_0Front,
      b"6.1" => Self::Ch6_1,
      b"6.1(back)" => Self::Ch6_1Back,
      b"6.1(front)" => Self::Ch6_1Front,
      b"7.0" => Self::Ch7_0,
      b"7.0(front)" => Self::Ch7_0Front,
      b"7.1" => Self::Ch7_1,
      b"7.1(wide-side)" => Self::Ch7_1Wide,
      b"7.1(wide)" => Self::Ch7_1WideBack,
      b"7.1.2" => Self::Ch7_1_2,
      b"7.1.4" => Self::Ch7_1_4Back,
      b"7.2.3" => Self::Ch7_2_3,
      b"9.1.4" => Self::Ch9_1_4Back,
      b"9.1.6" => Self::Ch9_1_6,
      b"22.2" => Self::Ch22_2,
      b"hexagonal" => Self::Hexagonal,
      b"octagonal" => Self::Octagonal,
      b"hexadecagonal" => Self::Hexadecagonal,
      b"cube" => Self::Cube,
      b"ambisonic1" => Self::Ambisonic1,
      b"ambisonic2" => Self::Ambisonic2,
      b"ambisonic3" => Self::Ambisonic3,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

#[cfg(test)]
mod tests;
