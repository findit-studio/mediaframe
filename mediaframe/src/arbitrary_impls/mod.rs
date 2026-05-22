// Centralised `arbitrary::Arbitrary` impls for the descriptor vocabulary.
//
// Hand-written (no `#[derive]` on the type definitions) so private fields stay
// encapsulated and `try_new` validated types come out valid by construction.
// Split across three cluster files for parallel ownership; everything is
// re-exported via the module's `impl` items so cross-cluster cascades just
// resolve naturally.
//
//   strings.rs   — open string enums w/ `Other(SmolStr)` (codec×3, container,
//                  subtitle::Format, audio open formats).
//   coded.rs     — closed FFmpeg-coded enums w/ `from_u32` + colour / frame /
//                  pixel-format / disposition structs and enums.
//   composite.rs — audio composite metadata (Loudness/Fingerprint/CoverArt/Tags),
//                  capture (Device/GeoLocation), lang::Language.

mod coded;
mod composite;
mod strings;

/// `impl arbitrary::Arbitrary for $Ty` that decodes the raw `u32` through the
/// type's lossless `from_u32` constructor. Covers every variant (including the
/// `Unknown(u32)` / `Reserved(_)` arms) and stays a single line per type.
#[allow(unused_macros)]
macro_rules! arb_via_code {
  ($($ty:path),* $(,)?) => { $(
    impl<'a> ::arbitrary::Arbitrary<'a> for $ty {
      fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
        Ok(<$ty>::from_u32(<u32 as ::arbitrary::Arbitrary>::arbitrary(u)?))
      }
    }
  )* };
}
#[allow(unused_imports)]
pub(crate) use arb_via_code;

/// `impl arbitrary::Arbitrary for $Ty` for *strictly closed* coded enums —
/// those WITHOUT an `Unknown(u32)` / `Reserved(_)` escape arm. Picks
/// uniformly from the listed named variants via `Unstructured::choose`.
///
/// For low-cardinality closed enums (3-or-so variants) the previous
/// `arb_via_code!` path would skew the value space to the
/// default-fallback case (a raw u32 only lands on `1` or `2` ~3-in-4-
/// billion of the time), making most named variants effectively
/// unreachable in fuzz / arbitrary-driven property tests. This macro
/// guarantees every named variant is reachable.
#[allow(unused_macros)]
macro_rules! arb_via_named_variants {
  ($ty:path, [$($variant:ident),+ $(,)?]) => {
    impl<'a> ::arbitrary::Arbitrary<'a> for $ty {
      fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
        const NAMED: &[$ty] = &[$(<$ty>::$variant),+];
        Ok(*u.choose(NAMED)?)
      }
    }
  };
}
#[allow(unused_imports)]
pub(crate) use arb_via_named_variants;

/// `impl arbitrary::Arbitrary for $Ty` for closed coded enums WITH an
/// `Unknown(u32)` escape arm: 50/50 between picking uniformly from the
/// listed named variants (via `Unstructured::choose`) and round-tripping
/// an arbitrary `u32` through `from_u32` (which exercises the `Unknown`
/// arm for non-canonical codes). Guarantees both paths are commonly hit
/// — `arb_via_code!` alone biases low-cardinality enums towards the
/// `Unknown(_)` arm.
#[allow(unused_macros)]
macro_rules! arb_via_code_weighted {
  ($ty:path, [$($variant:ident),+ $(,)?]) => {
    impl<'a> ::arbitrary::Arbitrary<'a> for $ty {
      fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
        if <bool as ::arbitrary::Arbitrary>::arbitrary(u)? {
          const NAMED: &[$ty] = &[$(<$ty>::$variant),+];
          Ok(*u.choose(NAMED)?)
        } else {
          Ok(<$ty>::from_u32(<u32 as ::arbitrary::Arbitrary>::arbitrary(u)?))
        }
      }
    }
  };
}
#[allow(unused_imports)]
pub(crate) use arb_via_code_weighted;

