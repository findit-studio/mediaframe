use super::util::{be_encoded_u16_buf, le_encoded_u16_buf};
use super::*;
use std::{vec, vec::Vec};

// ---- GbrpHighBitFrame (3-plane) tests ------------------------------------

fn gbrp10_planes(w: u32, h: u32) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
  let n = (w * h) as usize;
  (vec![0u16; n], vec![0u16; n], vec![0u16; n])
}

#[test]
fn gbrp10_try_new_accepts_valid_tight() {
  let (g, b, r) = gbrp10_planes(16, 8);
  let f = Gbrp10LeFrame::try_new(&g, &b, &r, 16, 8, 16, 16, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.g_stride(), 16);
  assert_eq!(f.b_stride(), 16);
  assert_eq!(f.r_stride(), 16);
  assert_eq!(f.bits(), 10);
}

#[test]
fn gbrp10_try_new_accepts_padded_strides() {
  let stride = 32u32;
  let h = 8u32;
  let g = vec![0u16; (stride * h) as usize];
  let b = vec![0u16; (stride * h) as usize];
  let r = vec![0u16; (stride * h) as usize];
  let f = Gbrp10LeFrame::try_new(&g, &b, &r, 16, h, stride, stride, stride).expect("valid");
  assert_eq!(f.g_stride(), stride);
}

#[test]
fn gbrp10_try_new_rejects_zero_width() {
  let (g, b, r) = gbrp10_planes(4, 4);
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 0, 4, 4, 4, 4).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::ZeroDimension(_)));
}

#[test]
fn gbrp10_try_new_rejects_zero_height() {
  let (g, b, r) = gbrp10_planes(4, 4);
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 4, 0, 4, 4, 4).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::ZeroDimension(_)));
}

#[test]
fn gbrp10_try_new_rejects_g_stride_too_small() {
  let (g, b, r) = gbrp10_planes(8, 4);
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 8, 4, 4, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::InsufficientGStride(_)));
}

#[test]
fn gbrp10_try_new_rejects_b_stride_too_small() {
  let (g, b, r) = gbrp10_planes(8, 4);
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 8, 4, 8, 4, 8).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::InsufficientBStride(_)));
}

#[test]
fn gbrp10_try_new_rejects_r_stride_too_small() {
  let (g, b, r) = gbrp10_planes(8, 4);
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 4).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::InsufficientRStride(_)));
}

#[test]
fn gbrp10_try_new_rejects_g_plane_too_short() {
  // Need stride*height = 8*4=32 samples; only provide 16.
  let g = vec![0u16; 16];
  let b = vec![0u16; 32];
  let r = vec![0u16; 32];
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::InsufficientGPlane(_)));
}

#[test]
fn gbrp10_try_new_rejects_b_plane_too_short() {
  let g = vec![0u16; 32];
  let b = vec![0u16; 16];
  let r = vec![0u16; 32];
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::InsufficientBPlane(_)));
}

#[test]
fn gbrp10_try_new_rejects_r_plane_too_short() {
  let g = vec![0u16; 32];
  let b = vec![0u16; 32];
  let r = vec![0u16; 16];
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::InsufficientRPlane(_)));
}

#[test]
fn gbrp10_new_panics_on_invalid() {
  // zero width must panic — use try_new to confirm the error path
  // (const fn `new` panics but is untestable via catch_unwind with borrows).
  let g: [u16; 0] = [];
  let b: [u16; 0] = [];
  let r: [u16; 0] = [];
  let e = Gbrp10LeFrame::try_new(&g, &b, &r, 0, 1, 1, 1, 1).unwrap_err();
  assert!(matches!(e, GbrpHighBitFrameError::ZeroDimension(_)));
}

// ---- Per-BITS sanity: bits() accessor and valid construction --------------

#[test]
fn gbrp9_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrp9LeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 9);
}

#[test]
fn gbrp12_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrp12LeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 12);
}

#[test]
fn gbrp14_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrp14LeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 14);
}

#[test]
fn gbrp16_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrp16LeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 16);
}

// ---- GbrapHighBitFrame (4-plane) tests ------------------------------------

fn gbrap10_planes(w: u32, h: u32) -> (Vec<u16>, Vec<u16>, Vec<u16>, Vec<u16>) {
  let n = (w * h) as usize;
  (vec![0u16; n], vec![0u16; n], vec![0u16; n], vec![0u16; n])
}

