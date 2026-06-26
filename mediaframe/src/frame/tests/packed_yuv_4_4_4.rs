use crate::frame::{
  Ayuv64BeFrame, Ayuv64FrameError, Ayuv64LeFrame, AyuvFrame, AyuvFrameError, UyvaFrame,
  UyvaFrameError, V30XFrame, V30XFrameError, V410BeFrame, V410FrameError, V410LeFrame, VuyaFrame,
  VuyaFrameError, VuyxFrame, VuyxFrameError, Vyu444Frame, Vyu444FrameError, Xv36BeFrame,
  Xv36FrameError, Xv36LeFrame, Xv48BeFrame, Xv48FrameError, Xv48LeFrame,
};
use std::vec;

const fn zero_buf<const N: usize>() -> [u32; N] {
  [0u32; N]
}

#[test]
fn v410_frame_try_new_accepts_valid_tight() {
  // Tight stride: stride == width.
  let buf = zero_buf::<16>();
  let f = V410LeFrame::try_new(&buf, 4, 4, 4).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 4);
  assert_eq!(f.packed().len(), 16);
}

#[test]
fn v410_frame_try_new_accepts_oversized_stride() {
  let buf = zero_buf::<32>();
  V410LeFrame::try_new(&buf, 4, 4, 8).unwrap();
}

#[test]
fn v410_frame_try_new_rejects_zero_dimension() {
  let buf = zero_buf::<16>();
  assert!(matches!(
    V410LeFrame::try_new(&buf, 0, 4, 4),
    Err(V410FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    V410LeFrame::try_new(&buf, 4, 0, 4),
    Err(V410FrameError::ZeroDimension(_))
  ));
}

#[test]
fn v410_frame_try_new_rejects_stride_too_small() {
  let buf = zero_buf::<16>();
  assert!(matches!(
    V410LeFrame::try_new(&buf, 4, 4, 3),
    Err(V410FrameError::InsufficientStride(_))
  ));
}

#[test]
fn v410_frame_try_new_rejects_short_plane() {
  let buf = zero_buf::<8>();
  assert!(matches!(
    V410LeFrame::try_new(&buf, 4, 4, 4),
    Err(V410FrameError::InsufficientPlane(_))
  ));
}

#[test]
fn v410_frame_accessors_round_trip() {
  let buf = zero_buf::<32>();
  let f = V410LeFrame::try_new(&buf, 4, 4, 8).unwrap();
  assert_eq!(f.packed().len(), 32);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 8);
}

#[test]
#[should_panic(expected = "invalid V410Frame:")]
fn v410_frame_new_panics_on_invalid() {
  let buf = zero_buf::<8>();
  let _ = V410LeFrame::new(&buf, 4, 4, 4); // InsufficientPlane
}

#[test]
fn v410_le_frame_default_is_le() {
  // Phase 4: default `<const BE: bool = false>` exposed via `is_be()`.
  let buf = zero_buf::<16>();
  let f = V410LeFrame::try_new(&buf, 4, 4, 4).unwrap();
  assert!(!f.is_be());
}

#[test]
fn v410_be_frame_alias_constructs() {
  // Phase 4: `V410BeFrame` alias resolves to `V410Frame<'_, true>`.
  let buf = zero_buf::<16>();
  let f = V410BeFrame::try_new(&buf, 4, 4, 4).unwrap();
  assert!(f.is_be());
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
}

#[test]
fn v30x_frame_try_new_accepts_valid_tight() {
  // Tight stride: stride == width.
  let buf = zero_buf::<16>();
  let f = V30XFrame::try_new(&buf, 4, 4, 4).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 4);
  assert_eq!(f.packed().len(), 16);
}

#[test]
fn v30x_frame_try_new_accepts_oversized_stride() {
  let buf = zero_buf::<32>();
  V30XFrame::try_new(&buf, 4, 4, 8).unwrap();
}

