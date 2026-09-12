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
/// **Closed** — deliberately *not* `#[non_exhaustive]`. The
/// trichotomy is the whole of the *reporting* domain and has been
/// stable for twenty-five years: a container states that its audio
/// bit rate is fixed, varies freely, or varies around a target, and
/// there is no fourth thing to state. The near misses are not
/// members: CVBR is a shape of [`Self::Vbr`], and CRF / CQP are
/// *encoder* knobs, describing how a file was produced rather than a
/// property the stream reports. A downstream `match` is therefore
/// exhaustive and the compiler proves it handles every mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::bit_rate_mode")
)]
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
  /// set is closed (`Other(Utf8Bytes)`-free); unrecognised codes are
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
  /// default. This is the read door of the **binary** serde leg — the one
  /// a format that is not `is_human_readable` takes — so adversarial /
  /// corrupt wire values fail loudly rather than masquerading as `Cbr`.
  /// A human-readable format carries the slug instead, and refuses an
  /// unrecognised name through [`FromStr`](core::str::FromStr).
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

/// The error [`BitRateMode`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed: the input is deliberately not retained (these types
/// are available at the crate's no-alloc tier, where there is nowhere to
/// put an owned copy, and the input is attacker-controlled on the
/// deserialization path). `#[non_exhaustive]` keeps it constructible only
/// here, so it can grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a bit-rate-mode name")]
#[non_exhaustive]
pub struct ParseBitRateModeError;

impl core::str::FromStr for BitRateMode {
  type Err = ParseBitRateModeError;

  /// Parses the canonical slug [`Self::as_str`] renders — the exact
  /// inverse of [`Display`](core::fmt::Display).
  ///
  /// # Errors
  ///
  /// Returns [`ParseBitRateModeError`] for any input
  /// outside this closed vocabulary.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s.as_bytes());
    Ok(match folded {
      b"cbr" => Self::Cbr,
      b"vbr" => Self::Vbr,
      b"abr" => Self::Abr,
      _ => return Err(ParseBitRateModeError),
    })
  }
}

#[cfg(test)]
mod tests;
