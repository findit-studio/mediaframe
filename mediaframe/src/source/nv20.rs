//! NV20 — semi-planar 4:2:2, 10-bit, **low-bit-packed**
//! (`AV_PIX_FMT_NV20LE` / `AV_PIX_FMT_NV20BE`).
//!
//! 4:2:2 semi-planar twin of [`super::P210`]: the same Y + interleaved-
//! UV plane shape (full-width Y, half-width interleaved UV at full
//! height) and the same `u16` element type, but each `u16` packs its 10
//! active bits in the **low** 10 positions (`value & 0x03FF`, high 6
//! zero) rather than P210's high 10. See
//! [`Nv20Frame`](crate::frame::Nv20Frame) for the full layout and the
//! authoritative FFmpeg-descriptor evidence that this is genuinely
//! 2-plane semi-planar (one `u16` per sample), not a tight bit-packed
//! stream.
//!
//! The marker carries `<const BE: bool = false>`: `Nv20` (=
//! `Nv20<false>`) is the LE source (`AV_PIX_FMT_NV20LE`);
//! `Nv20<true>` is the BE source (`AV_PIX_FMT_NV20BE`). The walker
//! [`nv20_to_endian::<S, BE>`] propagates `BE` from
//! [`Nv20Frame<'_, BE>`](crate::frame::Nv20Frame) into the sinker
//! dispatch; the kernel masks/normalizes each `u16` per `BE`.

use crate::frame::Nv20Frame;

walker! {
  semi_planar_be {
    /// Zero-sized marker for the NV20 source format. Used as the `F` type
    /// parameter on `MixedSinker`. `<const BE: bool>` defaults to `false`
    /// (LE — `AV_PIX_FMT_NV20LE`); `Nv20` resolves to `Nv20<false>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Nv20,
    frame: Nv20Frame<'_, BE>,
    frame_le: Nv20Frame<'_, false>,
    row: Nv20Row,
    sink: Nv20Sink,
    walker: nv20_to,
    walker_endian: nv20_to_endian,
    elem_type: u16,
    chroma_field: uv,
    chroma_plane: uv,
    chroma_stride: uv_stride,
    chroma_elems_per_row: |w| w,
    chroma_v: full,
    row_doc: "One output row of an NV20 source handed to an [`Nv20Sink`].\n\n\
              Carries borrows to the source slices (full-width Y, half-width interleaved\n\
              UV — full-height, one UV row per Y row) plus the row index and\n\
              matrix/range carry-throughs. Each `u16` element is low-bit-packed (10\n\
              active bits in the low 10 of each element). Endianness is recorded on the\n\
              parent [`Nv20Frame<'_, BE>`](crate::frame::Nv20Frame) / sinker, not on the\n\
              Row itself — the kernel monomorphizes on `BE` at the sinker dispatch.",
    walker_doc: "Walks an [`Nv20Frame`](crate::frame::Nv20Frame) row by row into the sink.\n\
                 Each Y row has its own corresponding UV row (4:2:2 — full-height chroma).\n\
                 Propagates `<const BE: bool>` from the frame into [`Nv20Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Nv20Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_y_len: usize,
    last_uv_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Nv20Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Nv20Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_y_len = row.y().len();
      self.last_uv_len = row.uv().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Nv20Sink for CountingSink {}

  #[test]
  fn nv20_walker_visits_every_row_once() {
    // 8×4 frame. 4:2:2 → chroma is half-width (4 pairs = 8 u16) at
    // full height (4 rows). Y is 8 u16 × 4 rows.
    let y = std::vec![0u16; 8 * 4];
    let uv = std::vec![0u16; 8 * 4];
    let frame = Nv20Frame::new(&y, &uv, 8, 4, 8, 8);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_y_len: 0,
      last_uv_len: 0,
      last_row_idx: 0,
    };
    nv20_to(&frame, true, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_y_len, 8); // full-width Y
    assert_eq!(sink.last_uv_len, 8); // half-width interleaved = width u16
    assert_eq!(sink.last_row_idx, 3);
  }

  // Compile-pass regression mirroring the `semi_planar_be` arm guarantee
  // (cf. `p010_to_explicit_turbofish_one_generic_compiles`): the macro
  // emits an LE-only `nv20_to` wrapper alongside the const-generic
  // `nv20_to_endian` so explicit-turbofish callers like
  // `nv20_to::<MySink>(...)` keep compiling.
  #[test]
  fn nv20_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Nv20Sink>() {
      let _: fn(&crate::frame::Nv20LeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        nv20_to::<S>;
    }
  }
}
