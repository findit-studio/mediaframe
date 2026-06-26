use crate::frame::{WidthAlignment, WidthAlignmentRequirement};

use super::super::{Nv20BeFrame, Nv20Frame, Nv20FrameError, Nv20FramePlane, Nv20LeFrame};
use super::util::{be_encoded_u16_buf, le_encoded_u16_buf};
use std::vec;

// 8×4 NV20 frame: Y is 8 u16/row × 4 rows; UV is half-width (4 pairs =
// 8 u16) at full height (4 rows) → 8 u16/row × 4 rows.
fn nv20_planes() -> (std::vec::Vec<u16>, std::vec::Vec<u16>) {
  (vec![0u16; 8 * 4], vec![0u16; 8 * 4])
}

#[test]
fn nv20_try_new_accepts_valid_tight() {
  let (y, uv) = nv20_planes();
  let f = Nv20LeFrame::try_new(&y, &uv, 8, 4, 8, 8).expect("valid");
  assert_eq!(f.width(), 8);
  assert_eq!(f.height(), 4);
  assert_eq!(f.y_stride(), 8);
  assert_eq!(f.uv_stride(), 8);
  assert!(!f.is_be());
}

#[test]
fn nv20_try_new_accepts_valid_padded_strides() {
  let y = vec![0u16; 16 * 4];
  let uv = vec![0u16; 16 * 4];
  let f = Nv20LeFrame::try_new(&y, &uv, 8, 4, 16, 16).expect("valid");
  assert_eq!(f.y_stride(), 16);
  assert_eq!(f.uv_stride(), 16);
}

#[test]
fn nv20_be_alias_reports_be() {
  let (y, uv) = nv20_planes();
  let f = Nv20BeFrame::try_new(&y, &uv, 8, 4, 8, 8).expect("valid");
  assert!(f.is_be());
}

