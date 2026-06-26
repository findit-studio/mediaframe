//! Walker spec for the `Grayf16` source format (FFmpeg `grayf16{le,be}`).
//!
//! Single `half::f16` luma plane. Nominal range `[0.0, 1.0]`; HDR > 1.0 is
//! permitted. Stride is in f16 elements. No chroma planes exist. The
//! half-float twin of [`super::Grayf32`].
//!
//! The marker carries `<const BE: bool = false>`: `Grayf16` (= `Grayf16<false>`)
//! is the LE source; `Grayf16<true>` is the BE source. Two walker entry points
//! are emitted: [`grayf16_to`] is an LE-only compatibility wrapper preserving
//! the single-generic signature `grayf16_to::<S>`;
//! [`grayf16_to_endian::<S, BE>`] is the const-generic entry point for
//! BE-aware callers, propagating `BE` from [`Grayf16Frame<'_, BE>`] into the
//! sinker dispatch. The kernel reinterprets each `f16` via byte-swapped `u16`
//! bits when `BE = true`.

use crate::frame::Grayf16Frame;

walker! {
  planar1_be {
    /// Marker type for the `Grayf16` source format (16-bit half-float luma).
    ///
    /// Nominal luma range `[0.0, 1.0]`; HDR values > 1.0 are permitted.
    /// Out-of-range values are clamped during output conversion, not at frame
    /// construction time. `<const BE: bool>` defaults to `false` (LE).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Grayf16,
    frame: Grayf16Frame,
    row: Grayf16Row,
    sink: Grayf16Sink,
    walker: grayf16_to,
    walker_endian: grayf16_to_endian,
    elem_type: half::f16,
    row_doc: "A single row from a [`Grayf16Frame`](crate::frame::Grayf16Frame) — `width` f16 luma samples.",
    walker_doc: "Walks a [`Grayf16Frame<'_, BE>`] row by row, dispatching each \
                 row to the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Grayf16Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Grayf16Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_y_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Grayf16Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Grayf16Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_y_len = row.y().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Grayf16Sink<false> for CountingSink {}

  #[test]
  fn grayf16_walker_visits_every_row_once() {
    // 4 px × 4 rows = 16 f16 elements (tight stride)
    let buf = std::vec![half::f16::from_f32(0.5); 16];
    let frame = Grayf16Frame::new(&buf, 4, 4, 4);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_y_len: 0,
      last_row_idx: 0,
    };
    grayf16_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_y_len, 4); // width f16 elements per row
    assert_eq!(sink.last_row_idx, 3);
  }

  // Compile-pass regression mirroring the `planar1_be` arm guarantee on the
  // sibling Grayf32 source: the macro emits an LE-only `grayf16_to` wrapper
  // alongside the const-generic `grayf16_to_endian` so explicit-turbofish
  // callers like `grayf16_to::<MySink>(...)` keep compiling (function-position
  // const-generic defaults aren't allowed).
  #[test]
  fn grayf16_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Grayf16Sink>() {
      let _: fn(&crate::frame::Grayf16LeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        grayf16_to::<S>;
    }
  }
}