#[test]
fn v30x_frame_try_new_rejects_zero_dimension() {
  let buf = zero_buf::<16>();
  assert!(matches!(
    V30XFrame::try_new(&buf, 0, 4, 4),
    Err(V30XFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    V30XFrame::try_new(&buf, 4, 0, 4),
    Err(V30XFrameError::ZeroDimension(_))
  ));
}

#[test]
fn v30x_frame_try_new_rejects_stride_too_small() {
  let buf = zero_buf::<16>();
  assert!(matches!(
    V30XFrame::try_new(&buf, 4, 4, 3),
    Err(V30XFrameError::InsufficientStride(_))
  ));
}

#[test]
fn v30x_frame_try_new_rejects_short_plane() {
  let buf = zero_buf::<8>();
  assert!(matches!(
    V30XFrame::try_new(&buf, 4, 4, 4),
    Err(V30XFrameError::InsufficientPlane(_))
  ));
}

#[test]
fn v30x_frame_accessors_round_trip() {
  let buf = zero_buf::<32>();
  let f = V30XFrame::try_new(&buf, 4, 4, 8).unwrap();
  assert_eq!(f.packed().len(), 32);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 8);
}

#[test]
#[should_panic(expected = "invalid V30XFrame:")]
fn v30x_frame_new_panics_on_invalid() {
  let buf = zero_buf::<8>();
  let _ = V30XFrame::new(&buf, 4, 4, 4); // InsufficientPlane
}

#[test]
fn xv36_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u16; 4 * 4 * 4]; // 4 px × 4 channels × 4 rows
  let f = Xv36LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn xv36_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u16; 4 * 4 * 8]; // stride=32 > width*4=16
  Xv36LeFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn xv36_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u16; 16];
  assert!(matches!(
    Xv36LeFrame::try_new(&buf, 0, 4, 16),
    Err(Xv36FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Xv36LeFrame::try_new(&buf, 4, 0, 16),
    Err(Xv36FrameError::ZeroDimension(_))
  ));
}

#[test]
fn xv36_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u16; 64];
  // width=4, width*4=16; stride=12 < 16
  assert!(matches!(
    Xv36LeFrame::try_new(&buf, 4, 4, 12),
    Err(Xv36FrameError::InsufficientStride(_))
  ));
}

#[test]
fn xv36_frame_try_new_rejects_short_plane() {
  let buf = vec![0u16; 32]; // need 16*4 = 64
  assert!(matches!(
    Xv36LeFrame::try_new(&buf, 4, 4, 16),
    Err(Xv36FrameError::InsufficientPlane(_))
  ));
}

#[test]
fn xv36_frame_try_new_checked_accepts_msb_aligned() {
  // Use LE-encoded byte storage so the test passes on every host. On BE,
  // raw `0xABC0` as host-native u16 has bytes `[AB, C0]`; the validator's
  // `u16::from_le` swap reads them as `0xC0AB` (low nibble = 0xB ≠ 0 →
  // false reject). `le_encoded_u16_buf` produces u16 whose in-memory
  // bytes ARE the LE wire bytes on every host.
  let buf = super::util::le_encoded_u16_buf(&[0xABC0u16; 64]);
  Xv36LeFrame::try_new_checked(&buf, 4, 4, 16).unwrap();
}

#[test]
fn xv36_frame_try_new_checked_rejects_low_bits_set() {
  // Same host-independence consideration as the accepts test above.
  let mut intended = std::vec![0xABC0u16; 64];
  intended[5] = 0xABCD; // low 4 bits = 0xD ≠ 0 (in active row range)
  let buf = super::util::le_encoded_u16_buf(&intended);
  let err = Xv36LeFrame::try_new_checked(&buf, 4, 4, 16).unwrap_err();
  assert!(matches!(err, Xv36FrameError::SampleLowBitsSetAt(_)));
}

// ---- BE/LE try_new_checked normalization regression tests ------------
//
// Codex PR #107 finding: `Xv36Frame::try_new_checked` previously
// tested raw `u16` byte-storage words against the low-nibble mask,
// which on a little-endian host falsely rejected valid BE-encoded
// XV36 samples (and could mis-judge true low-bit-set BE samples).
// These tests pin the post-fix behavior on every host: the validator
// normalizes via `u16::from_be`/`u16::from_le` per the `BE` flag
// before the check, mirroring PR #89 `b9a6c19` for high-bit planar.