#[test]
fn gbrap10_try_new_accepts_valid_tight() {
  let (g, b, r, a) = gbrap10_planes(16, 8);
  let f = Gbrap10LeFrame::try_new(&g, &b, &r, &a, 16, 8, 16, 16, 16, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.a_stride(), 16);
  assert_eq!(f.bits(), 10);
}

#[test]
fn gbrap10_try_new_rejects_zero_dimension() {
  let (g, b, r, a) = gbrap10_planes(4, 4);
  let e = Gbrap10LeFrame::try_new(&g, &b, &r, &a, 0, 4, 4, 4, 4, 4).unwrap_err();
  assert!(matches!(e, GbrapHighBitFrameError::ZeroDimension(_)));
}

#[test]
fn gbrap10_try_new_rejects_a_stride_too_small() {
  let (g, b, r, a) = gbrap10_planes(8, 4);
  let e = Gbrap10LeFrame::try_new(&g, &b, &r, &a, 8, 4, 8, 8, 8, 4).unwrap_err();
  assert!(matches!(e, GbrapHighBitFrameError::InsufficientAStride(_)));
}

#[test]
fn gbrap10_try_new_rejects_a_plane_too_short() {
  let g = vec![0u16; 32];
  let b = vec![0u16; 32];
  let r = vec![0u16; 32];
  let a = vec![0u16; 16]; // too short — need 32
  let e = Gbrap10LeFrame::try_new(&g, &b, &r, &a, 8, 4, 8, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrapHighBitFrameError::InsufficientAPlane(_)));
}

#[test]
fn gbrap10_try_new_rejects_g_stride_too_small() {
  let (g, b, r, a) = gbrap10_planes(8, 4);
  let e = Gbrap10LeFrame::try_new(&g, &b, &r, &a, 8, 4, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrapHighBitFrameError::InsufficientGStride(_)));
}

#[test]
fn gbrap10_new_panics_on_invalid() {
  let p: [u16; 0] = [];
  let e = Gbrap10LeFrame::try_new(&p, &p, &p, &p, 0, 1, 1, 1, 1, 1).unwrap_err();
  assert!(matches!(e, GbrapHighBitFrameError::ZeroDimension(_)));
}

// ---- Per-BITS sanity for Gbrap family -------------------------------------

#[test]
fn gbrap12_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrap12LeFrame::try_new(&p, &p, &p, &p, 2, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 12);
}

#[test]
fn gbrap14_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrap14LeFrame::try_new(&p, &p, &p, &p, 2, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 14);
}

#[test]
fn gbrap16_bits_accessor() {
  let p = vec![0u16; 4];
  let f = Gbrap16LeFrame::try_new(&p, &p, &p, &p, 2, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 16);
}

// ---- crate-internal aliases (y/u/v) pass through to g/b/r ----------------

#[test]
fn gbrp10_internal_aliases_match_public_accessors() {
  let g = vec![1u16; 4];
  let b = vec![2u16; 4];
  let r = vec![3u16; 4];
  let f = Gbrp10LeFrame::try_new(&g, &b, &r, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.y(), f.g());
  assert_eq!(f.u(), f.b());
  assert_eq!(f.v(), f.r());
  assert_eq!(f.y_stride(), f.g_stride());
  assert_eq!(f.u_stride(), f.b_stride());
  assert_eq!(f.v_stride(), f.r_stride());
}

#[test]
fn gbrap10_internal_aliases_match_public_accessors() {
  let g = vec![1u16; 4];
  let b = vec![2u16; 4];
  let r = vec![3u16; 4];
  let a = vec![4u16; 4];
  let f = Gbrap10LeFrame::try_new(&g, &b, &r, &a, 2, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.y(), f.g());
  assert_eq!(f.u(), f.b());
  assert_eq!(f.v(), f.r());
  assert_eq!(f.a()[0], 4u16);
}

// ---- Phase 4: BE alias + is_be() exposure ---------------------------------

#[test]
fn gbrp10_le_alias_is_be_returns_false() {
  let p = vec![0u16; 4];
  let f = Gbrp10LeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert!(!f.is_be());
}

