//! Audio channel **order** — how a layout's channels are arranged,
//! mirroring FFmpeg's `AVChannelOrder`.

use derive_more::{Display, IsVariant};

/// How the channels in a
/// [`ChannelLayoutDescription`](crate::audio::ChannelLayoutDescription)
/// are ordered.
///
/// Mirrors FFmpeg's `AVChannelOrder`. The stable wire integers are the
/// `repr(u32)` discriminants and match the [`Self::to_u32`] /
/// [`Self::from_u32`] mapping; [`Self::as_str`] / [`FromStr`] are the
/// text form.
///
/// **Closed** — deliberately *not* `#[non_exhaustive]`, and with no
/// `Other(Utf8Bytes)` escape. `AVChannelOrder` is itself a closed
/// taxonomy: every layout FFmpeg can describe is unspecified, native,
/// custom or ambisonic, so there is no vendor space for an escape arm to
/// preserve. A raw discriminant outside the four is a corrupt read, not
/// a value, and [`Self::from_u32`] decodes it to [`Self::Unspecified`];
/// [`Self::try_from_u32`] is the door that refuses it instead.
///
/// This is the *arrangement* axis. What the arrangement adds up to has
/// its own vocabulary — [`ChannelLayout`](crate::audio::ChannelLayout)
/// names layouts (`"5.1"`, `"quad"`, `"22.2"`), and the two travel
/// together on [`ChannelLayoutDescription`](crate::audio::ChannelLayoutDescription).
///
/// [`FromStr`]: core::str::FromStr
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::channel_order")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[repr(u32)]
pub enum ChannelOrder {
  /// Channel order is unknown / not communicated by the source
  /// (`"unspecified"`).
  #[default]
  Unspecified = 0,
  /// Native order (`"native"`): positions identified by a bitmask of
  /// well-known channel-position bits — FFmpeg's `AV_CH_*`, carried by
  /// [`ChannelLayoutDescription::native_mask`](crate::audio::ChannelLayoutDescription::native_mask).
  Native = 1,
  /// Custom order (`"custom"`): the channels are listed explicitly in
  /// [`ChannelLayoutDescription::custom_channels`](crate::audio::ChannelLayoutDescription::custom_channels).
  Custom = 2,
  /// Ambisonic order (`"ambisonic"`), optionally with an extra
  /// non-diegetic stereo pair (FFmpeg-style).
  Ambisonic = 3,
}

impl ChannelOrder {
  /// Canonical lowercase slug — the text form
  /// [`Display`](core::fmt::Display) prints and
  /// [`FromStr`](core::str::FromStr) reads back.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Unspecified => "unspecified",
      Self::Native => "native",
      Self::Custom => "custom",
      Self::Ambisonic => "ambisonic",
    }
  }

  /// Stable `u32` wire id — the `repr(u32)` discriminant. `0` always
  /// means [`Self::Unspecified`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    *self as u32
  }

  /// Decode from the wire id produced by [`Self::to_u32`]. Unrecognised
  /// values map to [`Self::Unspecified`] — the set is closed, so an
  /// unrecognised code is not preserved.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      1 => Self::Native,
      2 => Self::Custom,
      3 => Self::Ambisonic,
      _ => Self::Unspecified,
    }
  }

  /// Strict counterpart to [`Self::from_u32`]: returns [`None`] for any
  /// code outside the enumerated set instead of silently mapping it to
  /// the default. This is the read door of the **binary** serde leg —
  /// the one a format that is not `is_human_readable` takes — so
  /// adversarial / corrupt wire values fail loudly rather than
  /// masquerading as [`Self::Unspecified`]. A human-readable format
  /// carries the slug instead, and refuses an unrecognised name through
  /// [`FromStr`](core::str::FromStr).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_from_u32(v: u32) -> Option<Self> {
    match v {
      0 => Some(Self::Unspecified),
      1 => Some(Self::Native),
      2 => Some(Self::Custom),
      3 => Some(Self::Ambisonic),
      _ => None,
    }
  }
}

roster!(
  ChannelOrder,
  "channel order",
  [Unspecified, Native, Custom, Ambisonic]
);

/// The error [`ChannelOrder`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Its own type rather than a shared one: an input that names no channel
/// *order* and an input that names no channel *layout* are different
/// failures, and the type is what says which.
///
/// Opaque and sealed: the rejected input is deliberately not retained —
/// the input is attacker-controlled on any deserialization path, and
/// this vocabulary allocates nothing of its own. `#[non_exhaustive]`
/// keeps the error constructible here only, so it can grow structure
/// later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a channel-order name")]
#[non_exhaustive]
pub struct ParseChannelOrderError;

impl core::str::FromStr for ChannelOrder {
  type Err = ParseChannelOrderError;

  /// Parses the canonical slug [`Self::as_str`] renders — the exact
  /// inverse of [`Display`](core::fmt::Display), folding ASCII case and
  /// nothing else (`"native"`, `"Native"`, `"NATIVE"`).
  ///
  /// # Errors
  ///
  /// Returns [`ParseChannelOrderError`] for any input outside this
  /// closed vocabulary, the empty string included. Note that
  /// [`Self::from_u32`] absorbs an unrecognised *code* into
  /// [`Self::Unspecified`] while this door rejects an unrecognised
  /// *name*: a corrupt discriminant read out of FFmpeg memory has no
  /// spelling to fall back on, a misspelled configuration value does.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s.as_bytes());
    Ok(match folded {
      b"unspecified" => Self::Unspecified,
      b"native" => Self::Native,
      b"custom" => Self::Custom,
      b"ambisonic" => Self::Ambisonic,
      _ => return Err(ParseChannelOrderError),
    })
  }
}

#[cfg(test)]
mod tests;
