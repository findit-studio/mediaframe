//! Walker spec for the `Yaf16` source format
//! (FFmpeg `yaf16{le,be}` / `AV_PIX_FMT_YAF16{LE,BE}`).
//!
//! Single `half::f16` plane packed as `[Y0, A0, Y1, A1, ...]`. Each pixel
//! occupies 2 f16 elements; stride is in f16 elements and must be `≥ width × 2`
//! (may include row padding). Alpha is real source α at element slot 1 of every
//! pixel pair. The half-float twin of [`super::Ya16`].
//!
//! The marker carries `<const BE: bool = false>`: `Yaf16` (= `Yaf16<false>`) is
//! the LE source; `Yaf16<true>` is the BE source. Two walker entry points are
//! emitted: [`yaf16_to`] is an LE-only compatibility wrapper preserving the
//! single-generic signature `yaf16_to::<S>`; [`yaf16_to_endian::<S, BE>`] is
//! the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Yaf16Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Yaf16Frame;

walker! {
  packed_be {
    /// Marker type for the `Yaf16` source format (16-bit half-float gray +
    /// alpha, 2 f16/pixel). `<const BE: bool>` defaults to `false` (LE).
    ///
    /// Packed layout per pixel: `[Y(f16), A(f16)]`. Alpha is real source
    /// transparency and is passed through to RGBA outputs. Nominal range
    /// `[0.0, 1.0]`; HDR > 1.0 is permitted and clamped at output.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Yaf16,
    frame: Yaf16Frame,
    row: Yaf16Row,
    sink: Yaf16Sink,
    walker: yaf16_to,
    walker_endian: yaf16_to_endian,
    buf_field: packed,
    elem_type: half::f16,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Yaf16`] source — `width × 2` f16 elements (2 f16 per pixel:\n",
      "Y then A).\n",
      "\n",
      "f16 slot layout per pixel:\n",
      "\n",
      "| f16 slot | Field |\n",
      "|----------|-------|\n",
      "| 0        | Y (luma, 16-bit half-float)   |\n",
      "| 1        | A (real α, 16-bit half-float) |\n",
      "\n",
      "The walker does not interpret the f16 elements — it passes the raw packed\n",
      "slice to the sink. Endianness is recorded on the parent\n",
      "[`Yaf16Frame<'_, BE>`] / sinker, not on the Row itself — the kernel\n",
      "monomorphizes on `BE` at the sinker dispatch.",
    ),
    walker_doc: "Walks a [`Yaf16Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Yaf16Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Yaf16Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_packed_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Yaf16Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Yaf16Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_packed_len = row.packed().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Yaf16Sink<false> for CountingSink {}

  // Compile-pass regression mirroring the `packed_be` arm guarantee on the
  // sibling Ya16 source: the macro emits an LE-only `yaf16_to` wrapper
  // alongside the const-generic `yaf16_to_endian` so explicit-turbofish
  // callers like `yaf16_to::<MySink>(...)` keep compiling (function-position
  // const-generic defaults aren't allowed).
  #[test]
  fn yaf16_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Yaf16Sink>() {
      let _: fn(
        &crate::frame::Yaf16LeFrame<'_>,
        bool,
        crate::color::Matrix,
        &mut S,
      ) -> Result<(), S::Error> = yaf16_to::<S>;
    }
  }

  #[test]
  fn yaf16_walker_visits_every_row_once() {
    // 4 px × 2 f16 × 4 rows = 32 f16 elements (tight stride)
    let buf = std::vec![half::f16::ZERO; 32];
    let frame = Yaf16Frame::new(&buf, 4, 4, 8);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_packed_len: 0,
      last_row_idx: 0,
    };
    yaf16_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_packed_len, 8); // width × 2 f16 elements per row
    assert_eq!(sink.last_row_idx, 3);
  }
}
