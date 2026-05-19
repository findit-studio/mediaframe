//! Source pixel-format kernels — marker ZSTs, per-row borrow types
//! (`*Row<'a>`), per-format `Sink` subtraits, and walker fns that
//! iterate `crate::frame::*Frame<'a, BE>` row-by-row dispatching to
//! a [`PixelSink`].
//!
//! Gated per family — match `crate::frame::*` features (yuv-planar,
//! yuv-semi-planar, yuva, yuv-packed, yuv-444-packed, y2xx, v210,
//! rgb, rgb-float, rgb-legacy, gbr, gray, bayer, xyz, mono).

// === Core traits (always available) ===

/// Base trait for sinks that consume rows of a source pixel format.
///
/// One sub-module and kernel per source pixel-format family. The
/// walker fns iterate `crate::frame::*Frame<'a, BE>` row-by-row
/// and dispatch each row to a `PixelSink` implementation.
pub trait PixelSink {
  /// The shape of one input unit chosen by the per-format subtrait —
  /// e.g. [`Yuv420pRow`] for YUV 4:2:0, one row at a time.
  type Input<'a>;

  /// The error type surfaced by this sink. Use
  /// [`core::convert::Infallible`] for sinks that can't fail — the
  /// compiler eliminates the `Result` branching at the call sites.
  type Error;

  /// Called by the walker exactly once per frame, **before** any
  /// [`process`](Self::process) call, with the source frame's
  /// dimensions.
  ///
  /// Default is `Ok(())`.
  #[allow(unused_variables)]
  fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
    Ok(())
  }

  /// Consume one input unit. Called by the kernel once per unit (one
  /// row, for the row-granular kernels currently shipped). Input
  /// borrows may be invalidated after the call returns —
  /// implementations must not retain them.
  fn process(&mut self, input: Self::Input<'_>) -> Result<(), Self::Error>;
}

/// Sealed marker trait identifying a source pixel format.
///
/// Used as a type parameter on sinks that specialize per source.
/// Implementors are the zero-sized markers in [`crate::source`].
pub trait SourceFormat: sealed::Sealed {}

/// Internal module implementing the sealed-trait pattern for
/// [`SourceFormat`]. External crates cannot name `Sealed`, so they
/// cannot implement [`SourceFormat`] themselves.
pub(crate) mod sealed {
  /// Crate-private marker trait used to prevent downstream
  /// implementations of [`super::SourceFormat`].
  pub trait Sealed {}
}

// === Macros (internal — `marker!` is also re-exported at crate root) ===

#[macro_use]
mod marker_macro;
#[macro_use]
mod walker_macro;

// === Per-format modules (feature-gated) ===

// --- yuv-planar ---
#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv420p;
#[cfg(feature = "yuv-planar")]
pub use yuv420p::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv420p9;
#[cfg(feature = "yuv-planar")]
pub use yuv420p9::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv420p10;
#[cfg(feature = "yuv-planar")]
pub use yuv420p10::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv420p12;
#[cfg(feature = "yuv-planar")]
pub use yuv420p12::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv420p14;
#[cfg(feature = "yuv-planar")]
pub use yuv420p14::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv420p16;
#[cfg(feature = "yuv-planar")]
pub use yuv420p16::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv422p;
#[cfg(feature = "yuv-planar")]
pub use yuv422p::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv422p9;
#[cfg(feature = "yuv-planar")]
pub use yuv422p9::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv422p10;
#[cfg(feature = "yuv-planar")]
pub use yuv422p10::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv422p12;
#[cfg(feature = "yuv-planar")]
pub use yuv422p12::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv422p14;
#[cfg(feature = "yuv-planar")]
pub use yuv422p14::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv422p16;
#[cfg(feature = "yuv-planar")]
pub use yuv422p16::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv440p;
#[cfg(feature = "yuv-planar")]
pub use yuv440p::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv440p10;
#[cfg(feature = "yuv-planar")]
pub use yuv440p10::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv440p12;
#[cfg(feature = "yuv-planar")]
pub use yuv440p12::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv444p;
#[cfg(feature = "yuv-planar")]
pub use yuv444p::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv444p9;
#[cfg(feature = "yuv-planar")]
pub use yuv444p9::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv444p10;
#[cfg(feature = "yuv-planar")]
pub use yuv444p10::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv444p12;
#[cfg(feature = "yuv-planar")]
pub use yuv444p12::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv444p14;
#[cfg(feature = "yuv-planar")]
pub use yuv444p14::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv444p16;
#[cfg(feature = "yuv-planar")]
pub use yuv444p16::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv411p;
#[cfg(feature = "yuv-planar")]
pub use yuv411p::*;

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod yuv410p;
#[cfg(feature = "yuv-planar")]
pub use yuv410p::*;