#[test]
fn nv20_try_new_rejects_zero_dim() {
  let (y, uv) = nv20_planes();
  assert!(matches!(
    Nv20LeFrame::try_new(&y, &uv, 0, 4, 8, 8),
    Err(Nv20FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Nv20LeFrame::try_new(&y, &uv, 8, 0, 8, 8),
    Err(Nv20FrameError::ZeroDimension(_))
  ));
}

#[test]
fn nv20_try_new_rejects_odd_width() {
  let (y, uv) = nv20_planes();
  let e = Nv20LeFrame::try_new(&y, &uv, 7, 4, 8, 8).unwrap_err();
  assert!(matches!(
    e,
    Nv20FrameError::WidthAlignment(WidthAlignment {
      required: WidthAlignmentRequirement::Even,
      ..
    })
  ));
}

#[test]
fn nv20_try_new_accepts_odd_height() {
  // 4:2:2 chroma is full-height with no parity constraint on height.
  let y = vec![0u16; 8 * 3];
  let uv = vec![0u16; 8 * 3];
  let f = Nv20LeFrame::try_new(&y, &uv, 8, 3, 8, 8).expect("odd height ok");
  assert_eq!(f.height(), 3);
}

#[test]
fn nv20_try_new_rejects_short_y_stride() {
  let (y, uv) = nv20_planes();
  let e = Nv20LeFrame::try_new(&y, &uv, 8, 4, 7, 8).unwrap_err();
  assert!(matches!(e, Nv20FrameError::InsufficientYStride(_)));
}

#[test]
fn nv20_try_new_rejects_short_uv_stride() {
  let (y, uv) = nv20_planes();
  // uv_stride must be >= width (8) — the chroma row holds `width` u16.
  let e = Nv20LeFrame::try_new(&y, &uv, 8, 4, 8, 6).unwrap_err();
  assert!(matches!(e, Nv20FrameError::InsufficientUvStride(_)));
}

#[test]
fn nv20_try_new_rejects_odd_uv_stride() {
  // An odd u16-element stride (>= width) swaps U/V on alternate rows.
  let y = vec![0u16; 9 * 4];
  let uv = vec![0u16; 9 * 4];
  let e = Nv20LeFrame::try_new(&y, &uv, 8, 4, 9, 9).unwrap_err();
  assert!(matches!(e, Nv20FrameError::UvStrideOdd(_)));
}

#[test]
fn nv20_try_new_rejects_short_y_plane() {
  let y = vec![0u16; 8 * 3]; // one row short for height 4
  let uv = vec![0u16; 8 * 4];
  let e = Nv20LeFrame::try_new(&y, &uv, 8, 4, 8, 8).unwrap_err();
  assert!(matches!(e, Nv20FrameError::InsufficientYPlane(_)));
}

#[test]
fn nv20_try_new_rejects_short_uv_plane() {
  // 4:2:2 chroma is full-height: uv needs uv_stride * height samples.
  let y = vec![0u16; 8 * 4];
  let uv = vec![0u16; 8 * 3]; // one chroma row short
  let e = Nv20LeFrame::try_new(&y, &uv, 8, 4, 8, 8).unwrap_err();
  assert!(matches!(e, Nv20FrameError::InsufficientUvPlane(_)));
}

#[test]
fn nv20_accessors_round_trip() {
  let y = vec![1u16; 8 * 4];
  let uv = vec![2u16; 8 * 4];
  let f = Nv20Frame::<false>::new(&y, &uv, 8, 4, 8, 8);
  assert_eq!(f.y().len(), 8 * 4);
  assert_eq!(f.uv().len(), 8 * 4);
  assert_eq!(f.y()[0], 1);
  assert_eq!(f.uv()[0], 2);
}

#[test]
#[should_panic(expected = "invalid Nv20Frame")]
fn nv20_new_panics_on_invalid() {
  let (y, uv) = nv20_planes();
  let _ = Nv20LeFrame::new(&y, &uv, 0, 4, 8, 8);
}

// ---- Nv20Frame::try_new_checked --------------------------------------
//
// NV20 packs its 10 active bits in the LOW 10 of each `u16`; the high 6
// must be zero (`value & 0xFC00 == 0`). This is the exact inverse of the
// high-bit-packed P210 family, whose `try_new_checked` rejects non-zero
// LOW 6 bits. The host-independent `*_encoded_u16_buf` helpers build the
// wire byte layout so the asserted logical values hold on both LE and BE
// hosts after the validator's `from_le` / `from_be` normalization.

#[test]
fn nv20_try_new_checked_accepts_max_low_packed_value() {
  // 0x03FF = all ten low bits set, high six zero — the largest valid
  // NV20 sample. Must pass on both LE and BE hosts.
  let intended_y = vec![0x03FFu16; 8 * 4];
  let intended_uv = vec![0x03FFu16; 8 * 4];
  let y = le_encoded_u16_buf(&intended_y);
  let uv = le_encoded_u16_buf(&intended_uv);
  let f = Nv20LeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8).expect("0x03FF max valid passes");
  assert_eq!(f.width(), 8);
  assert_eq!(f.height(), 4);
}

#[test]
fn nv20_try_new_checked_rejects_y_high_bit_set_le() {
  // A single Y sample with a stray high bit (P210-shaped data wrongly
  // handed to NV20). LE-encoded so the logical 0x8000 holds on any host.
  let mut intended_y = vec![0x0040u16; 8 * 4];
  intended_y[3 * 8 + 5] = 0x8000;
  let y = le_encoded_u16_buf(&intended_y);
  let uv = le_encoded_u16_buf(&[0x0040u16; 8 * 4]);
  let e = Nv20LeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8).unwrap_err();
  match e {
    Nv20FrameError::StrayHighBits(p) => {
      assert_eq!(p.plane(), Nv20FramePlane::Y);
      assert_eq!(p.value(), 0x8000);
      assert_eq!(p.index(), 3 * 8 + 5);
    }
    other => panic!("expected StrayHighBits, got {other:?}"),
  }
}

