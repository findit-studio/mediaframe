#![allow(clippy::useless_vec)]

use super::{
  util::{be_encoded_u16_buf, le_encoded_u16_buf},
  *,
};
use std::{vec, vec::Vec};

// ---- Yuv444pFrame16::try_new_checked ---------------------------------

fn p444_planes_10bit() -> (Vec<u16>, Vec<u16>, Vec<u16>) {
  // 4:4:4: chroma is FULL-width, full-height (1:1 with Y).
  let y = le_encoded_u16_buf(&vec![64u16; 16 * 8]);
  let u = le_encoded_u16_buf(&vec![512u16; 16 * 8]);
  let v = le_encoded_u16_buf(&vec![512u16; 16 * 8]);
  (y, u, v)
}

#[test]
fn yuv444p10_try_new_checked_accepts_in_range_samples() {
  let (y, u, v) = p444_planes_10bit();
  let f = Yuv444p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).expect("valid 10-bit");
  assert_eq!(f.width(), 16);
  assert_eq!(f.bits(), 10);
}

#[test]
fn yuv444p10_try_new_checked_accepts_max_valid_value() {
  let y = le_encoded_u16_buf(&vec![1023u16; 16 * 8]);
  let u = le_encoded_u16_buf(&vec![1023u16; 16 * 8]);
  let v = le_encoded_u16_buf(&vec![1023u16; 16 * 8]);
  Yuv444p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).expect("max valid passes");
}

#[test]
fn yuv444p10_try_new_checked_rejects_y_high_bit_set() {
  let mut intended_y = vec![0u16; 16 * 8];
  intended_y[2 * 16 + 9] = 0x8000;
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&vec![512u16; 16 * 8]);
  let v = le_encoded_u16_buf(&vec![512u16; 16 * 8]);
  let e = Yuv444p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv444p10_try_new_checked_rejects_u_plane_sample_in_full_width_chroma() {
  // 4:4:4-specific: the offending sample is in the FULL-WIDTH
  // chroma plane, at column 13 (which doesn't exist in 4:2:0/4:2:2
  // half-width chroma). Forces the scan to extend across the full
  // chroma width.
  let mut intended_u = vec![512u16; 16 * 8];
  intended_u[3 * 16 + 13] = 1024;
  let y = le_encoded_u16_buf(&vec![0u16; 16 * 8]);
  let u = le_encoded_u16_buf(&intended_u);
  let v = le_encoded_u16_buf(&vec![512u16; 16 * 8]);
  let e = Yuv444p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv444p10_try_new_checked_rejects_v_plane_sample() {
  // LE-encoded byte storage so the test asserts the same logical values
  // on every host (see 4:2:0 sibling for the BE-host failure mode).
  let intended_y = vec![0u16; 16 * 8];
  let intended_u = vec![512u16; 16 * 8];
  let mut intended_v = vec![512u16; 16 * 8];
  intended_v[7 * 16 + 15] = 0xFFFF; // last chroma sample
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_u);
  let v = le_encoded_u16_buf(&intended_v);
  let e = Yuv444p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv444p14_try_new_checked_rejects_above_16383() {
  let mut intended_y = vec![8192u16; 16 * 8];
  intended_y[42] = 16384; // just above 14-bit max
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&vec![8192u16; 16 * 8]);
  let v = le_encoded_u16_buf(&vec![8192u16; 16 * 8]);
  let e = Yuv444p14Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv444p16_try_new_checked_accepts_full_u16_range() {
  let y = vec![65535u16; 16 * 8];
  let u = vec![32768u16; 16 * 8];
  let v = vec![32768u16; 16 * 8];
  Yuv444p16Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16)
    .expect("every u16 value is in range at 16 bits");
}

// ---- Yuv440p10/12 checked-constructor tests ---------------------------

fn p440_planes_10bit() -> (Vec<u16>, Vec<u16>, Vec<u16>) {
  // 4:4:0: chroma is FULL-width × HALF-height (8 / 2 = 4 chroma rows).
  let y = le_encoded_u16_buf(&vec![64u16; 16 * 8]);
  let u = le_encoded_u16_buf(&vec![512u16; 16 * 4]);
  let v = le_encoded_u16_buf(&vec![512u16; 16 * 4]);
  (y, u, v)
}

