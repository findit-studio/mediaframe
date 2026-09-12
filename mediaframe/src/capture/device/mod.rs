//! EXIF capture-device metadata — make/model of the camera (or other
//! capture device) that produced a media file.
//!
//! Mirrors the long-standing `MediaMeta.device_make` / `device_model`
//! free-form `Utf8Bytes` fields on findit-proto: a pair of small inline
//! strings holding e.g. `"Apple"` / `"iPhone 15 Pro"` or `"Sony"` /
//! `"ILCE-7M4"`. Empty string means absent (never `Option<Utf8Bytes>`)
//! per the mediaframe convention shared with the codec module.

use smol_bytes::Utf8Bytes;

/// EXIF-style capture device descriptor — manufacturer + model.
///
/// Sourced from EXIF tags `Make` (`0x010f`) / `Model` (`0x0110`) on
/// still images and from `com.apple.quicktime.make` /
/// `com.apple.quicktime.model` (and equivalent vendor) atoms on
/// MOV/MP4 video, as well as findit-proto's `MediaMeta.device_*`
/// `Utf8Bytes` fields.
///
/// Both fields are private `Utf8Bytes`s — the empty string is the
/// sentinel for "absent" so callers never need `Option<Utf8Bytes>`
/// (matches the codec / source-tagging convention elsewhere in this
/// crate). Use [`Self::is_empty`] to detect the fully-absent state.
// `serde(default)` keeps sparse / older-schema JSON deserializable: missing
// fields fall back to the type-level `Default` impl (`Device::new()` — both
// `make` and `model` empty), matching the empty-string-means-absent convention.
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(default)
)]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::capture_device")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Device {
  make: Utf8Bytes,
  model: Utf8Bytes,
}

impl Default for Device {
  /// Delegates to [`Device::new`] — both `make` and `model` empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl Device {
  /// Constructs an all-empty `Device` (both `make` and `model` set
  /// to the empty string).
  ///
  /// The empty string is the sentinel for "absent"; callers use
  /// [`Self::with_make`] / [`Self::with_model`] (or the setters) to
  /// populate either side.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      make: Utf8Bytes::new(),
      model: Utf8Bytes::new(),
    }
  }

  /// Returns the manufacturer (camera "make"), e.g. `"Apple"` /
  /// `"Sony"`. An empty string means absent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn make(&self) -> &str {
    self.make.as_str()
  }

  /// Returns the camera model, e.g. `"iPhone 15 Pro"` / `"ILCE-7M4"`.
  /// An empty string means absent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn model(&self) -> &str {
    self.model.as_str()
  }

  /// Sets the manufacturer (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_make(mut self, make: impl Into<Utf8Bytes>) -> Self {
    self.make = make.into();
    self
  }

  /// Sets the manufacturer in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_make(&mut self, make: impl Into<Utf8Bytes>) -> &mut Self {
    self.make = make.into();
    self
  }

  /// Sets the camera model (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_model(mut self, model: impl Into<Utf8Bytes>) -> Self {
    self.model = model.into();
    self
  }

  /// Sets the camera model in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_model(&mut self, model: impl Into<Utf8Bytes>) -> &mut Self {
    self.model = model.into();
    self
  }

  /// Returns `true` when both `make` and `model` are empty — i.e. no
  /// capture-device metadata is recorded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_empty(&self) -> bool {
    self.make.is_empty() && self.model.is_empty()
  }
}

#[cfg(test)]
mod tests;
