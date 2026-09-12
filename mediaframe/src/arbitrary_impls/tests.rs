use ::arbitrary::{Arbitrary, Unstructured};

// Fixed byte buffer drives a deterministic stream of `Arbitrary` decodes
// across N rounds. We don't care that the values are "random" — we care
// that the impls don't panic, that validated types come out valid, and
// that closed enums round-trip through their code.
fn drive<F: FnMut(&mut Unstructured<'_>)>(seed: u64, rounds: usize, mut body: F) {
  // Mix the seed into a 4 KiB buffer so each round gets fresh bytes.
  let mut bytes = ::std::vec![0u8; 4096];
  for (i, b) in bytes.iter_mut().enumerate() {
    *b = ((seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ i as u64) & 0xff) as u8;
  }
  let mut u = Unstructured::new(&bytes);
  for _ in 0..rounds {
    body(&mut u);
  }
}

#[test]
fn geo_location_invariant_lat_lon_in_range() {
  drive(0xA11CE, 256, |u| {
    let g = crate::capture::GeoLocation::arbitrary(u).unwrap();
    assert!(
      (-90.0..=90.0).contains(&g.lat()),
      "lat out of range: {}",
      g.lat()
    );
    assert!(
      (-180.0..=180.0).contains(&g.lon()),
      "lon out of range: {}",
      g.lon()
    );
    if let Some(alt) = g.altitude() {
      assert!(
        alt.is_finite(),
        "altitude must be finite when Some, got {alt}"
      );
    }
  });
}

#[test]
fn fingerprint_invariant_algorithm_non_empty() {
  drive(0xB0B, 256, |u| {
    let fp = crate::audio::Fingerprint::arbitrary(u).unwrap();
    assert!(!fp.algorithm().is_empty(), "algorithm must be non-empty");
  });
}

#[test]
fn cover_art_invariant_mime_and_data_non_empty() {
  drive(0xC0FFEE, 256, |u| {
    let c = crate::audio::CoverArt::arbitrary(u).unwrap();
    assert!(!c.mime().is_empty(), "mime must be non-empty");
    assert!(!c.data().is_empty(), "data must be non-empty");
  });
}

#[test]
fn smoke_yields_values_for_representative_types() {
  drive(0xD1CE, 64, |u| {
    let _ = crate::codec::VideoCodec::arbitrary(u).unwrap();
    let _ = crate::color::Info::arbitrary(u).unwrap();
    let _ = crate::frame::FrameRate::arbitrary(u).unwrap();
    let _ = crate::lang::LanguageId::arbitrary(u).unwrap();
    let _ = crate::disposition::TrackDisposition::arbitrary(u).unwrap();
  });
}

// Like `drive`, but builds a fresh `Unstructured` per round seeded
// with a different byte buffer — needed for reachability tests, since
// `Arbitrary` consumes bytes from the same `Unstructured` and a
// single 4 KiB buffer exhausts quickly into all-zero fallbacks
// (biasing every per-round decode to the same low-index variant).
fn drive_per_round<F: FnMut(&mut Unstructured<'_>)>(seed: u64, rounds: usize, mut body: F) {
  let mut state = seed
    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
    .wrapping_add(0xDEAD_BEEF_CAFE_F00D);
  for _ in 0..rounds {
    let mut bytes = ::std::vec![0u8; 64];
    for b in bytes.iter_mut() {
      // SplitMix64-ish: advance state, then mix into the byte.
      state = state
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9);
      let mixed = state ^ (state >> 27);
      *b = (mixed.wrapping_mul(0x94D0_49BB_1331_11EB) >> 56) as u8;
    }
    let mut u = Unstructured::new(&bytes);
    body(&mut u);
  }
}

