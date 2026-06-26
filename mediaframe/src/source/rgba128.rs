//! Packed RGBA128 source (`AV_PIX_FMT_RGBA128{LE,BE}`) — 32 bits per channel,
//! `u32` element order `R, G, B, A`. Stride in u32 elements (≥ `4 * width`).
//!
//! The full-bit integer twin of [`super::Rgba64`]; widened `u16` → `u32`.
//! All 32 bits per channel are active (no stray-bit contract); alpha is real.
//!
//! The marker carries `<const BE: bool = false>`: `Rgba128` (= `Rgba128<false>`)
//! is the LE source; `Rgba128<true>` is the BE source. The walker
//! [`rgba128_to::<BE>`] propagates `BE` from [`Rgba128Frame<'_, BE>`] into the
//! sinker dispatch.
//!
//! Outputs:
//! - `with_rgb`      — drop alpha, narrow each R/G/B channel `>> 24`, pack as R, G, B.
//! - `with_rgba`     — all four channels narrowed `>> 24`; source alpha passed through.
//! - `with_rgb_u16`  — drop alpha, narrow each R/G/B channel `>> 16`, pack as R, G, B.
//! - `with_rgba_u16` — all four channels narrowed `>> 16`; source alpha preserved.
//! - `with_luma`     — Y′ from R/G/B after narrowing to u8 (alpha ignored).
//! - `with_luma_u16` — Y′ computed at u8 precision (matching `with_luma`'s
//!   output) and zero-extended to u16; alpha ignored. Same convention as
//!   the 8-bit-source family; not native high-bit luma precision.
//! - `with_hsv`      — HSV via u8 RGB staging (alpha ignored).

use crate::frame::Rgba128Frame;

walker! {
  packed_be {
    /// Zero-sized marker for the packed **RGBA128** source format
    /// (`AV_PIX_FMT_RGBA128{LE,BE}`). `<const BE: bool>` defaults to `false`
    /// (LE); the alias `Rgba128` resolves to `Rgba128<false>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Rgba128,
    frame: Rgba128Frame,
    row: Rgba128Row,
    sink: Rgba128Sink,
    walker: rgba128_to,
    walker_endian: rgba128_to_endian,
    buf_field: rgba128,
    elem_type: u32,
    row_elems: |w| w * 4,
    row_doc: "One row of an [`Rgba128`] source — `width * 4` u32 elements \
              (`R, G, B, A` per pixel, each channel 32 bits; alpha is real). \
              Endianness is recorded on the parent [`Rgba128Frame<'_, BE>`] / \
              sinker, not on the Row itself.",
    walker_doc: "Walks an [`Rgba128Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Rgba128Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Rgba128Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_width: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Rgba128Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Rgba128Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_width = row.rgba128().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Rgba128Sink<false> for CountingSink {}

  #[test]
  fn rgba128_walker_visits_every_row_once() {
    // width=4, stride=16 (4*4), height=4 → plane needs 64 u32 elements
    let buf = std::vec![0u32; 16 * 4];
    let frame = Rgba128Frame::new(&buf, 4, 4, 16);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_width: 0,
      last_row_idx: 0,
    };
    rgba128_to(&frame, true, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_width, 16); // width * 4 u32 elements per row
    assert_eq!(sink.last_row_idx, 3);
  }

  // Compile-pass regression mirroring the `packed_be` arm guarantee on the
  // sibling Rgba64 source: the macro emits an LE-only `rgba128_to` wrapper
  // alongside the const-generic `rgba128_to_endian` so explicit-turbofish
  // callers like `rgba128_to::<MySink>(...)` keep compiling (function-position
  // const-generic defaults aren't allowed).
  #[test]
  fn rgba128_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Rgba128Sink>() {
      let _: fn(&crate::frame::Rgba128LeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        rgba128_to::<S>;
    }
  }
}