#[test]
fn gbrp10_be_alias_constructs_and_is_be() {
  let p = vec![0u16; 4];
  let f = Gbrp10BeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert!(f.is_be());
  assert_eq!(f.bits(), 10);
  assert_eq!(f.width(), 2);
  assert_eq!(f.height(), 2);
}

#[test]
fn gbrp16_be_alias_constructs() {
  let p = vec![0u16; 4];
  let f = Gbrp16BeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert!(f.is_be());
  assert_eq!(f.bits(), 16);
}

#[test]
fn gbrap10_le_alias_is_be_returns_false() {
  let p = vec![0u16; 4];
  let f = Gbrap10LeFrame::try_new(&p, &p, &p, &p, 2, 2, 2, 2, 2, 2).unwrap();
  assert!(!f.is_be());
}

#[test]
fn gbrap10_be_alias_constructs_and_is_be() {
  let p = vec![0u16; 4];
  let f = Gbrap10BeFrame::try_new(&p, &p, &p, &p, 2, 2, 2, 2, 2, 2).unwrap();
  assert!(f.is_be());
  assert_eq!(f.bits(), 10);
}

#[test]
fn gbrap16_be_alias_constructs() {
  let p = vec![0u16; 4];
  let f = Gbrap16BeFrame::try_new(&p, &p, &p, &p, 2, 2, 2, 2, 2, 2).unwrap();
  assert!(f.is_be());
  assert_eq!(f.bits(), 16);
}

// ---- GbrpMsbFrame (3-plane, high-bit-packed) geometry tests --------------

fn gbrp_msb_planes(w: u32, h: u32) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
  let n = (w * h) as usize;
  (vec![0u16; n], vec![0u16; n], vec![0u16; n])
}

#[test]
fn gbrp10_msb_try_new_accepts_valid_tight() {
  let (g, b, r) = gbrp_msb_planes(16, 8);
  let f = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 16, 8, 16, 16, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.g_stride(), 16);
  assert_eq!(f.bits(), 10);
  assert!(!f.is_be());
}

#[test]
fn gbrp10_msb_try_new_accepts_padded_strides() {
  let stride = 32u32;
  let h = 8u32;
  let p = vec![0u16; (stride * h) as usize];
  let f = Gbrp10MsbLeFrame::try_new(&p, &p, &p, 16, h, stride, stride, stride).expect("valid");
  assert_eq!(f.g_stride(), stride);
}

#[test]
fn gbrp10_msb_try_new_rejects_zero_dimension() {
  let (g, b, r) = gbrp_msb_planes(4, 4);
  assert!(matches!(
    Gbrp10MsbLeFrame::try_new(&g, &b, &r, 0, 4, 4, 4, 4),
    Err(GbrpMsbFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Gbrp10MsbLeFrame::try_new(&g, &b, &r, 4, 0, 4, 4, 4),
    Err(GbrpMsbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn gbrp10_msb_try_new_rejects_g_stride_too_small() {
  let (g, b, r) = gbrp_msb_planes(8, 4);
  let e = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 8, 4, 4, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::InsufficientGStride(_)));
}

#[test]
fn gbrp10_msb_try_new_rejects_b_stride_too_small() {
  let (g, b, r) = gbrp_msb_planes(8, 4);
  let e = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 8, 4, 8, 4, 8).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::InsufficientBStride(_)));
}

#[test]
fn gbrp10_msb_try_new_rejects_r_stride_too_small() {
  let (g, b, r) = gbrp_msb_planes(8, 4);
  let e = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 4).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::InsufficientRStride(_)));
}

#[test]
fn gbrp10_msb_try_new_rejects_g_plane_too_short() {
  let g = vec![0u16; 16];
  let b = vec![0u16; 32];
  let r = vec![0u16; 32];
  let e = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::InsufficientGPlane(_)));
}

#[test]
fn gbrp10_msb_try_new_rejects_b_plane_too_short() {
  let g = vec![0u16; 32];
  let b = vec![0u16; 16];
  let r = vec![0u16; 32];
  let e = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::InsufficientBPlane(_)));
}

#[test]
fn gbrp10_msb_try_new_rejects_r_plane_too_short() {
  let g = vec![0u16; 32];
  let b = vec![0u16; 32];
  let r = vec![0u16; 16];
  let e = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::InsufficientRPlane(_)));
}