// Reachability — the strictly closed coded enums (no escape arm
// arm) MUST visit every named variant under arbitrary-driven sampling.
// Codex round-1 finding: feeding raw `u32::arbitrary` into a 3-arm
// `from_u32` skewed ~3-in-4-billion of the value space to the
// non-default named variants, making them effectively unreachable.
// `arb_via_named_variants!` now picks uniformly from the named set.
#[test]
fn reachability_small_closed_coded_enums_hit_all_named() {
  // Sets keyed on the `to_u32()` code — `BitRateMode` / `TrackOrigin`
  // aren't `Ord` (nor `Hash`-keyed here), and a `u32`-keyed `BTreeSet`
  // needs no hasher.
  use ::std::collections::BTreeSet;
  let mut br: BTreeSet<u32> = BTreeSet::new();
  let mut co: BTreeSet<u32> = BTreeSet::new();
  drive_per_round(0x12C0DE5_u64, 2048, |u| {
    br.insert(crate::audio::BitRateMode::arbitrary(u).unwrap().to_u32());
    co.insert(crate::audio::ChannelOrder::arbitrary(u).unwrap().to_u32());
  });
  assert_eq!(br.len(), 3, "BitRateMode coverage: {br:?}");
  assert_eq!(co.len(), 4, "ChannelOrder coverage: {co:?}");
}

/// The channel household's two records have no invariant across their
/// fields, so the generator draws each one independently — and the
/// incoherent combinations that draw makes reachable are the point.
/// A generator that only produced well-formed FFmpeg layouts would leave
/// exactly the values a consumer is most likely to mishandle unreachable.
#[test]
fn reachability_channel_records_reach_their_incoherent_combinations() {
  use crate::audio::{ChannelLayoutDescription, ChannelOrder, ChannelSpec};

  let mut indices = 0u32;
  let mut raw_ids = 0u32;
  let mut labelled = false;
  let mut custom_without_channels = false;
  let mut native_without_mask = false;
  let mut populated = false;
  let mut described = false;
  drive_per_round(0xC4A11E1_u64, 2048, |u| {
    let spec = ChannelSpec::arbitrary(u).unwrap();
    indices |= spec.index();
    raw_ids |= spec.raw_id();
    labelled |= !spec.label().is_empty();

    let d = ChannelLayoutDescription::arbitrary(u).unwrap();
    custom_without_channels |= d.order() == ChannelOrder::Custom && d.custom_channels().is_empty();
    native_without_mask |= d.order() == ChannelOrder::Native && d.native_mask().is_none();
    populated |= !d.custom_channels().is_empty();
    described |= !d.text().is_empty();
  });
  assert_ne!(indices, 0, "spec index never left zero");
  assert_ne!(raw_ids, 0, "spec raw_id never left zero");
  assert!(labelled, "spec label was never populated");
  assert!(
    custom_without_channels,
    "a Custom order with no channel list is unreachable"
  );
  assert!(
    native_without_mask,
    "a Native order with no mask is unreachable"
  );
  assert!(populated, "custom_channels was never non-empty");
  assert!(described, "text was never populated");
}

/// `TrackOrigin` became an open string enum in 0.5.0, so it generates
/// the way the other name vocabularies do: every named variant plus the
/// `Other(_)` escape must be reachable.
#[test]
fn reachability_track_origin_hits_all_named_and_escape() {
  use crate::subtitle::TrackOrigin;
  use ::std::collections::BTreeSet;
  let mut named: BTreeSet<u32> = BTreeSet::new();
  let mut saw_other = false;
  drive_per_round(0x7401C1_u64, 2048, |u| {
    match TrackOrigin::arbitrary(u).unwrap().to_u32() {
      Some(code) => {
        named.insert(code);
      }
      None => saw_other = true,
    }
  });
  assert_eq!(named.len(), 4, "TrackOrigin named coverage: {named:?}");
  assert!(saw_other, "TrackOrigin `Other` arm never generated");
}

// Reachability — a small name vocabulary with an `Other(Utf8Bytes)` arm
// (`arb_via_code_weighted!`) MUST visit every named variant AND the
// `Other(_)` arm. `Rotation` is a typical 4-named + `Other(Utf8Bytes)`
// case; uniform raw `u32` previously almost never landed on `0..=3`.
#[test]
fn reachability_weighted_coded_enum_hits_all_named_and_unknown() {
  use crate::frame::Rotation;
  let mut saw_d0 = false;
  let mut saw_d90 = false;
  let mut saw_d180 = false;
  let mut saw_d270 = false;
  let mut saw_other = false;
  drive_per_round(0x20C0DE5_u64, 2048, |u| {
    match Rotation::arbitrary(u).unwrap() {
      Rotation::D0 => saw_d0 = true,
      Rotation::D90 => saw_d90 = true,
      Rotation::D180 => saw_d180 = true,
      Rotation::D270 => saw_d270 = true,
      Rotation::Other(_) => saw_other = true,
    }
  });
  assert!(
    saw_d0 && saw_d90 && saw_d180 && saw_d270 && saw_other,
    "Rotation coverage: D0={saw_d0} D90={saw_d90} D180={saw_d180} D270={saw_d270} Other={saw_other}"
  );
}

