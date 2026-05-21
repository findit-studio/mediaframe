//! Embedded audio cover art — a typed `(mime, data)` pair carried in
//! ID3v2 `APIC` frames / MP4 `covr` atoms / Vorbis `METADATA_BLOCK_PICTURE` /
//! FLAC `PICTURE` blocks.

use smol_str::SmolStr;

/// Embedded cover-art image for an audio stream.
///
/// `mime` is the IANA media-type string for the picture payload
/// (`"image/jpeg"`, `"image/png"`, …); `data` is the raw encoded
/// image bytes — opaque to this crate. Both must be non-empty (an
/// empty mime or empty payload is not a meaningful cover-art
/// attachment); use [`AudioCoverArt::try_new`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AudioCoverArt {
  mime: SmolStr,
  data: std::vec::Vec<u8>,
}

impl Default for AudioCoverArt {
  /// Synthetic `Default` — `mime: "application/octet-stream"`,
  /// `data: [0u8]`. The public constructor [`Self::try_new`] still
  /// rejects empty mime / empty data; the default here exists
  /// purely as a decoder seed for the `buffa` wire layer (which
  /// requires `DefaultInstance: Default`). Don't use this for real
  /// cover art — go through [`Self::try_new`].
  fn default() -> Self {
    Self {
      mime: SmolStr::new_static("application/octet-stream"),
      data: std::vec![0u8],
    }
  }
}

/// Error returned by [`AudioCoverArt::try_new`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum AudioCoverArtError {
  /// `mime` was empty — IANA media types are mandatory.
  #[error("audio cover-art mime type is empty")]
  EmptyMime,
  /// `data` was empty — a zero-byte image is not a meaningful
  /// attachment.
  #[error("audio cover-art data is empty")]
  EmptyData,
}

impl AudioCoverArt {
  /// Constructs an `AudioCoverArt` from a mime type and raw bytes.
  /// Rejects empty `mime` with [`AudioCoverArtError::EmptyMime`] and
  /// empty `data` with [`AudioCoverArtError::EmptyData`].
  pub fn try_new(
    mime: impl Into<SmolStr>,
    data: impl Into<std::vec::Vec<u8>>,
  ) -> Result<Self, AudioCoverArtError> {
    let mime = mime.into();
    if mime.is_empty() {
      return Err(AudioCoverArtError::EmptyMime);
    }
    let data = data.into();
    if data.is_empty() {
      return Err(AudioCoverArtError::EmptyData);
    }
    Ok(Self { mime, data })
  }

  /// Returns the IANA media type (`"image/jpeg"`, `"image/png"`,
  /// …).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn mime(&self) -> &str {
    self.mime.as_str()
  }

  /// Returns the raw encoded image bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn data(&self) -> &[u8] {
    self.data.as_slice()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::vec;

  #[test]
  fn try_new_happy_path() {
    let art = AudioCoverArt::try_new("image/jpeg", vec![0xFFu8, 0xD8, 0xFF]).unwrap();
    assert_eq!(art.mime(), "image/jpeg");
    assert_eq!(art.data(), &[0xFF, 0xD8, 0xFF]);
  }

  #[test]
  fn try_new_rejects_empty_mime() {
    let err = AudioCoverArt::try_new("", vec![1u8, 2, 3]).unwrap_err();
    assert_eq!(err, AudioCoverArtError::EmptyMime);
  }

  #[test]
  fn try_new_rejects_empty_data() {
    let err = AudioCoverArt::try_new("image/png", vec![]).unwrap_err();
    assert_eq!(err, AudioCoverArtError::EmptyData);
  }
}
