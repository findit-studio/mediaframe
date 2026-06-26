use super::{
  util::{be_encoded_u16_buf, le_encoded_u16_buf},
  *,
};
use std::vec;

// ---- YUVA high-bit BE-host regression tests ---------------------------
//
// `Yuva420pFrame16` / `Yuva422pFrame16` / `Yuva444pFrame16` document a
// **LE-encoded byte-layout** contract on their `&[u16]` planes (the
// FFmpeg `*LE` byte buffer reinterpreted as `u16`). The
// `try_new_checked` validators must therefore normalize each sample
// with `u16::from_le` before comparing against `max_valid`; otherwise
// a valid LE-encoded plane on a BE host has every `u16` byte-swapped
// relative to the intended logical value and the validator falsely
// rejects every row.
//
// These tests build the planes explicitly from LE-encoded bytes via
// `to_le_bytes` and read back as `&[u16]` via `from_ne_bytes`. On an
// LE host the resulting `u16` values are identical to the intended
// literals; on a BE host every `u16` is byte-swapped relative to the
// intent, exercising the `u16::from_le` normalization inside the
// validators. Without that normalization the validators would falsely
// reject every valid LE-encoded plane on a BE host.
//
// Each family covers (1) a positive case — a logical LE buffer of
// valid samples (including the alpha plane) that must be accepted on
// both LE and BE hosts — and (2) a negative case where a sample is
// invalid even after `from_le` normalization, ensuring the validator
// still surfaces real errors. Negative cases place the bad sample on
// the alpha plane to give that plane its own dedicated coverage.

// ---- Yuva420p10 -------------------------------------------------------

#[test]
fn yuva420p10_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 10-bit-low-packed white = 1023 (LE bytes [0xFF, 0x03]). The alpha
  // plane is full-width × full-height; chroma is half × half.
  let intended_y = vec![1023u16; 16 * 8];
  let intended_uv = vec![512u16; 8 * 4];
  let intended_a = vec![1023u16; 16 * 8];
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  let a = le_encoded_u16_buf(&intended_a);
  Yuva420p10Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16)
    .expect("LE-encoded valid yuva420p10le must be accepted on both LE and BE hosts");
}

#[test]
fn yuva420p10_try_new_checked_rejects_le_encoded_alpha_out_of_range_on_any_host() {
  // After `u16::from_le` normalization the offending alpha sample is
  // 1024 (just above the 10-bit max of 1023). On both LE and BE hosts
  // the validator must catch this — the LE-encoded byte buffer carries
  // the logical value 1024 in `a[3 * 16 + 5]`.
  let intended_y = vec![0u16; 16 * 8];
  let intended_uv = vec![512u16; 8 * 4];
  let mut intended_a = vec![1023u16; 16 * 8];
  intended_a[3 * 16 + 5] = 1024;
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  let a = le_encoded_u16_buf(&intended_a);
  let e = Yuva420p10Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16).unwrap_err();
  assert!(matches!(e, Yuva420pFrame16Error::SampleOutOfRange(_)));
}

// ---- Yuva422p10 -------------------------------------------------------

#[test]
fn yuva422p10_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 4:2:2 geometry: Y/A are full-width × full-height; U/V are
  // half-width × full-height. 10-bit white = 1023 (LE bytes
  // [0xFF, 0x03]).
  let intended_y = vec![1023u16; 16 * 8];
  let intended_uv = vec![512u16; 8 * 8];
  let intended_a = vec![1023u16; 16 * 8];
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  let a = le_encoded_u16_buf(&intended_a);
  Yuva422p10Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16)
    .expect("LE-encoded valid yuva422p10le must be accepted on both LE and BE hosts");
}

#[test]
fn yuva422p10_try_new_checked_rejects_le_encoded_alpha_out_of_range_on_any_host() {
  // Plant an out-of-range logical alpha sample (1024 > 10-bit max
  // 1023) in the LE byte buffer. The validator must surface the
  // normalized logical value on both LE and BE hosts.
  let intended_y = vec![0u16; 16 * 8];
  let intended_uv = vec![512u16; 8 * 8];
  let mut intended_a = vec![1023u16; 16 * 8];
  intended_a[2 * 16 + 7] = 1024;
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  let a = le_encoded_u16_buf(&intended_a);
  let e = Yuva422p10Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16).unwrap_err();
  assert!(matches!(e, Yuva422pFrame16Error::SampleOutOfRange(_)));
}

// ---- Yuva444p10 -------------------------------------------------------

