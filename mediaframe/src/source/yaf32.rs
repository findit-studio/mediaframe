//! Walker spec for the `Yaf32` source format
//! (FFmpeg `yaf32{le,be}` / `AV_PIX_FMT_YAF32{LE,BE}`).
//!
//! Single `f32` plane packed as `[Y0, A0, Y1, A1, ...]`. Each pixel occupies
//! 2 f32 elements; stride is in f32 elements and must be `≥ width × 2` (may
//! include row padding). Alpha is real source α at element slot 1 of every
//! pixel pair. The single-precision twin of [`super::Yaf16`].
//!
//! The marker carries `<const BE: bool = false>`: `Yaf32` (= `Yaf32<false>`) is
//! the LE source; `Yaf32<true>` is the BE source. Two walker entry points are
//! emitted: [`yaf32_to`] is an LE-only compatibility wrapper preserving the
//! single-generic signature `yaf32_to::<S>`; [`yaf32_to_endian::<S, BE>`] is
//! the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Yaf32Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Yaf32Frame;

walker! {
  packed_be {
    /// Marker type for the `Yaf32` source format (32-bit float gray + alpha,
    /// 2 f32/pixel). `<const BE: bool>` defaults to `false` (LE).
    ///
    /// Packed layout per pixel: `[Y(f32), A(f32)]`. Alpha is real source
    /// transparency and is passed through to RGBA outputs. Nominal range
    /// `[0.0, 1.0]`; HDR > 1.0 is permitted and clamped at output.
    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    marker: Yaf32,
    frame: Yaf32Frame,
    row: Yaf32Row,
    sink: Yaf32Sink,
    walker: yaf32_to,
    walker_endian: yaf32_to_endian,
    buf_field: packed,
    elem_type: f32,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Yaf32`] source — `width × 2` f32 elements (2 f32 per pixel:\n",
      "Y then A).\n",
      "\n",
      "f32 slot layout per pixel:\n",
      "\n",
      "| f32 slot | Field |\n",
      "|----------|-------|\n",
      "| 0        | Y (luma, 32-bit float)   |\n",
      "| 1        | A (real α, 32-bit float) |\n",
      "\n",
      "The walker does not interpret the f32 elements — it passes the raw packed\n",
      "slice to the sink. Endianness is recorded on the parent\n",
      "[`Yaf32Frame<'_, BE>`] / sinker, not on the Row itself — the kernel\n",
      "monomorphizes on `BE` at the sinker dispatch.",
    ),
    walker_doc: "Walks a [`Yaf32Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Yaf32Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Yaf32Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_packed_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Yaf32Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Yaf32Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_packed_len = row.packed().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Yaf32Sink<false> for CountingSink {}

  // Compile-pass regression mirroring the `packed_be` arm guarantee on the
  // sibling Ya16 source: the macro emits an LE-only `yaf32_to` wrapper
  // alongside the const-generic `yaf32_to_endian` so explicit-turbofish
  // callers like `yaf32_to::<MySink>(...)` keep compiling (function-position
  // const-generic defaults aren't allowed).
  #[test]
  fn yaf32_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Yaf32Sink>() {
      let _: fn(
        &crate::frame::Yaf32LeFrame<'_>,
        bool,
        crate::color::Matrix,
        &mut S,
      ) -> Result<(), S::Error> = yaf32_to::<S>;
    }
  }

  #[test]
  fn yaf32_walker_visits_every_row_once() {
    // 4 px × 2 f32 × 4 rows = 32 f32 elements (tight stride)
    let buf = std::vec![0.0f32; 32];
    let frame = Yaf32Frame::new(&buf, 4, 4, 8);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_packed_len: 0,
      last_row_idx: 0,
    };
    yaf32_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_packed_len, 8); // width × 2 f32 elements per row
    assert_eq!(sink.last_row_idx, 3);
  }
}
