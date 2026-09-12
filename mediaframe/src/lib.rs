#![doc = include_str!("../README.md")]
//!
//! # Feature tiers
//!
//! The crate builds at three tiers, and the tier decides how *open*
//! its vocabularies are.
//!
//! | Tier | Features | Vocabularies |
//! |---|---|---|
//! | no-alloc | (none) | **closed** — an unrecognised slug is rejected |
//! | alloc | `alloc` | open — an unrecognised slug rides `Other(Utf8Bytes)` |
//! | std | `std` (implies `alloc`) | as `alloc`, plus `std::error::Error` |
//!
//! `Other(Utf8Bytes)` needs a heap, so it exists only at the `alloc` /
//! `std` tier. At the no-alloc tier the same enums are closed and their
//! [`FromStr`](core::str::FromStr) returns the vocabulary's own error
//! instead: **an error beats a wrong value**, and collapsing an unknown
//! name onto a named variant would be a wrong value. The *wire shape* is
//! the same at every tier (a slug either way) — only the openness
//! differs.
//!
//! Since 0.5.0 the **error type follows the tier too**. Where the escape
//! arm exists the parse cannot fail, so `FromStr::Err` is
//! [`Infallible`](core::convert::Infallible) and a caller can discharge it
//! with an irrefutable `let Ok(x) = s.parse::<T>();`. The vocabulary's own
//! `Parse*Error` stays exported and is still what the no-alloc tier
//! returns. This applies to the vocabularies compiled at *every* tier —
//! the colour enums, [`PixelFormat`](pixel_format::PixelFormat), and the
//! frame orientation enums. Vocabularies that only exist at the `alloc`
//! tier have had `Err = Infallible` all along.
//!
//! Every gate on an alloc-tier item is spelled
//! `any(feature = "std", feature = "alloc")` rather than bare
//! `feature = "alloc"`, so the item cannot evaporate for a dependant
//! that turns on `std` alone.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

// Alias `alloc as std` on no_std + alloc builds so code can use
// `std::vec::Vec` etc. uniformly across feature combos. When the
// `std` feature is on, the real `std` crate is already in scope via
// the prelude. The `unused_extern_crates` allow silences a
// rust-2018-idioms false positive — the alias is needed at use-time
// even though rustc can't see that statically.
#[cfg(all(not(feature = "std"), feature = "alloc"))]
#[allow(unused_extern_crates)]
extern crate alloc as std;

#[cfg(feature = "std")]
#[allow(unused_extern_crates)]
extern crate std;

/// The doc text every `*Row::for_tests` carries, written once.
///
/// Declared here, ahead of `mod frame` and `mod source`, because both
/// hold row types and `macro_rules!` is textually scoped. One text in
/// one place — the same rule the row door itself enforces on colour
/// intent.
///
/// Gated on exactly the features that own a row door, because the text
/// is reachable only from one: the `walker!` arms under [`source`],
/// plus `source::pal8` (`mono`), `source::xyz12` (`xyz`) and
/// `frame::bayer` (`bayer`). Without the gate the lean build defines a
/// macro nothing expands, which `unused_macros` rejects.
///
/// All fifteen format features are on the list, which is a fact about
/// today's doors and not a synonym for "every format": it is written
/// out feature by feature so a format that loses its last door leaves
/// the list and the gate stays honest. The umbrella `frame` feature
/// cannot stand in for the disjunction either — it implies the format
/// features, not the reverse, so `--features rgb` alone would leave the
/// macro undefined at an expansion site.
#[cfg(any(
  feature = "yuv-planar",
  feature = "yuv-semi-planar",
  feature = "yuva",
  feature = "yuv-packed",
  feature = "yuv-444-packed",
  feature = "y2xx",
  feature = "v210",
  feature = "rgb",
  feature = "rgb-float",
  feature = "rgb-legacy",
  feature = "gbr",
  feature = "gray",
  feature = "bayer",
  feature = "xyz",
  feature = "mono",
))]
macro_rules! row_test_door_doc {
  () => {
    " Builds a row directly — **kernel-parity test scaffolding, not API.**\n\
      \n\
      [`Self::new`] is `pub(crate)`. Production rows come from the walkers,\n\
      which read the colour selector off the sink, so on the public surface\n\
      a row cannot be conjured beside the description that chose it. This\n\
      door is the one named exception to that.\n\
      \n\
      It exists because a kernel-parity suite drives a single row kernel\n\
      without materialising a frame, and there is no other way to reach one\n\
      from outside this crate. Not hypothetical: a census on 2026-08-19\n\
      found **493 such constructions across 85 files and 52 row types** in\n\
      `pixon` alone, every one of them test code.\n\
      \n\
      Takes exactly what `new` takes, colour selector included, so the two\n\
      cannot drift. **No stability promise** — it is `#[doc(hidden)]` and\n\
      may change or vanish in any release. If you are not testing a row\n\
      kernel, walk a frame instead."
  };
}

