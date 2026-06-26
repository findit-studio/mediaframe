//! Packed **BGR8** source (`AV_PIX_FMT_BGR8`) — packed RGB 3:3:2, one byte per
//! pixel: bits [7:6]=B2, [5:3]=G3, [2:0]=R3. No alpha. No unused bits.
//!
//! Outputs (Tier 7):
//! - `with_rgb`      — expand each channel to u8 via bit-replication, pack as `R, G, B`.
//! - `with_rgba`     — same + constant α=`0xFF`.
//! - `with_rgb_u16`  — native 3/3/2-bit precision, low-bit aligned in `u16`.
//! - `with_rgba_u16` — same + constant α=`0xFFFF`.
//! - `with_luma`     — Y′ luma staged through u8 RGB scratch.
//! - `with_luma_u16` — zero-extended u8 luma widened to u16.
//! - `with_hsv`      — HSV staged through u8 RGB scratch.

use crate::frame::Bgr8Frame;

walker! {
  packed {
    /// Zero-sized marker for the packed **BGR8** (`AV_PIX_FMT_BGR8`) source format.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Bgr8,
    frame: Bgr8Frame<'_>,
    row: Bgr8Row,
    sink: Bgr8Sink,
    walker: bgr8_to,
    buf_field: bgr8,
    elem_type: u8,
    row_elems: |w| w,
    row_doc: "One output row of a [`Bgr8`] source — `width` bytes,\n\
              one byte per pixel.\n\
              \n\
              Bit layout per byte (packed RGB 3:3:2, `(msb)2B 3G 3R(lsb)`):\n\
              \n\
              | Bits  | Field |\n\
              |-------|-------|\n\
              | 7:6   | B (2 bits, range [0, 3]) |\n\
              | 5:3   | G (3 bits, range [0, 7]) |\n\
              | 2:0   | R (3 bits, range [0, 7]) |\n\
              \n\
              Channel positions reversed vs [`crate::source::Rgb8`].\n\
              No source alpha; RGBA outputs force α=`0xFF` / `0xFFFF`.",
    walker_doc: "Walks a [`Bgr8Frame`](crate::frame::Bgr8Frame) row by row into the sink.",
  }
}
