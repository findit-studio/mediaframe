use super::*;
use ::std::string::ToString;

/// Every named variant's slug is the name FFmpeg's
/// `channel_layout_map[]` gives the constant that variant is named
/// after — transcribed from `libavutil/channel_layout.c` at the pinned
/// n9.0 tag, and re-checked against `av_channel_layout_describe` output
/// for every `AV_CHANNEL_LAYOUT_*` initializer in that release.
///
/// This exists because a third of these arms *look* wrong:
///
/// * FFmpeg gives the unqualified `"5.0"` / `"5.1"` / `"7.1(wide)"` to
///   the **back**-speaker layouts and qualifies the side ones;
/// * for the 5.1.2 family the qualifier runs the other way;
/// * `_BACK` in `5POINT1POINT4_BACK`, `7POINT1POINT4_BACK` and
///   `9POINT1POINT4_BACK` never reaches the slug at all;
/// * `STEREO_DOWNMIX` is named `"downmix"`, and `2_1` / `2_2` are named
///   `"3.0(back)"` / `"quad(side)"`.
///
/// mediaframe had the 5.x pairs the other way round until 0.4.0, so an
/// FFmpeg-sourced `"5.1"` landed on the variant whose documentation
/// promised side speakers. Nothing but a transcribed table catches that,
/// so here is the whole table; `ambisonic*` are excluded because FFmpeg
/// models ambisonics as a channel *order*, not a map entry.
///
/// This is the map **in full** — see
/// `the_map_is_transcribed_in_full`.
const MAP: &[(ChannelLayout, &str, &str)] = &[
  // (variant, `AV_CH_LAYOUT_` suffix it is named after, FFmpeg's name)
  (ChannelLayout::Mono, "MONO", "mono"),
  (ChannelLayout::Stereo, "STEREO", "stereo"),
  (ChannelLayout::StereoDownmix, "STEREO_DOWNMIX", "downmix"),
  (ChannelLayout::Binaural, "BINAURAL", "binaural"),
  (ChannelLayout::Ch2_1, "2POINT1", "2.1"),
  (ChannelLayout::Ch3_0, "SURROUND", "3.0"),
  (ChannelLayout::Ch3_0Back, "2_1", "3.0(back)"),
  (ChannelLayout::Ch3_1, "3POINT1", "3.1"),
  (ChannelLayout::Ch3_1_2, "3POINT1POINT2", "3.1.2"),
  (ChannelLayout::Ch4_0, "4POINT0", "4.0"),
  (ChannelLayout::Ch4_1, "4POINT1", "4.1"),
  (ChannelLayout::Quad, "QUAD", "quad"),
  (ChannelLayout::QuadSide, "2_2", "quad(side)"),
  (ChannelLayout::Ch5_0, "5POINT0", "5.0(side)"),
  (ChannelLayout::Ch5_0Back, "5POINT0_BACK", "5.0"),
  (ChannelLayout::Ch5_1, "5POINT1", "5.1(side)"),
  (ChannelLayout::Ch5_1Back, "5POINT1_BACK", "5.1"),
  (ChannelLayout::Ch5_1_2, "5POINT1POINT2", "5.1.2"),
  (
    ChannelLayout::Ch5_1_2Back,
    "5POINT1POINT2_BACK",
    "5.1.2(back)",
  ),
  (ChannelLayout::Ch5_1_4Back, "5POINT1POINT4_BACK", "5.1.4"),
  (ChannelLayout::Ch6_0, "6POINT0", "6.0"),
  (ChannelLayout::Ch6_0Front, "6POINT0_FRONT", "6.0(front)"),
  (ChannelLayout::Ch6_1, "6POINT1", "6.1"),
  (ChannelLayout::Ch6_1Back, "6POINT1_BACK", "6.1(back)"),
  (ChannelLayout::Ch6_1Front, "6POINT1_FRONT", "6.1(front)"),
  (ChannelLayout::Ch7_0, "7POINT0", "7.0"),
  (ChannelLayout::Ch7_0Front, "7POINT0_FRONT", "7.0(front)"),
  (ChannelLayout::Ch7_1, "7POINT1", "7.1"),
  (ChannelLayout::Ch7_1Wide, "7POINT1_WIDE", "7.1(wide-side)"),
  (
    ChannelLayout::Ch7_1WideBack,
    "7POINT1_WIDE_BACK",
    "7.1(wide)",
  ),
  (ChannelLayout::Ch7_1_2, "7POINT1POINT2", "7.1.2"),
  (ChannelLayout::Ch7_1_4Back, "7POINT1POINT4_BACK", "7.1.4"),
  (ChannelLayout::Ch7_2_3, "7POINT2POINT3", "7.2.3"),
  (ChannelLayout::Ch9_1_4Back, "9POINT1POINT4_BACK", "9.1.4"),
  (ChannelLayout::Ch9_1_6, "9POINT1POINT6", "9.1.6"),
  (ChannelLayout::Ch22_2, "22POINT2", "22.2"),
  (ChannelLayout::Hexagonal, "HEXAGONAL", "hexagonal"),
  (ChannelLayout::Octagonal, "OCTAGONAL", "octagonal"),
  (
    ChannelLayout::Hexadecagonal,
    "HEXADECAGONAL",
    "hexadecagonal",
  ),
  (ChannelLayout::Cube, "CUBE", "cube"),
];

