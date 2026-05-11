//! Pixel-format and color-metadata types shared across the
//! colconv / mediadecode / scenesdetect stack.
//!
//! This crate is the lowest layer of the findit-studio video stack:
//! pure data types (no SIMD, no decoder, no codec). Both
//! [`colconv`](https://crates.io/crates/colconv) (color-conversion
//! kernels) and [`mediadecode`](https://crates.io/crates/mediadecode)
//! (decoder traits + adapters) consume it, so downstream crates like
//! `scenesdetect` can interoperate without taking on the heavier
//! dependencies of either.
//!
//! ## Modules
//!
//! - [`color`] — ITU-T H.273 color metadata: [`color::ColorMatrix`],
//!   [`color::ColorPrimaries`], [`color::ColorTransfer`],
//!   [`color::ColorRange`], [`color::ColorInfo`],
//!   [`color::ChromaLocation`].
//! - [`cfa`] — color-filter-array (Bayer) descriptions:
//!   [`cfa::BayerPattern`].
//! - [`frame`] — frame structural primitives ([`frame::Dimensions`],
//!   [`frame::Rect`], [`frame::Plane`], [`frame::VideoFrame`],
//!   [`frame::TimestampedFrame`]) plus typed `*Frame<'a, BE>` borrow
//!   types gated behind per-family feature flags (see below).
//!
//! ## Per-family feature flags
//!
//! Enable only the pixel-format families you need, or use the `frame`
//! umbrella to opt into all of them at once.
//!
//! | Feature           | Formats                                              |
//! |-------------------|------------------------------------------------------|
//! | `yuv-planar`      | Yuv420p / 422p / 444p / 440p / 411p / 410p + 9-16bit |
//! | `yuv-semi-planar` | NV12 / 16 / 21 / 24 / 42, P010 / 210 / 410 families  |
//! | `yuva`            | YUVA planar 8-bit + high-bit                         |
//! | `yuv-packed`      | YUYV422, UYVY422, YVYU422, UYYVYY411                 |
//! | `yuv-444-packed`  | V410, XV30, XV36, AYUV64, VUYA, VUYX, V30X           |
//! | `y2xx`            | Y210 / Y212 / Y216                                   |
//! | `v210`            | V210                                                 |
//! | `rgb`             | Rgb24/Bgr24/Rgba/Bgra + 16-bit family                |
//! | `rgb-float`       | Rgbf32 / Rgbf16                                      |
//! | `rgb-legacy`      | Rgb444/555/565 + Bgr counterparts                    |
//! | `gbr`             | Gbrp / Gbrap + 9-16bit + float                       |
//! | `gray`            | Gray8-16, Grayf32, Ya8/16                            |
//! | `bayer`           | Bayer 8-16bit, 4 patterns                            |
//! | `xyz`             | Xyz12                                                |
//! | `mono`            | Monoblack / Monowhite / Pal8                         |
//! | `frame`           | umbrella — enables every sub-feature above           |
//!
//! ## no_std
//!
//! Default-feature `std` is on. Disable for `no_std`; enable `alloc`
//! for `no_std + alloc`. The `color` and `cfa` modules work without
//! either feature (pure enums / Copy types).

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

// Alias `alloc as std` on no_std + alloc builds so code can use
// `std::vec::Vec` etc. uniformly across feature combos. When the
// `std` feature is on, the real `std` crate is already in scope via
// the prelude. The `unused_extern_crates` allow silences a
// rust-2018-idioms false positive — the alias is needed at use-time
// even though rustc can't see that statically.
#[cfg(all(not(feature = "std"), feature = "alloc"))]
#[allow(unused_extern_crates)]
extern crate alloc as std;

#[cfg(feature = "std")]
#[allow(unused_extern_crates)]
extern crate std;

pub mod cfa;
pub mod color;
pub mod frame;
pub mod pixel_format;
pub mod source;

pub use source::{PixelSink, SourceFormat};
