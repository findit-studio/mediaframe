//! `fn(g: &mut quickcheck::Gen) -> T` helpers — one per descriptor type —
//! referenced via container-level `#[quickcheck(arbitrary = "…")]` on each type's
//! `quickcheck_richderive::Arbitrary` derive.
//!
//! Split across three cluster files for parallel ownership (same axis as
//! [`arbitrary_impls`](crate::arbitrary_impls)):
//!
//!   strings.rs   — open string enums w/ `Other(Utf8Bytes)` (codec×3, container,
//!                  image, subtitle::Format, audio open formats).
//!   coded.rs     — closed FFmpeg-coded enums w/ `from_u32` + colour / frame /
//!                  pixel-format / disposition structs and enums.
//!   composite.rs — audio composite metadata (Loudness/Fingerprint/CoverArt/Tags),
//!                  capture (Device/GeoLocation), lang (Language/ScriptSubtag/
//!                  Region/LanguageId).
//!
//! These helpers do **not** route through `arbitrary::Unstructured` — they
//! consume `quickcheck::Gen` directly. The two `Arbitrary` features
//! (`arbitrary` and `quickcheck`) are independent: enable either one alone.

/// Emit one `pub(crate) fn $name(g) -> $ty` for an **open string enum** —
/// 50/50 a curated slug or an arbitrary string, both through `FromStr`.
///
/// `FromStr` is the canonicalising constructor: a named slug yields the
/// named variant, only a non-named slug yields `Other`. Routing the
/// arbitrary-string branch through it too (rather than `Other(Utf8Bytes)`
/// directly) guarantees a string equal to a named slug becomes that named
/// variant — never a malformed `Other("h264")` that serde would
/// canonicalise to `H264` on the round trip. An arbitrary string is
/// virtually never a named slug, so the `Other` arm stays well-covered.
macro_rules! qc_open_string_enum {
  ($name:ident, $ty:ty, [$($slug:literal),+ $(,)?]) => {
    pub(crate) fn $name(g: &mut ::quickcheck::Gen) -> $ty {
      const SAMPLES: &[&str] = &[$($slug),+];
      let s = if $crate::quickcheck_helpers::coin(g) {
        ::std::string::String::from(*g.choose(SAMPLES).unwrap())
      } else {
        $crate::quickcheck_helpers::arb_string(g)
      };
      <$ty as ::core::str::FromStr>::from_str(&s).unwrap()
    }
  };
}
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
mod tests;