#[test]
fn yuva444p10_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 4:4:4 geometry: every plane (Y, U, V, A) is full-width ×
  // full-height. 10-bit white = 1023 (LE bytes [0xFF, 0x03]).
  let intended_full = vec![1023u16; 16 * 8];
  let intended_chroma = vec![512u16; 16 * 8];
  let y = le_encoded_u16_buf(&intended_full);
  let u = le_encoded_u16_buf(&intended_chroma);
  let v = le_encoded_u16_buf(&intended_chroma);
  let a = le_encoded_u16_buf(&intended_full);
  Yuva444p10Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 16, 16, 16)
    .expect("LE-encoded valid yuva444p10le must be accepted on both LE and BE hosts");
}

#[test]
fn yuva444p10_try_new_checked_rejects_le_encoded_alpha_out_of_range_on_any_host() {
  // Plant an out-of-range logical alpha sample (1024 > 10-bit max
  // 1023). The validator must catch this regardless of host endianness.
  let intended_y = vec![0u16; 16 * 8];
  let intended_chroma = vec![512u16; 16 * 8];
  let mut intended_a = vec![1023u16; 16 * 8];
  intended_a[4 * 16 + 9] = 1024;
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_chroma);
  let v = le_encoded_u16_buf(&intended_chroma);
  let a = le_encoded_u16_buf(&intended_a);
  let e = Yuva444p10Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 16, 16, 16).unwrap_err();
  assert!(matches!(e, Yuva444pFrame16Error::SampleOutOfRange(_)));
}

// ---- BE checked-constructor regressions -------------------------------
//
// `try_new_checked` on `*BeFrame` (i.e. `<const BE = true>`) MUST
// normalize via `u16::from_be` before the range check. See
// `subsampled_4_2_0_high_bit::p010_be_try_new_checked_*` tests for
// the full rationale.

#[test]
fn yuva420p10_be_try_new_checked_accepts_be_encoded_buffer_on_any_host() {
  let intended_y = vec![1023u16; 16 * 8];
  let intended_uv = vec![512u16; 8 * 4];
  let intended_a = vec![1023u16; 16 * 8];
  let y = be_encoded_u16_buf(&intended_y);
  let u = be_encoded_u16_buf(&intended_uv);
  let v = be_encoded_u16_buf(&intended_uv);
  let a = be_encoded_u16_buf(&intended_a);
  Yuva420p10BeFrame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16)
    .expect("BE-encoded valid yuva420p10be must be accepted on both LE and BE hosts");
}

#[test]
fn yuva420p10_be_try_new_checked_rejects_be_encoded_alpha_out_of_range() {
  // Logical 1024 (just above 10-bit max) on the alpha plane —
  // BE-encoded — must be rejected on every host.
  let intended_y = vec![1023u16; 16 * 8];
  let intended_uv = vec![512u16; 8 * 4];
  let mut intended_a = vec![1023u16; 16 * 8];
  intended_a[3 * 16 + 5] = 1024;
  let y = be_encoded_u16_buf(&intended_y);
  let u = be_encoded_u16_buf(&intended_uv);
  let v = be_encoded_u16_buf(&intended_uv);
  let a = be_encoded_u16_buf(&intended_a);
  let e = Yuva420p10BeFrame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16).unwrap_err();
  assert!(matches!(e, Yuva420pFrame16Error::SampleOutOfRange(_)));
}

// ---- Yuva420p12 (non-FFmpeg-enum 12-bit low-bit-packed) ---------------
//
// `BITS == 12` was relaxed in for the non-FFmpeg `yuva420p12le` layout
// (WebCodecs I420AP12 / non-FFmpeg decoders). 12 active low bits, high
// 4 bits zero — `try_new_checked` rejects stray high bits (anything
// above the 12-bit max of 4095, i.e. any bit in the 0xF000 mask).

#[test]
fn yuva420p12_try_new_accepts_bits_12() {
  // BITS=12 must now construct (was previously rejected as unsupported).
  let y = vec![0u16; 16 * 8];
  let uv = vec![0u16; 8 * 4];
  let a = vec![0u16; 16 * 8];
  let f = Yuva420p12Frame::try_new(&y, &uv, &uv, &a, 16, 8, 16, 8, 8, 16)
    .expect("BITS=12 yuva420p12le must construct");
  assert_eq!(f.bits(), 12);
  assert_eq!(f.width(), 16);
  assert_eq!(f.height(), 8);
}

