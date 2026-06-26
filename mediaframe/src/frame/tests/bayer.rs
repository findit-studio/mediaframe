use super::util::{be_encoded_u16_buf, le_encoded_u16_buf};
use super::*;
use crate::PixelSink;
use core::convert::Infallible;
use std::vec;

// ----- BayerFrame (8-bit) -----

#[test]
fn bayer_try_new_accepts_valid_tight() {
  let data = vec![0u8; 16 * 8];
  let f = BayerFrame::try_new(&data, 16, 8, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.stride(), 16);
}

#[test]
fn bayer_try_new_accepts_padded_stride() {
  let data = vec![0u8; 24 * 8];
  let f = BayerFrame::try_new(&data, 16, 8, 24).expect("padded stride valid");
  assert_eq!(f.stride(), 24);
}

#[test]
fn bayer_try_new_rejects_zero_dim() {
  let data = vec![0u8; 16 * 8];
  let e = BayerFrame::try_new(&data, 0, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrameError::ZeroDimension(_)));
  let e = BayerFrame::try_new(&data, 16, 0, 16).unwrap_err();
  assert!(matches!(e, BayerFrameError::ZeroDimension(_)));
}

#[test]
fn bayer_try_new_accepts_odd_width() {
  // Cropped Bayer planes can have odd dimensions; the kernel
  // handles partial 2×2 tiles via edge clamping.
  let data = vec![0u8; 15 * 8];
  let f = BayerFrame::try_new(&data, 15, 8, 15).expect("odd width valid");
  assert_eq!(f.width(), 15);
}

#[test]
fn bayer_try_new_accepts_odd_height() {
  let data = vec![0u8; 16 * 7];
  let f = BayerFrame::try_new(&data, 16, 7, 16).expect("odd height valid");
  assert_eq!(f.height(), 7);
}

#[test]
fn bayer_try_new_accepts_odd_width_and_height() {
  let data = vec![0u8; 15 * 7];
  let f = BayerFrame::try_new(&data, 15, 7, 15).expect("odd both valid");
  assert_eq!(f.width(), 15);
  assert_eq!(f.height(), 7);
}

#[test]
fn bayer_try_new_accepts_1x1() {
  let data = vec![42u8];
  let f = BayerFrame::try_new(&data, 1, 1, 1).expect("1x1 valid");
  assert_eq!(f.width(), 1);
  assert_eq!(f.height(), 1);
}

#[test]
fn bayer_try_new_rejects_stride_under_width() {
  let data = vec![0u8; 16 * 8];
  let e = BayerFrame::try_new(&data, 16, 8, 8).unwrap_err();
  assert!(matches!(e, BayerFrameError::InsufficientStride(_)));
}

#[test]
fn bayer_try_new_rejects_short_plane() {
  let data = vec![0u8; 10];
  let e = BayerFrame::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrameError::InsufficientPlane(_)));
}

#[test]
#[should_panic(expected = "invalid BayerFrame")]
fn bayer_new_panics_on_invalid() {
  let data = vec![0u8; 10];
  let _ = BayerFrame::new(&data, 16, 8, 16);
}

// ----- BayerFrame16 (high-bit-depth) -----

#[test]
fn bayer16_try_new_rejects_unsupported_bits() {
  let data = vec![0u16; 16 * 8];
  let e = BayerFrame16::<11>::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::UnsupportedBits(_)));
  let e = BayerFrame16::<8>::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::UnsupportedBits(_)));
}

#[test]
fn bayer16_try_new_accepts_each_supported_bits() {
  let data = vec![0u16; 16 * 8];
  Bayer10Frame::try_new(&data, 16, 8, 16).expect("10");
  Bayer12Frame::try_new(&data, 16, 8, 16).expect("12");
  Bayer14Frame::try_new(&data, 16, 8, 16).expect("14");
  Bayer16Frame::try_new(&data, 16, 8, 16).expect("16");
}

#[test]
fn bayer16_try_new_accepts_odd_dims() {
  let data = vec![0u16; 15 * 7];
  let f = Bayer12Frame::try_new(&data, 15, 7, 15).expect("odd both valid");
  assert_eq!(f.width(), 15);
  assert_eq!(f.height(), 7);
}

#[test]
fn bayer16_try_new_accepts_low_packed_12bit() {
  // 12-bit low-packed: every value ≤ 4095 is valid.
  let mut data = vec![2048u16; 16 * 8];
  data[7] = 4095; // max valid 12-bit
  data[42] = 0; // black
  Bayer12Frame::try_new(&data, 16, 8, 16).expect("12-bit low-packed");
}