// --- yuv-semi-planar ---
#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv12;
#[cfg(feature = "yuv-semi-planar")]
pub use nv12::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv16;
#[cfg(feature = "yuv-semi-planar")]
pub use nv16::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv21;
#[cfg(feature = "yuv-semi-planar")]
pub use nv21::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv24;
#[cfg(feature = "yuv-semi-planar")]
pub use nv24::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv42;
#[cfg(feature = "yuv-semi-planar")]
pub use nv42::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p010;
#[cfg(feature = "yuv-semi-planar")]
pub use p010::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p012;
#[cfg(feature = "yuv-semi-planar")]
pub use p012::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p016;
#[cfg(feature = "yuv-semi-planar")]
pub use p016::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p210;
#[cfg(feature = "yuv-semi-planar")]
pub use p210::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p212;
#[cfg(feature = "yuv-semi-planar")]
pub use p212::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p216;
#[cfg(feature = "yuv-semi-planar")]
pub use p216::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p410;
#[cfg(feature = "yuv-semi-planar")]
pub use p410::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p412;
#[cfg(feature = "yuv-semi-planar")]
pub use p412::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod p416;
#[cfg(feature = "yuv-semi-planar")]
pub use p416::*;

// --- yuva ---
#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva420p;
#[cfg(feature = "yuva")]
pub use yuva420p::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva420p9;
#[cfg(feature = "yuva")]
pub use yuva420p9::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva420p10;
#[cfg(feature = "yuva")]
pub use yuva420p10::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva420p16;
#[cfg(feature = "yuva")]
pub use yuva420p16::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva422p;
#[cfg(feature = "yuva")]
pub use yuva422p::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva422p9;
#[cfg(feature = "yuva")]
pub use yuva422p9::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva422p10;
#[cfg(feature = "yuva")]
pub use yuva422p10::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva422p12;
#[cfg(feature = "yuva")]
pub use yuva422p12::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva422p16;
#[cfg(feature = "yuva")]
pub use yuva422p16::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva444p;
#[cfg(feature = "yuva")]
pub use yuva444p::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva444p9;
#[cfg(feature = "yuva")]
pub use yuva444p9::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva444p10;
#[cfg(feature = "yuva")]
pub use yuva444p10::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva444p12;
#[cfg(feature = "yuva")]
pub use yuva444p12::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva444p14;
#[cfg(feature = "yuva")]
pub use yuva444p14::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva444p16;
#[cfg(feature = "yuva")]
pub use yuva444p16::*;

// --- yuv-packed ---
#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod yuyv422;
#[cfg(feature = "yuv-packed")]
pub use yuyv422::*;

#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod uyvy422;
#[cfg(feature = "yuv-packed")]
pub use uyvy422::*;

#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod yvyu422;
#[cfg(feature = "yuv-packed")]
pub use yvyu422::*;

#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod uyyvyy411;
#[cfg(feature = "yuv-packed")]
pub use uyyvyy411::*;

// --- yuv-444-packed ---
#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod v410;
#[cfg(feature = "yuv-444-packed")]
pub use v410::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod v30x;
#[cfg(feature = "yuv-444-packed")]
pub use v30x::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod xv36;
#[cfg(feature = "yuv-444-packed")]
pub use xv36::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod ayuv64;
#[cfg(feature = "yuv-444-packed")]
pub use ayuv64::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod vuya;
#[cfg(feature = "yuv-444-packed")]
pub use vuya::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod vuyx;
#[cfg(feature = "yuv-444-packed")]
pub use vuyx::*;

// --- y2xx ---
#[cfg(feature = "y2xx")]
#[cfg_attr(docsrs, doc(cfg(feature = "y2xx")))]
mod y210;
#[cfg(feature = "y2xx")]
pub use y210::*;

#[cfg(feature = "y2xx")]
#[cfg_attr(docsrs, doc(cfg(feature = "y2xx")))]
mod y212;
#[cfg(feature = "y2xx")]
pub use y212::*;

#[cfg(feature = "y2xx")]
#[cfg_attr(docsrs, doc(cfg(feature = "y2xx")))]
mod y216;
#[cfg(feature = "y2xx")]
pub use y216::*;

// --- v210 ---
#[cfg(feature = "v210")]
#[cfg_attr(docsrs, doc(cfg(feature = "v210")))]
mod v210;
#[cfg(feature = "v210")]
pub use v210::*;

