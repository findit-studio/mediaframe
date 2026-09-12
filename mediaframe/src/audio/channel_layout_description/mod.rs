//! The full structural description of an audio channel layout — order,
//! channel count, mask, per-channel list, and the backend's rendering.
//!
//! **Three layers, and this is the third.**
//!
//! 1. [`ChannelOrder`] — how the channels are arranged (Native bitmask /
//!    Custom per-channel list / Ambisonic / Unspecified), matching
//!    FFmpeg's `AVChannelOrder` taxonomy.
//! 2. [`ChannelSpec`] — for a custom-order layout, one entry per
//!    channel: an index, a backend-specific raw id, and an optional
//!    label.
//! 3. [`ChannelLayoutDescription`] — the bundle: order + channel count +
//!    the layout's name + native bitmask (when applicable) + custom
//!    channel list (when applicable) + the backend's own free-form
//!    rendering.
//!
//! **The name and the description are different things.**
//! [`ChannelLayout`] is the *named* vocabulary — a closed-ish roster of
//! the layouts FFmpeg's `channel_layout_map[]` spells (`"5.1"`,
//! `"quad(side)"`, `"22.2"`), with an `Other(Utf8Bytes)` escape for a name
//! it does not carry. It answers *which layout is this*. This household
//! answers *what is this layout made of*, and holds the name as one
//! field among six ([`known_kind`](ChannelLayoutDescription::known_kind)).
//!
//! Neither subsumes the other. A layout with a custom channel ordering
//! has no name at all — [`ChannelLayout::default`] is the "absent"
//! sentinel that field then carries — while its channel list is exactly
//! what a consumer needs. A stream tagged `"5.1"` and nothing else has a
//! name and no mask. Keeping them apart is what lets a value be honest
//! about which of the two it actually knows.

use smol_bytes::Utf8Bytes;
use std::vec::Vec;

use crate::audio::{ChannelLayout, ChannelOrder, ChannelSpec};

/// Audio channel layout, described in full — order + channel count +
/// identification.
///
/// The bundle FFmpeg's `AVChannelLayout` carries through to consumers,
/// rendered as plain Rust data:
///
/// - [`order`](Self::order) — Native / Custom / Ambisonic / Unspecified.
/// - [`channels`](Self::channels) — total count.
/// - [`known_kind`](Self::known_kind) — the layout's *name*, as a
///   [`ChannelLayout`]. [`ChannelLayout::default`] (the `Other("")`
///   absent sentinel) when no well-known shape matches.
/// - [`native_mask`](Self::native_mask) — `Some(bitmask)` for
///   [`ChannelOrder::Native`] / [`ChannelOrder::Ambisonic`], `None`
///   otherwise.
/// - [`custom_channels`](Self::custom_channels) — populated for
///   [`ChannelOrder::Custom`] layouts; one [`ChannelSpec`] per channel.
/// - [`text`](Self::text) — the backend's own free-form rendering, e.g.
///   FFmpeg's `av_channel_layout_describe` output (`"5.1(side)"`,
///   `"3 channels (FL+FR+LFE)"`).
///
/// With the `serde` feature the wire form is a map of those six names,
/// each field in its own shape: `order` as its `u32` code, `known_kind`
/// as its canonical slug, `native_mask` as a nullable integer,
/// `custom_channels` as an array of [`ChannelSpec`] maps.
///
/// **No invariant across the fields.** An incoherent combination — a
/// `Custom` order with an empty channel list, a `Native` order with no
/// mask — is exactly as constructible through the public setters as a
/// coherent one, so the derive rejects nothing the builders would have
/// accepted, and the fuzz generators deliberately reach those
/// combinations. A consumer that needs coherence checks it.
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(default)
)]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::channel_layout_description")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelLayoutDescription {
  order: ChannelOrder,
  channels: u32,
  known_kind: ChannelLayout,
  native_mask: Option<u64>,
  custom_channels: Vec<ChannelSpec>,
  text: Utf8Bytes,
}

impl Default for ChannelLayoutDescription {
  /// Delegates to [`ChannelLayoutDescription::new`] with zero channels —
  /// the "uninitialized" sentinel [`Self::is_empty`] recognises.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new(0)
  }
}