#[test]
fn xv36_be_frame_try_new_checked_accepts_be_encoded_msb_aligned_on_any_host() {
  // Logical sample 0xABC0 (low 4 bits zero) encoded BE as wire bytes
  // [0xAB, 0xC0]. Use `from_ne_bytes` so the byte storage of the u16
  // matches the wire bytes on every host (mirrors PR #95 round 2 fix
  // for `pack12_le`); then the validator's `u16::from_be` recovers
  // logical 0xABC0 regardless of host endianness.
  let mut buf = vec![0u16; 64];
  let be_word = u16::from_ne_bytes([0xAB, 0xC0]); // wire bytes [0xAB, 0xC0] on any host
  buf.fill(be_word);
  Xv36BeFrame::try_new_checked(&buf, 4, 4, 16).expect("valid BE-encoded MSB-aligned XV36");
}

#[test]
fn xv36_be_frame_try_new_checked_rejects_be_encoded_low_bits_set_on_any_host() {
  // Logical sample 0xABCD (low 4 bits = 0xD ≠ 0) encoded BE as wire
  // bytes [0xAB, 0xCD]. Validator must reject after BE normalization.
  let mut buf = vec![0u16; 64];
  let be_word = u16::from_ne_bytes([0xAB, 0xCD]); // wire bytes [0xAB, 0xCD] on any host
  buf[5] = be_word;
  let err = Xv36BeFrame::try_new_checked(&buf, 4, 4, 16).unwrap_err();
  assert!(matches!(err, Xv36FrameError::SampleLowBitsSetAt(_)));
}

#[test]
fn xv36_be_frame_try_new_checked_rejects_be_encoded_low_nibble_only() {
  // Logical sample 0x000D — low nibble only set. Pre-fix on a LE
  // host this was 0x0D00 (low nibble 0) and falsely *accepted*. Post-
  // fix the BE normalization restores 0x000D and rejects.
  let mut buf = vec![0u16; 64];
  let be_word = u16::from_ne_bytes([0x00, 0x0D]); // wire bytes [0x00, 0x0D] on any host
  buf[7] = be_word;
  let err = Xv36BeFrame::try_new_checked(&buf, 4, 4, 16).unwrap_err();
  assert!(matches!(err, Xv36FrameError::SampleLowBitsSetAt(_)));
}

#[test]
fn xv36_le_frame_try_new_checked_accepts_le_encoded_msb_aligned_on_any_host() {
  // Symmetric LE counterpart: logical 0xABC0 encoded LE as wire bytes
  // [0xC0, 0xAB]. Use `from_ne_bytes` so the u16 byte storage matches
  // the wire bytes on every host; the validator's `u16::from_le`
  // recovers logical 0xABC0 regardless of host endianness.
  let mut buf = vec![0u16; 64];
  let le_word = u16::from_ne_bytes([0xC0, 0xAB]); // wire bytes [0xC0, 0xAB] on any host
  buf.fill(le_word);
  Xv36LeFrame::try_new_checked(&buf, 4, 4, 16).expect("valid LE-encoded MSB-aligned XV36");
}

#[test]
fn xv36_le_frame_try_new_checked_rejects_le_encoded_low_bits_set_on_any_host() {
  let mut buf = vec![0u16; 64];
  let le_word = u16::from_ne_bytes([0xCD, 0xAB]); // wire bytes [0xCD, 0xAB] = logical 0xABCD on any host
  buf[3] = le_word;
  let err = Xv36LeFrame::try_new_checked(&buf, 4, 4, 16).unwrap_err();
  assert!(matches!(err, Xv36FrameError::SampleLowBitsSetAt(_)));
}

#[test]
fn xv36_frame_accessors_round_trip() {
  let buf = vec![0u16; 64];
  let f = Xv36LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.packed().len(), 64);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
}

#[test]
fn xv36_le_frame_default_is_le() {
  // Phase 4: default `<const BE: bool = false>` exposed via `is_be()`.
  let buf = vec![0u16; 64];
  let f = Xv36LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert!(!f.is_be());
}

#[test]
fn xv36_be_frame_alias_constructs() {
  // Phase 4: `Xv36BeFrame` alias resolves to `Xv36Frame<'_, true>`.
  let buf = vec![0u16; 64];
  let f = Xv36BeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert!(f.is_be());
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
}

