//! Audio bit-rate mode — the classic Constant / Variable / Average
//! Bit Rate trichotomy.

use derive_more::{Display, IsVariant};

/// Audio bit-rate mode — Constant / Variable / Average.
///
/// Read from container metadata (e.g. MKV `BitRateMode` /
/// FFmpeg-derived encoder hints). The default is [`Self::Cbr`] —
/// most legacy and broadcast pipelines emit constant-bit-rate audio
/// unless explicitly told otherwise; this mirrors the conservative
/// default downstream encoders pick when no mode tag is present.
///
/// `#[non_exhaustive]` keeps future additions non-breaking.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum BitRateMode {
  /// Constant bit rate (`"cbr"`).
  #[default]
  Cbr,
  /// Variable bit rate (`"vbr"`).
  Vbr,
  /// Average bit rate (`"abr"`).
  Abr,
}

impl BitRateMode {
  /// Lowercase canonical slug (`"cbr"`/`"vbr"`/`"abr"`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Cbr => "cbr",
      Self::Vbr => "vbr",
      Self::Abr => "abr",
    }
  }

  /// Stable `u32` wire id: `0`/`1`/`2` for `Cbr`/`Vbr`/`Abr`. Stable
  /// and append-only.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Cbr => 0,
      Self::Vbr => 1,
      Self::Abr => 2,
    }
  }

  /// Decode from the wire id produced by [`Self::to_u32`].
  /// Unrecognised values map to the [`Self::default`] (`Cbr`) — the
  /// set is closed (`Other(SmolStr)`-free); unrecognised codes are
  /// not preserved.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Cbr,
      1 => Self::Vbr,
      2 => Self::Abr,
      _ => Self::Cbr,
    }
  }

  /// Strict counterpart to [`Self::from_u32`]: returns `None` for any code
  /// outside the enumerated set, instead of silently mapping it to the
  /// default. Used by the strict deserialize path so adversarial / corrupt
  /// wire values fail loudly rather than masquerading as `Cbr`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_from_u32(v: u32) -> Option<Self> {
    match v {
      0 => Some(Self::Cbr),
      1 => Some(Self::Vbr),
      2 => Some(Self::Abr),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::std::string::ToString;

  #[test]
  fn default_is_cbr() {
    assert_eq!(BitRateMode::default(), BitRateMode::Cbr);
  }

  #[test]
  fn as_str_round_trips() {
    assert_eq!(BitRateMode::Cbr.as_str(), "cbr");
    assert_eq!(BitRateMode::Vbr.as_str(), "vbr");
    assert_eq!(BitRateMode::Abr.as_str(), "abr");
  }

  #[test]
  fn u32_round_trip_named_variants() {
    for v in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
      assert_eq!(BitRateMode::from_u32(v.to_u32()), v);
    }
  }

  #[test]
  fn display_matches_as_str() {
    assert_eq!(BitRateMode::Cbr.to_string(), "cbr");
    assert_eq!(BitRateMode::Vbr.to_string(), "vbr");
    assert_eq!(BitRateMode::Abr.to_string(), "abr");
  }

  #[test]
  fn is_variant_predicates() {
    assert!(BitRateMode::Cbr.is_cbr());
    assert!(BitRateMode::Vbr.is_vbr());
    assert!(BitRateMode::Abr.is_abr());
  }
}
