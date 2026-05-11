#[allow(unused_imports)]
use super::*;

// `util` is shared by the high-bit Frame tests
// (subsampled_4_2_2_high_bit, subsampled_4_4_4_high_bit, y2xx,
// packed_yuv_4_4_4). Gate the helper module behind the union of those
// consumer features so it doesn't trip dead-code lints when the
// frame-test surface is otherwise empty (e.g. miri default-feature
// builds without `--features frame`).
#[cfg(any(feature = "yuv-planar", feature = "yuv-444-packed", feature = "y2xx",))]
mod util;

#[cfg(feature = "bayer")]
mod bayer;
#[cfg(feature = "gray")]
mod grayf32;
#[cfg(feature = "rgb-legacy")]
mod legacy_rgb;
#[cfg(feature = "rgb")]
mod packed_rgb_10bit;
#[cfg(feature = "rgb")]
mod packed_rgb_16bit;
#[cfg(feature = "rgb")]
mod packed_rgb_8bit;
#[cfg(feature = "rgb-float")]
mod packed_rgb_f16;
#[cfg(feature = "rgb-float")]
mod packed_rgb_float;
#[cfg(feature = "yuv-packed")]
mod packed_yuv_4_1_1;
#[cfg(feature = "yuv-444-packed")]
mod packed_yuv_4_4_4;
#[cfg(feature = "yuv-packed")]
mod packed_yuv_8bit;
#[cfg(feature = "mono")]
mod pal8;
#[cfg(feature = "yuv-planar")]
mod planar_8bit;
#[cfg(feature = "gbr")]
mod planar_gbr_8bit;
#[cfg(feature = "gbr")]
mod planar_gbr_float;
#[cfg(feature = "gbr")]
mod planar_gbr_high_bit;
#[cfg(feature = "yuv-semi-planar")]
mod semi_planar_8bit;
#[cfg(feature = "yuv-planar")]
mod subsampled_4_2_0_high_bit;
#[cfg(feature = "yuv-planar")]
mod subsampled_4_2_2_high_bit;
#[cfg(feature = "yuv-planar")]
mod subsampled_4_4_4_high_bit;
#[cfg(feature = "v210")]
mod v210;
#[cfg(feature = "xyz")]
mod xyz12;
#[cfg(feature = "y2xx")]
mod y2xx;
#[cfg(feature = "gray")]
mod ya16;
#[cfg(feature = "gray")]
mod ya8;
#[cfg(feature = "yuva")]
mod yuva_high_bit;

// ---- 32-bit overflow regressions --------------------------------------
//
// `u32 * u32` can exceed `usize::MAX` only on 32-bit targets (wasm32,
// i686). Gate the tests so they actually run on those hosts under CI
// cross builds; on 64-bit they're trivially uninteresting (the
// product always fits). These tests stay in `tests/mod.rs` because
// they exercise both `Yuv420pFrame` (planar 8-bit family) and
// `Nv12Frame` (semi-planar 8-bit family) — cross-cutting between
// the per-family submodules above.

#[cfg(all(target_pointer_width = "32", feature = "yuv-planar"))]
#[test]
fn yuv420p_try_new_rejects_y_geometry_overflow() {
  // 0x1_0000 * 0x1_0000 = 2^32, which overflows a 32-bit `usize`
  // (max = 2^32 − 1). Even so the odd-width check passes, so we
  // actually reach `checked_mul` and hit `GeometryOverflow`.
  let big: u32 = 0x1_0000;
  let y: [u8; 0] = [];
  let u: [u8; 0] = [];
  let v: [u8; 0] = [];
  let e = Yuv420pFrame::try_new(&y, &u, &v, big, big, big, big / 2, big / 2).unwrap_err();
  assert!(matches!(e, Yuv420pFrameError::GeometryOverflow { .. }));
}

#[cfg(all(target_pointer_width = "32", feature = "yuv-semi-planar"))]
#[test]
fn nv12_try_new_rejects_geometry_overflow() {
  let big: u32 = 0x1_0000;
  let y: [u8; 0] = [];
  let uv: [u8; 0] = [];
  let e = Nv12Frame::try_new(&y, &uv, big, big, big, big).unwrap_err();
  assert!(matches!(e, Nv12FrameError::GeometryOverflow { .. }));
}