#[test]
fn yuva420p12_try_new_checked_accepts_le_encoded_buffer_on_any_host() {
  // 12-bit-low-packed white = 4095 (LE bytes [0xFF, 0x0F]).
  let intended_y = vec![4095u16; 16 * 8];
  let intended_uv = vec![2048u16; 8 * 4];
  let intended_a = vec![4095u16; 16 * 8];
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  let a = le_encoded_u16_buf(&intended_a);
  Yuva420p12Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16)
    .expect("LE-encoded valid yuva420p12le must be accepted on both LE and BE hosts");
}

#[test]
fn yuva420p12_try_new_checked_rejects_stray_high_bits_on_any_host() {
  // 0x1000 (= 4096) sets a bit in the 0xF000 high-nibble mask — one
  // above the 12-bit max of 4095. `try_new_checked` must reject it on
  // every host (LE-encoded byte buffer carries logical 4096).
  let intended_y = vec![0u16; 16 * 8];
  let intended_uv = vec![2048u16; 8 * 4];
  let mut intended_a = vec![4095u16; 16 * 8];
  intended_a[3 * 16 + 5] = 0x1000; // stray high bit set
  let y = le_encoded_u16_buf(&intended_y);
  let u = le_encoded_u16_buf(&intended_uv);
  let v = le_encoded_u16_buf(&intended_uv);
  let a = le_encoded_u16_buf(&intended_a);
  let e = Yuva420p12Frame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16).unwrap_err();
  assert!(matches!(e, Yuva420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuva420p12_be_alias_constructs_and_reports_be() {
  // The const-generic `BE` alias resolves and reports its byte order.
  let y = vec![0u16; 16 * 8];
  let uv = vec![0u16; 8 * 4];
  let a = vec![0u16; 16 * 8];
  let f = Yuva420p12BeFrame::try_new(&y, &uv, &uv, &a, 16, 8, 16, 8, 8, 16).unwrap();
  assert!(f.is_be());
  assert_eq!(f.bits(), 12);
}

#[test]
fn yuva420p12_be_try_new_checked_accepts_be_encoded_buffer_on_any_host() {
  // 12-bit-low-packed white = 4095, BE-encoded. The `try_new_checked`
  // validator normalizes via `u16::from_be` before the range check, so a
  // valid BE buffer must be accepted on both LE and BE hosts.
  let intended_y = vec![4095u16; 16 * 8];
  let intended_uv = vec![2048u16; 8 * 4];
  let intended_a = vec![4095u16; 16 * 8];
  let y = be_encoded_u16_buf(&intended_y);
  let u = be_encoded_u16_buf(&intended_uv);
  let v = be_encoded_u16_buf(&intended_uv);
  let a = be_encoded_u16_buf(&intended_a);
  Yuva420p12BeFrame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16)
    .expect("BE-encoded valid yuva420p12be must be accepted on both LE and BE hosts");
}

#[test]
fn yuva420p12_be_try_new_checked_rejects_stray_high_bits_on_any_host() {
  // 0x1000 (= 4096) sets a bit in the 0xF000 high-nibble mask — one
  // above the 12-bit max of 4095 — on the alpha plane, BE-encoded. After
  // `u16::from_be` normalization the validator must reject it on every
  // host.
  let intended_y = vec![4095u16; 16 * 8];
  let intended_uv = vec![2048u16; 8 * 4];
  let mut intended_a = vec![4095u16; 16 * 8];
  intended_a[3 * 16 + 5] = 0x1000; // stray high bit set
  let y = be_encoded_u16_buf(&intended_y);
  let u = be_encoded_u16_buf(&intended_uv);
  let v = be_encoded_u16_buf(&intended_uv);
  let a = be_encoded_u16_buf(&intended_a);
  let e = Yuva420p12BeFrame::try_new_checked(&y, &u, &v, &a, 16, 8, 16, 8, 8, 16).unwrap_err();
  assert!(matches!(e, Yuva420pFrame16Error::SampleOutOfRange(_)));
}

#[test]
fn yuva420p_frame16_still_rejects_bits_14() {
  // Only 9/10/12/16 are accepted for YUVA 4:2:0; 14-bit has no decoder.
  let y = vec![0u16; 16 * 8];
  let uv = vec![0u16; 8 * 4];
  let a = vec![0u16; 16 * 8];
  let e = Yuva420pFrame16::<14, false>::try_new(&y, &uv, &uv, &a, 16, 8, 16, 8, 8, 16).unwrap_err();
  assert!(matches!(e, Yuva420pFrame16Error::UnsupportedBits(_)));
}
