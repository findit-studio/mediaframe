use derive_more::{Display, IsVariant};

/// Identifies which of the three HSV planes refer to.
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

  /// Returns the mutable buffers for the H, S, and V planes as a tuple.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn hsv(&mut self) -> (&mut [u8], &mut [u8], &mut [u8]) {
    (self.h, self.s, self.v)
  }

  /// Consumes the mutable buffers for the H, S, and V planes and returns them as a tuple.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn into_hsv(self) -> (&'a mut [u8], &'a mut [u8], &'a mut [u8]) {
    (self.h, self.s, self.v)
  }

  /// Returns the mutable buffer for the H plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn h(&mut self) -> &mut [u8] {
    self.h
  }

  /// Returns the mutable buffer for the S plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn s(&mut self) -> &mut [u8] {
    self.s
  }

  /// Returns the mutable buffer for the V plane.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn v(&mut self) -> &mut [u8] {
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
