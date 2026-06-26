//! Packed YUV 4:4:4 8-bit `VYU444` source — 24bpp capture format
//! (FFmpeg `AV_PIX_FMT_VYU444`). Each pixel is a u8 triple
//! `V(8) ‖ Y(8) ‖ U(8)` — there is **no alpha channel** (three bytes
//! per pixel, not four). It is the no-alpha sibling of
//! [`crate::source::Vuya`] / [`crate::source::Vuyx`] with the padding
//! byte dropped entirely. See [`Vyu444Frame`](crate::frame::Vyu444Frame)
//! for layout details.
//!
//! Outputs are produced via:
//! - `with_rgb` — packed YUV → RGB 8-bit pipeline.
//! - `with_rgba` — packed YUV → RGBA 8-bit pipeline; α forced to
//!   `0xFF` (the source carries no alpha).
//! - `with_luma` — extracts the Y byte (byte 1 of each pixel)
//!   directly.
//! - `with_hsv` — stages an internal RGB scratch and runs the
//!   existing `rgb_to_hsv_row` kernel.
//!
//! VYU444 has no u16 output paths — it is an 8-bit source.

use crate::frame::Vyu444Frame;

walker! {
  packed {
    /// Zero-sized marker for the packed **VYU444** source format.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Vyu444,
    frame: Vyu444Frame<'_>,
    row: Vyu444Row,
    sink: Vyu444Sink,
    walker: vyu444_to,
    buf_field: packed,
    elem_type: u8,
    row_elems: |w| w * 3,
    row_doc: concat!(
      "One row of a [`Vyu444`] source — `width × 3` bytes (3 channels per\n",
      "pixel: V, Y, U; there is no alpha channel).\n",
      "\n",
      "Byte layout per pixel:\n",
      "\n",
      "| Byte offset | Field |\n",
      "|-------------|-------|\n",
      "| 0           | V     |\n",
      "| 1           | Y     |\n",
      "| 2           | U     |\n",
      "\n",
      "The walker does not interpret the bytes — it passes the raw packed\n",
      "slice to the sink. Byte-level channel extraction happens in the\n",
      "row-kernel layer.\n",
      "\n",
      "Full range: `[0, 255]` (8-bit). Limited range Y: `[16, 235]`,\n",
      "limited range chroma: `[16, 240]`.",
    ),
    walker_doc: "Walks a [`Vyu444Frame`](crate::frame::Vyu444Frame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::{PixelSink, color::Matrix, frame::Vyu444Frame};
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_packed_len: usize,
    last_row_idx: usize,
  }
  impl PixelSink for CountingSink {
    type Input<'r> = Vyu444Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Vyu444Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_packed_len = row.packed().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }
  impl Vyu444Sink for CountingSink {}

  #[test]
  fn vyu444_walker_visits_every_row_once() {
    // 4 px × 3 channels × 4 rows = 48 bytes
    let buf = std::vec![0u8; 4 * 3 * 4];
    let frame = Vyu444Frame::new(&buf, 4, 4, 12);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_packed_len: 0,
      last_row_idx: 0,
    };
    vyu444_to(&frame, false, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_packed_len, 12); // width × 3 bytes per row
    assert_eq!(sink.last_row_idx, 3);
  }
}
