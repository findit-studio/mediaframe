//! Audio-stream descriptor vocabulary — channel layout, sample /
//! container format, bit-rate mode, EBU R128 loudness, fingerprint,
//! embedded metadata tags + cover art.

pub mod bit_rate_mode;
pub mod channel_layout;
pub mod cover_art;
pub mod fingerprint;
pub mod format;
pub mod loudness;
pub mod tags;

pub use bit_rate_mode::BitRateMode;
pub use channel_layout::ChannelLayout;
pub use cover_art::{AudioCoverArt, AudioCoverArtError};
pub use fingerprint::{AudioFingerprint, AudioFingerprintError};
pub use format::{AudioContainerFormat, AudioFormat};
pub use loudness::Loudness;
pub use tags::AudioTags;