/// The ambisonic groupings, which have no `channel_layout_map[]` entry
/// to transcribe: FFmpeg models ambisonics as a channel *order*
/// (`AV_CHANNEL_ORDER_AMBISONIC`), so these slugs are this crate's own.
const AMBISONIC: &[(ChannelLayout, &str)] = &[
  (ChannelLayout::Ambisonic1, "ambisonic1"),
  (ChannelLayout::Ambisonic2, "ambisonic2"),
  (ChannelLayout::Ambisonic3, "ambisonic3"),
];

#[test]
fn channel_layout_slugs_match_ffmpegs_map() {
  for (layout, constant, ffmpeg_name) in MAP {
    assert_eq!(
      layout.as_str(),
      *ffmpeg_name,
      "AV_CH_LAYOUT_{constant} is named {ffmpeg_name:?} by FFmpeg, not {:?}",
      layout.as_str()
    );
    assert_eq!(
      ffmpeg_name.parse::<ChannelLayout>().unwrap(),
      *layout,
      "FFmpeg's {ffmpeg_name:?} must read back as the AV_CH_LAYOUT_{constant} variant"
    );
  }
}

#[test]
fn ambisonic_slugs_round_trip() {
  for (layout, slug) in AMBISONIC {
    assert_eq!(layout.as_str(), *slug);
    assert_eq!(slug.parse::<ChannelLayout>().unwrap(), *layout);
  }
}

/// A variant cannot join the roster without a transcribed map row (or a
/// place among the ambisonics). Without this, a new layout could be
/// added with an invented slug and every other test here would still
/// pass — the roster witness only proves the variant is *listed*, not
/// that anyone checked what FFmpeg calls it.
#[test]
fn every_rostered_variant_has_a_transcribed_source() {
  assert_eq!(
    MAP.len() + AMBISONIC.len(),
    ChannelLayout::ROSTER.len(),
    "roster has {} entries but only {} are accounted for by the FFmpeg map \
     ({}) plus the ambisonics ({})",
    ChannelLayout::ROSTER.len(),
    MAP.len() + AMBISONIC.len(),
    MAP.len(),
    AMBISONIC.len()
  );
  for layout in ChannelLayout::ROSTER {
    assert!(
      MAP.iter().any(|(mapped, ..)| mapped == layout)
        || AMBISONIC.iter().any(|(mapped, _)| mapped == layout),
      "{layout} is rostered but appears in neither table"
    );
  }
}