// --- rgb ---
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod rgb24;
#[cfg(feature = "rgb")]
pub use rgb24::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod bgr24;
#[cfg(feature = "rgb")]
pub use bgr24::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod rgba;
#[cfg(feature = "rgb")]
pub use rgba::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod bgra;
#[cfg(feature = "rgb")]
pub use bgra::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod argb;
#[cfg(feature = "rgb")]
pub use argb::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod abgr;
#[cfg(feature = "rgb")]
pub use abgr::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod xrgb;
#[cfg(feature = "rgb")]
pub use xrgb::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod rgbx;
#[cfg(feature = "rgb")]
pub use rgbx::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod xbgr;
#[cfg(feature = "rgb")]
pub use xbgr::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod bgrx;
#[cfg(feature = "rgb")]
pub use bgrx::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod rgb48;
#[cfg(feature = "rgb")]
pub use rgb48::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod bgr48;
#[cfg(feature = "rgb")]
pub use bgr48::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod rgba64;
#[cfg(feature = "rgb")]
pub use rgba64::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod bgra64;
#[cfg(feature = "rgb")]
pub use bgra64::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod x2rgb10;
#[cfg(feature = "rgb")]
pub use x2rgb10::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod x2bgr10;
#[cfg(feature = "rgb")]
pub use x2bgr10::*;

// --- rgb-float ---
#[cfg(feature = "rgb-float")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-float")))]
mod rgbf32;
#[cfg(feature = "rgb-float")]
pub use rgbf32::*;

#[cfg(feature = "rgb-float")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-float")))]
mod rgbf16;
#[cfg(feature = "rgb-float")]
pub use rgbf16::*;

// --- rgb-legacy ---
#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod rgb444;
#[cfg(feature = "rgb-legacy")]
pub use rgb444::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod rgb555;
#[cfg(feature = "rgb-legacy")]
pub use rgb555::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod rgb565;
#[cfg(feature = "rgb-legacy")]
pub use rgb565::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod bgr444;
#[cfg(feature = "rgb-legacy")]
pub use bgr444::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod bgr555;
#[cfg(feature = "rgb-legacy")]
pub use bgr555::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod bgr565;
#[cfg(feature = "rgb-legacy")]
pub use bgr565::*;

// --- gbr ---
#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrp;
#[cfg(feature = "gbr")]
pub use gbrp::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrap;
#[cfg(feature = "gbr")]
pub use gbrap::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrp9;
#[cfg(feature = "gbr")]
pub use gbrp9::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrp10;
#[cfg(feature = "gbr")]
pub use gbrp10::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrp12;
#[cfg(feature = "gbr")]
pub use gbrp12::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrp14;
#[cfg(feature = "gbr")]
pub use gbrp14::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrp16;
#[cfg(feature = "gbr")]
pub use gbrp16::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrap10;
#[cfg(feature = "gbr")]
pub use gbrap10::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrap12;
#[cfg(feature = "gbr")]
pub use gbrap12::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrap14;
#[cfg(feature = "gbr")]
pub use gbrap14::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrap16;
#[cfg(feature = "gbr")]
pub use gbrap16::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrpf32;
#[cfg(feature = "gbr")]
pub use gbrpf32::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrapf32;
#[cfg(feature = "gbr")]
pub use gbrapf32::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrpf16;
#[cfg(feature = "gbr")]
pub use gbrpf16::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod gbrapf16;
#[cfg(feature = "gbr")]
pub use gbrapf16::*;

// --- gray ---
#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray8;
#[cfg(feature = "gray")]
pub use gray8::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray9;
#[cfg(feature = "gray")]
pub use gray9::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray10;
#[cfg(feature = "gray")]
pub use gray10::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray12;
#[cfg(feature = "gray")]
pub use gray12::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray14;
#[cfg(feature = "gray")]
pub use gray14::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray16;
#[cfg(feature = "gray")]
pub use gray16::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod grayf32;
#[cfg(feature = "gray")]
pub use grayf32::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod ya8;
#[cfg(feature = "gray")]
pub use ya8::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod ya16;
#[cfg(feature = "gray")]
pub use ya16::*;

// --- mono ---
#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod monoblack;
#[cfg(feature = "mono")]
pub use monoblack::*;

#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod monowhite;
#[cfg(feature = "mono")]
pub use monowhite::*;

#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod pal8;
#[cfg(feature = "mono")]
pub use pal8::*;

// --- bayer ---
#[cfg(feature = "bayer")]
#[cfg_attr(docsrs, doc(cfg(feature = "bayer")))]
mod bayer;
#[cfg(feature = "bayer")]
pub use bayer::*;

#[cfg(feature = "bayer")]
#[cfg_attr(docsrs, doc(cfg(feature = "bayer")))]
mod bayer16;
#[cfg(feature = "bayer")]
pub use bayer16::*;

// --- xyz ---
#[cfg(feature = "xyz")]
#[cfg_attr(docsrs, doc(cfg(feature = "xyz")))]
mod xyz12;
#[cfg(feature = "xyz")]
pub use xyz12::*;

mod hsv;
pub use hsv::*;
