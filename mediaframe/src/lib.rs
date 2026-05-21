#![doc = include_str!("../README.md")]
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

/// Hand-written [`arbitrary::Arbitrary`] impls for the descriptor vocabulary
/// (codecs, container/subtitle/audio formats, capture, language, colour, pixel
/// format, frame geometry/orientation, disposition). All generation goes through
/// the types' public constructors so private fields stay encapsulated and
/// `try_new` validated types come out valid by construction. Mirrors the
/// surface covered by [`serde`](serde_impls) — the same descriptor set the
/// storage / wire layers serialize.
#[cfg(feature = "arbitrary")]
mod arbitrary_impls;
/// Audio-stream descriptor vocabulary — channel layout, sample /
/// container format, bit-rate mode, EBU R128 loudness, fingerprint,
/// embedded metadata tags + cover art. Requires the `alloc` feature
/// (`std` includes it) for the `Other(SmolStr)` escape arms and the
/// `Vec<u8>` payloads.
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod audio;
#[cfg(feature = "buffa")]
mod buffa;
/// EXIF / capture-metadata vocabulary — capture device, geographic
/// location (with ISO-6709 parse/format). Requires the `alloc`
/// feature (`std` includes it) because the constituent types lean on
/// `SmolStr` / `std::string::String` for their text surface.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod capture;
/// Stream-descriptor codec/format/layout vocabulary for video, audio, and
/// subtitle tracks. Requires the `alloc` feature (`std` includes it) for
/// the `Other(SmolStr)` escape arms.
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod codec;
pub mod color;
/// Top-level multimedia container-format vocabulary. Requires the
/// `alloc` feature (`std` includes it) for the `Other(SmolStr)`
/// escape arm.
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod container;
/// FFmpeg `AV_DISPOSITION_*` bitflags shared across all track types
/// (video / audio / subtitle).
pub mod disposition;
pub mod frame;
/// Validated BCP-47 language tag wrapping `icu_locale_core` subtags
/// (`Copy`, heap-free representation; `to_bcp47() -> String` and
/// `Display` need the allocator).
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod lang;
pub mod pixel_format;
/// `fn(&mut quickcheck::Gen) -> T` helpers consumed by the per-type
/// `#[quickcheck(with = "…")]` attributes on each descriptor's
/// `quickcheck-richderive::Arbitrary` derive. The derive emits the actual
/// `impl quickcheck::Arbitrary for T` blocks; this module owns the bodies.
/// Same surface as [`arbitrary_impls`] (39 descriptor-vocabulary types) but
/// the two are independent — quickcheck does **not** bridge through arbitrary.
#[cfg(feature = "quickcheck")]
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
/// feature (`std` includes it) for the [`subtitle::Format`]'s
/// `Other(SmolStr)` escape arm.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod subtitle;

pub use source::{PixelSink, SourceFormat};