/// FFmpeg n9.0's `channel_layout_map[]` has exactly forty entries, and
/// [`MAP`] transcribes all forty — so this vocabulary names every layout
/// FFmpeg can put a name to, and `Other` carries only what the map has
/// no entry for.
///
/// The count is the pin. `every_rostered_variant_has_a_transcribed_source`
/// catches a variant added without a map row; this catches the reverse —
/// a map row nobody noticed, which is how `"5.1.2"`, `"9.1.6"` and
/// `"binaural"` sat in the escape for three releases. `ffmpeg -layouts`
/// prints the map if it ever needs recounting against a newer FFmpeg;
/// a bump that adds a layout lands here, deliberately, as a red test
/// rather than as silence.
#[test]
fn the_map_is_transcribed_in_full() {
  const FFMPEG_N9_MAP_ENTRIES: usize = 40;
  assert_eq!(
    MAP.len(),
    FFMPEG_N9_MAP_ENTRIES,
    "FFmpeg n9.0 names {FFMPEG_N9_MAP_ENTRIES} layouts; {} are transcribed",
    MAP.len()
  );
}

/// The four 5.x slugs were swapped in 0.4.0. Pin both readings so a
/// well-meaning "fix" cannot quietly put them back.
#[test]
fn the_unqualified_five_point_slugs_are_the_back_layouts() {
  assert_eq!("5.0".parse(), Ok(ChannelLayout::Ch5_0Back));
  assert_eq!("5.1".parse(), Ok(ChannelLayout::Ch5_1Back));
  assert_eq!("5.0(side)".parse(), Ok(ChannelLayout::Ch5_0));
  assert_eq!("5.1(side)".parse(), Ok(ChannelLayout::Ch5_1));
  assert_eq!(ChannelLayout::Ch5_0Back.as_str(), "5.0");
  assert_eq!(ChannelLayout::Ch5_1Back.as_str(), "5.1");
  assert_eq!(ChannelLayout::Ch5_0.as_str(), "5.0(side)");
  assert_eq!(ChannelLayout::Ch5_1.as_str(), "5.1(side)");
}

/// The 7.1-wide pair crosses exactly like the 5.x pairs — the
/// unqualified `"7.1(wide)"` is `AV_CH_LAYOUT_7POINT1_WIDE_BACK`, and
/// `AV_CH_LAYOUT_7POINT1_WIDE` is the qualified `"7.1(wide-side)"`.
/// `mediadecode` 0.5.0 had this pair the other way round.
#[test]
fn the_unqualified_wide_slug_is_the_back_layout() {
  assert_eq!("7.1(wide)".parse(), Ok(ChannelLayout::Ch7_1WideBack));
  assert_eq!("7.1(wide-side)".parse(), Ok(ChannelLayout::Ch7_1Wide));
  assert_eq!(ChannelLayout::Ch7_1WideBack.as_str(), "7.1(wide)");
  assert_eq!(ChannelLayout::Ch7_1Wide.as_str(), "7.1(wide-side)");
}

/// Two ways the `(back)` qualifier does *not* behave like the 5.x one,
/// both of which would be silently "fixed" wrong by anyone
/// generalising from that family.
#[test]
fn the_back_qualifier_is_not_a_rule() {
  // 5.1.2 runs the opposite way to 5.1: here the *unqualified* slug is
  // the side layout and the back one carries the qualifier.
  assert_eq!("5.1.2".parse(), Ok(ChannelLayout::Ch5_1_2));
  assert_eq!("5.1.2(back)".parse(), Ok(ChannelLayout::Ch5_1_2Back));
  assert_eq!(ChannelLayout::Ch5_1_2.as_str(), "5.1.2");
  assert_eq!(ChannelLayout::Ch5_1_2Back.as_str(), "5.1.2(back)");

  // The `_BACK` in these three constants marks a top-back height pair,
  // not surround placement, so no qualifier reaches the slug.
  assert_eq!(ChannelLayout::Ch5_1_4Back.as_str(), "5.1.4");
  assert_eq!(ChannelLayout::Ch7_1_4Back.as_str(), "7.1.4");
  assert_eq!(ChannelLayout::Ch9_1_4Back.as_str(), "9.1.4");
  for slug in ["5.1.4(back)", "7.1.4(back)", "9.1.4(back)"] {
    assert!(
      slug.parse::<ChannelLayout>().unwrap().is_other(),
      "{slug} is not a name FFmpeg gives anything"
    );
  }
}

