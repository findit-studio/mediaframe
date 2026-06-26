//! MSB-packed planar YUV 4:4:4 10-bit (`AV_PIX_FMT_YUV444P10MSB{LE,BE}`) —
//! three full-resolution `u16` planes (Y, U, V; chroma is 1:1 with luma in
//! 4:4:4).
//!
//! Samples are stored in the **high** 10 bits of each `u16` element (low 6
//! bits zero), matching FFmpeg's `yuv444p10msb{le,be}` descriptors
//! (`shift = 6`). This is the exact inverse of the low-bit-packed
//! [`super::Yuv444p10`] (`yuv444p10{le,be}`, samples in the low 10), so it
//! carries a **dedicated** [`Yuv444pMsbFrame`](crate::frame::Yuv444pMsbFrame)
//! whose [`try_new_checked`](crate::frame::Yuv444pMsbFrame::try_new_checked)
//! rejects stray **low** bits — mirroring the just-added GBR MSB pair
//! ([`super::Gbrp10Msb`]).
//!
//! The marker carries `<const BE: bool = false>`: `Yuv444p10Msb`
//! (= `Yuv444p10Msb<false>`) is the LE source; `Yuv444p10Msb<true>` is the BE
//! source. The walker [`yuv444p10_msb_to::<BE>`] propagates `BE` from
//! [`Yuv444p10MsbFrame<'_, BE>`](crate::frame::Yuv444p10MsbFrame) into the
//! sinker dispatch.

use crate::frame::{Yuv444p10MsbFrame, Yuv444pMsbFrame};

walker! {
  planar3_bits_be {
    /// Zero-sized marker for the MSB-packed planar YUV 4:4:4 10-bit source
    /// format (`AV_PIX_FMT_YUV444P10MSB{LE,BE}`). `<const BE: bool>` defaults
    /// to `false`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Yuv444p10Msb,
    frame: Yuv444p10MsbFrame,
    generic_frame: Yuv444pMsbFrame,
    bits: 10,
    row: Yuv444p10MsbRow,
    sink: Yuv444p10MsbSink,
    walker: yuv444p10_msb_to,
    walker_endian: yuv444p10_msb_to_endian,
    walker_inner: yuv444p10_msb_walker,
    elem_type: u16,
    row_doc: "One output row of a [`Yuv444p10Msb`] source — three full-width\n\
              `u16` planes in Y / U / V order (samples in the high 10 bits,\n\
              low 6 zero).",
    walker_doc: "Walks a [`Yuv444p10MsbFrame<'_, BE>`](crate::frame::Yuv444p10MsbFrame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::color::Matrix;

  // Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
  // (cf. `gbrp10_msb::tests`): the macro emits an LE-only `yuv444p10_msb_to`
  // wrapper alongside the const-generic `yuv444p10_msb_to_endian` so
  // explicit-turbofish callers like `yuv444p10_msb_to::<MySink>(...)` keep
  // compiling.
  #[test]
  fn yuv444p10_msb_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Yuv444p10MsbSink>() {
      let _: fn(
        &crate::frame::Yuv444p10MsbLeFrame<'_>,
        bool,
        Matrix,
        &mut S,
      ) -> Result<(), S::Error> = yuv444p10_msb_to::<S>;
    }
  }
}