#[test]
fn yuv440p10_try_new_checked_accepts_in_range_samples() {
  let (y, u, v) = p440_planes_10bit();
  let f = Yuv440p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).expect("valid 10-bit");
  assert_eq!(f.width(), 16);
  assert_eq!(f.bits(), 10);
}

#[test]
fn yuv440p10_try_new_checked_rejects_y_high_bit_set() {
  let mut intended_y = vec![0u16; 16 * 8];
  intended_y[2 * 16 + 9] = 0x8000;
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&vec![512u16; 16 * 4]);
  let v = le_encoded_u16_buf(&vec![512u16; 16 * 4]);
  let e = Yuv440p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv440p10_try_new_checked_rejects_u_plane_sample_in_full_width_chroma() {
  // 4:4:0-specific: chroma is full-width × half-height. Plant the
  // bad sample at column 13 (would be out of range for half-width
  // 4:2:0/4:2:2 chroma) on the last chroma row (index 3 for height
  // 8 ⇒ 4 chroma rows).
  let mut intended_u = vec![512u16; 16 * 4];
  intended_u[3 * 16 + 13] = 1024;
  let y = le_encoded_u16_buf(&vec![0u16; 16 * 8]);
  let u = le_encoded_u16_buf(&intended_u);
  let v = le_encoded_u16_buf(&vec![512u16; 16 * 4]);
  let e = Yuv440p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv440p10_try_new_checked_rejects_v_plane_sample() {
  // LE-encoded byte storage so the test asserts the same logical values
  // on every host (see 4:2:0 sibling for the BE-host failure mode).
  let intended_y = vec![0u16; 16 * 8];
  let intended_u = vec![512u16; 16 * 4];
  let mut intended_v = vec![512u16; 16 * 4];
  intended_v[3 * 16 + 15] = 0xFFFF; // last chroma sample of the last chroma row
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_u);
  let v = le_encoded_u16_buf(&intended_v);
  let e = Yuv440p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv440p12_try_new_checked_rejects_above_4095() {
  let mut intended_y = vec![2048u16; 16 * 8];
  intended_y[42] = 4096; // just above 12-bit max
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&vec![2048u16; 16 * 4]);
  let v = le_encoded_u16_buf(&vec![2048u16; 16 * 4]);
  let e = Yuv440p12Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

// ---- Host-independent BE-host regressions (codex round-2) -----------
//
// See `subsampled_4_2_0_high_bit.rs` for the full rationale: build
// planes from LE-encoded bytes so on BE hosts the validator's
// `u16::from_le` normalization is exercised end-to-end.

#[test]
fn yuv444p10_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 4:4:4: chroma is full-width × full-height.
  let intended_y = vec![1023u16; 16 * 8];
  let intended_uv = vec![512u16; 16 * 8];
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  Yuv444p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16)
    .expect("LE-encoded valid yuv444p10le must be accepted on both LE and BE hosts");
}

#[test]
fn yuv444p14_try_new_checked_rejects_le_encoded_out_of_range_on_any_host() {
  // After `from_le` normalization, the offending sample is 16384
  // (just above 14-bit max 16383).
  let intended_y = vec![8192u16; 16 * 8];
  let intended_u = vec![8192u16; 16 * 8];
  let mut intended_v = vec![8192u16; 16 * 8];
  intended_v[3 * 16 + 11] = 16384;
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_u);
  let v = le_encoded_u16_buf(&intended_v);
  let e = Yuv444p14Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuv440p10_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 4:4:0: chroma is full-width × half-height.
  let intended_y = vec![1023u16; 16 * 8];
  let intended_uv = vec![512u16; 16 * 4];
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  Yuv440p10Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16)
    .expect("LE-encoded valid yuv440p10le must be accepted on both LE and BE hosts");
}

