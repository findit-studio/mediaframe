//! Planar GBR + A 32-bit-per-channel (`AV_PIX_FMT_GBRAP32{LE,BE}`) — four
//! full-resolution `u32` planes in **G, B, R, A** order (FFmpeg convention).
//!
//! All 32 bits of each `u32` element are active (full `u32` range); alpha is
//! real per-pixel α (1:1 with G), not padding. FFmpeg added this planar
//! 32-bit RGBA format for Vulkan FFv1 decoding.
//!
//! The marker carries `<const BE: bool = false>`: `Gbrap32`
//! (= `Gbrap32<false>`) is the LE source (`AV_PIX_FMT_GBRAP32LE`);
//! `Gbrap32<true>` is the BE source (`AV_PIX_FMT_GBRAP32BE`). The walker
//! [`gbrap32_to::<BE>`] propagates `BE` from
//! [`Gbrap32Frame<'_, BE>`](crate::frame::Gbrap32Frame) into the sinker
//! dispatch; the kernel byte-swaps each `u32` per `BE`.
//!
//! # API naming
//!
//! The public row API exposes only the externally-correct `g()` / `b()` /
//! `r()` / `a()` accessors; the marker / frame carry the `g/b/r` channel
//! order natively.

use crate::{
  PixelSink, SourceFormat,
  color::Matrix,
  frame::{Gbrap32Frame, Gbrap32LeFrame},
  source::sealed::Sealed,
};

/// Zero-sized marker for the planar GBRAP 32-bit source format
/// (`AV_PIX_FMT_GBRAP32{LE,BE}`). `<const BE: bool>` defaults to `false`
/// (LE — `AV_PIX_FMT_GBRAP32LE`); `Gbrap32` resolves to `Gbrap32<false>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Gbrap32<const BE: bool = false>;

impl<const BE: bool> Sealed for Gbrap32<BE> {}
impl<const BE: bool> SourceFormat for Gbrap32<BE> {}

/// One output row of a [`Gbrap32`] source — four full-width `u32` slices in
/// G / B / R / A order (full 32-bit range). Use [`Self::g`] / [`Self::b`] /
/// [`Self::r`] / [`Self::a`]. Endianness is recorded on the parent
/// [`Gbrap32Frame<'_, BE>`](crate::frame::Gbrap32Frame) / sinker, not on the
/// Row itself.
#[derive(Debug, Clone, Copy)]
pub struct Gbrap32Row<'a> {
  g: &'a [u32],
  b: &'a [u32],
  r: &'a [u32],
  a: &'a [u32],
  row: usize,
  matrix: Matrix,
  full_range: bool,
}

impl<'a> Gbrap32Row<'a> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn new(
    g: &'a [u32],
    b: &'a [u32],
    r: &'a [u32],
    a: &'a [u32],
    row: usize,
    matrix: Matrix,
    full_range: bool,
  ) -> Self {
    Self {
      g,
      b,
      r,
      a,
      row,
      matrix,
      full_range,
    }
  }

  /// Green plane row — `width` `u32` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn g(&self) -> &'a [u32] {
    self.g
  }
  /// Blue plane row — `width` `u32` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn b(&self) -> &'a [u32] {
    self.b
  }
  /// Red plane row — `width` `u32` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn r(&self) -> &'a [u32] {
    self.r
  }
  /// Alpha plane row — `width` `u32` elements (opaque = `u32::MAX`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn a(&self) -> &'a [u32] {
    self.a
  }
  /// Output row index within the frame (0-based).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row(&self) -> usize {
    self.row
  }
  /// YUV/RGB conversion matrix carried through from the kernel call.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn matrix(&self) -> Matrix {
    self.matrix
  }
  /// Full-range vs limited-range flag carried through from the kernel call.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn full_range(&self) -> bool {
    self.full_range
  }
}

/// Sinks that consume rows of a [`Gbrap32`] source. The `<const BE>`
/// parameter encodes the source byte-order — sinkers typically impl for one
/// specific `BE` matching their stored `MixedSinker<Gbrap32<BE>>`
/// monomorphization. Defaults to `false` (LE) for back-compat.
pub trait Gbrap32Sink<const BE: bool = false>:
  for<'a> PixelSink<Input<'a> = Gbrap32Row<'a>>
{
}

