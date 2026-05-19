use super::*;
use std::vec;

// ---- Rgb24Frame --------------------------------------------------------
//
// Single-plane 8-bit packed RGB. `stride` is in bytes (≥ 3 * width);
// `plane.len() >= stride * height`. No width parity constraint.

#[test]
fn rgb24_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 3];
  Rgb24Frame::try_new(&buf, 16, 4, 48).expect("valid");
}

#[test]
fn rgb24_frame_try_new_accepts_oversized_stride() {
  // stride > 3 * width (row padding) is allowed.
  let buf = vec![0u8; 64 * 4];
  Rgb24Frame::try_new(&buf, 16, 4, 64).expect("padded stride is valid");
}

#[test]
fn rgb24_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 3];
  assert!(matches!(
    Rgb24Frame::try_new(&buf, 0, 4, 48),
    Err(Rgb24FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Rgb24Frame::try_new(&buf, 16, 0, 48),
    Err(Rgb24FrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb24_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 3];
  assert!(matches!(
    Rgb24Frame::try_new(&buf, 16, 4, 47),
    Err(Rgb24FrameError::InsufficientStride(_))
  ));
}

#[test]
fn rgb24_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 3];
  assert!(matches!(
    Rgb24Frame::try_new(&small, 16, 4, 48),
    Err(Rgb24FrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Rgb24Frame")]
fn rgb24_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = Rgb24Frame::new(&buf, 16, 4, 48);
}

// ---- Bgr24Frame --------------------------------------------------------
//
// Mirrors Rgb24Frame: same single-plane layout, channel order is
// purely a marker / accessor distinction. Validation is identical in
// shape so we re-test the variants to catch typos in the parallel
// implementation.

#[test]
fn bgr24_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 3];
  Bgr24Frame::try_new(&buf, 16, 4, 48).expect("valid");
}

#[test]
fn bgr24_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 3];
  assert!(matches!(
    Bgr24Frame::try_new(&buf, 0, 4, 48),
    Err(Bgr24FrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr24_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 3];
  assert!(matches!(
    Bgr24Frame::try_new(&buf, 16, 4, 47),
    Err(Bgr24FrameError::InsufficientStride(_))
  ));
}

#[test]
fn bgr24_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 3];
  assert!(matches!(
    Bgr24Frame::try_new(&small, 16, 4, 48),
    Err(Bgr24FrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Bgr24Frame")]
fn bgr24_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = Bgr24Frame::new(&buf, 16, 4, 48);
}

// ---- RgbaFrame --------------------------------------------------------
//
// Single-plane 8-bit packed RGBA. `stride` is in bytes (≥ 4 * width);
// `plane.len() >= stride * height`. No width parity constraint.

#[test]
fn rgba_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  RgbaFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn rgba_frame_try_new_accepts_oversized_stride() {
  // stride > 4 * width (row padding) is allowed.
  let buf = vec![0u8; 96 * 4];
  RgbaFrame::try_new(&buf, 16, 4, 96).expect("padded stride is valid");
}

#[test]
fn rgba_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    RgbaFrame::try_new(&buf, 0, 4, 64),
    Err(RgbaFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    RgbaFrame::try_new(&buf, 16, 0, 64),
    Err(RgbaFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgba_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    RgbaFrame::try_new(&buf, 16, 4, 63),
    Err(RgbaFrameError::InsufficientStride(_))
  ));
}

#[test]
fn rgba_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    RgbaFrame::try_new(&small, 16, 4, 64),
    Err(RgbaFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid RgbaFrame")]
fn rgba_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = RgbaFrame::new(&buf, 16, 4, 64);
}

// ---- BgraFrame --------------------------------------------------------
//
// Mirrors RgbaFrame: same single-plane layout, channel order is
// purely a marker / accessor distinction. Validation is identical in
// shape so we re-test the variants to catch typos in the parallel
// implementation.

#[test]
fn bgra_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  BgraFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn bgra_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    BgraFrame::try_new(&buf, 0, 4, 64),
    Err(BgraFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgra_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    BgraFrame::try_new(&buf, 16, 4, 63),
    Err(BgraFrameError::InsufficientStride(_))
  ));
}

