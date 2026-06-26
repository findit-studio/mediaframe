//! MSB-packed planar GBR 10-bit (`AV_PIX_FMT_GBRP10MSB{LE,BE}`) — three
//! full-resolution `u16` planes in **G, B, R** order (FFmpeg convention).
//!
//! Samples are stored in the **high** 10 bits of each `u16` element (low 6
//! bits zero), matching FFmpeg's `gbrp10msb{le,be}` descriptors (`shift = 6`).
//! This is the exact inverse of the low-bit-packed [`super::Gbrp10`]
//! (`gbrp10{le,be}`, samples in the low 10), so it carries a **dedicated**
//! [`GbrpMsbFrame`](crate::frame::GbrpMsbFrame) whose
//! [`try_new_checked`](crate::frame::GbrpMsbFrame::try_new_checked) rejects
//! stray **low** bits.
//!
//! The marker carries `<const BE: bool = false>`: `Gbrp10Msb`
//! (= `Gbrp10Msb<false>`) is the LE source; `Gbrp10Msb<true>` is the BE
//! source. The walker [`gbrp10_msb_to::<BE>`] propagates `BE` from
//! [`Gbrp10MsbFrame<'_, BE>`](crate::frame::Gbrp10MsbFrame) into the sinker
//! dispatch.

use crate::frame::{Gbrp10MsbFrame, GbrpMsbFrame};

walker! {
  planar3_bits_be {
    /// Zero-sized marker for the MSB-packed planar GBR 10-bit source format
    /// (`AV_PIX_FMT_GBRP10MSB{LE,BE}`). `<const BE: bool>` defaults to `false`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Gbrp10Msb,
    frame: Gbrp10MsbFrame,
    generic_frame: GbrpMsbFrame,
    bits: 10,
    row: Gbrp10MsbRow,
    sink: Gbrp10MsbSink,
    walker: gbrp10_msb_to,
    walker_endian: gbrp10_msb_to_endian,
    walker_inner: gbrp10_msb_walker,
    elem_type: u16,
    row_doc: "One output row of a [`Gbrp10Msb`] source — three full-width\n\
              `u16` planes in G / B / R order (samples in the high 10 bits,\n\
              low 6 zero).",
    walker_doc: "Walks a [`Gbrp10MsbFrame<'_, BE>`](crate::frame::Gbrp10MsbFrame) row by row into the sink.",
  }
}

impl<'a> Gbrp10MsbRow<'a> {
  /// Green plane row — full width, 10 active bits in the high 10 of each `u16`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn g(&self) -> &'a [u16] {
    self.y()
  }
  /// Blue plane row — full width.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn b(&self) -> &'a [u16] {
    self.u()
  }
  /// Red plane row — full width.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn r(&self) -> &'a [u16] {
    self.v()
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::color::Matrix;

  // Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
  // (cf. `gbrp10::tests`): the macro emits an LE-only `gbrp10_msb_to` wrapper
  // alongside the const-generic `gbrp10_msb_to_endian` so explicit-turbofish
  // callers like `gbrp10_msb_to::<MySink>(...)` keep compiling.
  #[test]
  fn gbrp10_msb_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Gbrp10MsbSink>() {
      let _: fn(&crate::frame::Gbrp10MsbLeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        gbrp10_msb_to::<S>;
    }
  }
}
