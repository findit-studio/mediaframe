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