// ---- XV48 (16-bit packed 4:4:4, full-depth sibling of XV36) ----------
//
// Geometry-only validation: XV48 is 16-bit native (all bits active), so
// unlike XV36 there is no low-bit-alignment invariant / `try_new_checked`.

#[test]
fn xv48_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u16; 4 * 4 * 4]; // 4 px × 4 channels × 4 rows
  let f = Xv48LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn xv48_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u16; 4 * 4 * 8]; // stride=32 > width*4=16
  Xv48LeFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn xv48_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u16; 16];
  assert!(matches!(
    Xv48LeFrame::try_new(&buf, 0, 4, 16),
    Err(Xv48FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Xv48LeFrame::try_new(&buf, 4, 0, 16),
    Err(Xv48FrameError::ZeroDimension(_))
  ));
}

#[test]
fn xv48_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u16; 64];
  // width=4, width*4=16; stride=12 < 16
  assert!(matches!(
    Xv48LeFrame::try_new(&buf, 4, 4, 12),
    Err(Xv48FrameError::InsufficientStride(_))
  ));
}

#[test]
fn xv48_frame_try_new_rejects_short_plane() {
  let buf = vec![0u16; 32]; // need 16*4 = 64
  assert!(matches!(
    Xv48LeFrame::try_new(&buf, 4, 4, 16),
    Err(Xv48FrameError::InsufficientPlane(_))
  ));
}

#[test]
fn xv48_frame_accessors_round_trip() {
  let buf = vec![0u16; 64];
  let f = Xv48LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.packed().len(), 64);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
}

#[test]
fn xv48_le_frame_default_is_le() {
  let buf = vec![0u16; 64];
  let f = Xv48LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert!(!f.is_be());
}

#[test]
fn xv48_be_frame_alias_constructs() {
  // `Xv48BeFrame` alias resolves to `Xv48Frame<'_, true>`.
  let buf = vec![0u16; 64];
  let f = Xv48BeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert!(f.is_be());
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
}

#[test]
#[should_panic(expected = "invalid Xv48Frame:")]
fn xv48_frame_new_panics_on_invalid() {
  let buf = vec![0u16; 32]; // need width*4*height = 64; too short
  let _ = Xv48LeFrame::new(&buf, 4, 4, 16); // InsufficientPlane
}

#[test]
fn vuya_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 4 * 4 * 4]; // 4 px × 4 bytes × 4 rows
  let f = VuyaFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn vuya_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u8; 4 * 4 * 8]; // stride=32 > width*4=16
  VuyaFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn vuya_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 64];
  assert!(matches!(
    VuyaFrame::try_new(&buf, 0, 4, 16),
    Err(VuyaFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    VuyaFrame::try_new(&buf, 4, 0, 16),
    Err(VuyaFrameError::ZeroDimension(_))
  ));
}

#[test]
fn vuya_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 64];
  // width=4, width*4=16 bytes; stride=12 < 16
  assert!(matches!(
    VuyaFrame::try_new(&buf, 4, 4, 12),
    Err(VuyaFrameError::InsufficientStride(_))
  ));
}

#[test]
fn vuya_frame_try_new_rejects_short_plane() {
  let buf = vec![0u8; 32]; // need 16*4 = 64 bytes
  assert!(matches!(
    VuyaFrame::try_new(&buf, 4, 4, 16),
    Err(VuyaFrameError::InsufficientPlane(_))
  ));
}

#[test]
fn vuya_frame_accessors_round_trip() {
  let buf = vec![0u8; 128]; // stride=32, height=4 → 128 bytes
  let f = VuyaFrame::try_new(&buf, 4, 4, 32).unwrap();
  assert_eq!(f.packed().len(), 128);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 32);
}

#[test]
fn vuyx_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 4 * 4 * 4]; // 4 px × 4 bytes × 4 rows
  let f = VuyxFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn vuyx_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u8; 4 * 4 * 8]; // stride=32 > width*4=16
  VuyxFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn vuyx_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 64];
  assert!(matches!(
    VuyxFrame::try_new(&buf, 0, 4, 16),
    Err(VuyxFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    VuyxFrame::try_new(&buf, 4, 0, 16),
    Err(VuyxFrameError::ZeroDimension(_))
  ));
}

