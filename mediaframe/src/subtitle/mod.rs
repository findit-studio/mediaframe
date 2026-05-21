//! Subtitle-stream descriptor vocabulary — format + track origin.
//!
//! Two orthogonal axes describing a subtitle track:
//!
//! - [`Format`] — the file / demuxer-tag form (`"srt"` /
//!   `"webvtt"` / `"ass"` / image-based `"hdmv_pgs_subtitle"` / …).
//!   Distinct from [`crate::codec::SubtitleCodec`]: the *format* is
//!   how the bytes are packaged on disk, the *codec* is how they
//!   are encoded.
//! - [`TrackOrigin`] — where the bytes came from
//!   ([`TrackOrigin::Embedded`] inside the container,
//!   [`TrackOrigin::Sidecar`] file next to it, or
//!   [`TrackOrigin::External`] download / OCR / ASR).
//!
//! Both types are pure media-stream descriptor vocabulary; they have
//! no per-cue / per-event content. The corresponding wire impls live
//! in [`crate::buffa`] behind the `buffa` feature.

pub mod format;
pub mod track_origin;

pub use format::Format;
pub use track_origin::TrackOrigin;