// Every one of `SampleFormat`'s 12 named variants must be reachable —
// plus the `Other(_)` escape arm. A weaker "some named appears" check
// (Codex round-2 finding) would pass even if half the slug list were
// missing.
#[test]
fn reachability_sample_format_all_named_plus_arms() {
  use crate::audio::SampleFormat;
  use ::std::collections::BTreeSet;
  let mut named: BTreeSet<::std::string::String> = BTreeSet::new();
  let mut saw_other = false;
  drive_per_round(0x3F0_FEED_u64, 4096, |u| {
    match SampleFormat::arbitrary(u).unwrap() {
      SampleFormat::Other(_) => saw_other = true,
      other => {
        named.insert(::std::string::String::from(other.as_str()));
      }
    }
  });
  assert_eq!(
    named.len(),
    12,
    "missing named SampleFormat variants; observed: {named:?}"
  );
  assert!(saw_other, "SampleFormat: never observed `Other(_)`");
}

// The range-weighted large coded enums must actually reach a broad set
// of named codes — `arb_via_code!` (uniform `u32`) hit the named range
// for `Matrix` / `Primaries` essentially never (Codex round-2 finding).
#[test]
fn reachability_range_weighted_enums_hit_named_codes() {
  use ::std::collections::BTreeSet;
  let mut matrix: BTreeSet<u32> = BTreeSet::new();
  let mut primaries: BTreeSet<u32> = BTreeSet::new();
  let mut transfer: BTreeSet<u32> = BTreeSet::new();
  let mut pixel: BTreeSet<u32> = BTreeSet::new();
  drive_per_round(0x4A_C0DE5_u64, 8192, |u| {
    matrix.extend(crate::color::Matrix::arbitrary(u).unwrap().to_u32());
    primaries.extend(crate::color::Primaries::arbitrary(u).unwrap().to_u32());
    transfer.extend(crate::color::Transfer::arbitrary(u).unwrap().to_u32());
    pixel.extend(
      crate::pixel_format::PixelFormat::arbitrary(u)
        .unwrap()
        .to_u32(),
    );
  });
  // Count distinct codes within each type's named range.
  let in_range = |s: &BTreeSet<u32>, max: u32| s.iter().filter(|&&c| c <= max).count();
  assert!(
    in_range(&matrix, 17) >= 3,
    "Matrix named-range coverage too low: {matrix:?}"
  );
  // `Matrix::Bt601` is the domain-extension variant at `DOMAIN_EXT_BASE`
  // — must be reached by the hand-written 3-way `Matrix` impl, not just
  // the rare full-`u32` fallback.
  assert!(
    matrix.contains(&crate::color::DOMAIN_EXT_BASE),
    "Matrix::Bt601 (DOMAIN_EXT_BASE) never generated"
  );
  assert!(
    in_range(&primaries, 22) >= 3,
    "Primaries named-range coverage too low: {primaries:?}"
  );
  assert!(
    in_range(&transfer, 18) >= 3,
    "Transfer named-range coverage too low: {transfer:?}"
  );
  // PixelFormat draws from a curated 6-slug list plus the escape.
  assert!(
    in_range(&pixel, 947) >= 3,
    "PixelFormat named-range coverage too low: {} distinct",
    in_range(&pixel, 947)
  );
}

// For a name vocabulary, `as_str().parse() == x` is the round-trip
// contract — the generator must never mint a value its own text form
// cannot express.
#[test]
fn generated_vocabulary_values_round_trip_through_their_name() {
  drive(0xE11E, 128, |u| {
    macro_rules! rt {
      ($ty:path) => {{
        let v = <$ty>::arbitrary(u).unwrap();
        assert_eq!(v.as_str().parse::<$ty>(), Ok(v.clone()), "{v:?}");
      }};
    }
    rt!(crate::color::Matrix);
    rt!(crate::pixel_format::PixelFormat);
    rt!(crate::frame::Rotation);
    let d = crate::disposition::TrackDisposition::arbitrary(u).unwrap();
    assert_eq!(
      crate::disposition::TrackDisposition::from_u32(d.to_u32()),
      d
    );
  });
}

