//! Packed RGB96 source (`AV_PIX_FMT_RGB96{LE,BE}`) — 32 bits per channel,
//! `u32` element order `R, G, B`. Stride in u32 elements (≥ `3 * width`).
//!
//! The full-bit integer twin of [`super::Rgb48`]; widened `u16` → `u32`.
//! All 32 bits per channel are active (no stray-bit contract).
//!
//! The marker carries `<const BE: bool = false>`: `Rgb96` (= `Rgb96<false>`)
//! is the LE source; `Rgb96<true>` is the BE source. The walker
//! [`rgb96_to::<BE>`] propagates `BE` from [`Rgb96Frame<'_, BE>`] into the
//! sinker dispatch.
//!
//! Outputs:
//! - `with_rgb`      — narrow each channel `>> 24`, pack as R, G, B.
//! - `with_rgba`     — same narrow + alpha = `0xFF`.
//! - `with_rgb_u16`  — narrow each channel `>> 16`, pack as R, G, B.
//! - `with_rgba_u16` — same narrow + alpha = `0xFFFF`.
//! - `with_luma`     — Y′ from R/G/B after narrowing to u8.
//! - `with_luma_u16` — Y′ computed at u8 precision (matching `with_luma`'s
//!   output) and zero-extended to u16. Same convention as the 8-bit-source
//!   family; not native high-bit luma precision.
//! - `with_hsv`      — HSV via u8 RGB staging.

use crate::frame::Rgb96Frame;

walker! {
  packed_be {
    /// Zero-sized marker for the packed **RGB96** source format
    /// (`AV_PIX_FMT_RGB96{LE,BE}`). `<const BE: bool>` defaults to `false`
    /// (LE); the alias `Rgb96` resolves to `Rgb96<false>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Rgb96,
    frame: Rgb96Frame,
    row: Rgb96Row,
    sink: Rgb96Sink,
    walker: rgb96_to,
    walker_endian: rgb96_to_endian,
    buf_field: rgb96,
    elem_type: u32,
    row_elems: |w| w * 3,
    row_doc: "One row of an [`Rgb96`] source — `width * 3` u32 elements \
              (`R, G, B` per pixel, each channel 32 bits). Endianness is \
              recorded on the parent [`Rgb96Frame<'_, BE>`] / sinker, not on \
              the Row itself — the kernel monomorphizes on `BE` at the \
              sinker dispatch.",
    walker_doc: "Walks an [`Rgb96Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Rgb96Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Rgb96Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_width: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Rgb96Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Rgb96Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_width = row.rgb96().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Rgb96Sink<false> for CountingSink {}

  #[test]
  fn rgb96_walker_visits_every_row_once() {
    // width=4, stride=12 (3*4), height=4 → plane needs 48 u32 elements
    let buf = std::vec![0u32; 12 * 4];
    let frame = Rgb96Frame::new(&buf, 4, 4, 12);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_width: 0,
      last_row_idx: 0,
    };
    rgb96_to(&frame, true, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_width, 12); // width * 3 u32 elements per row
    assert_eq!(sink.last_row_idx, 3);
  }

  // Compile-pass regression mirroring the `packed_be` arm guarantee on the
  // sibling Rgb48 source: the macro emits an LE-only `rgb96_to` wrapper
  // alongside the const-generic `rgb96_to_endian` so explicit-turbofish
  // callers like `rgb96_to::<MySink>(...)` keep compiling (function-position
  // const-generic defaults aren't allowed).
  #[test]
  fn rgb96_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Rgb96Sink>() {
      let _: fn(&crate::frame::Rgb96LeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        rgb96_to::<S>;
    }
  }
}
