use derive_more::{Display, IsVariant};

/// Identifies which of the three HSV planes a
/// [`MixedSinkerError::InsufficientHsvPlane`] refers to.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, IsVariant)]
#[display("{}", self.as_str())]
pub enum HsvPlane {
  /// Hue plane.
  H,
  /// Saturation plane.
  S,
  /// Value plane.
  V,
}

impl HsvPlane {
  /// Returns a string identifier for this plane, used in error messages.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::H => "H",
      Self::S => "S",
      Self::V => "V",
    }
  }
}

/// Mutable references to the three HSV planes of an HSV source frame.
#[derive(Debug)]
pub struct HsvFrameMut<'a> {
  h: &'a mut [u8],
  s: &'a mut [u8],
  v: &'a mut [u8],
}

impl<'a> HsvFrameMut<'a> {
  /// Constructs a new [`HsvFrameMut`] from the three plane buffers.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(h: &'a mut [u8], s: &'a mut [u8], v: &'a mut [u8]) -> Self {
    Self { h, s, v }
  }

  /// Returns the mutable buffer for the H plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn h(&'a mut self) -> &'a mut [u8] {
    self.h
  }

  /// Returns the mutable buffer for the S plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn s(&'a mut self) -> &'a mut [u8] {
    self.s
  }

  /// Returns the mutable buffer for the V plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&'a mut self) -> &'a mut [u8] {
    self.v
  }
}

/// Immutable references to the three HSV planes of an HSV source frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HsvFrame<'a> {
  h: &'a [u8],
  s: &'a [u8],
  v: &'a [u8],
}

impl<'a> HsvFrame<'a> {
  /// Constructs a new [`HsvFrame`] from the three plane buffers.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(h: &'a [u8], s: &'a [u8], v: &'a [u8]) -> Self {
    Self { h, s, v }
  }

  /// Returns the buffer for the H plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn h(&self) -> &'a [u8] {
    self.h
  }

  /// Returns the buffer for the S plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn s(&self) -> &'a [u8] {
    self.s
  }

  /// Returns the buffer for the V plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&self) -> &'a [u8] {
    self.v
  }
}