#[test]
fn nv20_try_new_checked_rejects_y_high_bit_set_be() {
  // BE-encoded buffer: `from_be` normalization must recover the logical
  // 0x8000 on any host and reject it.
  let mut intended_y = vec![0x0040u16; 8 * 4];
  intended_y[8 + 2] = 0x8000;
  let y = be_encoded_u16_buf(&intended_y);
  let uv = be_encoded_u16_buf(&[0x0040u16; 8 * 4]);
  let e = Nv20BeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8).unwrap_err();
  match e {
    Nv20FrameError::StrayHighBits(p) => {
      assert_eq!(p.plane(), Nv20FramePlane::Y);
      assert_eq!(p.value(), 0x8000);
    }
    other => panic!("expected StrayHighBits, got {other:?}"),
  }
}

#[test]
fn nv20_try_new_checked_rejects_uv_high_bit_set_le() {
  // The offending sample is on the LAST chroma row (row 3), which only
  // exists because 4:2:2 chroma is full-height. A 4:2:0-style half-
  // height scan would stop at row 1 and miss it.
  let mut intended_uv = vec![0x0040u16; 8 * 4];
  intended_uv[3 * 8 + 7] = 0x0400; // bit 10 set — just above the 10-bit range
  let y = le_encoded_u16_buf(&[0x0040u16; 8 * 4]);
  let uv = le_encoded_u16_buf(&intended_uv);
  let e = Nv20LeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8).unwrap_err();
  match e {
    Nv20FrameError::StrayHighBits(p) => {
      assert_eq!(p.plane(), Nv20FramePlane::Uv);
      assert_eq!(p.value(), 0x0400);
      assert_eq!(p.index(), 3 * 8 + 7);
    }
    other => panic!("expected StrayHighBits, got {other:?}"),
  }
}

#[test]
fn nv20_try_new_checked_rejects_uv_high_bit_set_be() {
  let mut intended_uv = vec![0x0040u16; 8 * 4];
  intended_uv[2 * 8 + 4] = 0xFFFF;
  let y = be_encoded_u16_buf(&[0x0040u16; 8 * 4]);
  let uv = be_encoded_u16_buf(&intended_uv);
  let e = Nv20BeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8).unwrap_err();
  match e {
    Nv20FrameError::StrayHighBits(p) => {
      assert_eq!(p.plane(), Nv20FramePlane::Uv);
      assert_eq!(p.value(), 0xFFFF);
    }
    other => panic!("expected StrayHighBits, got {other:?}"),
  }
}

#[test]
fn nv20_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // Valid low-bit-packed NV20LE content (max 0x03FF) must be accepted on
  // both LE and BE hosts after `from_le` normalization.
  let y = le_encoded_u16_buf(&[0x03FFu16; 8 * 4]);
  let uv = le_encoded_u16_buf(&[0x0200u16; 8 * 4]);
  Nv20LeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8)
    .expect("LE-encoded valid NV20 must be accepted on both LE and BE hosts");
}

#[test]
fn nv20_try_new_still_accepts_high_bit_contaminated_data() {
  // The geometry-only `try_new` does NOT scan samples, so P210-shaped
  // high-bit data (which `try_new_checked` rejects) is still accepted.
  // This pins that `try_new` is unchanged.
  let y = vec![0xFFC0u16; 8 * 4]; // 1023 << 6 — P210 white, stray high bits
  let uv = vec![0x8000u16; 8 * 4];
  let f = Nv20LeFrame::try_new(&y, &uv, 8, 4, 8, 8).expect("geometry-only try_new accepts");
  assert_eq!(f.y()[0], 0xFFC0);
  // And the checked constructor rejects the very same buffer.
  let e = Nv20LeFrame::try_new_checked(&y, &uv, 8, 4, 8, 8).unwrap_err();
  assert!(matches!(e, Nv20FrameError::StrayHighBits(_)));
}

#[test]
fn nv20_try_new_checked_propagates_geometry_error() {
  // Geometry validation runs first and short-circuits before any scan.
  let (y, uv) = nv20_planes();
  assert!(matches!(
    Nv20LeFrame::try_new_checked(&y, &uv, 0, 4, 8, 8),
    Err(Nv20FrameError::ZeroDimension(_))
  ));
}
