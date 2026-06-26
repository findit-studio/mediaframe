//! Packed **BGR4** source (`AV_PIX_FMT_BGR4`) — packed RGB 1:2:1 bitstream,
//! 4 bits per pixel, two pixels per byte. Each 4-bit nibble holds one 1:2:1
//! pixel: bit [3]=B1, bits [2:1]=G2, bit [0]=R1. Within each byte the **first
//! (even) pixel is the high nibble `[7:4]`** and the second (odd) pixel is the
//! low nibble `[3:0]`. No alpha.
//!
//! Outputs (Tier 7):
//! - `with_rgb`      — expand each channel to u8 via bit-replication, pack as `R, G, B`.
//! - `with_rgba`     — same + constant α=`0xFF`.
//! - `with_rgb_u16`  — native 1/2/1-bit precision, low-bit aligned in `u16`.
//! - `with_rgba_u16` — same + constant α=`0xFFFF`.
//! - `with_luma`     — Y′ luma staged through u8 RGB scratch.
//! - `with_luma_u16` — zero-extended u8 luma widened to u16.
//! - `with_hsv`      — HSV staged through u8 RGB scratch.

use crate::frame::Bgr4Frame;

walker! {
  packed {
    /// Zero-sized marker for the packed **BGR4** (`AV_PIX_FMT_BGR4`) source format.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Bgr4,
    frame: Bgr4Frame<'_>,
    row: Bgr4Row,
    sink: Bgr4Sink,
    walker: bgr4_to,
    buf_field: bgr4,
    elem_type: u8,
    row_elems: |w| w.div_ceil(2),
    row_doc: "One output row of a [`Bgr4`] source — `width.div_ceil(2)`\n\
              bytes, two pixels packed per byte.\n\
              \n\
              Within-byte pixel order: the first (even) pixel is the high\n\
              nibble `[7:4]`; the second (odd) pixel is the low nibble `[3:0]`.\n\
              For odd widths the final byte's low nibble is unused.\n\
              \n\
              Bit layout per 4-bit nibble (packed RGB 1:2:1, `(msb)1B 2G 1R(lsb)`):\n\
              \n\
              | Bits (within nibble) | Field |\n\
              |----------------------|-------|\n\
              | 3   | B (1 bit, range [0, 1]) |\n\
              | 2:1 | G (2 bits, range [0, 3]) |\n\
              | 0   | R (1 bit, range [0, 1]) |\n\
              \n\
              Channel positions reversed vs [`crate::source::Rgb4`].\n\
              No source alpha; RGBA outputs force α=`0xFF` / `0xFFFF`.",
    walker_doc: "Walks a [`Bgr4Frame`](crate::frame::Bgr4Frame) row by row into the sink.",
  }
}
