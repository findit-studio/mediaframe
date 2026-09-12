// Centralised `arbitrary::Arbitrary` impls for the descriptor vocabulary.
//
// Hand-written (no `#[derive]` on the type definitions) so private fields stay
// encapsulated and `try_new` validated types come out valid by construction.
// Split across three cluster files for parallel ownership; everything is
// re-exported via the module's `impl` items so cross-cluster cascades just
// resolve naturally.
//
//   strings.rs   — open string enums w/ `Other(Utf8Bytes)` (codec×3, container,
//                  image, subtitle::Format, audio open formats).
//   coded.rs     — the FFmpeg-coded name vocabularies + colour / frame /
//                  pixel-format / disposition structs and enums.
//   composite.rs — audio composite metadata (Loudness/Fingerprint/CoverArt/Tags),
//                  capture (Device/GeoLocation), lang (Language/ScriptSubtag/
//                  Region/LanguageId).

mod coded;
mod composite;
mod strings;

/// `impl arbitrary::Arbitrary for $Ty` that decodes the raw `u32` through the
/// type's `from_u32`. Only the bit set uses it: every `u32` is a meaningful
/// value there, so a uniform draw covers the space.
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
/// those with no escape arm at all. Picks uniformly from the listed named
/// variants via `Unstructured::choose`.
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

/// `impl arbitrary::Arbitrary for $Ty` for open string enums: 50/50 picks a
/// curated slug or an arbitrary string — **both routed through `FromStr`**.
///
/// `FromStr` is the canonicalising constructor: a named slug yields the
/// named variant, only a non-named slug yields `Other`. Going through it
/// (rather than `Other(Utf8Bytes::from(s))` directly) guarantees every
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
mod tests;