/// `impl arbitrary::Arbitrary for $Ty` for closed coded enums WITH an
/// `Unknown(u32)` arm whose named codes span a contiguous-ish low-integer
/// range (typical of FFmpeg `AV*` coded enums — colour, pixel-format —
/// where listing every named variant in a macro would be unwieldy). 50/50
/// between an in-`0..=max_named` `u32` pick (most of which land on named
/// variants; gaps fall on `Unknown`, fine for fuzz coverage) and an
/// arbitrary full-range `u32` (broad `Unknown` exercise). `arb_via_code!`
/// alone almost never reaches the named range for these — e.g.
/// `PixelFormat`'s 270 named codes span `0..=947` out of the full `u32`.
#[allow(unused_macros)]
macro_rules! arb_via_code_weighted_range {
  ($ty:path, max_named = $max:expr) => {
    impl<'a> ::arbitrary::Arbitrary<'a> for $ty {
      fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
        let code = if <bool as ::arbitrary::Arbitrary>::arbitrary(u)? {
          u.int_in_range(0u32..=$max)?
        } else {
          <u32 as ::arbitrary::Arbitrary>::arbitrary(u)?
        };
        Ok(<$ty>::from_u32(code))
      }
    }
  };
}
#[allow(unused_imports)]
pub(crate) use arb_via_code_weighted_range;

/// `impl arbitrary::Arbitrary for $Ty` for open string enums: 50/50 picks a
/// curated slug or an arbitrary string — **both routed through `FromStr`**.
///
/// `FromStr` is the canonicalising constructor: a named slug yields the
/// named variant, only a non-named slug yields `Other`. Going through it
/// (rather than `Other(SmolStr::from(s))` directly) guarantees every
/// generated value is canonical / round-trippable — a string that happens
/// to equal a named slug becomes that named variant, never a malformed
/// `Other("h264")` that serde would canonicalise to `H264` on the round
/// trip (Codex round-4 finding). An arbitrary string is virtually never a
/// named slug, so the `Other` arm stays well-covered.
#[allow(unused_macros)]
macro_rules! arb_open_string_enum {
  ($ty:path, [$($slug:literal),+ $(,)?]) => {
    impl<'a> ::arbitrary::Arbitrary<'a> for $ty {
      fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
        const SAMPLES: &[&str] = &[$($slug),+];
        // `FromStr` for these enums is `Infallible`.
        if <bool as ::arbitrary::Arbitrary>::arbitrary(u)? {
          Ok(<$ty as ::core::str::FromStr>::from_str(u.choose(SAMPLES)?).unwrap())
        } else {
          let s = <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?;
          Ok(<$ty as ::core::str::FromStr>::from_str(&s).unwrap())
        }
      }
    }
  };
}
#[allow(unused_imports)]
pub(crate) use arb_open_string_enum;

#[cfg(test)]
mod tests {
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
      let _ = crate::lang::Language::arbitrary(u).unwrap();
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