#[test]
fn yuv440p12_try_new_checked_rejects_le_encoded_out_of_range_on_any_host() {
  let intended_y = vec![2048u16; 16 * 8];
  let mut intended_u = vec![2048u16; 16 * 4];
  intended_u[2 * 16 + 5] = 4096;
  let intended_v = vec![2048u16; 16 * 4];
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_u);
  let v = le_encoded_u16_buf(&intended_v);
  let e = Yuv440p12Frame::try_new_checked(&y, &u, &v, 16, 8, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuv420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn p410_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 4:4:4 PnFrame444: chroma is full-width × full-height with 2 u16
  // per pair ⇒ each UV row holds 2 * width u16 elements.
  let intended_y = vec![0xFFC0u16; 16 * 8];
  let intended_uv = vec![0x8000u16; 32 * 8];
  let y = le_encoded_u16_buf(&intended_y);
  let uv = le_encoded_u16_buf(&intended_uv);
  P410Frame::try_new_checked(&y, &uv, 16, 8, 16, 32)
    .expect("LE-encoded valid P410 must be accepted on both LE and BE hosts");
}

#[test]
fn p410_try_new_checked_rejects_le_encoded_low_bits_on_any_host() {
  // Logical 0x03FF (low 6 bits all set) on the UV plane.
  let intended_y = vec![0xFFC0u16; 16 * 8];
  let mut intended_uv = vec![0x8000u16; 32 * 8];
  intended_uv[4 * 32 + 17] = 0x03FF;
  let y = le_encoded_u16_buf(&intended_y);
  let uv = le_encoded_u16_buf(&intended_uv);
  let e = P410Frame::try_new_checked(&y, &uv, 16, 8, 16, 32).unwrap_err();
  assert!(matches!(e, PnFrameError::SampleLowBitsSet(_)));
}

// ---- Yuv444pMsbFrame (3-plane, high-bit-packed) geometry tests ------------

fn yuv444p_msb_planes(w: u32, h: u32) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
  let n = (w * h) as usize;
  (vec![0u16; n], vec![0u16; n], vec![0u16; n])
}

#[test]
fn yuv444p10_msb_try_new_accepts_valid_tight() {
  let (y, u, v) = yuv444p_msb_planes(16, 8);
  let f = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 16, 8, 16, 16, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.y_stride(), 16);
  assert_eq!(f.bits(), 10);
  assert!(!f.is_be());
}

#[test]
fn yuv444p10_msb_try_new_accepts_padded_strides() {
  let stride = 32u32;
  let h = 8u32;
  let p = vec![0u16; (stride * h) as usize];
  let f = Yuv444p10MsbLeFrame::try_new(&p, &p, &p, 16, h, stride, stride, stride).expect("valid");
  assert_eq!(f.y_stride(), stride);
}

#[test]
fn yuv444p10_msb_try_new_rejects_zero_dimension() {
  let (y, u, v) = yuv444p_msb_planes(4, 4);
  assert!(matches!(
    Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 0, 4, 4, 4, 4),
    Err(Yuv444pMsbFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 4, 0, 4, 4, 4),
    Err(Yuv444pMsbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn yuv444p10_msb_try_new_rejects_y_stride_too_small() {
  let (y, u, v) = yuv444p_msb_planes(8, 4);
  let e = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 8, 4, 4, 8, 8).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::InsufficientYStride(_)));
}

#[test]
fn yuv444p10_msb_try_new_rejects_u_stride_too_small() {
  // 4:4:4 chroma is full-width, so u_stride must be >= width.
  let (y, u, v) = yuv444p_msb_planes(8, 4);
  let e = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 8, 4, 8, 4, 8).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::InsufficientUStride(_)));
}

#[test]
fn yuv444p10_msb_try_new_rejects_v_stride_too_small() {
  let (y, u, v) = yuv444p_msb_planes(8, 4);
  let e = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 8, 4, 8, 8, 4).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::InsufficientVStride(_)));
}

#[test]
fn yuv444p10_msb_try_new_rejects_y_plane_too_short() {
  let y = vec![0u16; 16];
  let u = vec![0u16; 32];
  let v = vec![0u16; 32];
  let e = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::InsufficientYPlane(_)));
}