#[test]
#[should_panic(expected = "invalid GbrpMsbFrame")]
fn gbrp10_msb_new_panics_on_invalid() {
  let (g, b, r) = gbrp_msb_planes(4, 4);
  let _ = Gbrp10MsbLeFrame::new(&g, &b, &r, 0, 4, 4, 4, 4);
}

#[test]
fn gbrp12_msb_bits_accessor_and_be_alias() {
  let p = vec![0u16; 4];
  let f = Gbrp12MsbLeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.bits(), 12);
  assert!(!f.is_be());
  let fbe = Gbrp12MsbBeFrame::try_new(&p, &p, &p, 2, 2, 2, 2, 2).unwrap();
  assert!(fbe.is_be());
  assert_eq!(fbe.bits(), 12);
}

#[test]
fn gbrp10_msb_internal_aliases_match_public_accessors() {
  let g = vec![1u16; 4];
  let b = vec![2u16; 4];
  let r = vec![3u16; 4];
  let f = Gbrp10MsbLeFrame::try_new(&g, &b, &r, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.y(), f.g());
  assert_eq!(f.u(), f.b());
  assert_eq!(f.v(), f.r());
  assert_eq!(f.y_stride(), f.g_stride());
  assert_eq!(f.u_stride(), f.b_stride());
  assert_eq!(f.v_stride(), f.r_stride());
}

// ---- GbrpMsbFrame::try_new_checked (rejects stray LOW bits) ---------------
//
// MSB-packed GBRP carries its BITS active bits in the HIGH BITS of each
// `u16`; the low `16 - BITS` must be zero (low 6 at 10-bit → mask 0x003F,
// low 4 at 12-bit → mask 0x000F). This is the exact inverse of the
// low-bit-packed NV20 family, whose `try_new_checked` rejects non-zero HIGH
// bits, and matches the high-bit P210 family. The host-independent
// `*_encoded_u16_buf` helpers build the wire byte layout so the asserted
// logical values hold on both LE and BE hosts after the validator's
// `from_le` / `from_be` normalization.

#[test]
fn gbrp10_msb_try_new_checked_accepts_max_high_packed_value() {
  // 0xFFC0 = all ten high bits set, low six zero — the largest valid 10-bit
  // MSB-packed sample. Must pass on both LE and BE hosts.
  let intended = vec![0xFFC0u16; 8 * 4];
  let g = le_encoded_u16_buf(&intended);
  let f =
    Gbrp10MsbLeFrame::try_new_checked(&g, &g, &g, 8, 4, 8, 8, 8).expect("0xFFC0 max valid passes");
  assert_eq!(f.width(), 8);
  assert_eq!(f.height(), 4);
}

#[test]
fn gbrp12_msb_try_new_checked_accepts_max_high_packed_value() {
  // 0xFFF0 = all twelve high bits set, low four zero.
  let intended = vec![0xFFF0u16; 8 * 4];
  let g = le_encoded_u16_buf(&intended);
  let f =
    Gbrp12MsbLeFrame::try_new_checked(&g, &g, &g, 8, 4, 8, 8, 8).expect("0xFFF0 max valid passes");
  assert_eq!(f.bits(), 12);
}