/// Declares a vocabulary's `ROSTER` **and** the compile-time witness that
/// keeps it complete, from one list of variant names.
///
/// The list is written once. From it the macro emits:
///
/// 1. `pub const ROSTER: &'static [Self]` — the named variants in
///    declaration order, as a **slice** so the count stays out of the
///    type and growing the vocabulary remains a minor change; and
/// 2. an exhaustive `match` beside the type. `#[non_exhaustive]` does not
///    bind the defining crate, so that `match` really is exhaustive here:
///    a new variant makes it `E0004` and the compiler names the variant
///    that was added. Updating this one list fixes both artefacts at
///    once, which is why they are not two lists that can drift.
///
/// The escape arm is spelled at the call site rather than assumed,
/// because the two families gate it differently: vocabularies living in
/// `alloc`-only modules carry `Other` unconditionally (`escape:`), while
/// those compiled at every tier carry it behind
/// `any(feature = "std", feature = "alloc")` (`alloc_escape:`). The
/// escape is deliberately **not** a roster member — the roster is the set
/// of names this build knows, and the escape is the arm holding a name it
/// does not.
///
/// Declared here, ahead of the vocabulary modules, because
/// `macro_rules!` is textually scoped.
macro_rules! roster {
  ($ty:ident, $noun:literal, [$($variant:ident),+ $(,)?] $(,)?) => {
    roster!(@emit $ty, $noun, [$($variant),+], {});
  };
  ($ty:ident, $noun:literal, [$($variant:ident),+ $(,)?], escape: $escape:ident $(,)?) => {
    roster!(@emit $ty, $noun, [$($variant),+], { $ty::$escape(_) => (), });
  };
  ($ty:ident, $noun:literal, [$($variant:ident),+ $(,)?], alloc_escape: $escape:ident $(,)?) => {
    roster!(@emit $ty, $noun, [$($variant),+], {
      #[cfg(any(feature = "std", feature = "alloc"))]
      $ty::$escape(_) => (),
    });
  };
  (@emit $ty:ident, $noun:literal, [$($variant:ident),+], { $($escape_arm:tt)* }) => {
    impl $ty {
      #[doc = concat!(" Every ", $noun, " this vocabulary names, in declaration order.")]
      ///
      /// A slice rather than an array: how many names this build carries
      /// is a fact about the release, not part of the type, so a later
      /// addition stays a minor change.
      ///
      /// The open escape arm is not a member. The roster answers "which
      /// names does this build know", and the escape is precisely the arm
      /// that carries a name it does not — listing it would need a slug
      /// to put in it, and there isn't one.
      pub const ROSTER: &'static [Self] = &[$(Self::$variant),+];
    }

    // The witness that `ROSTER` above is complete: adding a variant makes
    // this `match` non-exhaustive, and the compiler names the variant
    // that is missing from the roster.
    const _: () = {
      #[allow(dead_code)]
      fn every_variant_is_rostered(v: &$ty) {
        match v {
          $($ty::$variant => (),)+
          $($escape_arm)*
        }
      }
    };
  };
}