/// Walks a [`Gbrap32Frame<'_, BE>`](crate::frame::Gbrap32Frame) row by row,
/// dispatching each row to the sink. Propagates `<const BE: bool>` from the
/// frame into [`Gbrap32Sink<BE>`]. Use the LE-only [`gbrap32_to`] wrapper for
/// pre-Phase-4 explicit-turbofish callers.
pub fn gbrap32_to_endian<S, const BE: bool>(
  src: &Gbrap32Frame<'_, BE>,
  full_range: bool,
  matrix: Matrix,
  sink: &mut S,
) -> Result<(), S::Error>
where
  S: Gbrap32Sink<BE>,
{
  sink.begin_frame(src.width(), src.height())?;

  let w = src.width() as usize;
  let h = src.height() as usize;
  let g_stride = src.g_stride() as usize;
  let b_stride = src.b_stride() as usize;
  let r_stride = src.r_stride() as usize;
  let a_stride = src.a_stride() as usize;

  let g_plane = src.g();
  let b_plane = src.b();
  let r_plane = src.r();
  let a_plane = src.a();

  for row in 0..h {
    let g = &g_plane[row * g_stride..row * g_stride + w];
    let b = &b_plane[row * b_stride..row * b_stride + w];
    let r = &r_plane[row * r_stride..row * r_stride + w];
    let a = &a_plane[row * a_stride..row * a_stride + w];
    sink.process(Gbrap32Row::new(g, b, r, a, row, matrix, full_range))?;
  }
  Ok(())
}

/// LE-only back-compat wrapper preserving the pre-Phase-4 walker signature.
/// Forwards to [`gbrap32_to_endian`] with `BE = false`.
///
/// Rust forbids defaults on function-position const-generic parameters, so an
/// explicit-turbofish caller (`gbrap32_to::<MySink>(...)`) needs this
/// single-generic wrapper to keep compiling. BE-aware callers should use
/// [`gbrap32_to_endian`] directly.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn gbrap32_to<S>(
  src: &Gbrap32LeFrame<'_>,
  full_range: bool,
  matrix: Matrix,
  sink: &mut S,
) -> Result<(), S::Error>
where
  S: Gbrap32Sink<false>,
{
  gbrap32_to_endian::<S, false>(src, full_range, matrix, sink)
}

#[cfg(all(test, feature = "std"))]
mod tests {
  use super::*;
  use crate::PixelSink;
  use core::convert::Infallible;

  struct CountingSink {
    rows_seen: usize,
    last_g_len: usize,
    last_a_len: usize,
    last_row_idx: usize,
  }

  impl PixelSink for CountingSink {
    type Input<'r> = Gbrap32Row<'r>;
    type Error = Infallible;
    fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
      Ok(())
    }
    fn process(&mut self, row: Gbrap32Row<'_>) -> Result<(), Infallible> {
      self.rows_seen += 1;
      self.last_g_len = row.g().len();
      self.last_a_len = row.a().len();
      self.last_row_idx = row.row();
      Ok(())
    }
  }

  impl Gbrap32Sink for CountingSink {}

  #[test]
  fn gbrap32_walker_visits_every_row_once() {
    let buf = std::vec![0xDEAD_BEEFu32; 4 * 4];
    let frame = Gbrap32LeFrame::new(&buf, &buf, &buf, &buf, 4, 4, 4, 4, 4, 4);
    let mut sink = CountingSink {
      rows_seen: 0,
      last_g_len: 0,
      last_a_len: 0,
      last_row_idx: 0,
    };
    gbrap32_to(&frame, true, Matrix::Bt709, &mut sink).unwrap();
    assert_eq!(sink.rows_seen, 4);
    assert_eq!(sink.last_g_len, 4);
    assert_eq!(sink.last_a_len, 4);
    assert_eq!(sink.last_row_idx, 3);
  }

  // Compile-pass regression: the hand-written `gbrap32_to` keeps the
  // single-generic signature so explicit-turbofish callers compile (mirrors
  // `gbrapf32::tests`). BE-aware callers should use `gbrap32_to_endian`.
  #[test]
  fn gbrap32_to_explicit_turbofish_one_generic_compiles() {
    #[allow(clippy::type_complexity)]
    fn _check<S: Gbrap32Sink>() {
      let _: fn(&Gbrap32LeFrame<'_>, bool, Matrix, &mut S) -> Result<(), S::Error> =
        gbrap32_to::<S>;
    }
  }
}
