//! YUVA 4:2:0 planar 12‑bit, low‑bit‑packed (`yuva420p12le` — no
//! FFmpeg `AV_PIX_FMT` enum).
//!
//! FFmpeg ships no `yuva420p12` pixel format, but non‑FFmpeg decoders
//! and WebCodecs `I420AP12` produce this layout, so it is supported as
//! a source. Storage mirrors [`super::Yuva420p10`] — three planes for
//! Y / U / V at the standard 4:2:0 layout (Y full‑size, U / V
//! half‑width × half‑height) — plus a fourth full‑resolution alpha
//! plane (1:1 with Y; only chroma is subsampled in 4:2:0). Sample
//! width is **`u16`** (12 active bits in the low bits of each element;
//! the high 4 bits are zero). Runs on the same Q15 i32 kernel family
//! as the 9 / 10‑bit YUVA 4:2:0 siblings.

use crate::frame::Yuva420pFrame16;

walker! {
  planar4_bits_be {
    /// Zero‑sized marker for the YUVA 4:2:0 **12‑bit** low‑bit‑packed
    /// source format (`yuva420p12le` — no FFmpeg enum).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Yuva420p12,
    frame: Yuva420pFrame16<'_, 12, BE>,
    frame_le: Yuva420pFrame16<'_, 12, false>,
    generic_frame: Yuva420pFrame16<'_, BITS, BE>,
    bits: 12,
    row: Yuva420p12Row,
    sink: Yuva420p12Sink,
    walker: yuva420p12_to,
    walker_endian: yuva420p12_to_endian,
    walker_inner: yuva420p12_walker,
    elem_type: u16,
    chroma_h: half,
    chroma_v: half,
    row_doc: "One output row of a [`Yuva420p12`] source.",
    walker_doc: "Walks a [`Yuva420p12Frame`](crate::frame::Yuva420p12Frame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::color::Matrix;

  // Mirrors the yuva420p10 turbofish regression: the macro emits an
  // LE-only `yuva420p12_to` wrapper alongside the const-generic
  // `yuva420p12_to_endian`, so explicit-turbofish callers keep
  // compiling.
  #[test]
  fn yuva420p12_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Yuva420p12Sink>() {
      let _: fn(
        &crate::frame::Yuva420p12LeFrame<'_>,
        bool,
        Matrix,
        &mut S,
      ) -> Result<(), S::Error> = yuva420p12_to::<S>;
    }
  }
}