/// `AV_CH_LAYOUT_7POINT1_TOP_BACK` is a deprecated **alias** of
/// `5POINT1POINT2_BACK`, not a layout of its own — one mask, and
/// therefore one variant. `mediadecode` 0.5.0 spends two variants on it,
/// which makes its `Ch7_1TopBack` unreachable from its own FFmpeg
/// adapter.
#[test]
fn the_top_back_alias_is_not_a_second_layout() {
  assert!("7.1(top-back)".parse::<ChannelLayout>().unwrap().is_other());
  assert_eq!(
    ChannelLayout::ROSTER
      .iter()
      .filter(|layout| layout.as_str() == "5.1.2(back)")
      .count(),
    1
  );
}

/// With the whole map named, the escape only ever carries something the
/// map has no entry for. `av_channel_layout_describe` renders such a
/// layout as its channel list, which is what this uses — a shape FFmpeg
/// can never turn into a map name, unlike the layout slugs that kept
/// getting promoted out of the escape as this vocabulary grew.
#[test]
fn a_layout_outside_the_map_lands_in_other() {
  let v: ChannelLayout = "fl+fr+tfl".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "fl+fr+tfl");
  assert_eq!(v.to_string(), "fl+fr+tfl");
}

#[test]
fn display_matches_as_str() {
  assert_eq!(ChannelLayout::Stereo.to_string(), "stereo");
  assert_eq!(ChannelLayout::Ch5_1.to_string(), "5.1(side)");
  assert_eq!(
    ChannelLayout::Other(Utf8Bytes::from("custom_layout")).to_string(),
    "custom_layout"
  );
}

#[test]
fn is_variant_predicates() {
  assert!(ChannelLayout::Mono.is_mono());
  assert!(ChannelLayout::Stereo.is_stereo());
  assert!(ChannelLayout::Ch5_1.is_ch_5_1());
  assert!(ChannelLayout::Ch5_1_2Back.is_ch_5_1_2_back());
  assert!(ChannelLayout::QuadSide.is_quad_side());
  assert!(ChannelLayout::Other(Utf8Bytes::from("x")).is_other());
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with `Self::other` running that same lookup, so
/// one **named** meaning is one value under the derived `Eq` / `Hash`.
#[test]
fn channellayout_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &["mono", "stereo", "5.1", "7.1", "quad"];
  for (i, slug) in SLUGS.iter().enumerate() {
    assert!(
      !slug.bytes().any(|b| b.is_ascii_uppercase()),
      "slug {slug:?} is not lowercase-canonical"
    );
    for prior in &SLUGS[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two variants fold onto {slug:?}"
      );
    }
    let v: ChannelLayout = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), *slug, "`{slug}` is not its own canonical form");
  }
  assert_eq!("mono", "MONO".parse::<ChannelLayout>().unwrap().as_str());

  // `other()` heals a canonical name to the named variant...
  assert_eq!(ChannelLayout::other("MONO"), ChannelLayout::Mono);
  assert_eq!(ChannelLayout::other("mono"), ChannelLayout::Mono);

  // ...but a genuine stranger keeps its own spelling verbatim — the
  // escape is a passthrough, not a fold target.
  let escaped: ChannelLayout = "MONO_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "MONO_X");
  assert_eq!(ChannelLayout::other("MONO_X"), escaped);
}
#[test]
fn channel_layout_unwrap_other_borrowed_view() {
  let v = ChannelLayout::other("fl+fr+tfl");
  assert_eq!(v.unwrap_other_ref().as_str(), "fl+fr+tfl");
  assert!(v.try_unwrap_other_ref().is_ok());
  assert!(ChannelLayout::Stereo.try_unwrap_other_ref().is_err());
}

/// The runtime half of the `ROSTER` contract for `ChannelLayout` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(
    ChannelLayout::ROSTER,
    "ChannelLayout",
    ChannelLayout::as_str,
  );
}
