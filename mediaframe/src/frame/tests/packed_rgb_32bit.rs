use super::*;
use std::vec;

// ---- Rgb96Frame tests --------------------------------------------------------

#[test]
fn rgb96_try_new_happy_path() {
  // width=2, stride=6, height=3 → plane needs 18 u32 elements
  let buf = vec![0u32; 18];
  let f = Rgb96LeFrame::try_new(&buf, 2, 3, 6).unwrap();
  assert_eq!(f.width(), 2);
  assert_eq!(f.height(), 3);
  assert_eq!(f.stride(), 6);
  assert_eq!(f.rgb96().len(), 18);
  assert!(!f.is_be());
}

#[test]
fn rgb96_stride_too_small() {
  let buf = vec![0u32; 18];
  // stride=5 < 3*2=6
  assert!(Rgb96LeFrame::try_new(&buf, 2, 3, 5).is_err());
}

#[test]
fn rgb96_plane_too_short() {
  // stride=6, height=3 → need 18; supply only 17
  let buf = vec![0u32; 17];
  assert!(Rgb96LeFrame::try_new(&buf, 2, 3, 6).is_err());
}

#[test]
fn rgb96_zero_dimension() {
  let buf = vec![0u32; 18];
  assert!(Rgb96LeFrame::try_new(&buf, 0, 3, 6).is_err());
  assert!(Rgb96LeFrame::try_new(&buf, 2, 0, 6).is_err());
}

#[test]
fn rgb96_be_frame_alias_constructs() {
  let buf = vec![0u32; 18];
  let f = Rgb96BeFrame::try_new(&buf, 2, 3, 6).unwrap();
  assert!(f.is_be());
  assert_eq!(f.width(), 2);
  assert_eq!(f.height(), 3);
}

#[test]
fn rgb96_try_new_rejects_width_overflow() {
  let buf = vec![0u32; 0];
  let too_big = (u32::MAX / 3) + 1;
  assert!(matches!(
    Rgb96LeFrame::try_new(&buf, too_big, 1, u32::MAX),
    Err(Rgb96FrameError::WidthOverflow(p)) if p.width() == too_big
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb96_try_new_rejects_geometry_overflow() {
  // Only meaningful on 32-bit targets (wasm32, i686) where
  // `stride * height` as `usize` can overflow. Pick a width small
  // enough that `3 * width <= stride` so we pass the InsufficientStride
  // check and reach the geometry-overflow check.
  let buf: [u32; 0] = [];
  let width: u32 = 0x5555; // 3 * width = 0xFFFF, ≤ stride
  let stride: u32 = 0x1_0000;
  let height: u32 = 0x1_0000; // stride * height = 2^32 → overflows usize on 32-bit
  let res = Rgb96LeFrame::try_new(&buf, width, height, stride);
  assert!(
    matches!(res, Err(Rgb96FrameError::GeometryOverflow(_))),
    "expected GeometryOverflow, got {:?}",
    res
  );
}

#[test]
#[should_panic(expected = "invalid Rgb96Frame")]
fn rgb96_new_panics_on_invalid() {
  let buf = vec![0u32; 10];
  let _ = Rgb96LeFrame::new(&buf, 2, 3, 6);
}

// ---- Rgba128Frame tests ------------------------------------------------------

#[test]
fn rgba128_try_new_happy_path() {
  // width=2, stride=8 (4*2), height=3 → plane needs 24 u32 elements
  let buf = vec![0u32; 24];
  let f = Rgba128LeFrame::try_new(&buf, 2, 3, 8).unwrap();
  assert_eq!(f.width(), 2);
  assert_eq!(f.height(), 3);
  assert_eq!(f.stride(), 8);
  assert_eq!(f.rgba128().len(), 24);
  assert!(!f.is_be());
}

#[test]
fn rgba128_stride_too_small() {
  let buf = vec![0u32; 24];
  // stride=7 < 4*2=8
  assert!(Rgba128LeFrame::try_new(&buf, 2, 3, 7).is_err());
}

#[test]
fn rgba128_plane_too_short() {
  // stride=8, height=3 → need 24; supply only 23
  let buf = vec![0u32; 23];
  assert!(Rgba128LeFrame::try_new(&buf, 2, 3, 8).is_err());
}

#[test]
fn rgba128_zero_dimension() {
  let buf = vec![0u32; 24];
  assert!(Rgba128LeFrame::try_new(&buf, 0, 3, 8).is_err());
  assert!(Rgba128LeFrame::try_new(&buf, 2, 0, 8).is_err());
}

#[test]
fn rgba128_be_frame_alias_constructs() {
  let buf = vec![0u32; 24];
  let f = Rgba128BeFrame::try_new(&buf, 2, 3, 8).unwrap();
  assert!(f.is_be());
}

#[test]
fn rgba128_try_new_rejects_width_overflow() {
  let buf = vec![0u32; 0];
  let too_big = (u32::MAX / 4) + 1;
  assert!(matches!(
    Rgba128LeFrame::try_new(&buf, too_big, 1, u32::MAX),
    Err(Rgba128FrameError::WidthOverflow(p)) if p.width() == too_big
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgba128_try_new_rejects_geometry_overflow() {
  // Only meaningful on 32-bit targets (wasm32, i686) where
  // `stride * height` as `usize` can overflow. Pick a width small
  // enough that `4 * width <= stride` so we pass the InsufficientStride
  // check and reach the geometry-overflow check.
  let buf: [u32; 0] = [];
  let width: u32 = 0x3FFF; // 4 * width = 0xFFFC, ≤ stride
  let stride: u32 = 0x1_0000;
  let height: u32 = 0x1_0000; // stride * height = 2^32 → overflows usize on 32-bit
  let res = Rgba128LeFrame::try_new(&buf, width, height, stride);
  assert!(
    matches!(res, Err(Rgba128FrameError::GeometryOverflow(_))),
    "expected GeometryOverflow, got {:?}",
    res
  );
}

#[test]
#[should_panic(expected = "invalid Rgba128Frame")]
fn rgba128_new_panics_on_invalid() {
  let buf = vec![0u32; 10];
  let _ = Rgba128LeFrame::new(&buf, 2, 3, 8);
}