#[test]
fn vuyx_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 64];
  // width=4, width*4=16 bytes; stride=12 < 16
  assert!(matches!(
    VuyxFrame::try_new(&buf, 4, 4, 12),
    Err(VuyxFrameError::InsufficientStride(_))
  ));
}

#[test]
fn vuyx_frame_try_new_rejects_short_plane() {
  let buf = vec![0u8; 32]; // need 16*4 = 64 bytes
  assert!(matches!(
    VuyxFrame::try_new(&buf, 4, 4, 16),
    Err(VuyxFrameError::InsufficientPlane(_))
  ));
}

#[test]
fn vuyx_frame_accessors_round_trip() {
  let buf = vec![0u8; 128]; // stride=32, height=4 → 128 bytes
  let f = VuyxFrame::try_new(&buf, 4, 4, 32).unwrap();
  assert_eq!(f.packed().len(), 128);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 32);
}

#[test]
fn ayuv_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 4 * 4 * 4]; // 4 px × 4 bytes × 4 rows
  let f = AyuvFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn ayuv_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u8; 4 * 4 * 8]; // stride=32 > width*4=16
  AyuvFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn ayuv_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 64];
  assert!(matches!(
    AyuvFrame::try_new(&buf, 0, 4, 16),
    Err(AyuvFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    AyuvFrame::try_new(&buf, 4, 0, 16),
    Err(AyuvFrameError::ZeroDimension(_))
  ));
}

#[test]
fn ayuv_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 64];
  // width=4, width*4=16 bytes; stride=12 < 16
  assert!(matches!(
    AyuvFrame::try_new(&buf, 4, 4, 12),
    Err(AyuvFrameError::InsufficientStride(_))
  ));
}

#[test]
fn ayuv_frame_try_new_rejects_short_plane() {
  let buf = vec![0u8; 32]; // need 16*4 = 64 bytes
  assert!(matches!(
    AyuvFrame::try_new(&buf, 4, 4, 16),
    Err(AyuvFrameError::InsufficientPlane(_))
  ));
}

#[test]
fn ayuv_frame_accessors_round_trip() {
  let buf = vec![0u8; 128]; // stride=32, height=4 → 128 bytes
  let f = AyuvFrame::try_new(&buf, 4, 4, 32).unwrap();
  assert_eq!(f.packed().len(), 128);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 32);
}

#[test]
fn uyva_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 4 * 4 * 4]; // 4 px × 4 bytes × 4 rows
  let f = UyvaFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn uyva_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u8; 4 * 4 * 8]; // stride=32 > width*4=16
  UyvaFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn uyva_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 64];
  assert!(matches!(
    UyvaFrame::try_new(&buf, 0, 4, 16),
    Err(UyvaFrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    UyvaFrame::try_new(&buf, 4, 0, 16),
    Err(UyvaFrameError::ZeroDimension(_))
  ));
}

#[test]
fn uyva_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 64];
  // width=4, width*4=16 bytes; stride=12 < 16
  assert!(matches!(
    UyvaFrame::try_new(&buf, 4, 4, 12),
    Err(UyvaFrameError::InsufficientStride(_))
  ));
}

#[test]
fn uyva_frame_try_new_rejects_short_plane() {
  let buf = vec![0u8; 32]; // need 16*4 = 64 bytes
  assert!(matches!(
    UyvaFrame::try_new(&buf, 4, 4, 16),
    Err(UyvaFrameError::InsufficientPlane(_))
  ));
}

#[test]
fn uyva_frame_accessors_round_trip() {
  let buf = vec![0u8; 128]; // stride=32, height=4 → 128 bytes
  let f = UyvaFrame::try_new(&buf, 4, 4, 32).unwrap();
  assert_eq!(f.packed().len(), 128);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 32);
}

#[test]
fn vyu444_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u8; 4 * 3 * 4]; // 4 px × 3 bytes × 4 rows
  let f = Vyu444Frame::try_new(&buf, 4, 4, 12).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 12);
  assert_eq!(f.packed().len(), 48);
}

