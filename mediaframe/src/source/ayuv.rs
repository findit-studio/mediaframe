//! Packed YUV 4:4:4 8-bit `AYUV` source — 32bpp alpha-first capture
//! format (FFmpeg `AV_PIX_FMT_AYUV`). Each pixel is a u8 quadruple
//! `A(8) ‖ Y(8) ‖ U(8) ‖ V(8)` where the A byte is **real alpha**
//! (source transparency). It is the 8-bit, alpha-first channel
//! re-ordering of [`crate::source::Vuya`]. See
//! [`AyuvFrame`](crate::frame::AyuvFrame) for layout details.
//!
//! Outputs are produced via:
//! - `with_rgb` — packed YUV → RGB 8-bit pipeline; alpha discarded.
//! - `with_rgba` — packed YUV → RGBA 8-bit pipeline; source α
//!   passed through from byte 0 of each pixel.
//! - `with_luma` — extracts the Y byte (byte 1 of each pixel)
//!   directly.
//! - `with_hsv` — stages an internal RGB scratch and runs the
//!   existing `rgb_to_hsv_row` kernel.
//!
//! AYUV has no u16 output paths — it is an 8-bit source.

use crate::frame::AyuvFrame;

walker! {
  packed {
    /// Zero-sized marker for the packed **AYUV** source format.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Ayuv,
    frame: AyuvFrame<'_>,
    row: AyuvRow,
    sink: AyuvSink,
    walker: ayuv_to,
    buf_field: packed,
    elem_type: u8,
    row_elems: |w| w * 4,
    row_doc: concat!(
      "One row of an [`Ayuv`] source — `width × 4` bytes (4 channels per\n",
      "pixel: A, Y, U, V; the A byte is real source alpha).\n",
      "\n",
      "Byte layout per pixel:\n",
      "\n",
      "| Byte offset | Field |\n",
      "|-------------|-------|\n",
      "| 0           | A (real source α — passed through to RGBA outputs) |\n",
      "| 1           | Y     |\n",
      "| 2           | U     |\n",
      "| 3           | V     |\n",
      "\n",
      "The walker does not interpret the bytes — it passes the raw packed\n",
      "slice to the sink. Byte-level channel extraction happens in the\n",
      "row-kernel layer.\n",
      "\n",
      "Full range: `[0, 255]` (8-bit). Limited range Y: `[16, 235]`,\n",
      "limited range chroma: `[16, 240]`.",
    ),
    walker_doc: "Walks an [`AyuvFrame`](crate::frame::AyuvFrame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::AyuvFrame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_packed_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = AyuvRow<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: AyuvRow<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_packed_len = row.packed().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl AyuvSink for CountingSink {}

  #[test]
  fn ayuv_walker_visits_every_row_once() {
    // 4 px × 4 channels × 4 rows = 64 bytes
    let buf = std::vec![0u8; 4 * 4 * 4];
    let frame = AyuvFrame::new(&buf, 4, 4, 16);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_packed_len: 0,
      last_row_idx: 0,
    };
    ayuv_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_packed_len, 16); // width × 4 bytes per row
    assert_eq!(sink.last_row_idx, 3);
  }
}
