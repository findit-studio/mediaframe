//! Packed **BGR4_BYTE** source (`AV_PIX_FMT_BGR4_BYTE`) — packed RGB 1:2:1, one
//! byte per pixel: bit [3]=B1, bits [2:1]=G2, bit [0]=R1. The top 4 bits [7:4]
//! are unused padding. No alpha.
//!
//! Outputs (Tier 7):
//! - `with_rgb`      — expand each channel to u8 via bit-replication, pack as `R, G, B`.
//! - `with_rgba`     — same + constant α=`0xFF`.
//! - `with_rgb_u16`  — native 1/2/1-bit precision, low-bit aligned in `u16`.
//! - `with_rgba_u16` — same + constant α=`0xFFFF`.
//! - `with_luma`     — Y′ luma staged through u8 RGB scratch.
//! - `with_luma_u16` — zero-extended u8 luma widened to u16.
//! - `with_hsv`      — HSV staged through u8 RGB scratch.

use crate::frame::Bgr4ByteFrame;

walker! {
  packed {
    /// Zero-sized marker for the packed **BGR4_BYTE** (`AV_PIX_FMT_BGR4_BYTE`) source format.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Bgr4Byte,
    frame: Bgr4ByteFrame<'_>,
    row: Bgr4ByteRow,
    sink: Bgr4ByteSink,
    walker: bgr4_byte_to,
    buf_field: bgr4_byte,
    elem_type: u8,
    row_elems: |w| w,
    row_doc: "One output row of a [`Bgr4Byte`] source — `width` bytes,\n\
              one byte per pixel (the pixel lives in the low nibble).\n\
              \n\
              Bit layout per byte (packed RGB 1:2:1, `(msb)1B 2G 1R(lsb)`):\n\
              \n\
              | Bits  | Field |\n\
              |-------|-------|\n\
              | 7:4   | padding (ignored on read) |\n\
              | 3     | B (1 bit, range [0, 1]) |\n\
              | 2:1   | G (2 bits, range [0, 3]) |\n\
              | 0     | R (1 bit, range [0, 1]) |\n\
              \n\
              Channel positions reversed vs [`crate::source::Rgb4Byte`].\n\
              No source alpha; RGBA outputs force α=`0xFF` / `0xFFFF`.",
    walker_doc: "Walks a [`Bgr4ByteFrame`](crate::frame::Bgr4ByteFrame) row by row into the sink.",
  }
}
