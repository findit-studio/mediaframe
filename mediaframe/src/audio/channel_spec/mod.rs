//! One channel's entry in a custom-ordered layout — index, backend raw
//! id, and an optional label.

use smol_bytes::Utf8Bytes;

/// One entry in a
/// [`ChannelLayoutDescription::custom_channels`](crate::audio::ChannelLayoutDescription::custom_channels)
/// list — the per-channel description for a
/// [`ChannelOrder::Custom`](crate::audio::ChannelOrder::Custom) layout.
///
/// Three fields, no relation between them: a position in the layout, the
/// backend's own id for the channel that sits there (FFmpeg's `AVChannel`
/// integer, say), and the human-readable label the backend prints for it
/// (`"FL"`, `"LFE"`) — empty when there is none.
///
/// With the `serde` feature the wire form is a map of the three
/// accessors' names — `{"index": 0, "raw_id": 1, "label": "FL"}`. The
/// record carries no invariant (every field has a public unchecked
/// setter), so the derive is the whole story: there is nothing a
/// hand-written `Deserialize` would have to re-check. `serde(default)`
/// keeps sparse / older-schema documents readable — a missing field
/// falls back to the all-zero [`Default`].
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(default)
)]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::channel_spec")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelSpec {
  index: u32,
  raw_id: u32,
  label: Utf8Bytes,
}

impl Default for ChannelSpec {
  /// Delegates to [`ChannelSpec::new`] — channel `0`, raw id `0`, no
  /// label.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new(0, 0)
  }
}

impl ChannelSpec {
  /// Constructs a `ChannelSpec` with the given channel index and
  /// backend-specific raw id. The label starts empty; fill it in with
  /// [`Self::with_label`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(index: u32, raw_id: u32) -> Self {
    Self {
      index,
      raw_id,
      label: Utf8Bytes::new(),
    }
  }

  /// Index of this channel in the layout (0-based).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn index(&self) -> u32 {
    self.index
  }

  /// Backend-specific channel id (e.g. FFmpeg's `AVChannel` integer).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn raw_id(&self) -> u32 {
    self.raw_id
  }

  /// Human-readable label, or the empty string when unspecified.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn label(&self) -> &str {
    self.label.as_str()
  }

  /// Sets the channel index — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_index(mut self, v: u32) -> Self {
    self.index = v;
    self
  }

  /// Sets the raw id — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_raw_id(mut self, v: u32) -> Self {
    self.raw_id = v;
    self
  }

  /// Sets the label — consuming builder.
  ///
  /// Takes anything that converts into the [`Utf8Bytes`] seat: a `&str`, a
  /// `String`, or a `Utf8Bytes` the caller already holds. The `String`
  /// conversion takes ownership of the caller's buffer instead of copying it.
  ///
  /// **There is no fallible twin, and none is needed.** A caller that
  /// must not abort on a hostile or oversized label does the fallible
  /// part on its own side — reserve (`Vec::try_reserve`), decode, build
  /// the `String` — and hands the finished text here; this seat only
  /// stores what it is given. That is why a backend no longer has to
  /// drop a label it could not fit: the length limit that forced the
  /// choice is gone with the old carrier.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_label(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.label = v.into();
    self
  }

  /// Sets the channel index in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_index(&mut self, v: u32) -> &mut Self {
    self.index = v;
    self
  }

  /// Sets the raw id in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_raw_id(&mut self, v: u32) -> &mut Self {
    self.raw_id = v;
    self
  }

  /// Sets the label in place — same seat and same conversions as
  /// [`Self::with_label`], including its note on fallibility.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_label(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.label = v.into();
    self
  }
}

#[cfg(test)]
mod tests;