/// Hand-written [`arbitrary::Arbitrary`] impls for the descriptor vocabulary
/// (codecs, container/subtitle/audio formats, capture, language, colour, pixel
/// format, frame geometry/orientation, disposition). All generation goes through
/// the types' public constructors so private fields stay encapsulated and
/// `try_new` validated types come out valid by construction. Mirrors the
/// surface covered by [`serde`](serde_impls) — the same descriptor set the
/// storage / wire layers serialize.
#[cfg(feature = "arbitrary")]
mod arbitrary_impls;
/// Audio-stream descriptor vocabulary — channel layout (the name) and
/// channel layout description (the structure: order, mask, per-channel
/// list), sample / container format, bit-rate mode, EBU R128 loudness,
/// fingerprint, embedded metadata tags + cover art. Requires the `alloc`
/// feature (`std` includes it) for the `Other(Utf8Bytes)` escape arms and
/// the `Vec<u8>` payloads.
///
/// **Derive threshold.** Every open enum here carries `Unwrap` /
/// `TryUnwrap` for its `Other(Utf8Bytes)` arm. The pair generates three
/// methods per variant, so an enum in the hundreds pays that in compile
/// time for one reachable payload arm; the two 200-plus-variant codec
/// enums in [`codec`] are the crate's only exemptions. The line is
/// variant count, not principle.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod audio;
#[cfg(feature = "buffa")]
mod buffa;
/// EXIF / capture-metadata vocabulary — capture device, geographic
/// location (with ISO-6709 parse/format). Requires the `alloc`
/// feature (`std` includes it) because the constituent types lean on
/// `Utf8Bytes` / `std::string::String` for their text surface.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod capture;
/// Stream-descriptor codec/format/layout vocabulary for video, audio, and
/// subtitle tracks. Requires the `alloc` feature (`std` includes it) for
/// the `Other(Utf8Bytes)` escape arms.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod codec;
pub mod color;
/// Top-level multimedia container-format vocabulary. Requires the
/// `alloc` feature (`std` includes it) for the `Other(Utf8Bytes)`
/// escape arm.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod container;
/// FFmpeg `AV_DISPOSITION_*` bitflags shared across all track types
/// (video / audio / subtitle).
pub mod disposition;
pub mod frame;
/// Still-image vocabulary — standard photo formats (`jpeg`, `png`, `heif`,
/// …) plus a curated camera-RAW family (`dng`, `cr2`, `nef`, `arw`, …).
/// Requires the `alloc` feature (`std` includes it) for the
/// `Other(Utf8Bytes)` escape arm — same tier as [`container`] and [`audio`],
/// which this household's own module doc explains it exists to sit
/// beside.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod image;
/// BCP 47 language identity — the three validated subtag types
/// ([`lang::Language`], [`lang::ScriptSubtag`], [`lang::Region`]) and the
/// lossless whole tag that composes them ([`lang::LanguageId`]), read
/// against the two registries vendored under `xtask/vendor/` and generated
/// into [`lang::registry`].
///
/// Every fold a container's dirty tag needs is a column of a published
/// file rather than a rule written here: an mkv's `ger` and an mp4's `deu`
/// are both German, `iw` is `he`, `BU` is `MM`, and `en-Latn` composes as
/// `en` while `zh-Hans` composes as itself. Requires the `alloc` feature
/// (`std` includes it) for the `Utf8Bytes` text seat each subtag holds.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod lang;
/// The shared runtime checks for the `ROSTER` constants the `roster!`
/// macro emits — duplicate entries and slug collisions, the two faults a
/// compile-time completeness witness cannot see.
#[cfg(test)]
mod roster_tests;
// The case-sensitivity axis and the ASCII case-folding gate every
// `FromStr` in the crate declares and passes through, respectively.
// Private: the errors those parses return live with their vocabularies,
// one per type.
mod parse;
pub mod pixel_format;
/// `fn(&mut quickcheck::Gen) -> T` helpers consumed by the per-type
/// `#[quickcheck(arbitrary = "…")]` attributes on each descriptor's
/// `quickcheck-richderive::Arbitrary` derive. The derive emits the actual
/// `impl quickcheck::Arbitrary for T` blocks; this module owns the bodies.
/// Same surface as [`arbitrary_impls`] (44 descriptor-vocabulary types) but
/// the two are independent — quickcheck does **not** bridge through arbitrary.
#[cfg(feature = "quickcheck")]
#[cfg_attr(docsrs, doc(cfg(feature = "quickcheck")))]
pub mod quickcheck_helpers;
/// Centralised `serde` impls for the descriptor enums (the structs derive
/// serde at their definition sites). Open codec/format enums serialize as
/// their `as_str()` slug; closed FFmpeg-coded enums as their `to_u32()`
/// code — mirroring the storage backends.
#[cfg(feature = "serde")]
mod serde_impls;
pub mod source;
/// Subtitle-stream descriptor vocabulary — file / demuxer format
/// ([`subtitle::Format`]) and track-origin axis
/// ([`subtitle::TrackOrigin`]). Requires the `alloc`
/// feature (`std` includes it) for both types' `Other(Utf8Bytes)`
/// escape arms.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod subtitle;

pub use source::{PixelSink, SourceFormat};
