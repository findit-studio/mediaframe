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
