//! Walker spec for the `Gray32` source format (FFmpeg `gray32{le,be}`).
//!
//! Single `u32` luma plane, all 32 bits active. No chroma. The full-bit
//! integer twin of [`super::Gray16`]; widened `u16` → `u32`.
//!
//! The marker carries `<const BE: bool = false>`: `Gray32` (= `Gray32<false>`)
//! is the LE source; `Gray32<true>` is the BE source. Two walker entry points
//! are emitted: [`gray32_to`] is an LE-only compatibility wrapper preserving
//! the single-generic signature `gray32_to::<S>`; [`gray32_to_endian::<S, BE>`]
//! is the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Gray32Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Gray32Frame;

walker! {
  planar1_be {
    /// Marker type for the `Gray32` source format (32-bit integer u32).
    /// `<const BE: bool>` defaults to `false` (LE).
    marker: Gray32,
    frame: Gray32Frame,
    row: Gray32Row,
    sink: Gray32Sink,
    walker: gray32_to,
    walker_endian: gray32_to_endian,
    elem_type: u32,
    row_doc: "A single row from a [`Gray32Frame`](crate::frame::Gray32Frame).",
    walker_doc: "Walks a [`Gray32Frame<'_, BE>`] row by row, dispatching each \
                 row to the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Gray32Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Gray32Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_y_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Gray32Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Gray32Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_y_len = row.y().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Gray32Sink<false> for CountingSink {}

  #[test]
  fn gray32_walker_visits_every_row_once() {
    // 4 px × 4 rows = 16 u32 elements (tight stride)
    let buf = std::vec![0u32; 16];
    let frame = Gray32Frame::new(&buf, 4, 4, 4);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_y_len: 0,
      last_row_idx: 0,
    };
    gray32_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_y_len, 4); // width u32 elements per row
    assert_eq!(sink.last_row_idx, 3);
  }

  // Compile-pass regression mirroring the `planar1_be` arm guarantee on the
  // sibling Gray16 source: the macro emits an LE-only `gray32_to` wrapper
  // alongside the const-generic `gray32_to_endian` so explicit-turbofish
  // callers like `gray32_to::<MySink>(...)` keep compiling (function-position
  // const-generic defaults aren't allowed).
  #[test]
  fn gray32_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Gray32Sink>() {
      let _: fn(&crate::frame::Gray32LeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        gray32_to::<S>;
    }
  }
}