// Reachability — the four R5/R6-promoted variants (`M2ts`/`Threeg2` on
// `container::Format`, `Aifc` on `audio::ContainerFormat`, `Heic` on
// `image::Format`) must actually be generated, not merely present in
// the enum. Codex R6 finding: before their canonical slugs were added
// to `arb_open_string_enum!`'s curated seed lists, each was reachable
// only through the 50/50 branch's unconstrained-`String` path landing
// on an exact multi-character match by chance — negligible odds, not a
// real coverage guarantee. This test proves the seed-list fix actually
// closes that gap, for all four at once, so a future promotion that
// forgets its seed entry fails here rather than silently shipping
// unreachable.
#[test]
fn reachability_r5_r6_promoted_variants_are_generated() {
  let mut saw_m2ts = false;
  let mut saw_threeg2 = false;
  let mut saw_aifc = false;
  let mut saw_heic = false;
  drive_per_round(0x0025_1067_u64, 4096, |u| {
    saw_m2ts |= crate::container::Format::arbitrary(u).unwrap() == crate::container::Format::M2ts;
    saw_threeg2 |=
      crate::container::Format::arbitrary(u).unwrap() == crate::container::Format::Threeg2;
    saw_aifc |=
      crate::audio::ContainerFormat::arbitrary(u).unwrap() == crate::audio::ContainerFormat::Aifc;
    saw_heic |= crate::image::Format::arbitrary(u).unwrap() == crate::image::Format::Heic;
  });
  assert!(saw_m2ts, "container::Format::M2ts never generated");
  assert!(saw_threeg2, "container::Format::Threeg2 never generated");
  assert!(saw_aifc, "audio::ContainerFormat::Aifc never generated");
  assert!(saw_heic, "image::Format::Heic never generated");
}

// Arbitrary-generated values must survive a serde round-trip unchanged
// (Codex round-4/5 findings). Every `arbitrary` impl here generates only
// *canonical* values: named variants, `Other` slugs that are genuinely
// non-named, and — crucially — `Loudness` with FINITE floats (non-finite
// `f32` would JSON-serialize as `null` and fail to deserialize). A
// generator that produced `Other("s16")` or a NaN/inf `Loudness` field
// would fail this.
#[cfg(feature = "serde")]
#[test]
fn arbitrary_values_survive_serde_round_trip() {
  drive_per_round(0x5E2DE_u64, 4096, |u| {
    let sf = crate::audio::SampleFormat::arbitrary(u).unwrap();
    let json = serde_json::to_string(&sf).unwrap();
    let back: crate::audio::SampleFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sf, "SampleFormat lost identity via serde: {json}");

    let vc = crate::codec::VideoCodec::arbitrary(u).unwrap();
    let json = serde_json::to_string(&vc).unwrap();
    let back: crate::codec::VideoCodec = serde_json::from_str(&json).unwrap();
    assert_eq!(back, vc, "VideoCodec lost identity via serde: {json}");

    // `Loudness` is the serde-derived composite struct with `f32` fields —
    // the round-trip only holds if every field is finite.
    let ld = crate::audio::Loudness::arbitrary(u).unwrap();
    let json = serde_json::to_string(&ld).unwrap();
    let back: crate::audio::Loudness = serde_json::from_str(&json).unwrap();
    assert_eq!(back, ld, "Loudness lost identity via serde: {json}");
  });
}

// `Tags.language` (`Option<Language>`, serialized as buffa field 13) was
// omitted from the `Tags` generator (Codex round-7 finding) — every
// generated `Tags` had `language == None`. Both the absent (`None`) and
// present (`Some(_)`) states must be reachable. `Language` has no empty
// value, so `Some` is unconditionally wire-canonical.
#[test]
fn reachability_tags_language_hits_none_and_some() {
  let mut saw_none = false;
  let mut saw_some = false;
  drive_per_round(
    0x007A_651A_u64,
    1024,
    |u| match crate::audio::Tags::arbitrary(u).unwrap().language() {
      None => saw_none = true,
      Some(_) => saw_some = true,
    },
  );
  assert!(saw_none, "Tags.language never generated `None`");
  assert!(saw_some, "Tags.language never generated `Some(_)`");
}