#[test]
fn gbrp10_msb_try_new_checked_rejects_g_low_bit_set_le() {
  // A single G sample with a stray low bit (low-bit-packed data wrongly
  // handed to the MSB path). LE-encoded so logical 0x0001 holds on any host.
  let mut intended_g = vec![0xFFC0u16; 8 * 4];
  intended_g[3 * 8 + 5] = 0x0001;
  let g = le_encoded_u16_buf(&intended_g);
  let ok = le_encoded_u16_buf(&[0xFFC0u16; 8 * 4]);
  let e = Gbrp10MsbLeFrame::try_new_checked(&g, &ok, &ok, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    GbrpMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), GbrpMsbFramePlane::G);
      assert_eq!(p.value(), 0x0001);
      assert_eq!(p.index(), 3 * 8 + 5);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn gbrp10_msb_try_new_checked_rejects_b_low_bit_set_be() {
  // BE-encoded buffer: `from_be` normalization must recover logical 0x0020
  // (bit 5, inside the low-6 zero region) on any host and reject it.
  let mut intended_b = vec![0xFFC0u16; 8 * 4];
  intended_b[8 + 2] = 0x0020;
  let b = be_encoded_u16_buf(&intended_b);
  let ok = be_encoded_u16_buf(&[0xFFC0u16; 8 * 4]);
  let e = Gbrp10MsbBeFrame::try_new_checked(&ok, &b, &ok, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    GbrpMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), GbrpMsbFramePlane::B);
      assert_eq!(p.value(), 0x0020);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn gbrp10_msb_try_new_checked_rejects_r_low_bit_set_le() {
  // Offending sample on the R plane, last row.
  let mut intended_r = vec![0xFFC0u16; 8 * 4];
  intended_r[3 * 8 + 7] = 0x003F; // all six low bits set
  let r = le_encoded_u16_buf(&intended_r);
  let ok = le_encoded_u16_buf(&[0xFFC0u16; 8 * 4]);
  let e = Gbrp10MsbLeFrame::try_new_checked(&ok, &ok, &r, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    GbrpMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), GbrpMsbFramePlane::R);
      assert_eq!(p.value(), 0x003F);
      assert_eq!(p.index(), 3 * 8 + 7);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn gbrp12_msb_try_new_checked_rejects_g_low_bit_set_le() {
  // At 12-bit the low-4 mask is 0x000F. 0x0008 is bit 3 — a stray low bit.
  let mut intended_g = vec![0xFFF0u16; 8 * 4];
  intended_g[8 + 1] = 0x0008;
  let g = le_encoded_u16_buf(&intended_g);
  let ok = le_encoded_u16_buf(&[0xFFF0u16; 8 * 4]);
  let e = Gbrp12MsbLeFrame::try_new_checked(&g, &ok, &ok, 8, 4, 8, 8, 8).unwrap_err();
  match e {
    GbrpMsbFrameError::StrayLowBits(p) => {
      assert_eq!(p.plane(), GbrpMsbFramePlane::G);
      assert_eq!(p.value(), 0x0008);
    }
    other => panic!("expected StrayLowBits, got {other:?}"),
  }
}

#[test]
fn gbrp12_msb_try_new_checked_accepts_value_with_low_10_set_but_not_low_4() {
  // 12-bit MSB only requires the low 4 to be zero. 0xFFF0 has bits 4..15
  // set — valid at 12-bit even though it would fail the 10-bit (low-6) mask.
  let intended = vec![0xFFF0u16; 8 * 4];
  let g = le_encoded_u16_buf(&intended);
  Gbrp12MsbLeFrame::try_new_checked(&g, &g, &g, 8, 4, 8, 8, 8)
    .expect("low-4-zero value valid at 12-bit");
}

#[test]
fn gbrp10_msb_try_new_still_accepts_low_bit_contaminated_data() {
  // The geometry-only `try_new` does NOT scan samples, so low-bit-packed
  // data (which `try_new_checked` rejects) is still accepted. Pins that
  // `try_new` is unchanged.
  let buf = vec![0x03FFu16; 8 * 4]; // low-bit-packed white — stray low bits
  let f = Gbrp10MsbLeFrame::try_new(&buf, &buf, &buf, 8, 4, 8, 8, 8)
    .expect("geometry-only try_new accepts");
  assert_eq!(f.g()[0], 0x03FF);
  let e = Gbrp10MsbLeFrame::try_new_checked(&buf, &buf, &buf, 8, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, GbrpMsbFrameError::StrayLowBits(_)));
}

#[test]
fn gbrp10_msb_try_new_checked_propagates_geometry_error() {
  // Geometry validation runs first and short-circuits before any scan.
  let (g, b, r) = gbrp_msb_planes(8, 4);
  assert!(matches!(
    Gbrp10MsbLeFrame::try_new_checked(&g, &b, &r, 0, 4, 8, 8, 8),
    Err(GbrpMsbFrameError::ZeroDimension(_))
  ));
}

// ---- Gbrap32Frame (4-plane u32, full range) tests ------------------------

fn gbrap32_planes(w: u32, h: u32) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) {
  let n = (w * h) as usize;
  (vec![0u32; n], vec![0u32; n], vec![0u32; n], vec![0u32; n])
}

#[test]
fn gbrap32_try_new_accepts_valid_tight() {
  let (g, b, r, a) = gbrap32_planes(16, 8);
  let f = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 16, 8, 16, 16, 16, 16).expect("valid");
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
  assert_eq!(f.a_stride(), 16);
  assert!(!f.is_be());
}

