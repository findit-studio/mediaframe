use super::*;
use std::vec;

// ---- Rgb565Frame -----------------------------------------------------------

#[test]
fn rgb565_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 2 * 4];
  Rgb565Frame::try_new(&buf, 16, 4, 32).expect("valid tight stride");
}

#[test]
fn rgb565_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 32];
  assert!(matches!(
    Rgb565Frame::try_new(&buf, 0, 0, 0),
    Err(LegacyRgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb565_frame_try_new_rejects_width_overflow() {
  let buf = vec![0u8; 4];
  let w = u32::MAX / 2 + 1;
  assert!(matches!(
    Rgb565Frame::try_new(&buf, w, 1, w),
    Err(LegacyRgbFrameError::WidthOverflow(_))
  ));
}

#[test]
fn rgb565_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 2 * 4];
  // min stride = 2*16 = 32; supply 31
  assert!(matches!(
    Rgb565Frame::try_new(&buf, 16, 4, 31),
    Err(LegacyRgbFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb565_frame_try_new_rejects_geometry_overflow() {
  // Only meaningful on 32-bit targets where `stride * height` as `usize` can overflow.
  // stride=0x1_0000, height=0x1_0000 → product = 2^32 → overflows 32-bit usize.
  let buf: [u8; 0] = [];
  assert!(matches!(
    Rgb565Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(LegacyRgbFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn rgb565_frame_try_new_rejects_plane_too_short() {
  // Valid geometry: 16 px wide, 4 rows, stride=32 → need 128 bytes; give 127
  let buf = vec![0u8; 127];
  assert!(matches!(
    Rgb565Frame::try_new(&buf, 16, 4, 32),
    Err(LegacyRgbFrameError::InsufficientPlane(_))
  ));
}

// ---- Bgr565Frame -----------------------------------------------------------

#[test]
fn bgr565_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 2 * 4];
  Bgr565Frame::try_new(&buf, 16, 4, 32).expect("valid tight stride");
}

#[test]
fn bgr565_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 32];
  assert!(matches!(
    Bgr565Frame::try_new(&buf, 0, 0, 0),
    Err(LegacyRgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr565_frame_try_new_rejects_width_overflow() {
  let buf = vec![0u8; 4];
  let w = u32::MAX / 2 + 1;
  assert!(matches!(
    Bgr565Frame::try_new(&buf, w, 1, w),
    Err(LegacyRgbFrameError::WidthOverflow(_))
  ));
}

#[test]
fn bgr565_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 2 * 4];
  assert!(matches!(
    Bgr565Frame::try_new(&buf, 16, 4, 31),
    Err(LegacyRgbFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn bgr565_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Bgr565Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(LegacyRgbFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn bgr565_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 127];
  assert!(matches!(
    Bgr565Frame::try_new(&buf, 16, 4, 32),
    Err(LegacyRgbFrameError::InsufficientPlane(_))
  ));
}

// ---- Rgb555Frame -----------------------------------------------------------

#[test]
fn rgb555_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 2 * 4];
  Rgb555Frame::try_new(&buf, 16, 4, 32).expect("valid tight stride");
}

#[test]
fn rgb555_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 32];
  assert!(matches!(
    Rgb555Frame::try_new(&buf, 0, 0, 0),
    Err(LegacyRgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb555_frame_try_new_rejects_width_overflow() {
  let buf = vec![0u8; 4];
  let w = u32::MAX / 2 + 1;
  assert!(matches!(
    Rgb555Frame::try_new(&buf, w, 1, w),
    Err(LegacyRgbFrameError::WidthOverflow(_))
  ));
}

#[test]
fn rgb555_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 2 * 4];
  assert!(matches!(
    Rgb555Frame::try_new(&buf, 16, 4, 31),
    Err(LegacyRgbFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb555_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Rgb555Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(LegacyRgbFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn rgb555_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 127];
  assert!(matches!(
    Rgb555Frame::try_new(&buf, 16, 4, 32),
    Err(LegacyRgbFrameError::InsufficientPlane(_))
  ));
}

// ---- Bgr555Frame -----------------------------------------------------------

#[test]
fn bgr555_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 2 * 4];
  Bgr555Frame::try_new(&buf, 16, 4, 32).expect("valid tight stride");
}

#[test]
fn bgr555_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 32];
  assert!(matches!(
    Bgr555Frame::try_new(&buf, 0, 0, 0),
    Err(LegacyRgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr555_frame_try_new_rejects_width_overflow() {
  let buf = vec![0u8; 4];
  let w = u32::MAX / 2 + 1;
  assert!(matches!(
    Bgr555Frame::try_new(&buf, w, 1, w),
    Err(LegacyRgbFrameError::WidthOverflow(_))
  ));
}

#[test]
fn bgr555_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 2 * 4];
  assert!(matches!(
    Bgr555Frame::try_new(&buf, 16, 4, 31),
    Err(LegacyRgbFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn bgr555_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Bgr555Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(LegacyRgbFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn bgr555_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 127];
  assert!(matches!(
    Bgr555Frame::try_new(&buf, 16, 4, 32),
    Err(LegacyRgbFrameError::InsufficientPlane(_))
  ));
}

// ---- Rgb444Frame -----------------------------------------------------------

#[test]
fn rgb444_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 2 * 4];
  Rgb444Frame::try_new(&buf, 16, 4, 32).expect("valid tight stride");
}

#[test]
fn rgb444_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 32];
  assert!(matches!(
    Rgb444Frame::try_new(&buf, 0, 0, 0),
    Err(LegacyRgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb444_frame_try_new_rejects_width_overflow() {
  let buf = vec![0u8; 4];
  let w = u32::MAX / 2 + 1;
  assert!(matches!(
    Rgb444Frame::try_new(&buf, w, 1, w),
    Err(LegacyRgbFrameError::WidthOverflow(_))
  ));
}

#[test]
fn rgb444_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 2 * 4];
  assert!(matches!(
    Rgb444Frame::try_new(&buf, 16, 4, 31),
    Err(LegacyRgbFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb444_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Rgb444Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(LegacyRgbFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn rgb444_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 127];
  assert!(matches!(
    Rgb444Frame::try_new(&buf, 16, 4, 32),
    Err(LegacyRgbFrameError::InsufficientPlane(_))
  ));
}

// ---- Rgb565Frame::new panic -------------------------------------------------

#[test]
#[should_panic(expected = "invalid Rgb565Frame dimensions or plane length")]
fn rgb565_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Rgb565Frame::new(&buf, 16, 4, 32);
}

// ---- Bgr565Frame::new panic -------------------------------------------------

#[test]
#[should_panic(expected = "invalid Bgr565Frame dimensions or plane length")]
fn bgr565_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Bgr565Frame::new(&buf, 16, 4, 32);
}

// ---- Rgb555Frame::new panic -------------------------------------------------

#[test]
#[should_panic(expected = "invalid Rgb555Frame dimensions or plane length")]
fn rgb555_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Rgb555Frame::new(&buf, 16, 4, 32);
}

// ---- Bgr555Frame::new panic -------------------------------------------------

#[test]
#[should_panic(expected = "invalid Bgr555Frame dimensions or plane length")]
fn bgr555_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Bgr555Frame::new(&buf, 16, 4, 32);
}

// ---- Rgb444Frame::new panic -------------------------------------------------

#[test]
#[should_panic(expected = "invalid Rgb444Frame dimensions or plane length")]
fn rgb444_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Rgb444Frame::new(&buf, 16, 4, 32);
}

// ---- Bgr444Frame::new panic -------------------------------------------------

#[test]
#[should_panic(expected = "invalid Bgr444Frame dimensions or plane length")]
fn bgr444_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Bgr444Frame::new(&buf, 16, 4, 32);
}

// ---- Bgr444Frame -----------------------------------------------------------

#[test]
fn bgr444_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 2 * 4];
  Bgr444Frame::try_new(&buf, 16, 4, 32).expect("valid tight stride");
}

#[test]
fn bgr444_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 32];
  assert!(matches!(
    Bgr444Frame::try_new(&buf, 0, 0, 0),
    Err(LegacyRgbFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr444_frame_try_new_rejects_width_overflow() {
  let buf = vec![0u8; 4];
  let w = u32::MAX / 2 + 1;
  assert!(matches!(
    Bgr444Frame::try_new(&buf, w, 1, w),
    Err(LegacyRgbFrameError::WidthOverflow(_))
  ));
}

#[test]
fn bgr444_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 2 * 4];
  assert!(matches!(
    Bgr444Frame::try_new(&buf, 16, 4, 31),
    Err(LegacyRgbFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn bgr444_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Bgr444Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(LegacyRgbFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn bgr444_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 127];
  assert!(matches!(
    Bgr444Frame::try_new(&buf, 16, 4, 32),
    Err(LegacyRgbFrameError::InsufficientPlane(_))
  ));
}

// ============================================================
// Bit-packed RGB/BGR frames (8bpp byte + 4bpp bitstream)
// ============================================================

// ---- Rgb8Frame -------------------------------------------------------------

#[test]
fn rgb8_frame_try_new_accepts_valid_tight() {
  // 1 byte/pixel: 16 px wide, 4 rows, stride = 16 → 64 bytes.
  let buf = vec![0u8; 16 * 4];
  Rgb8Frame::try_new(&buf, 16, 4, 16).expect("valid tight stride");
}

#[test]
fn rgb8_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16];
  assert!(matches!(
    Rgb8Frame::try_new(&buf, 0, 0, 0),
    Err(PackedRgbBitFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb8_frame_try_new_rejects_stride_too_small() {
  // min stride = width = 16; supply 15
  let buf = vec![0u8; 16 * 4];
  assert!(matches!(
    Rgb8Frame::try_new(&buf, 16, 4, 15),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb8_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Rgb8Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(PackedRgbBitFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn rgb8_frame_try_new_rejects_plane_too_short() {
  // 16 px wide, 4 rows, stride=16 → need 64 bytes; give 63
  let buf = vec![0u8; 63];
  assert!(matches!(
    Rgb8Frame::try_new(&buf, 16, 4, 16),
    Err(PackedRgbBitFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Rgb8Frame dimensions or plane length")]
fn rgb8_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Rgb8Frame::new(&buf, 16, 4, 16);
}

// ---- Bgr8Frame -------------------------------------------------------------

#[test]
fn bgr8_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4];
  Bgr8Frame::try_new(&buf, 16, 4, 16).expect("valid tight stride");
}

#[test]
fn bgr8_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16];
  assert!(matches!(
    Bgr8Frame::try_new(&buf, 0, 0, 0),
    Err(PackedRgbBitFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr8_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4];
  assert!(matches!(
    Bgr8Frame::try_new(&buf, 16, 4, 15),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn bgr8_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Bgr8Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(PackedRgbBitFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn bgr8_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 63];
  assert!(matches!(
    Bgr8Frame::try_new(&buf, 16, 4, 16),
    Err(PackedRgbBitFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Bgr8Frame dimensions or plane length")]
fn bgr8_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Bgr8Frame::new(&buf, 16, 4, 16);
}

// ---- Rgb4ByteFrame ---------------------------------------------------------

#[test]
fn rgb4_byte_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4];
  Rgb4ByteFrame::try_new(&buf, 16, 4, 16).expect("valid tight stride");
}

#[test]
fn rgb4_byte_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16];
  assert!(matches!(
    Rgb4ByteFrame::try_new(&buf, 0, 0, 0),
    Err(PackedRgbBitFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb4_byte_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4];
  assert!(matches!(
    Rgb4ByteFrame::try_new(&buf, 16, 4, 15),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb4_byte_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Rgb4ByteFrame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(PackedRgbBitFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn rgb4_byte_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 63];
  assert!(matches!(
    Rgb4ByteFrame::try_new(&buf, 16, 4, 16),
    Err(PackedRgbBitFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Rgb4ByteFrame dimensions or plane length")]
fn rgb4_byte_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Rgb4ByteFrame::new(&buf, 16, 4, 16);
}

// ---- Bgr4ByteFrame ---------------------------------------------------------

#[test]
fn bgr4_byte_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 16 * 4];
  Bgr4ByteFrame::try_new(&buf, 16, 4, 16).expect("valid tight stride");
}

#[test]
fn bgr4_byte_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16];
  assert!(matches!(
    Bgr4ByteFrame::try_new(&buf, 0, 0, 0),
    Err(PackedRgbBitFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr4_byte_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 16 * 4];
  assert!(matches!(
    Bgr4ByteFrame::try_new(&buf, 16, 4, 15),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn bgr4_byte_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Bgr4ByteFrame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(PackedRgbBitFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn bgr4_byte_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 63];
  assert!(matches!(
    Bgr4ByteFrame::try_new(&buf, 16, 4, 16),
    Err(PackedRgbBitFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Bgr4ByteFrame dimensions or plane length")]
fn bgr4_byte_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Bgr4ByteFrame::new(&buf, 16, 4, 16);
}

// ---- Rgb4Frame (4bpp bitstream, 2 px/byte) ---------------------------------

#[test]
fn rgb4_frame_try_new_accepts_valid_tight() {
  // 4bpp: 16 px wide → min stride = 8 bytes; 4 rows → 32 bytes.
  let buf = vec![0u8; 8 * 4];
  Rgb4Frame::try_new(&buf, 16, 4, 8).expect("valid tight stride");
}

#[test]
fn rgb4_frame_try_new_accepts_odd_width_tight() {
  // Odd width 15 → min stride = div_ceil(15, 2) = 8 bytes; final byte's low
  // nibble is unused. 4 rows → 32 bytes.
  let buf = vec![0u8; 8 * 4];
  Rgb4Frame::try_new(&buf, 15, 4, 8).expect("valid tight odd-width stride");
}

#[test]
fn rgb4_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16];
  assert!(matches!(
    Rgb4Frame::try_new(&buf, 0, 0, 0),
    Err(PackedRgbBitFrameError::ZeroDimension(_))
  ));
}

#[test]
fn rgb4_frame_try_new_rejects_stride_too_small() {
  // min stride = div_ceil(16, 2) = 8; supply 7
  let buf = vec![0u8; 8 * 4];
  assert!(matches!(
    Rgb4Frame::try_new(&buf, 16, 4, 7),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[test]
fn rgb4_frame_try_new_rejects_odd_width_stride_too_small() {
  // Odd width 15 → min stride = div_ceil(15, 2) = 8; supply 7
  let buf = vec![0u8; 8 * 4];
  assert!(matches!(
    Rgb4Frame::try_new(&buf, 15, 4, 7),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn rgb4_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Rgb4Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(PackedRgbBitFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn rgb4_frame_try_new_rejects_plane_too_short() {
  // 16 px wide, 4 rows, stride=8 → need 32 bytes; give 31
  let buf = vec![0u8; 31];
  assert!(matches!(
    Rgb4Frame::try_new(&buf, 16, 4, 8),
    Err(PackedRgbBitFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Rgb4Frame dimensions or plane length")]
fn rgb4_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Rgb4Frame::new(&buf, 16, 4, 8);
}

// ---- Bgr4Frame (4bpp bitstream, 2 px/byte) ---------------------------------

#[test]
fn bgr4_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 8 * 4];
  Bgr4Frame::try_new(&buf, 16, 4, 8).expect("valid tight stride");
}

#[test]
fn bgr4_frame_try_new_accepts_odd_width_tight() {
  let buf = vec![0u8; 8 * 4];
  Bgr4Frame::try_new(&buf, 15, 4, 8).expect("valid tight odd-width stride");
}

#[test]
fn bgr4_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 16];
  assert!(matches!(
    Bgr4Frame::try_new(&buf, 0, 0, 0),
    Err(PackedRgbBitFrameError::ZeroDimension(_))
  ));
}

#[test]
fn bgr4_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 8 * 4];
  assert!(matches!(
    Bgr4Frame::try_new(&buf, 16, 4, 7),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[test]
fn bgr4_frame_try_new_rejects_odd_width_stride_too_small() {
  let buf = vec![0u8; 8 * 4];
  assert!(matches!(
    Bgr4Frame::try_new(&buf, 15, 4, 7),
    Err(PackedRgbBitFrameError::InsufficientStride(_))
  ));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn bgr4_frame_try_new_rejects_geometry_overflow() {
  let buf: [u8; 0] = [];
  assert!(matches!(
    Bgr4Frame::try_new(&buf, 1, 0x1_0000, 0x1_0000),
    Err(PackedRgbBitFrameError::GeometryOverflow(_))
  ));
}

#[test]
fn bgr4_frame_try_new_rejects_plane_too_short() {
  let buf = vec![0u8; 31];
  assert!(matches!(
    Bgr4Frame::try_new(&buf, 16, 4, 8),
    Err(PackedRgbBitFrameError::InsufficientPlane(_))
  ));
}

#[test]
#[should_panic(expected = "invalid Bgr4Frame dimensions or plane length")]
fn bgr4_frame_new_panics_on_invalid() {
  let buf = vec![0u8; 1];
  Bgr4Frame::new(&buf, 16, 4, 8);
}
