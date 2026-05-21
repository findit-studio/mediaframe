//! EBU R128 / ITU-R BS.1770 loudness measurement: integrated
//! loudness, loudness range, true peak, and sample peak.

/// EBU R128 / ITU-R BS.1770 loudness measurement.
///
/// A value object capturing the four canonical loudness scalars
/// emitted by an EBU R128 analysis pass (e.g. FFmpeg `ebur128`
/// filter, `libebur128`):
///
/// - `integrated_lufs` — programme-integrated loudness in LUFS
///   (a.k.a. LKFS). Typical broadcast targets: −23 LUFS
///   (EBU R128) / −24 LUFS (ATSC A/85).
/// - `range_lu`        — loudness range (LRA) in LU. The macro-
///   dynamic spread; the difference between the high- and low-
///   loudness regions of a programme.
/// - `true_peak_dbtp`  — true peak in dBTP (inter-sample peak as
///   estimated by 4× oversampling per BS.1770-4 Annex 2).
/// - `sample_peak_dbfs` — sample peak in dBFS (the raw PCM peak
///   absolute value, no oversampling).
///
/// The default is all-zero — a "silent / fresh measurement" sentinel
/// rather than a meaningful programme loudness.
///
/// `f32` storage precludes `Eq`/`Hash` (NaN ≠ NaN); the derives are
/// limited to `Debug`/`Clone`/`Copy`/`PartialEq`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loudness {
  integrated_lufs: f32,
  range_lu: f32,
  true_peak_dbtp: f32,
  sample_peak_dbfs: f32,
}

impl Default for Loudness {
  /// Delegates to [`Loudness::new`] — the all-zero
  /// "silent / fresh measurement" sentinel.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new(0.0, 0.0, 0.0, 0.0)
  }
}

impl Loudness {
  /// Constructs a `Loudness` measurement from the four canonical
  /// EBU R128 / BS.1770 scalars.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    integrated_lufs: f32,
    range_lu: f32,
    true_peak_dbtp: f32,
    sample_peak_dbfs: f32,
  ) -> Self {
    Self {
      integrated_lufs,
      range_lu,
      true_peak_dbtp,
      sample_peak_dbfs,
    }
  }

  /// Programme-integrated loudness in LUFS (a.k.a. LKFS).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn integrated_lufs(&self) -> f32 {
    self.integrated_lufs
  }

  /// Loudness range (LRA) in LU.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn range_lu(&self) -> f32 {
    self.range_lu
  }

  /// True peak in dBTP (BS.1770-4 4× oversampled inter-sample peak).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn true_peak_dbtp(&self) -> f32 {
    self.true_peak_dbtp
  }

  /// Sample peak in dBFS (raw PCM peak absolute value).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sample_peak_dbfs(&self) -> f32 {
    self.sample_peak_dbfs
  }

  /// Sets the integrated loudness (LUFS) — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_integrated_lufs(mut self, v: f32) -> Self {
    self.integrated_lufs = v;
    self
  }

  /// Sets the loudness range (LU) — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_range_lu(mut self, v: f32) -> Self {
    self.range_lu = v;
    self
  }

  /// Sets the true peak (dBTP) — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_true_peak_dbtp(mut self, v: f32) -> Self {
    self.true_peak_dbtp = v;
    self
  }

  /// Sets the sample peak (dBFS) — consuming builder.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_sample_peak_dbfs(mut self, v: f32) -> Self {
    self.sample_peak_dbfs = v;
    self
  }

  /// Sets the integrated loudness in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_integrated_lufs(&mut self, v: f32) -> &mut Self {
    self.integrated_lufs = v;
    self
  }

  /// Sets the loudness range in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_range_lu(&mut self, v: f32) -> &mut Self {
    self.range_lu = v;
    self
  }

  /// Sets the true peak in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_true_peak_dbtp(&mut self, v: f32) -> &mut Self {
    self.true_peak_dbtp = v;
    self
  }

  /// Sets the sample peak in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_sample_peak_dbfs(&mut self, v: f32) -> &mut Self {
    self.sample_peak_dbfs = v;
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_holds_supplied_fields() {
    let l = Loudness::new(-23.0, 7.5, -1.2, -3.4);
    assert_eq!(l.integrated_lufs(), -23.0);
    assert_eq!(l.range_lu(), 7.5);
    assert_eq!(l.true_peak_dbtp(), -1.2);
    assert_eq!(l.sample_peak_dbfs(), -3.4);
  }

  #[test]
  fn default_is_all_zero() {
    let l = Loudness::default();
    assert_eq!(l.integrated_lufs(), 0.0);
    assert_eq!(l.range_lu(), 0.0);
    assert_eq!(l.true_peak_dbtp(), 0.0);
    assert_eq!(l.sample_peak_dbfs(), 0.0);
  }

  #[test]
  fn with_chain_builds_full_value() {
    let l = Loudness::default()
      .with_integrated_lufs(-23.0)
      .with_range_lu(7.5)
      .with_true_peak_dbtp(-1.2)
      .with_sample_peak_dbfs(-3.4);
    assert_eq!(l, Loudness::new(-23.0, 7.5, -1.2, -3.4));
  }

  #[test]
  fn setters_mutate_in_place() {
    let mut l = Loudness::default();
    l.set_integrated_lufs(-16.0)
      .set_range_lu(5.0)
      .set_true_peak_dbtp(-0.5)
      .set_sample_peak_dbfs(-1.0);
    assert_eq!(l, Loudness::new(-16.0, 5.0, -0.5, -1.0));
  }
}
