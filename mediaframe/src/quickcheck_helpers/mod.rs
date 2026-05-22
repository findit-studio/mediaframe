//! `fn(g: &mut quickcheck::Gen) -> T` helpers — one per descriptor type —
//! referenced via container-level `#[quickcheck(arbitrary = "…")]` on each type's
//! `quickcheck_richderive::Arbitrary` derive.
//!
//! Split across three cluster files for parallel ownership (same axis as
//! [`arbitrary_impls`](crate::arbitrary_impls)):
//!
//!   strings.rs   — open string enums w/ `Other(SmolStr)` (codec×3, container,
//!                  subtitle::Format, audio open formats).
//!   coded.rs     — closed FFmpeg-coded enums w/ `from_u32` + colour / frame /
//!                  pixel-format / disposition structs and enums.
//!   composite.rs — audio composite metadata (Loudness/Fingerprint/CoverArt/Tags),
//!                  capture (Device/GeoLocation), lang::Language.
//!
//! These helpers do **not** route through `arbitrary::Unstructured` — they
//! consume `quickcheck::Gen` directly. The two `Arbitrary` features
//! (`arbitrary` and `quickcheck`) are independent: enable either one alone.

pub(crate) mod coded;
pub(crate) mod composite;
pub(crate) mod strings;

/// Picks a `bool` via `quickcheck::Gen` — short alias used in the cluster
/// helpers' 50/50 curated-slug-vs-`Other` branches.
#[inline]
#[allow(dead_code)] // referenced by helpers; lint trips on partial-feature builds
pub(crate) fn coin(g: &mut ::quickcheck::Gen) -> bool {
  <bool as ::quickcheck::Arbitrary>::arbitrary(g)
}

/// `String::arbitrary(g)` shorthand used by the helpers.
#[inline]
#[allow(dead_code)]
pub(crate) fn arb_string(g: &mut ::quickcheck::Gen) -> ::std::string::String {
  <::std::string::String as ::quickcheck::Arbitrary>::arbitrary(g)
}

#[cfg(test)]
mod tests {
  use ::quickcheck::{Arbitrary, Gen};

  /// Drives N rounds against a fresh `quickcheck::Gen` for a given `size`.
  fn drive<F: FnMut(&mut Gen)>(size: usize, rounds: usize, mut body: F) {
    let mut g = Gen::new(size);
    for _ in 0..rounds {
      body(&mut g);
    }
  }

  #[test]
  fn geo_location_invariant_lat_lon_in_range() {
    drive(64, 256, |g| {
      let geo = crate::capture::GeoLocation::arbitrary(g);
      assert!(
        (-90.0..=90.0).contains(&geo.lat()),
        "lat out of range: {}",
        geo.lat()
      );
      assert!(
        (-180.0..=180.0).contains(&geo.lon()),
        "lon out of range: {}",
        geo.lon()
      );
      if let Some(alt) = geo.altitude() {
        assert!(
          alt.is_finite(),
          "altitude must be finite when Some, got {alt}"
        );
      }
    });
  }

  #[test]
  fn fingerprint_invariant_algorithm_non_empty() {
    drive(64, 256, |g| {
      let fp = crate::audio::Fingerprint::arbitrary(g);
      assert!(!fp.algorithm().is_empty(), "algorithm must be non-empty");
    });
  }

  #[test]
  fn cover_art_invariant_mime_and_data_non_empty() {
    drive(64, 256, |g| {
      let c = crate::audio::CoverArt::arbitrary(g);
      assert!(!c.mime().is_empty(), "mime must be non-empty");
      assert!(!c.data().is_empty(), "data must be non-empty");
    });
  }

  #[test]
  fn smoke_yields_values_for_representative_types() {
    drive(64, 64, |g| {
      let _ = crate::codec::VideoCodec::arbitrary(g);
      let _ = crate::color::Info::arbitrary(g);
      let _ = crate::frame::FrameRate::arbitrary(g);
      let _ = crate::lang::Language::arbitrary(g);
      let _ = crate::disposition::TrackDisposition::arbitrary(g);
      let _ = crate::audio::Tags::arbitrary(g);
    });
  }

