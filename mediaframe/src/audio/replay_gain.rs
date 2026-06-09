//! ReplayGain — container-tagged loudness-normalization recommendation.

/// ReplayGain — container-tagged loudness-normalization recommendation.
///
/// A value object capturing the four canonical ReplayGain scalars
/// emitted by music taggers (Picard, foobar2000, mp3gain, etc.) and
/// surfaced by FFmpeg via `AV_PKT_DATA_REPLAYGAIN` side data or via
/// the `REPLAYGAIN_*` `AVDictionary` keys:
///
/// - `track_gain_db` — recommended gain to apply (dB) so this **track**
///   plays at the reference loudness. ReplayGain 2.0 uses −18 LUFS as
///   reference, so roughly `track_gain_db ≈ -18.0 - integrated_lufs`.
/// - `track_peak`    — maximum sample magnitude in the track, in
///   linear PCM amplitude (`0.0..=1.0` for non-clipped material; values
///   `> 1.0` are possible after lossy decoding). Players use it to
///   clamp `track_gain_db` so applying it cannot cause hard-clip.
/// - `album_gain_db` — recommended gain across the whole album (so
///   relative loudness between tracks of an album is preserved while
///   album-to-album loudness is normalized). `None` when the
///   container did not carry an album-level number (single-track
///   distribution, or a tagger that only ran per-track analysis).
/// - `album_peak`    — maximum sample magnitude across the whole
///   album, same units / `None` semantics as `album_gain_db`.
///
/// ReplayGain is **distinct from** [`crate::audio::Loudness`]:
///
/// - `Loudness` is the EBU R128 / ITU-R BS.1770 *measurement* of the
///   audio signal.
/// - `ReplayGain` is the *normalization recommendation* a tagger
///   wrote into the container (the delta from a reference level).
///
/// Both can be present and they aren't redundant: per-track
/// ReplayGain is derivable from per-track `Loudness`, but
/// `album_gain_db` / `album_peak` cannot be computed from a single
/// track's loudness measurement alone.
///
/// `f32` storage precludes `Eq`/`Hash` (NaN ≠ NaN); the derives are
/// limited to `Debug`/`Clone`/`Copy`/`PartialEq`. The default is
/// all-zero (gain) / all-none (album) — a "fresh / unset" sentinel
/// rather than a meaningful normalization.
// `serde(default)` keeps sparse / older-schema JSON deserializable: missing
// fields fall back to the type-level `Default` impl — the all-zero / all-none
// "fresh / unset" sentinel.
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(default)
)]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::replay_gain")
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplayGain {
  track_gain_db: f32,
  track_peak: f32,
  album_gain_db: Option<f32>,
  album_peak: Option<f32>,
}

impl Default for ReplayGain {
  /// Delegates to [`ReplayGain::new`] — the all-zero (track) /
  /// all-none (album) "fresh / unset" sentinel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new(0.0, 0.0, None, None)
  }
}

impl ReplayGain {
  /// Constructs a `ReplayGain` recommendation from the four
  /// canonical scalars.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    track_gain_db: f32,
    track_peak: f32,
    album_gain_db: Option<f32>,
    album_peak: Option<f32>,
  ) -> Self {
    Self {
      track_gain_db,
      track_peak,
      album_gain_db,
      album_peak,
    }
  }

  /// Track gain in dB.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn track_gain_db(&self) -> f32 {
    self.track_gain_db
  }

  /// Track peak in linear PCM amplitude (`0.0..=1.0` for non-clipped
  /// material; `> 1.0` is possible after lossy decoding).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn track_peak(&self) -> f32 {
    self.track_peak
  }

  /// Album gain in dB; `None` when the container carried no
  /// album-level number.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn album_gain_db(&self) -> Option<f32> {
    self.album_gain_db
  }

  /// Album peak in linear PCM amplitude; `None` when the container
  /// carried no album-level number.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn album_peak(&self) -> Option<f32> {
    self.album_peak
  }

  /// Sets the track gain (dB) — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_track_gain_db(mut self, v: f32) -> Self {
    self.track_gain_db = v;
    self
  }

  /// Sets the track peak — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_track_peak(mut self, v: f32) -> Self {
    self.track_peak = v;
    self
  }

  /// Sets the album gain (dB) — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_album_gain_db(mut self, v: Option<f32>) -> Self {
    self.album_gain_db = v;
    self
  }

  /// Sets the album peak — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_album_peak(mut self, v: Option<f32>) -> Self {
    self.album_peak = v;
    self
  }

  /// Sets the track gain in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_track_gain_db(&mut self, v: f32) -> &mut Self {
    self.track_gain_db = v;
    self
  }

  /// Sets the track peak in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_track_peak(&mut self, v: f32) -> &mut Self {
    self.track_peak = v;
    self
  }

  /// Sets the album gain in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_album_gain_db(&mut self, v: Option<f32>) -> &mut Self {
    self.album_gain_db = v;
    self
  }

  /// Sets the album peak in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_album_peak(&mut self, v: Option<f32>) -> &mut Self {
    self.album_peak = v;
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_holds_supplied_fields() {
    let g = ReplayGain::new(-6.4, 0.97, Some(-7.1), Some(0.99));
    assert_eq!(g.track_gain_db(), -6.4);
    assert_eq!(g.track_peak(), 0.97);
    assert_eq!(g.album_gain_db(), Some(-7.1));
    assert_eq!(g.album_peak(), Some(0.99));
  }

  #[test]
  fn default_is_zero_track_none_album() {
    let g = ReplayGain::default();
    assert_eq!(g.track_gain_db(), 0.0);
    assert_eq!(g.track_peak(), 0.0);
    assert_eq!(g.album_gain_db(), None);
    assert_eq!(g.album_peak(), None);
  }

  #[test]
  fn with_chain_builds_full_value() {
    let g = ReplayGain::default()
      .with_track_gain_db(-6.4)
      .with_track_peak(0.97)
      .with_album_gain_db(Some(-7.1))
      .with_album_peak(Some(0.99));
    assert_eq!(g, ReplayGain::new(-6.4, 0.97, Some(-7.1), Some(0.99)));
  }

  #[test]
  fn setters_mutate_in_place() {
    let mut g = ReplayGain::default();
    g.set_track_gain_db(-6.4)
      .set_track_peak(0.97)
      .set_album_gain_db(Some(-7.1))
      .set_album_peak(Some(0.99));
    assert_eq!(g, ReplayGain::new(-6.4, 0.97, Some(-7.1), Some(0.99)));
  }

  #[test]
  fn album_fields_are_independent() {
    // Track present, album missing — the common case for
    // single-track distribution.
    let g = ReplayGain::default()
      .with_track_gain_db(-6.4)
      .with_track_peak(0.97);
    assert_eq!(g.album_gain_db(), None);
    assert_eq!(g.album_peak(), None);
  }
}