#[test]
fn yuv444p10_msb_try_new_rejects_u_plane_too_short() {
  let y = vec![0u16; 32];
  let u = vec![0u16; 16];
  let v = vec![0u16; 32];
  let e = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::InsufficientUPlane(_)));
}

#[test]
fn yuv444p10_msb_try_new_rejects_v_plane_too_short() {
  let y = vec![0u16; 32];
  let u = vec![0u16; 32];
  let v = vec![0u16; 16];
  let e = Yuv444p10MsbLeFrame::try_new(&y, &u, &v, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::InsufficientVPlane(_)));
}

#[test]
#[should_panic(expected = "invalid Yuv444pMsbFrame")]
fn yuv444p10_msb_new_panics_on_invalid() {
  let (y, u, v) = yuv444p_msb_planes(4, 4);
  let _ = Yuv444p10MsbLeFrame::new(&y, &u, &v, 0, 4, 4, 4, 4);
}

#[test]
fn yuv444p12_msb_bits_accessor_and_be_alias() {
  let p = vec![0u16; 4];
  let f = Yuv444p12MsbLeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 12);
  assert!(!f.is_be());
  let fbe = Yuv444p12MsbBeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert!(fbe.is_be());
  assert_eq!(fbe.bits(), 12);
}

// ---- Yuv444pMsbFrame::try_new_checked (rejects stray LOW bits) ------------
//
// MSB-packed YUV 4:4:4 carries its BITS active bits in the HIGH BITS of each
// `u16`; the low `16 - BITS` must be zero (low 6 at 10-bit → mask 0x003F, low
// 4 at 12-bit → mask 0x000F). This is the exact inverse of the low-bit-packed
// `Yuv444pFrame16`, whose `try_new_checked` rejects out-of-range (stray-HIGH)
// samples. The host-independent `*_encoded_u16_buf` helpers build the wire
// byte layout so the asserted logical values hold on both LE and BE hosts
// after the validator's `from_le` / `from_be` normalization.

#[test]
fn yuv444p10_msb_try_new_checked_accepts_max_high_packed_value() {
  // 0xFFC0 = all ten high bits set, low six zero — the largest valid 10-bit
  // MSB-packed sample. Must pass on both LE and BE hosts.
  let intended = vec![0xFFC0u16; 8 * 4];
  let p = le_encoded_u16_buf(&intended);
  let f = Yuv444p10MsbLeFrame::try_new_checked(&p, &p, &p, 8, 4, 8, 8, 8)
    .expect("0xFFC0 max valid passes");
  assert_eq!(f.width(), 8);
  assert_eq!(f.height(), 4);
}

#[test]
fn yuv444p12_msb_try_new_checked_accepts_max_high_packed_value() {
  // 0xFFF0 = all twelve high bits set, low four zero.
  let intended = vec![0xFFF0u16; 8 * 4];
  let p = le_encoded_u16_buf(&intended);
  let f = Yuv444p12MsbLeFrame::try_new_checked(&p, &p, &p, 8, 4, 8, 8, 8)
    .expect("0xFFF0 max valid passes");
  assert_eq!(f.bits(), 12);
}

