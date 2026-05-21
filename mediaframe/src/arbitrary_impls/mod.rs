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