  #[test]
  fn reachability_small_coded_enums_hit_all_named() {
    use ::std::collections::HashSet;
    let mut br: HashSet<crate::audio::BitRateMode> = HashSet::new();
    let mut to: HashSet<crate::subtitle::TrackOrigin> = HashSet::new();
    drive(64, 2048, |g| {
      br.insert(crate::audio::BitRateMode::arbitrary(g));
      to.insert(crate::subtitle::TrackOrigin::arbitrary(g));
    });
    assert_eq!(br.len(), 3, "BitRateMode coverage: {br:?}");
    assert_eq!(to.len(), 3, "TrackOrigin coverage: {to:?}");
  }

  #[test]
  fn reachability_weighted_rotation_hits_named_and_unknown() {
    use crate::frame::Rotation;
    let mut saw_named = false;
    let mut saw_unknown = false;
    drive(64, 2048, |g| match Rotation::arbitrary(g) {
      Rotation::Unknown(_) => saw_unknown = true,
      _ => saw_named = true,
    });
    assert!(
      saw_named && saw_unknown,
      "Rotation missing arms: named={saw_named} unknown={saw_unknown}"
    );
  }

  // Every one of `SampleFormat`'s 12 named variants must be reachable —
  // plus the `Unknown(_)` and `Other(_)` escape arms. A weaker
  // "some named appears" check (Codex round-2 finding) would pass even
  // if half the slug list were missing.
  #[test]
  fn reachability_sample_format_all_named_plus_arms() {
    use crate::audio::SampleFormat;
    use ::std::collections::HashSet;
    let mut named: HashSet<::std::string::String> = HashSet::new();
    let mut saw_unknown = false;
    let mut saw_other = false;
    drive(64, 4096, |g| match SampleFormat::arbitrary(g) {
      SampleFormat::Unknown(_) => saw_unknown = true,
      SampleFormat::Other(_) => saw_other = true,
      other => {
        named.insert(other.as_str().to_string());
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

  // The range-weighted large coded enums must reach a broad set of named
  // codes — `arb_via_code!` (uniform `u32`) hit the named range for
  // `Matrix` / `Primaries` essentially never (Codex round-2 finding).
  #[test]
  fn reachability_range_weighted_enums_hit_named_codes() {
    use ::std::collections::HashSet;
    let mut matrix: HashSet<u32> = HashSet::new();
    let mut primaries: HashSet<u32> = HashSet::new();
    let mut transfer: HashSet<u32> = HashSet::new();
    let mut pixel: HashSet<u32> = HashSet::new();
    drive(64, 8192, |g| {
      matrix.insert(crate::color::Matrix::arbitrary(g).to_u32());
      primaries.insert(crate::color::Primaries::arbitrary(g).to_u32());
      transfer.insert(crate::color::Transfer::arbitrary(g).to_u32());
      pixel.insert(crate::pixel_format::PixelFormat::arbitrary(g).to_u32());
    });
    let in_range = |s: &HashSet<u32>, max: u32| s.iter().filter(|&&c| c <= max).count();
    assert!(
      in_range(&matrix, 17) >= 10,
      "Matrix named-range coverage too low: {matrix:?}"
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

  #[test]
  fn coded_enums_roundtrip_through_code() {
    drive(64, 128, |g| {
      let m = crate::color::Matrix::arbitrary(g);
      assert_eq!(crate::color::Matrix::from_u32(m.to_u32()), m);
      let p = crate::pixel_format::PixelFormat::arbitrary(g);
      assert_eq!(crate::pixel_format::PixelFormat::from_u32(p.to_u32()), p);
      let r = crate::frame::Rotation::arbitrary(g);
      assert_eq!(crate::frame::Rotation::from_u32(r.to_u32()), r);
      let d = crate::disposition::TrackDisposition::arbitrary(g);
      assert_eq!(
        crate::disposition::TrackDisposition::from_u32(d.to_u32()),
        d
      );
    });
  }
}