#[test]
fn vyu444_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u8; 4 * 6 * 4]; // stride=24 > width*3=12
  Vyu444Frame::try_new(&buf, 4, 4, 24).unwrap();
}

#[test]
fn vyu444_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u8; 48];
  assert!(matches!(
    Vyu444Frame::try_new(&buf, 0, 4, 12),
    Err(Vyu444FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Vyu444Frame::try_new(&buf, 4, 0, 12),
    Err(Vyu444FrameError::ZeroDimension(_))
  ));
}

#[test]
fn vyu444_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u8; 48];
  // width=4, width*3=12 bytes; stride=11 < 12
  assert!(matches!(
    Vyu444Frame::try_new(&buf, 4, 4, 11),
    Err(Vyu444FrameError::InsufficientStride(_))
  ));
}

#[test]
fn vyu444_frame_try_new_rejects_short_plane() {
  let buf = vec![0u8; 24]; // need 12*4 = 48 bytes
  assert!(matches!(
    Vyu444Frame::try_new(&buf, 4, 4, 12),
    Err(Vyu444FrameError::InsufficientPlane(_))
  ));
}

#[test]
fn vyu444_frame_accessors_round_trip() {
  let buf = vec![0u8; 96]; // stride=24, height=4 → 96 bytes
  let f = Vyu444Frame::try_new(&buf, 4, 4, 24).unwrap();
  assert_eq!(f.packed().len(), 96);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 24);
}

#[test]
fn ayuv64_frame_try_new_accepts_valid_tight() {
  let buf = vec![0u16; 4 * 4 * 4]; // 4 px × 4 u16 channels × 4 rows
  let f = Ayuv64LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 16);
  assert_eq!(f.packed().len(), 64);
}

#[test]
fn ayuv64_frame_try_new_accepts_oversized_stride() {
  let buf = vec![0u16; 4 * 4 * 8]; // stride=32 > width*4=16
  Ayuv64LeFrame::try_new(&buf, 4, 4, 32).unwrap();
}

#[test]
fn ayuv64_frame_try_new_rejects_zero_dimension() {
  let buf = vec![0u16; 64];
  assert!(matches!(
    Ayuv64LeFrame::try_new(&buf, 0, 4, 16),
    Err(Ayuv64FrameError::ZeroDimension(_))
  ));
  assert!(matches!(
    Ayuv64LeFrame::try_new(&buf, 4, 0, 16),
    Err(Ayuv64FrameError::ZeroDimension(_))
  ));
}

#[test]
fn ayuv64_frame_try_new_rejects_stride_too_small() {
  let buf = vec![0u16; 64];
  // width=4, width*4=16 u16 elements; stride=12 < 16
  assert!(matches!(
    Ayuv64LeFrame::try_new(&buf, 4, 4, 12),
    Err(Ayuv64FrameError::InsufficientStride(_))
  ));
}

#[test]
fn ayuv64_frame_try_new_rejects_short_plane() {
  let buf = vec![0u16; 32]; // need 16*4 = 64 u16 elements
  assert!(matches!(
    Ayuv64LeFrame::try_new(&buf, 4, 4, 16),
    Err(Ayuv64FrameError::InsufficientPlane(_))
  ));
}

#[test]
fn ayuv64_frame_accessors_round_trip() {
  let buf = vec![0u16; 128]; // stride=32, height=4 → 128 u16 elements
  let f = Ayuv64LeFrame::try_new(&buf, 4, 4, 32).unwrap();
  assert_eq!(f.packed().len(), 128);
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
  assert_eq!(f.stride(), 32);
}

#[test]
fn ayuv64_le_frame_default_is_le() {
  // Phase 4: default `<const BE: bool = false>` exposed via `is_be()`.
  let buf = vec![0u16; 64];
  let f = Ayuv64LeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert!(!f.is_be());
}

#[test]
fn ayuv64_be_frame_alias_constructs() {
  // Phase 4: `Ayuv64BeFrame` alias resolves to `Ayuv64Frame<'_, true>`.
  let buf = vec![0u16; 64];
  let f = Ayuv64BeFrame::try_new(&buf, 4, 4, 16).unwrap();
  assert!(f.is_be());
  assert_eq!(f.width(), 4);
  assert_eq!(f.height(), 4);
}
