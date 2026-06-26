//! MSB-packed planar YUV 4:4:4 12-bit (`AV_PIX_FMT_YUV444P12MSB{LE,BE}`) —
//! three full-resolution `u16` planes (Y, U, V; chroma is 1:1 with luma in
//! 4:4:4).
//!
//! Samples are stored in the **high** 12 bits of each `u16` element (low 4
//! bits zero), matching FFmpeg's `yuv444p12msb{le,be}` descriptors
//! (`shift = 4`). This is the exact inverse of the low-bit-packed
//! [`super::Yuv444p12`] (`yuv444p12{le,be}`, samples in the low 12), so it
//! carries a **dedicated** [`Yuv444pMsbFrame`](crate::frame::Yuv444pMsbFrame)
//! whose [`try_new_checked`](crate::frame::Yuv444pMsbFrame::try_new_checked)
//! rejects stray **low** bits — mirroring the just-added GBR MSB pair
//! ([`super::Gbrp12Msb`]).
//!
//! The marker carries `<const BE: bool = false>`: `Yuv444p12Msb`
//! (= `Yuv444p12Msb<false>`) is the LE source; `Yuv444p12Msb<true>` is the BE
//! source. The walker [`yuv444p12_msb_to::<BE>`] propagates `BE` from
//! [`Yuv444p12MsbFrame<'_, BE>`](crate::frame::Yuv444p12MsbFrame) into the
//! sinker dispatch.

use crate::frame::{Yuv444p12MsbFrame, Yuv444pMsbFrame};

walker! {
  planar3_bits_be {
    /// Zero-sized marker for the MSB-packed planar YUV 4:4:4 12-bit source
    /// format (`AV_PIX_FMT_YUV444P12MSB{LE,BE}`). `<const BE: bool>` defaults
    /// to `false`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Yuv444p12Msb,
    frame: Yuv444p12MsbFrame,
    generic_frame: Yuv444pMsbFrame,
    bits: 12,
    row: Yuv444p12MsbRow,
    sink: Yuv444p12MsbSink,
    walker: yuv444p12_msb_to,
    walker_endian: yuv444p12_msb_to_endian,
    walker_inner: yuv444p12_msb_walker,
    elem_type: u16,
    row_doc: "One output row of a [`Yuv444p12Msb`] source — three full-width\n\
              `u16` planes in Y / U / V order (samples in the high 12 bits,\n\
              low 4 zero).",
    walker_doc: "Walks a [`Yuv444p12MsbFrame<'_, BE>`](crate::frame::Yuv444p12MsbFrame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::color::Matrix;

  // Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
  // (cf. `gbrp12_msb::tests`): the macro emits an LE-only `yuv444p12_msb_to`
  // wrapper alongside the const-generic `yuv444p12_msb_to_endian`.
  #[test]
  fn yuv444p12_msb_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Yuv444p12MsbSink>() {
      let _: fn(
        &crate::frame::Yuv444p12MsbLeFrame<'_>,
        bool,
        Matrix,
        &mut S,
      ) -> Result<(), S::Error> = yuv444p12_msb_to::<S>;
    }
  }
}
