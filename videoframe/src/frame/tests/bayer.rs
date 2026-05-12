use super::*;
use crate::PixelSink;
use core::convert::Infallible;

// ----- BayerFrame (8-bit) -----

#[test]
fn bayer_try_new_accepts_valid_tight() {
  let data = std::vec![0u8; 16 * 8];
  let f = BayerFrame::try_new(&data, 16, 8, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.stride(), 16);
}

#[test]
fn bayer_try_new_accepts_padded_stride() {
  let data = std::vec![0u8; 24 * 8];
  let f = BayerFrame::try_new(&data, 16, 8, 24).expect("padded stride valid");
  assert_eq!(f.stride(), 24);
}

#[test]
fn bayer_try_new_rejects_zero_dim() {
  let data = std::vec![0u8; 16 * 8];
  let e = BayerFrame::try_new(&data, 0, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrameError::ZeroDimension(_)));
  let e = BayerFrame::try_new(&data, 16, 0, 16).unwrap_err();
  assert!(matches!(e, BayerFrameError::ZeroDimension(_)));
}

#[test]
fn bayer_try_new_accepts_odd_width() {
  // Cropped Bayer planes can have odd dimensions; the kernel
  // handles partial 2×2 tiles via edge clamping.
  let data = std::vec![0u8; 15 * 8];
  let f = BayerFrame::try_new(&data, 15, 8, 15).expect("odd width valid");
  assert_eq!(f.width(), 15);
}

#[test]
fn bayer_try_new_accepts_odd_height() {
  let data = std::vec![0u8; 16 * 7];
  let f = BayerFrame::try_new(&data, 16, 7, 16).expect("odd height valid");
  assert_eq!(f.height(), 7);
}

#[test]
fn bayer_try_new_accepts_odd_width_and_height() {
  let data = std::vec![0u8; 15 * 7];
  let f = BayerFrame::try_new(&data, 15, 7, 15).expect("odd both valid");
  assert_eq!(f.width(), 15);
  assert_eq!(f.height(), 7);
}

#[test]
fn bayer_try_new_accepts_1x1() {
  let data = std::vec![42u8];
  let f = BayerFrame::try_new(&data, 1, 1, 1).expect("1x1 valid");
  assert_eq!(f.width(), 1);
  assert_eq!(f.height(), 1);
}

#[test]
fn bayer_try_new_rejects_stride_under_width() {
  let data = std::vec![0u8; 16 * 8];
  let e = BayerFrame::try_new(&data, 16, 8, 8).unwrap_err();
  assert!(matches!(e, BayerFrameError::InsufficientStride(_)));
}

#[test]
fn bayer_try_new_rejects_short_plane() {
  let data = std::vec![0u8; 10];
  let e = BayerFrame::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrameError::InsufficientPlane(_)));
}

#[test]
#[should_panic(expected = "invalid BayerFrame")]
fn bayer_new_panics_on_invalid() {
  let data = std::vec![0u8; 10];
  let _ = BayerFrame::new(&data, 16, 8, 16);
}

// ----- BayerFrame16 (high-bit-depth) -----

#[test]
fn bayer16_try_new_rejects_unsupported_bits() {
  let data = std::vec![0u16; 16 * 8];
  let e = BayerFrame16::<11>::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::UnsupportedBits(_)));
  let e = BayerFrame16::<8>::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::UnsupportedBits(_)));
}

#[test]
fn bayer16_try_new_accepts_each_supported_bits() {
  let data = std::vec![0u16; 16 * 8];
  Bayer10Frame::try_new(&data, 16, 8, 16).expect("10");
  Bayer12Frame::try_new(&data, 16, 8, 16).expect("12");
  Bayer14Frame::try_new(&data, 16, 8, 16).expect("14");
  Bayer16Frame::try_new(&data, 16, 8, 16).expect("16");
}

#[test]
fn bayer16_try_new_accepts_odd_dims() {
  let data = std::vec![0u16; 15 * 7];
  let f = Bayer12Frame::try_new(&data, 15, 7, 15).expect("odd both valid");
  assert_eq!(f.width(), 15);
  assert_eq!(f.height(), 7);
}

#[test]
fn bayer16_try_new_accepts_low_packed_12bit() {
  // 12-bit low-packed: every value ≤ 4095 is valid.
  let mut data = std::vec![2048u16; 16 * 8];
  data[7] = 4095; // max valid 12-bit
  data[42] = 0; // black
  Bayer12Frame::try_new(&data, 16, 8, 16).expect("12-bit low-packed");
}

#[test]
fn bayer16_try_new_rejects_above_max_at_12bit() {
  let mut data = std::vec![2048u16; 16 * 8];
  data[42] = 4096; // just above 12-bit max
  let e = Bayer12Frame::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn bayer16_try_new_rejects_above_max_at_10bit() {
  let mut data = std::vec![512u16; 16 * 8];
  data[3] = 1024; // just above 10-bit max
  let e = Bayer10Frame::try_new(&data, 16, 8, 16).unwrap_err();
  assert!(matches!(e, BayerFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn bayer16_try_new_accepts_full_u16_range_at_16bit() {
  // At BITS=16 every u16 is valid.
  let mut data = std::vec![0u16; 16 * 8];
  data[7] = 0xFFFF;
  data[42] = 0x1234;
  Bayer16Frame::try_new(&data, 16, 8, 16).expect("any u16 valid at 16-bit");
}

#[test]
#[should_panic(expected = "invalid BayerFrame16")]
fn bayer16_new_panics_on_invalid() {
  let data = std::vec![0u16; 10];
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
  let raw = std::vec![0u8; (w * h) as usize];
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
  assert_eq!(sink.above_first, std::vec![1u8, 0, 1, 2]);
  assert_eq!(sink.below_first, std::vec![1u8, 2, 3, 2]);
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

  let raw = std::vec![42u8; 4]; // 4 columns, 1 row
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
  assert_eq!(sink.above_first, std::vec![42u8]);
  assert_eq!(sink.below_first, std::vec![42u8]);
}

/// `Bayer12Frame::try_new` rejects out-of-range samples as
/// `BayerFrame16Error::SampleOutOfRange` — a recoverable
/// `Result::Err`, not a panic. Sample-range validation is now
/// part of standard frame construction so the walker is fully
/// fallible.
#[test]
fn bayer12_try_new_rejects_sample_above_max() {
  let (w, h) = (4u32, 2u32);
  let mut raw = std::vec![100u16; (w * h) as usize];
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
  let raw = std::vec![0x8000u16; (w * h) as usize]; // MSB-aligned 12-bit midgray
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
  let mut raw = std::vec![100u16; (w * h) as usize];
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
  let raw = std::vec![0u16; (w * h) as usize];
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