#[test]
fn yuv444p10_msb_try_new_checked_rejects_y_low_bit_set_le() {
  // A single Y sample with a stray low bit (low-bit-packed data wrongly
  // handed to the MSB path). LE-encoded so logical 0x0001 holds on any host.
  let mut intended_y = vec![0xFFC0u16; 8 * 4];
  intended_y[3 * 8 + 5] = 0x0001;
  let y = le_encoded_u16_buf(&intended_y);
  let ok = le_encoded_u16_buf(&[0xFFC0u16; 8 * 4]);
  let e = Yuv444p10MsbLeFrame::try_new_checked(&y, &ok, &ok, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    Yuv444pMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), Yuv444pMsbFramePlane::Y);
      assert_eq!(p.value(), 0x0001);
      assert_eq!(p.index(), 3 * 8 + 5);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn yuv444p10_msb_try_new_checked_rejects_u_low_bit_set_be() {
  // BE-encoded buffer: `from_be` normalization must recover logical 0x0020
  // (bit 5, inside the low-6 zero region) on any host and reject it.
  let mut intended_u = vec![0xFFC0u16; 8 * 4];
  intended_u[8 + 2] = 0x0020;
  let u = be_encoded_u16_buf(&intended_u);
  let ok = be_encoded_u16_buf(&[0xFFC0u16; 8 * 4]);
  let e = Yuv444p10MsbBeFrame::try_new_checked(&ok, &u, &ok, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    Yuv444pMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), Yuv444pMsbFramePlane::U);
      assert_eq!(p.value(), 0x0020);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn yuv444p10_msb_try_new_checked_rejects_v_low_bit_set_le() {
  // Offending sample on the V plane, last row.
  let mut intended_v = vec![0xFFC0u16; 8 * 4];
  intended_v[3 * 8 + 7] = 0x003F; // all six low bits set
  let v = le_encoded_u16_buf(&intended_v);
  let ok = le_encoded_u16_buf(&[0xFFC0u16; 8 * 4]);
  let e = Yuv444p10MsbLeFrame::try_new_checked(&ok, &ok, &v, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    Yuv444pMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), Yuv444pMsbFramePlane::V);
      assert_eq!(p.value(), 0x003F);
      assert_eq!(p.index(), 3 * 8 + 7);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn yuv444p12_msb_try_new_checked_rejects_y_low_bit_set_be() {
  // At 12-bit the low-4 mask is 0x000F. 0x0008 is bit 3 — a stray low bit.
  // BE-encoded so `from_be` normalization recovers it on any host.
  let mut intended_y = vec![0xFFF0u16; 8 * 4];
  intended_y[8 + 1] = 0x0008;
  let y = be_encoded_u16_buf(&intended_y);
  let ok = be_encoded_u16_buf(&[0xFFF0u16; 8 * 4]);
  let e = Yuv444p12MsbBeFrame::try_new_checked(&y, &ok, &ok, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    Yuv444pMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), Yuv444pMsbFramePlane::Y);
      assert_eq!(p.value(), 0x0008);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn yuv444p12_msb_try_new_checked_accepts_value_with_low_10_set_but_not_low_4() {
  // 12-bit MSB only requires the low 4 to be zero. 0xFFF0 has bits 4..15 set
  // — valid at 12-bit even though it would fail the 10-bit (low-6) mask.
  let intended = vec![0xFFF0u16; 8 * 4];
  let p = le_encoded_u16_buf(&intended);
  Yuv444p12MsbLeFrame::try_new_checked(&p, &p, &p, 8, 4, 8, 8, 8)
    .expect("low-4-zero value valid at 12-bit");
}

#[test]
fn yuv444p10_msb_try_new_still_accepts_low_bit_contaminated_data() {
  // The geometry-only `try_new` does NOT scan samples, so low-bit-packed data
  // (which `try_new_checked` rejects) is still accepted. Pins that `try_new`
  // is unchanged.
  let buf = vec![0x03FFu16; 8 * 4]; // low-bit-packed white — stray low bits
  let f = Yuv444p10MsbLeFrame::try_new(&buf, &buf, &buf, 8, 4, 8, 8, 8)
    .expect("geometry-only try_new accepts");
  assert_eq!(f.y()[0], 0x03FF);
  let e = Yuv444p10MsbLeFrame::try_new_checked(&buf, &buf, &buf, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Yuv444pMsbFrameError::StrayLowBits(_)));
}

#[test]
fn yuv444p10_msb_try_new_checked_propagates_geometry_error() {
  // Geometry validation runs first and short-circuits before any scan.
  let (y, u, v) = yuv444p_msb_planes(8, 4);
  assert!(matches!(
    Yuv444p10MsbLeFrame::try_new_checked(&y, &u, &v, 0, 4, 8, 8, 8),
    Err(Yuv444pMsbFrameError::ZeroDimension(_))
  ));
}
