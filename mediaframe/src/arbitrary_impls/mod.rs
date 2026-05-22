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

/// `impl arbitrary::Arbitrary for $Ty` for open string enums: 50/50 picks a
/// curated slug (round-trips through total `FromStr`) or builds an
/// `Other(SmolStr::from(<arbitrary String>))`. Constructing `Self::Other` is
/// allowed because this module is *inside* the crate (non_exhaustive applies
/// at the API surface, not in-crate).
#[allow(unused_macros)]
macro_rules! arb_open_string_enum {
  ($ty:path, [$($slug:literal),+ $(,)?]) => {
    impl<'a> ::arbitrary::Arbitrary<'a> for $ty {
      fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
        const SAMPLES: &[&str] = &[$($slug),+];
        if <bool as ::arbitrary::Arbitrary>::arbitrary(u)? {
          // `FromStr` for these enums is `Infallible`.
          Ok(<$ty as ::core::str::FromStr>::from_str(u.choose(SAMPLES)?).unwrap())
        } else {
          let s = <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?;
          Ok(<$ty>::Other(::smol_str::SmolStr::from(s)))
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
    use ::std::collections::HashSet;
    let mut br: HashSet<crate::audio::BitRateMode> = HashSet::new();
    let mut to: HashSet<crate::subtitle::TrackOrigin> = HashSet::new();
    drive_per_round(0x12C0DE5_u64, 2048, |u| {
      br.insert(crate::audio::BitRateMode::arbitrary(u).unwrap());
      to.insert(crate::subtitle::TrackOrigin::arbitrary(u).unwrap());
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
  #[test]
  fn reachability_sample_format_reaches_all_three_arms() {
    use crate::audio::SampleFormat;
    let mut saw_named = false;
    let mut saw_unknown = false;
    let mut saw_other = false;
    drive_per_round(0x3F0_FEED_u64, 2048, |u| {
      match SampleFormat::arbitrary(u).unwrap() {
        SampleFormat::Unknown(_) => saw_unknown = true,
        SampleFormat::Other(_) => saw_other = true,
        _ => saw_named = true,
      }
    });
    assert!(
      saw_named && saw_unknown && saw_other,
      "SampleFormat arms: named={saw_named} unknown={saw_unknown} other={saw_other}"
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
}