  // Reachability — the strictly closed coded enums (no `Unknown(u32)`
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
    let mut to: BTreeSet<u32> = BTreeSet::new();
    drive_per_round(0x12C0DE5_u64, 2048, |u| {
      br.insert(crate::audio::BitRateMode::arbitrary(u).unwrap().to_u32());
      to.insert(crate::subtitle::TrackOrigin::arbitrary(u).unwrap().to_u32());
    });
    assert_eq!(br.len(), 3, "BitRateMode coverage: {br:?}");
    assert_eq!(to.len(), 3, "TrackOrigin coverage: {to:?}");
  }

  // Reachability — a small coded enum with an `Unknown(u32)` arm
  // (`arb_via_code_weighted!`) MUST visit every named variant AND the
  // `Unknown(_)` arm. `Rotation` is a typical 4-named + `Unknown(u32)`
  // case; uniform raw `u32` previously almost never landed on `0..=3`.
  #[test]
  fn reachability_weighted_coded_enum_hits_all_named_and_unknown() {
    use crate::frame::Rotation;
    let mut saw_d0 = false;
    let mut saw_d90 = false;
    let mut saw_d180 = false;
    let mut saw_d270 = false;
    let mut saw_unknown = false;
    drive_per_round(0x20C0DE5_u64, 2048, |u| {
      match Rotation::arbitrary(u).unwrap() {
        Rotation::D0 => saw_d0 = true,
        Rotation::D90 => saw_d90 = true,
        Rotation::D180 => saw_d180 = true,
        Rotation::D270 => saw_d270 = true,
        Rotation::Unknown(_) => saw_unknown = true,
      }
    });
    assert!(
      saw_d0 && saw_d90 && saw_d180 && saw_d270 && saw_unknown,
      "Rotation coverage: D0={saw_d0} D90={saw_d90} D180={saw_d180} D270={saw_d270} Unknown={saw_unknown}"
    );
  }

  // Reachability — `SampleFormat` has BOTH `Unknown(u32)` and
  // `Other(SmolStr)`. The previous open-string-enum macro routed only
  // through slugs / `Other`, leaving `Unknown(_)` unreachable. The
  // bespoke 3-way generator MUST hit all three arms.
  // Every one of `SampleFormat`'s 12 named variants must be reachable —
  // plus the `Unknown(_)` and `Other(_)` escape arms. A weaker
  // "some named appears" check (Codex round-2 finding) would pass even
  // if half the slug list were missing.
  #[test]
  fn reachability_sample_format_all_named_plus_arms() {
    use crate::audio::SampleFormat;
    use ::std::collections::BTreeSet;
    let mut named: BTreeSet<::std::string::String> = BTreeSet::new();
    let mut saw_unknown = false;
    let mut saw_other = false;
    drive_per_round(0x3F0_FEED_u64, 4096, |u| {
      match SampleFormat::arbitrary(u).unwrap() {
        SampleFormat::Unknown(_) => saw_unknown = true,
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
    assert!(saw_unknown, "SampleFormat: never observed `Unknown(_)`");
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
      matrix.insert(crate::color::Matrix::arbitrary(u).unwrap().to_u32());
      primaries.insert(crate::color::Primaries::arbitrary(u).unwrap().to_u32());
      transfer.insert(crate::color::Transfer::arbitrary(u).unwrap().to_u32());
      pixel.insert(
        crate::pixel_format::PixelFormat::arbitrary(u)
          .unwrap()
          .to_u32(),
      );
    });
    // Count distinct codes within each type's named range.
    let in_range = |s: &BTreeSet<u32>, max: u32| s.iter().filter(|&&c| c <= max).count();
    assert!(
      in_range(&matrix, 17) >= 10,
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
      in_range(&primaries, 22) >= 8,
      "Primaries named-range coverage too low: {primaries:?}"
    );
    assert!(
      in_range(&transfer, 18) >= 10,
      "Transfer named-range coverage too low: {transfer:?}"
    );
    // PixelFormat: 270 named codes spread over 0..=947 — a generous floor.
    assert!(
      in_range(&pixel, 947) >= 40,
      "PixelFormat named-range coverage too low: {} distinct",
      in_range(&pixel, 947)
    );
  }

  // For coded enums, `from_u32(to_u32(x)) == x` is the lossless-roundtrip
  // contract. Verifies cluster B's macro applies the right code path.
  #[test]
  fn coded_enums_roundtrip_through_code() {
    drive(0xE11E, 128, |u| {
      let m = crate::color::Matrix::arbitrary(u).unwrap();
      assert_eq!(crate::color::Matrix::from_u32(m.to_u32()), m);
      let p = crate::pixel_format::PixelFormat::arbitrary(u).unwrap();
      assert_eq!(crate::pixel_format::PixelFormat::from_u32(p.to_u32()), p);
      let r = crate::frame::Rotation::arbitrary(u).unwrap();
      assert_eq!(crate::frame::Rotation::from_u32(r.to_u32()), r);
      let d = crate::disposition::TrackDisposition::arbitrary(u).unwrap();
      assert_eq!(
        crate::disposition::TrackDisposition::from_u32(d.to_u32()),
        d
      );
    });
  }

  // Arbitrary-generated values must survive a serde round-trip unchanged
  // (Codex round-4/5 findings). Every `arbitrary` impl here generates only
  // *canonical* values: `Unknown(v)` whose `v` is canonical, named variants,
  // `Other` slugs that are genuinely non-named, and — crucially — `Loudness`
  // with FINITE floats (non-finite `f32` would JSON-serialize as `null` and
  // fail to deserialize). A generator that produced `Other("s16")`,
  // `Unknown(<named code>)`, or a NaN/inf `Loudness` field would fail this.
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
      0x7A65_1A_u64,
      1024,
      |u| match crate::audio::Tags::arbitrary(u).unwrap().language() {
        None => saw_none = true,
        Some(_) => saw_some = true,
      },
    );
    assert!(saw_none, "Tags.language never generated `None`");
    assert!(saw_some, "Tags.language never generated `Some(_)`");
  }
}