#[test]
fn bayer16_try_new_rejects_above_max_at_12bit() {
  let mut data = vec![2048u16; 16 * 8];
  data[42] = 4096; // just above 12-bit max
  let e = Bayer12Frame::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn bayer16_try_new_rejects_above_max_at_10bit() {
  let mut data = vec![512u16; 16 * 8];
  data[3] = 1024; // just above 10-bit max
  let e = Bayer10Frame::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn bayer16_try_new_accepts_full_u16_range_at_16bit() {
  // At BITS=16 every u16 is valid.
  let mut data = vec![0u16; 16 * 8];
  data[7] = 0xFFFF;
  data[42] = 0x1234;
  Bayer16Frame::try_new(&data, 16, 8, 16).expect("any u16 valid at 16-bit");
}

#[test]
#[should_panic(expected = "invalid BayerFrame16")]
fn bayer16_new_panics_on_invalid() {
  let data = vec![0u16; 10];
  let _ = Bayer12Frame::new(&data, 16, 8, 16);
}

#[test]
fn bayer_walker_calls_sink_once_per_row() {
  struct CountSink {
    rows: u32,
  }
  impl PixelSink for CountSink {
    type Input<'a> = BayerRow<'a>;
    type Error = Infallible;
    fn process(&mut self, _row: BayerRow<'_>) -> Result<(), Self::Error> {
      self.rows += 1;
      Ok(())
    }
  }
  impl BayerSink for CountSink {}

  let (w, h) = (8u32, 6u32);
  let raw = vec![0u8; (w * h) as usize];
  let frame = BayerFrame::try_new(&raw, w, h, w).unwrap();
  let mut sink = CountSink { rows: 0 };
  bayer_to(
    &frame,
    BayerPattern::Rggb,
    BayerDemosaic::Bilinear,
    WhiteBalance::neutral(),
    ColorCorrectionMatrix::identity(),
    &mut sink,
  )
  .unwrap();
  assert_eq!(sink.rows, h);
}

/// Asserts the documented mirror-by-2 boundary contract: at the
/// top edge `above` is `mid_row(1)`, at the bottom edge `below`
/// is `mid_row(h - 2)`. A custom sink that captures the row
/// borrows directly can verify this without re-running the
/// kernel.
#[test]
fn bayer_walker_supplies_mirror_by_2_row_borrows() {
  /// Captures the first byte of `above` and `below` for each row.
  struct EdgeCapture {
    above_first: std::vec::Vec<u8>,
    below_first: std::vec::Vec<u8>,
  }
  impl PixelSink for EdgeCapture {
    type Input<'a> = BayerRow<'a>;
    type Error = Infallible;
    fn process(&mut self, row: BayerRow<'_>) -> Result<(), Self::Error> {
      self.above_first.push(row.above()[0]);
      self.below_first.push(row.below()[0]);
      Ok(())
    }
  }
  impl BayerSink for EdgeCapture {}

  // 4×4 plane where every row's first byte is the row index. So
  // mid_row(r)[0] == r, and mirror-by-2 should produce
  // above_first = [1, 0, 1, 2] and below_first = [1, 2, 3, 2].
  let raw: std::vec::Vec<u8> = (0..16u8).map(|i| i / 4).collect();
  let frame = BayerFrame::try_new(&raw, 4, 4, 4).unwrap();
  let mut sink = EdgeCapture {
    above_first: std::vec::Vec::new(),
    below_first: std::vec::Vec::new(),
  };
  bayer_to(
    &frame,
    BayerPattern::Rggb,
    BayerDemosaic::Bilinear,
    WhiteBalance::neutral(),
    ColorCorrectionMatrix::identity(),
    &mut sink,
  )
  .unwrap();
  // row 0: above = mid_row(1), below = mid_row(1)
  // row 1: above = mid_row(0), below = mid_row(2)
  // row 2: above = mid_row(1), below = mid_row(3)
  // row 3: above = mid_row(2), below = mid_row(2)  (mirror-by-2)
  assert_eq!(sink.above_first, vec![1u8, 0, 1, 2]);
  assert_eq!(sink.below_first, vec![1u8, 2, 3, 2]);
}

/// Same contract test for `height < 2` — falls back to replicate
/// (no mirror partner exists).
#[test]
fn bayer_walker_falls_back_to_replicate_when_height_below_2() {
  struct EdgeCapture {
    above_first: std::vec::Vec<u8>,
    below_first: std::vec::Vec<u8>,
  }
  impl PixelSink for EdgeCapture {
    type Input<'a> = BayerRow<'a>;
    type Error = Infallible;
    fn process(&mut self, row: BayerRow<'_>) -> Result<(), Self::Error> {
      self.above_first.push(row.above()[0]);
      self.below_first.push(row.below()[0]);
      Ok(())
    }
  }
  impl BayerSink for EdgeCapture {}

  let raw = vec![42u8; 4]; // 4 columns, 1 row
  let frame = BayerFrame::try_new(&raw, 4, 1, 4).unwrap();
  let mut sink = EdgeCapture {
    above_first: std::vec::Vec::new(),
    below_first: std::vec::Vec::new(),
  };
  bayer_to(
    &frame,
    BayerPattern::Rggb,
    BayerDemosaic::Bilinear,
    WhiteBalance::neutral(),
    ColorCorrectionMatrix::identity(),
    &mut sink,
  )
  .unwrap();
  // h=1: replicate fallback. above = below = mid = 42.
  assert_eq!(sink.above_first, vec![42u8]);
  assert_eq!(sink.below_first, vec![42u8]);
}

/// `Bayer12Frame::try_new` rejects out-of-range samples as
/// `BayerFrame16Error::SampleOutOfRange` — a recoverable
/// `Result::Err`, not a panic. Sample-range validation is now
/// part of standard frame construction so the walker is fully
/// fallible.
#[test]
fn bayer12_try_new_rejects_sample_above_max() {
  let (w, h) = (4u32, 2u32);
  let mut raw = vec![100u16; (w * h) as usize];
  raw[3] = 4096; // just above 12-bit max
  let e = Bayer12Frame::try_new(&raw, w, h, w).unwrap_err();
  let crate::frame::BayerFrame16Error::SampleOutOfRange(p) = e else {
    panic!("expected SampleOutOfRange, got {e:?}");
  };
  assert_eq!(p.index(), 3);
  assert_eq!(p.value(), 4096);
  assert_eq!(p.max_valid(), 4095);
}

/// Codex-recommended regression: MSB-aligned 12-bit midgray
/// (e.g., `2048 << 4 = 0x8000`) is exactly the common
/// packing-mismatch bug, where a caller forgot to right-shift
/// before constructing the `Bayer12Frame`. Now caught at
/// construction as `Result::Err` instead of a runtime panic.
#[test]
fn bayer12_try_new_rejects_msb_aligned_input() {
  let (w, h) = (4u32, 2u32);
  let raw = vec![0x8000u16; (w * h) as usize]; // MSB-aligned 12-bit midgray
  let e = Bayer12Frame::try_new(&raw, w, h, w).unwrap_err();
  let crate::frame::BayerFrame16Error::SampleOutOfRange(p) = e else {
    panic!("expected SampleOutOfRange, got {e:?}");
  };
  assert_eq!(p.value(), 0x8000);
  assert_eq!(p.max_valid(), 4095);
}

/// Codex-recommended partial-output regression: a Bayer12 frame
/// with a bad sample in a *later* row used to trigger a runtime
/// panic mid-walk; now `try_new` catches the bad sample upfront
/// and returns `Err`, so the user's output buffer is never
/// touched. (The `bayer16_to` walker can no longer be reached
/// with bad sample data because no `BayerFrame16<BITS>` value
/// can exist with out-of-range samples.)
#[test]
fn bayer12_try_new_rejects_bad_sample_in_later_row() {
  let (w, h) = (4u32, 8u32);
  let mut raw = vec![100u16; (w * h) as usize];
  let off = (6 * w) as usize + 2;
  raw[off] = 4096; // exceeds 12-bit max
  let e = Bayer12Frame::try_new(&raw, w, h, w).unwrap_err();
  let crate::frame::BayerFrame16Error::SampleOutOfRange(p) = e else {
    panic!("expected SampleOutOfRange, got {e:?}");
  };
  assert_eq!(p.value(), 4096);
  assert_eq!(p.max_valid(), 4095);
}

#[test]
fn bayer12_walker_calls_sink_once_per_row() {
  struct CountSink<const BITS: u32> {
    rows: u32,
  }
  impl<const BITS: u32> PixelSink for CountSink<BITS> {
    type Input<'a> = BayerRow16<'a, BITS>;
    type Error = Infallible;
    fn process(&mut self, _row: BayerRow16<'_, BITS>) -> Result<(), Self::Error> {
      self.rows += 1;
      Ok(())
    }
  }
  impl<const BITS: u32> BayerSink16<BITS> for CountSink<BITS> {}

  let (w, h) = (8u32, 6u32);
  let raw = vec![0u16; (w * h) as usize];
  let frame = Bayer12Frame::try_new(&raw, w, h, w).unwrap();
  let mut sink = CountSink::<12> { rows: 0 };
  bayer16_to(
    &frame,
    BayerPattern::Rggb,
    BayerDemosaic::Bilinear,
    WhiteBalance::neutral(),
    ColorCorrectionMatrix::identity(),
    &mut sink,
  )
  .unwrap();
  assert_eq!(sink.rows, h);
}

// ----- BayerFrame16 endian: host-independent LE wire-bytes -----
//
// The `&[u16]` plane of a `BayerFrame16<BITS, false>` is interpreted as
// **LE wire bytes**, consistent with the Y2xx /
// high-bit-YUV frames, and `try_new` normalizes via `from_le` before the
// range check. The plain-`vec![v; n]` LE tests above build planes from
// *native* `u16` values, so they only exercise the wire-bytes contract
// on a little-endian host (where native == LE) — they would silently
// pass for the wrong reason on a big-endian host.
//
// These companions build the plane from explicitly **LE-encoded bytes**
// (`to_le_bytes` → `from_ne_bytes`, via the shared `le_encoded_u16_buf`
// helper the Y2xx / P2xx / GBRP-high-bit tests use), so the in-memory
// `&[u16]` matches the LE wire layout on *any* host: on
// an LE host the buffer equals the literals, on a BE host every `u16` is
// byte-swapped, and `try_new`'s `from_le` recovers the intended value on
// both. This proves the wire-bytes interpretation host-independently
// (mirror of `yuv422p10_try_new_checked_accepts_le_encoded_buffer_on_any_host`).

#[test]
fn bayer16_le_try_new_accepts_le_encoded_plane_on_any_host() {
  // Intended 12-bit samples (all ≤ 4095) are valid; LE-encoding them
  // and reading host-native must still pass on every host because
  // `try_new` normalizes via `from_le` before the range check.
  let intended = vec![2048u16; 16 * 8];
  let buf = le_encoded_u16_buf(&intended);
  let f = Bayer12Frame::try_new(&buf, 16, 8, 16)
    .expect("LE-encoded valid 12-bit plane must be accepted on both LE and BE hosts");
  assert!(!f.is_be());
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
}

#[test]
fn bayer16_le_try_new_accepts_max_value_le_encoded_on_any_host() {
  // The intended 10-bit value 0x03FF (1023) is the max valid sample;
  // its LE wire bytes are [0xFF, 0x03]. Read host-native on a BE host
  // that is 0xFF03 = 65283, far above the 10-bit max — so without the
  // `from_le` normalization the validator would wrongly reject this
  // valid LE plane on a big-endian host. (LE mirror of the BE
  // `accepts_valid_be_encoded_plane` test below.)
  let intended = vec![0x03FFu16; 8 * 4];
  let buf = le_encoded_u16_buf(&intended);
  Bayer10Frame::try_new(&buf, 8, 4, 8)
    .expect("valid LE-encoded 10-bit plane should pass after from_le on any host");
}

#[test]
fn bayer16_le_try_new_rejects_out_of_range_le_encoded_sample_on_any_host() {
  // Intended logical value 4096 exceeds the 12-bit max (4095); after
  // `from_le` normalization the validator must report it as
  // SampleOutOfRange with the normalized logical value on any host,
  // not the raw (possibly byte-swapped) host read. (Negative companion
  // to the LE accept tests; mirror of
  // `yuv422p12_try_new_checked_rejects_le_encoded_out_of_range_on_any_host`.)
  let mut intended = vec![100u16; 4 * 2];
  intended[3] = 4096; // just above 12-bit max
  let buf = le_encoded_u16_buf(&intended);
  let e = Bayer12Frame::try_new(&buf, 4, 2, 4).unwrap_err();
  let crate::frame::BayerFrame16Error::SampleOutOfRange(p) = e else {
    panic!("expected SampleOutOfRange, got {e:?}");
  };
  assert_eq!(p.index(), 3);
  assert_eq!(p.value(), 4096); // normalized logical value
  assert_eq!(p.max_valid(), 4095);
}

// ----- BayerFrame16 endian (BE) -----
//
// These mirror the Y2xx BE tests (`y210_be_frame_*`). They are
// host-independent: an *intended* plane of logical low-packed samples
// is BE-encoded (`to_be_bytes` → `from_ne_bytes`, via the shared
// `be_encoded_u16_buf` helper) so the in-memory `&[u16]` matches the
// BE wire layout on any host. The frame validator's
// `from_be` normalization (and, later, the kernel's byte-swap) recovers
// the intended values on both LE and BE hosts.

#[test]
fn bayer16_be_frame_alias_constructs_and_reports_is_be() {
  // `Bayer12BeFrame` resolves to `BayerFrame16<'_, 12, true>`. The
  // intended samples (all ≤ 4095) are valid 12-bit; BE-encoding them
  // must still pass because `try_new` normalizes via `from_be`.
  let intended = vec![2048u16; 16 * 8];
  let buf = be_encoded_u16_buf(&intended);
  let f = Bayer12BeFrame::try_new(&buf, 16, 8, 16).expect("valid BE 12-bit plane");
  assert!(f.is_be());
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
}

#[test]
fn bayer16_le_frame_alias_is_default_and_not_be() {
  // The default `Bayer12Frame` (LE) and `BayerFrame16<'_, 12, false>`
  // are the same type and report `!is_be()`.
  let data = vec![0u16; 16 * 8];
  let f = Bayer12Frame::try_new(&data, 16, 8, 16).expect("LE default");
  assert!(!f.is_be());
}

#[test]
fn bayer16_be_try_new_accepts_valid_be_encoded_plane() {
  // A plane that is only valid once `from_be`-normalized: the intended
  // 10-bit value 0x03FF (1023) BE-encodes to bytes [0x03, 0xFF]; read
  // host-native on an LE host that is 0xFF03 = 65283, far above the
  // 10-bit max. Without `from_be` normalization the validator would
  // reject this valid BE plane. (Mirror of the Y210 BE accept test.)
  let intended = vec![0x03FFu16; 8 * 4];
  let buf = be_encoded_u16_buf(&intended);
  Bayer10BeFrame::try_new(&buf, 8, 4, 8)
    .expect("valid BE-encoded 10-bit plane should pass after from_be");
}

#[test]
fn bayer16_be_try_new_rejects_out_of_range_logical_sample() {
  // Intended logical value 4096 exceeds the 12-bit max (4095); after
  // `from_be` normalization the validator must report it as
  // SampleOutOfRange with the normalized value, not the raw host read.
  let mut intended = vec![100u16; 4 * 2];
  intended[3] = 4096; // just above 12-bit max
  let buf = be_encoded_u16_buf(&intended);
  let e = Bayer12BeFrame::try_new(&buf, 4, 2, 4).unwrap_err();
  let crate::frame::BayerFrame16Error::SampleOutOfRange(p) = e else {
    panic!("expected SampleOutOfRange, got {e:?}");
  };
  assert_eq!(p.index(), 3);
  assert_eq!(p.value(), 4096); // normalized logical value, not the byte-swapped host read
  assert_eq!(p.max_valid(), 4095);
}

#[test]
fn bayer16_to_endian_be_calls_sink_once_per_row() {
  struct CountSink<const BITS: u32> {
    rows: u32,
  }
  impl<const BITS: u32> PixelSink for CountSink<BITS> {
    type Input<'a> = BayerRow16<'a, BITS>;
    type Error = Infallible;
    fn process(&mut self, _row: BayerRow16<'_, BITS>) -> Result<(), Self::Error> {
      self.rows += 1;
      Ok(())
    }
  }
  // BE sink: impls `BayerSink16<BITS, true>` so the walker's
  // `S: BayerSink16<BITS, BE>` bound is satisfied for `BE = true`.
  impl<const BITS: u32> BayerSink16<BITS, true> for CountSink<BITS> {}

  let (w, h) = (8u32, 6u32);
  let intended = vec![2048u16; (w * h) as usize];
  let buf = be_encoded_u16_buf(&intended);
  let frame = Bayer12BeFrame::try_new(&buf, w, h, w).unwrap();
  assert!(frame.is_be());
  let mut sink = CountSink::<12> { rows: 0 };
  bayer16_to_endian::<12, true, _>(
    &frame,
    BayerPattern::Rggb,
    BayerDemosaic::Bilinear,
    WhiteBalance::neutral(),
    ColorCorrectionMatrix::identity(),
    &mut sink,
  )
  .unwrap();
  assert_eq!(sink.rows, h);
}
