//! Packed YUV 4:4:4 8-bit `UYVA` source — 32bpp chroma-first capture
//! format (FFmpeg `AV_PIX_FMT_UYVA`). Each pixel is a u8 quadruple
//! `U(8) ‖ Y(8) ‖ V(8) ‖ A(8)` where the A byte is **real alpha**
//! (source transparency). It is the chroma-first channel re-ordering
//! of [`crate::source::Vuya`]. See
//! [`UyvaFrame`](crate::frame::UyvaFrame) for layout details.
//!
//! Outputs are produced via:
//! - `with_rgb` — packed YUV → RGB 8-bit pipeline; alpha discarded.
//! - `with_rgba` — packed YUV → RGBA 8-bit pipeline; source α
//!   passed through from byte 3 of each pixel.
//! - `with_luma` — extracts the Y byte (byte 1 of each pixel)
//!   directly.
//! - `with_hsv` — stages an internal RGB scratch and runs the
//!   existing `rgb_to_hsv_row` kernel.
//!
//! UYVA has no u16 output paths — it is an 8-bit source.

use crate::frame::UyvaFrame;

walker! {
  packed {
    /// Zero-sized marker for the packed **UYVA** source format.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Uyva,
    frame: UyvaFrame<'_>,
    row: UyvaRow,
    sink: UyvaSink,
    walker: uyva_to,
    buf_field: packed,
    elem_type: u8,
    row_elems: |w| w * 4,
    row_doc: concat!(
      "One row of a [`Uyva`] source — `width × 4` bytes (4 channels per\n",
      "pixel: U, Y, V, A; the A byte is real source alpha).\n",
      "\n",
      "Byte layout per pixel:\n",
      "\n",
      "| Byte offset | Field |\n",
      "|-------------|-------|\n",
      "| 0           | U     |\n",
      "| 1           | Y     |\n",
      "| 2           | V     |\n",
      "| 3           | A (real source α — passed through to RGBA outputs) |\n",
      "\n",
      "The walker does not interpret the bytes — it passes the raw packed\n",
      "slice to the sink. Byte-level channel extraction happens in the\n",
      "row-kernel layer.\n",
      "\n",
      "Full range: `[0, 255]` (8-bit). Limited range Y: `[16, 235]`,\n",
      "limited range chroma: `[16, 240]`.",
    ),
    walker_doc: "Walks a [`UyvaFrame`](crate::frame::UyvaFrame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::UyvaFrame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_packed_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = UyvaRow<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: UyvaRow<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_packed_len = row.packed().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl UyvaSink for CountingSink {}

  #[test]
  fn uyva_walker_visits_every_row_once() {
    // 4 px × 4 channels × 4 rows = 64 bytes
    let buf = std::vec![0u8; 4 * 4 * 4];
    let frame = UyvaFrame::new(&buf, 4, 4, 16);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_packed_len: 0,
      last_row_idx: 0,
    };
    uyva_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_packed_len, 16); // width × 4 bytes per row
    assert_eq!(sink.last_row_idx, 3);
  }
}