#[test]
fn bgra_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    BgraFrame::try_new(&small, 16, 4, 64),
    Err(BgraFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid BgraFrame")]
fn bgra_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = BgraFrame::new(&buf, 16, 4, 64);
}

// ---- ArgbFrame --------------------------------------------------------
//
// Single-plane 8-bit packed ARGB. `stride` is in bytes (≥ 4 * width);
// `plane.len() >= stride * height`. No width parity constraint.

#[test]
fn argb_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  ArgbFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn argb_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u8; 96 * 4];
  ArgbFrame::try_new(&buf, 16, 4, 96).expect("padded stride is valid");
}

#[test]
fn argb_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    ArgbFrame::try_new(&buf, 0, 4, 64),
    Err(ArgbFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    ArgbFrame::try_new(&buf, 16, 0, 64),
    Err(ArgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn argb_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    ArgbFrame::try_new(&buf, 16, 4, 63),
    Err(ArgbFrameError::InsufficientStride(_))
  ));
}

#[test]
fn argb_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    ArgbFrame::try_new(&small, 16, 4, 64),
    Err(ArgbFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid ArgbFrame")]
fn argb_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = ArgbFrame::new(&buf, 16, 4, 64);
}

// ---- AbgrFrame --------------------------------------------------------

#[test]
fn abgr_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  AbgrFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn abgr_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    AbgrFrame::try_new(&buf, 0, 4, 64),
    Err(AbgrFrameError::ZeroDimension(_))
  ));
}

#[test]
fn abgr_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    AbgrFrame::try_new(&buf, 16, 4, 63),
    Err(AbgrFrameError::InsufficientStride(_))
  ));
}

#[test]
fn abgr_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    AbgrFrame::try_new(&small, 16, 4, 64),
    Err(AbgrFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid AbgrFrame")]
fn abgr_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = AbgrFrame::new(&buf, 16, 4, 64);
}

// ---- Padding-byte family (Ship 9d) -----------------------------------
//
// 4-byte single-plane formats with one ignored padding byte. Frame
// validation is the same shape as RgbaFrame/BgraFrame (4 bpp); each
// variant tested for at least one rejection path to catch typos.

#[test]
fn xrgb_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  XrgbFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn xrgb_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    XrgbFrame::try_new(&small, 16, 4, 64),
    Err(XrgbFrameError::InsufficientPlane(_))
  ));
}

#[test]
fn xrgb_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    XrgbFrame::try_new(&buf, 0, 4, 64),
    Err(XrgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn xrgb_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4 * 4];
  assert!(matches!(
    XrgbFrame::try_new(&buf, 16, 4, 63),
    Err(XrgbFrameError::InsufficientStride(_))
  ));
}

#[test]
#[should_panic(expected = "invalid XrgbFrame")]
fn xrgb_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = XrgbFrame::new(&buf, 16, 4, 64);
}

#[test]
fn rgbx_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  RgbxFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn rgbx_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    RgbxFrame::try_new(&small, 16, 4, 64),
    Err(RgbxFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid RgbxFrame")]
fn rgbx_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = RgbxFrame::new(&buf, 16, 4, 64);
}

#[test]
fn xbgr_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  XbgrFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn xbgr_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    XbgrFrame::try_new(&small, 16, 4, 64),
    Err(XbgrFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid XbgrFrame")]
fn xbgr_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = XbgrFrame::new(&buf, 16, 4, 64);
}

#[test]
fn bgrx_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4 * 4];
  BgrxFrame::try_new(&buf, 16, 4, 64).expect("valid");
}

#[test]
fn bgrx_frame_try_new_rejects_short_plane() {
  let small = vec![0u8; 16 * 4];
  assert!(matches!(
    BgrxFrame::try_new(&small, 16, 4, 64),
    Err(BgrxFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid BgrxFrame")]
fn bgrx_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 10];
  let _ = BgrxFrame::new(&buf, 16, 4, 64);
}