impl ChannelLayoutDescription {
  /// Constructs a minimal description with the given channel count.
  /// Every other field starts at its own absent value (`Unspecified`
  /// order, no name, no mask, no channel list, no rendering); fill them
  /// in with the `with_*` builders.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(channels: u32) -> Self {
    Self {
      order: ChannelOrder::Unspecified,
      channels,
      known_kind: ChannelLayout::Other(Utf8Bytes::new()),
      native_mask: None,
      custom_channels: Vec::new(),
      text: Utf8Bytes::new(),
    }
  }

  /// Channel ordering (Native / Custom / Ambisonic / Unspecified).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn order(&self) -> ChannelOrder {
    self.order
  }

  /// Total channel count.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn channels(&self) -> u32 {
    self.channels
  }

  /// The layout's name, or [`ChannelLayout::default`] — the `Other("")`
  /// absent sentinel — when no well-known shape matches.
  ///
  /// Borrowed rather than copied: [`ChannelLayout`] carries a `Utf8Bytes`
  /// in its escape arm and so is not `Copy`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn known_kind(&self) -> &ChannelLayout {
    &self.known_kind
  }

  /// Native-order bitmask of `AV_CH_*` channel positions, when
  /// applicable. `None` for Custom / Unspecified orders, or when the
  /// mask is zero.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn native_mask(&self) -> Option<u64> {
    self.native_mask
  }

  /// Per-channel descriptors for [`ChannelOrder::Custom`] layouts; empty
  /// otherwise.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn custom_channels(&self) -> &[ChannelSpec] {
    self.custom_channels.as_slice()
  }

  /// The backend's own free-form rendering of this layout — FFmpeg's
  /// `av_channel_layout_describe` output (`"5.1(side)"`,
  /// `"3 channels (FL+FR+LFE)"`), empty when there is none.
  ///
  /// Not a slug: this is what the backend printed, verbatim, and it is
  /// the seat that keeps a layout no vocabulary can name from being lost
  /// entirely. [`known_kind`](Self::known_kind) is the parsed name;
  /// this is the rendering.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn text(&self) -> &str {
    self.text.as_str()
  }

  /// `true` when every field is at its absent value — zero channels,
  /// `Unspecified` order, no name, no mask, no custom channels, no
  /// rendering. Useful as an "uninitialized" sentinel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_empty(&self) -> bool {
    self.channels == 0
      && self.order.is_unspecified()
      && self.known_kind == ChannelLayout::default()
      && self.native_mask.is_none()
      && self.custom_channels.is_empty()
      && self.text.is_empty()
  }

  /// Sets the order — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_order(mut self, v: ChannelOrder) -> Self {
    self.order = v;
    self
  }

  /// Sets the channel count — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_channels(mut self, v: u32) -> Self {
    self.channels = v;
    self
  }

  /// Sets the layout's name — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_known_kind(mut self, v: ChannelLayout) -> Self {
    self.known_kind = v;
    self
  }

  /// Sets the native-order bitmask — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_native_mask(mut self, v: Option<u64>) -> Self {
    self.native_mask = v;
    self
  }

  /// Sets the custom-order channel list — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_custom_channels(mut self, v: Vec<ChannelSpec>) -> Self {
    self.custom_channels = v;
    self
  }

  /// Sets the backend's rendering — consuming builder.
  ///
  /// Takes anything that converts into the [`Utf8Bytes`] seat: a `&str`, a
  /// `String`, or a `Utf8Bytes` the caller already holds. The `String`
  /// conversion takes ownership of the caller's buffer instead of copying it.
  ///
  /// **There is no fallible twin, and none is needed.** A backend
  /// rendering an arbitrary layout name does the fallible part on its
  /// own side — reserve (`Vec::try_reserve`), decode, build the
  /// `String` — and hands the finished text here; this seat only stores
  /// what it is given. That is why a long rendering no longer has to be
  /// dropped as "absent": the length limit that forced the choice is
  /// gone with the old carrier.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_text(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.text = v.into();
    self
  }

  /// Sets the order in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_order(&mut self, v: ChannelOrder) -> &mut Self {
    self.order = v;
    self
  }

  /// Sets the channel count in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_channels(&mut self, v: u32) -> &mut Self {
    self.channels = v;
    self
  }

  /// Sets the layout's name in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_known_kind(&mut self, v: ChannelLayout) -> &mut Self {
    self.known_kind = v;
    self
  }

  /// Sets the native-order bitmask in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_native_mask(&mut self, v: Option<u64>) -> &mut Self {
    self.native_mask = v;
    self
  }

  /// Sets the custom-order channel list in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_custom_channels(&mut self, v: Vec<ChannelSpec>) -> &mut Self {
    self.custom_channels = v;
    self
  }

  /// Sets the backend's rendering in place — same seat and same
  /// conversions as [`Self::with_text`], including its note on
  /// fallibility.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_text(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.text = v.into();
    self
  }

  /// Appends one channel to the custom-order list.
  ///
  /// Crate-private, and the `buffa` decoder is its only caller: a
  /// repeated field arrives one element per `merge_field` call, and the
  /// public seat is whole-`Vec` (`with_custom_channels` /
  /// `set_custom_channels`). Reading the list out and writing it back
  /// per element would make decoding quadratic in a length an untrusted
  /// peer chooses, which is a denial of service rather than a
  /// performance note.
  ///
  /// Not public because a caller assembling a description has the whole
  /// list in hand and the whole-`Vec` setter is the honest shape for
  /// that; this exists only because the wire hands them over one at a
  /// time.
  #[cfg(feature = "buffa")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn push_custom_channel(&mut self, v: ChannelSpec) -> &mut Self {
    self.custom_channels.push(v);
    self
  }
}

#[cfg(test)]
mod tests;
