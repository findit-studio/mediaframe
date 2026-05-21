//! Audio fingerprint — algorithm-tagged raw bytes.

use bytes::Bytes;
use smol_str::SmolStr;

/// Audio fingerprint value object — a free-text algorithm label
/// (`"chromaprint"`, `"acoustid"`, `"audiocrc32"`, …) plus the raw
/// fingerprint bytes the named algorithm produces.
///
/// The byte layout is opaque to this crate; the `algorithm` label is
/// the routing key that lets a downstream consumer interpret the
/// bytes correctly. The payload is held as [`bytes::Bytes`] (O(1)
/// clone). Empty `algorithm` is rejected because an unlabelled
/// fingerprint cannot be routed; empty `value` is **allowed** (some
/// algorithms emit an empty fingerprint for silence / sub-second
/// clips).
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(try_from = "FingerprintShadow")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint {
  algorithm: SmolStr,
  value: Bytes,
}

/// Deserialization shadow for [`Fingerprint`] — routes through
/// [`Fingerprint::try_new`] so the non-empty-`algorithm` invariant is
/// enforced on the way in, instead of being bypassed by a field-derived
/// `Deserialize`.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct FingerprintShadow {
  algorithm: SmolStr,
  value: Bytes,
}

#[cfg(feature = "serde")]
impl core::convert::TryFrom<FingerprintShadow> for Fingerprint {
  type Error = FingerprintError;
  fn try_from(s: FingerprintShadow) -> Result<Self, Self::Error> {
    Self::try_new(s.algorithm, s.value)
  }
}

impl Default for Fingerprint {
  /// Synthetic `Default` — `algorithm: "default"`, `value: []`. The
  /// public constructor [`Self::try_new`] still rejects empty
  /// algorithm; the default value here exists purely as a decoder
  /// seed for the `buffa` wire layer (which requires
  /// `DefaultInstance: Default`). Don't use this for real
  /// fingerprints — go through [`Self::try_new`].
  fn default() -> Self {
    Self {
      algorithm: SmolStr::new_inline("default"),
      value: Bytes::new(),
    }
  }
}

/// Error returned by [`Fingerprint::try_new`] when the inputs
/// are structurally invalid (empty `algorithm`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum FingerprintError {
  /// The `algorithm` label was empty — a fingerprint without an
  /// algorithm tag cannot be routed.
  #[error("audio fingerprint algorithm label is empty")]
  EmptyAlgorithm,
}

impl Fingerprint {
  /// Constructs an `Fingerprint` from an algorithm label and
  /// raw bytes. Rejects an empty `algorithm` with
  /// [`FingerprintError::EmptyAlgorithm`]. Empty `value` is
  /// allowed (some algorithms emit no bytes for silence / very
  /// short clips).
  pub fn try_new(
    algorithm: impl Into<SmolStr>,
    value: impl Into<Bytes>,
  ) -> Result<Self, FingerprintError> {
    let algorithm = algorithm.into();
    if algorithm.is_empty() {
      return Err(FingerprintError::EmptyAlgorithm);
    }
    Ok(Self {
      algorithm,
      value: value.into(),
    })
  }

  /// Returns the algorithm label (e.g. `"chromaprint"`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn algorithm(&self) -> &str {
    self.algorithm.as_str()
  }

  /// Returns the raw fingerprint bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn value(&self) -> &[u8] {
    self.value.as_ref()
  }

  /// Returns the fingerprint payload as a cheaply-cloneable
  /// [`bytes::Bytes`] handle (O(1) refcount bump, no copy).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn value_bytes(&self) -> Bytes {
    self.value.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::vec;

  #[test]
  fn try_new_happy_path() {
    let fp = Fingerprint::try_new("chromaprint", vec![1u8, 2, 3, 4]).unwrap();
    assert_eq!(fp.algorithm(), "chromaprint");
    assert_eq!(fp.value(), &[1, 2, 3, 4]);
  }

  #[test]
  fn try_new_rejects_empty_algorithm() {
    let err = Fingerprint::try_new("", vec![1u8]).unwrap_err();
    assert_eq!(err, FingerprintError::EmptyAlgorithm);
  }

  #[test]
  fn try_new_accepts_empty_value() {
    let fp = Fingerprint::try_new("acoustid", vec![]).unwrap();
    assert_eq!(fp.algorithm(), "acoustid");
    assert!(fp.value().is_empty());
  }
}