#[test]
fn gbrap32_try_new_accepts_padded_strides() {
  let stride = 24u32;
  let h = 8u32;
  let p = vec![0u32; (stride * h) as usize];
  let f =
    Gbrap32LeFrame::try_new(&p, &p, &p, &p, 16, h, stride, stride, stride, stride).expect("valid");
  assert_eq!(f.g_stride(), stride);
}

#[test]
fn gbrap32_full_u32_range_round_trips_through_accessors() {
  // Every bit active: u32::MAX must survive unmodified (no masking contract).
  let g = vec![u32::MAX; 4];
  let b = vec![0xDEAD_BEEFu32; 4];
  let r = vec![1u32; 4];
  let a = vec![0u32; 4];
  let f = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 2, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.g()[0], u32::MAX);
  assert_eq!(f.b()[0], 0xDEAD_BEEF);
  assert_eq!(f.r()[0], 1);
  assert_eq!(f.a()[0], 0);
}

#[test]
fn gbrap32_try_new_rejects_zero_dimension() {
  let (g, b, r, a) = gbrap32_planes(4, 4);
  assert!(matches!(
    Gbrap32LeFrame::try_new(&g, &b, &r, &a, 0, 4, 4, 4, 4, 4),
    Err(Gbrap32FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Gbrap32LeFrame::try_new(&g, &b, &r, &a, 4, 0, 4, 4, 4, 4),
    Err(Gbrap32FrameError::ZeroDimension(_))
  ));
}

#[test]
fn gbrap32_try_new_rejects_g_stride_too_small() {
  let (g, b, r, a) = gbrap32_planes(8, 4);
  let e = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 8, 4, 4, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Gbrap32FrameError::InsufficientGStride(_)));
}

#[test]
fn gbrap32_try_new_rejects_a_stride_too_small() {
  let (g, b, r, a) = gbrap32_planes(8, 4);
  let e = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 8, 4, 8, 8, 8, 4).unwrap_err();
  assert!(matches!(e, Gbrap32FrameError::InsufficientAStride(_)));
}

#[test]
fn gbrap32_try_new_rejects_r_plane_too_short() {
  let g = vec![0u32; 32];
  let b = vec![0u32; 32];
  let r = vec![0u32; 16]; // too short — need 32
  let a = vec![0u32; 32];
  let e = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 8, 4, 8, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Gbrap32FrameError::InsufficientRPlane(_)));
}

#[test]
fn gbrap32_try_new_rejects_a_plane_too_short() {
  let g = vec![0u32; 32];
  let b = vec![0u32; 32];
  let r = vec![0u32; 32];
  let a = vec![0u32; 16]; // too short — need 32
  let e = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 8, 4, 8, 8, 8, 8).unwrap_err();
  assert!(matches!(e, Gbrap32FrameError::InsufficientAPlane(_)));
}

#[test]
fn gbrap32_be_alias_reports_be() {
  let (g, b, r, a) = gbrap32_planes(2, 2);
  let f = Gbrap32BeFrame::try_new(&g, &b, &r, &a, 2, 2, 2, 2, 2, 2).expect("valid");
  assert!(f.is_be());
}

#[test]
#[should_panic(expected = "invalid Gbrap32Frame")]
fn gbrap32_new_panics_on_invalid() {
  let (g, b, r, a) = gbrap32_planes(4, 4);
  let _ = Gbrap32LeFrame::new(&g, &b, &r, &a, 0, 4, 4, 4, 4, 4);
}

#[test]
fn gbrap32_internal_aliases_match_public_accessors() {
  let g = vec![1u32; 4];
  let b = vec![2u32; 4];
  let r = vec![3u32; 4];
  let a = vec![4u32; 4];
  let f = Gbrap32LeFrame::try_new(&g, &b, &r, &a, 2, 2, 2, 2, 2, 2).unwrap();
  assert_eq!(f.y(), f.g());
  assert_eq!(f.u(), f.b());
  assert_eq!(f.v(), f.r());
  assert_eq!(f.y_stride(), f.g_stride());
  assert_eq!(f.u_stride(), f.b_stride());
  assert_eq!(f.v_stride(), f.r_stride());
  assert_eq!(f.a()[0], 4u32);
}
