//! MSB-packed planar GBR 12-bit (`AV_PIX_FMT_GBRP12MSB{LE,BE}`) — three
//! full-resolution `u16` planes in **G, B, R** order (FFmpeg convention).
//!
//! Samples are stored in the **high** 12 bits of each `u16` element (low 4
//! bits zero), matching FFmpeg's `gbrp12msb{le,be}` descriptors (`shift = 4`).
//! This is the exact inverse of the low-bit-packed [`super::Gbrp12`]
//! (`gbrp12{le,be}`, samples in the low 12), so it carries a **dedicated**
//! [`GbrpMsbFrame`](crate::frame::GbrpMsbFrame) whose
//! [`try_new_checked`](crate::frame::GbrpMsbFrame::try_new_checked) rejects
//! stray **low** bits.
//!
//! The marker carries `<const BE: bool = false>`: `Gbrp12Msb`
//! (= `Gbrp12Msb<false>`) is the LE source; `Gbrp12Msb<true>` is the BE
//! source. The walker [`gbrp12_msb_to::<BE>`] propagates `BE` from
//! [`Gbrp12MsbFrame<'_, BE>`](crate::frame::Gbrp12MsbFrame) into the sinker
//! dispatch.

use crate::frame::{Gbrp12MsbFrame, GbrpMsbFrame};

walker! {
  planar3_bits_be {
    /// Zero-sized marker for the MSB-packed planar GBR 12-bit source format
    /// (`AV_PIX_FMT_GBRP12MSB{LE,BE}`). `<const BE: bool>` defaults to `false`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Gbrp12Msb,
    frame: Gbrp12MsbFrame,
    generic_frame: GbrpMsbFrame,
    bits: 12,
    row: Gbrp12MsbRow,
    sink: Gbrp12MsbSink,
    walker: gbrp12_msb_to,
    walker_endian: gbrp12_msb_to_endian,
    walker_inner: gbrp12_msb_walker,
    elem_type: u16,
    row_doc: "One output row of a [`Gbrp12Msb`] source — three full-width\n\
              `u16` planes in G / B / R order (samples in the high 12 bits,\n\
              low 4 zero).",
    walker_doc: "Walks a [`Gbrp12MsbFrame<'_, BE>`](crate::frame::Gbrp12MsbFrame) row by row into the sink.",
  }
}

impl<'a> Gbrp12MsbRow<'a> {
  /// Green plane row — full width, 12 active bits in the high 12 of each `u16`.
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
  // (cf. `gbrp10::tests`): the macro emits an LE-only `gbrp12_msb_to` wrapper
  // alongside the const-generic `gbrp12_msb_to_endian`.
  #[test]
  fn gbrp12_msb_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Gbrp12MsbSink>() {
      let _: fn(&crate::frame::Gbrp12MsbLeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        gbrp12_msb_to::<S>;
    }
  }
}
