# Changelog

All notable changes to this crate are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] May 19, 2026

### Added

- **`buffa`** — optional `buffa` wire serialization for the colour /
  frame / HDR vocabulary (hand-written `Message`/`DefaultInstance`,
  no codegen); lets downstream proto schemas extern-map
  `.videoframe.v1` → `::videoframe`.
- **`color`/`frame`** — lossless `Unknown(u32)` catch-all on every
  colour enum, `Rotation`, and `DcpTargetGamut`: unrecognised /
  future / corrupt wire ids round-trip verbatim instead of collapsing
  to a default.
- **`color`** — `DOMAIN_EXT_BASE` + `ColorMatrix::Bt601`
  (videoframe-domain superset id, disjoint from FFmpeg/H.273 codes).
- **`color`/`frame`** — `ContentLightLevel`, `ChromaCoord`,
  `MasteringDisplay`, `HdrStaticMetadata` (SMPTE ST 2086 / FFmpeg
  HDR10 static side-data); `Rotation`; `SampleAspectRatio`.
- **xtask** — `check` verifies colour-enum numbering against the
  pinned FFmpeg n8.1 header (vendored `ffmpeg-color.txt`).

### Breakage

- **`color`** — `ColorPrimaries`/`ColorTransfer`/`ColorMatrix`/
  `ColorRange`/`ChromaLocation` renumbered to exact FFmpeg n8.1 /
  ITU-T H.273 code points; `to_u32`/`from_u32` now lossless.
- **`color::ColorTransfer`** — `Bt470M`/`Bt470Bg` renamed to
  `Gamma22`/`Gamma28` (FFmpeg-canonical names for the identical
  transfer code 4/5; slugs / `Display` unchanged).
- **`color::ColorMatrix`** — `Default` changed `Bt709` →
  `Unspecified` (FFmpeg `AVCOL_SPC_UNSPECIFIED`); `ColorInfo`
  default/`UNSPECIFIED` `matrix` likewise.
- **`color::ChromaCoord`** — `x`/`y` widened `u16` → `u32` so
  out-of-range wire values are preserved losslessly (no saturation).
- **`frame::Rotation`** — no longer `#[repr(u32)]`; gains
  `Unknown(u32)`.

### Changes

- **`buffa`** — standalone-enum codec elides on the type's `Default`
  (FFmpeg `UNSPECIFIED`), not proto3 wire-zero, so code `0` (e.g.
  `ColorMatrix::Rgb`) is no longer conflated with "absent".
- **`source::xyz12`** — `xyz12_to` requires a concrete
  `DcpTargetGamut`; passing `Unknown(_)` panics with a descriptive
  message instead of silently decoding as DCI-P3.

## [0.2.0] May 12, 2026

### Added

- Add bayer structures

### Breakage

- **`cfa`** - remove cfa mod

### Changes

- Make all error enums follows tuple enum errors

## [0.1.0] May 11, 2026

This is the first release line. Nothing has been published to
crates.io yet; everything below describes the shape of the
forthcoming `0.1.0`.

### Added

- **`color`** — ITU-T H.273 enums (`ColorMatrix`, `ColorPrimaries`,
  `ColorTransfer`, `ColorRange`, `ChromaLocation`) bundled into
  `ColorInfo`. Plus `DcpTargetGamut` for DCI-XYZ target-gamut
  selection. Each enum exposes `pub const fn as_str() -> &'static
  str` returning the FFmpeg-style wire slug, and a
  `derive_more::Display` impl routes through `as_str()` so the two
  cannot drift.
- **`cfa`** — Bayer mosaic descriptor (`BayerPattern`).
- **`pixel_format`** — single `PixelFormat` enum covering **every**
  pixel format in FFmpeg `n8.1`'s `AVPixelFormat` (254 variants
  excluding GPU-resident HW formats) plus cinema-RAW additions.
  ~270 variants total. `Unknown(u32)` preserves the raw wire value
  so `from_u32(to_u32(x)) == x` for every `x: u32`.
- **`frame::Dimensions`**, **`frame::Rect`**, **`frame::Plane<B>`** —
  structural primitives (always available).
- **`frame::VideoFrame<P, B>`** — runtime-tagged frame: dimensions,
  pixel format `P`, up to 4 `Plane<B>`, optional visible-rect crop,
  `ColorInfo`. **No timestamp**, no backend extras — pure pixel
  data. Generic over `P` (typically `PixelFormat`) and `B` (buffer
  type — `&'a [u8]` / `Vec<u8>` / `Bytes` / refcounted FFmpeg buffer).
- **`frame::TimestampedFrame<F>`** — orthogonal time-carrying wrapper
  bundling `Option<mediatime::Timestamp>` PTS + duration around any
  inner `F`. Composition over inheritance: pixel data stays
  independent of any timekeeping convention. Use with
  `VideoFrame<P, B>` for runtime-tagged decoder output or with
  typed `*Frame<'a, BE>` borrow views for conversion pipelines.
- **Typed `*Frame<'a, BE>` borrow types** (per-family feature-gated)
  — ~70 zero-copy validated borrow views covering planar YUV
  (4:2:0 / 4:2:2 / 4:4:4 / 4:4:0 / 4:1:1 / 4:1:0 at 8 / 9 / 10 / 12
  / 14 / 16-bit), planar YUVA (same matrix), semi-planar YUV (NV12
  / 16 / 21 / 24 / 42 + P010 / 210 / 410 families), packed YUV
  (YUYV422 / UYVY422 / YVYU422 / UYYVYY411 / V210 / V410 / XV30 /
  XV36 / AYUV64 / VUYA / VUYX / Y210 / Y212 / Y216), packed RGB
  (Rgb24 / Bgr24 / Rgba / Bgra / Argb / Abgr / Xrgb / Rgbx / Xbgr /
  Bgrx / Rgb48 / Bgr48 / Rgba64 / Bgra64 / X2Rgb10 / X2Bgr10),
  packed RGB float (Rgbf32 / Rgbf16), packed legacy RGB (Rgb444 /
  555 / 565 + Bgr counterparts), planar GBR / GBRA at 8 / 9-16 /
  float, grayscale (Gray8 / 9-16 / f32 / Ya8 / Ya16), Bayer 8 /
  10 / 12 / 14 / 16-bit × 4 patterns, Xyz12, and Pal8 / Monoblack /
  Monowhite. Each `*Frame<'a, BE>` carries a `<const BE: bool =
  false>` parameter selecting endianness; row kernels handle the
  byte-swap under the hood.
- **`source`** — per-format marker ZSTs (`Yuv420p`, `Nv12`,
  `Rgb24`, …), `*Row<'a>` borrow types, `*Sink` subtraits, and
  `*_to` walker fns that iterate Frame → Row → `PixelSink`. The
  `walker!` macro generates the marker / Row / Sink / walker
  quartet uniformly per format. The companion `marker!` macro
  generates the canonical marker shape (`pub struct Foo(())` with
  `pub const fn new()` constructor — private `()` field locks
  shape evolution to additive changes only).
- **`PixelSink`** + **`SourceFormat`** sealed traits re-exported at
  the crate root.
- **`xtask`** — dev-only Cargo subcommand. `cargo xtask sync`
  fetches FFmpeg's `libavutil/pixfmt.h` from the pinned release tag
  (currently `n8.1`) and writes the lowercase slug list to
  `xtask/vendor/ffmpeg-pixfmts.txt`. `cargo xtask check` diffs the
  vendored list against `PixelFormat::as_str()` and fails on any
  missing variant. Vendoring only the slug list (not the LGPL
  header verbatim) sidesteps the license question.

### Conventions

- **No public fields anywhere.** Every struct exposes private fields
  via `pub const fn` getters + `pub const fn new(...)` constructors
  + `#[must_use]` `with_*` consuming builders + `set_*` in-place
  setters. Applies to color types, frame primitives, all error
  payloads, and marker ZSTs.
- **Sealed-trait pattern** on `SourceFormat`: external crates can
  introspect but not extend the format set.
- **Single-source-of-truth display strings**: every enum's `Display`
  impl is derived through its `pub const fn as_str()` — no risk of
  drift between the two surfaces.
- **`derive_more::IsVariant`** on every enum (color, cfa,
  pixel_format, every `*FrameError`). Callers get `is_<variant>()`
  predicates for free.

### `*FrameError` shape

All 65 `*FrameError` enums use **newtype-tuple variants** wrapping
private-field payload structs (no struct-style variants). Pattern:

```rust
pub enum FooFrameError {
    Bar(Bar),
    Baz(Baz),
}
```

#### Shared error payloads

Common shapes live at the top of `videoframe::frame` and are reused
across every error enum that has the matching shape — variant
names carry plane / unit semantics, payload carries shape-only data:

- `ZeroDimension { width, height }`
- `DimensionOverflow { width, height }`
- `InsufficientStride { stride, min }` — wraps every
  `Insufficient*Stride` variant across the Y / U / V / A / G / B / R /
  Uv / Vu plane axes. Variant name conveys per-plane / per-unit
  semantics.
- `InsufficientPlane { expected, actual }` — wraps every
  `Insufficient*Plane` variant.
- `GeometryOverflow { stride, rows }`
- `OddWidth { width }`
- `WidthNotMultipleOf4 { width }`
- `WidthOverflow { width }`
- `UnsupportedBits { bits }`

Naming follows the **`Insufficient*` family** rather than the
historical `*TooShort` / `*TooSmall` style (e.g.
`InsufficientYPlane`, `InsufficientYStride`).

Rare / unique shapes get local payload structs adjacent to their
consumer enum: `Yuv420pFrame16SampleOutOfRange`,
`Yuva420pFrame16SampleOutOfRange`, `Yuva422pFrame16SampleOutOfRange`,
`Yuva444pFrame16SampleOutOfRange`, `BayerSampleOutOfRange`,
`PnSampleLowBitsSet`, `Xv36SampleLowBitsSetAt`, `PnUvStrideOdd`.

#### `Display` impls

Each payload struct derives `thiserror::Error` and owns its own
`#[error("...")]` message. Enum variants delegate via
`#[error(transparent)]` — display routes through the payload's
own `Display` impl. Trade-off: per-enum format-identifying
prefixes (e.g. "V210Frame: zero dimension width=X height=Y")
drop in favor of canonical payload-owned messages; format
identity lives on the typed enum (`V210FrameError`) itself.

#### Generated accessors

Every `*FrameError` derives `derive_more::{IsVariant, TryUnwrap,
Unwrap}` with `#[unwrap(ref, ref_mut)]` + `#[try_unwrap(ref,
ref_mut)]` modifiers. Each variant gets:

- `is_<variant>() -> bool`
- `unwrap_<variant>(self) -> Payload`
- `unwrap_<variant>_ref(&self) -> &Payload`
- `unwrap_<variant>_mut(&mut self) -> &mut Payload`
- `try_unwrap_<variant>(self) -> Result<Payload, Self>`
- `try_unwrap_<variant>_ref(&self) -> Result<&Payload, &Self>`
- `try_unwrap_<variant>_mut(&mut self) -> Result<&mut Payload, &mut Self>`

### Feature flags

- `default = ["std"]` — `std` and `alloc` features, mediatime,
  derive_more (`is_variant` + `display`), thiserror always pulled
  in (small, no_std-friendly).
- **Per-family feature flags** gate the typed `*Frame<'a, BE>`
  validators and the matching `source::*` walker quartet so
  consumers compile only the formats they actually use:

  | Feature           | Formats                                                  |
  |-------------------|----------------------------------------------------------|
  | `yuv-planar`      | Yuv420p / 422p / 444p / 440p / 411p / 410p + 9-16 bit    |
  | `yuv-semi-planar` | NV12 / 16 / 21 / 24 / 42, P010 / 210 / 410 family        |
  | `yuva`            | YUVA planar 8-bit + 9-16 bit                             |
  | `yuv-packed`      | YUYV422, UYVY422, YVYU422, UYYVYY411                     |
  | `yuv-444-packed`  | V410, XV30, XV36, AYUV64, VUYA, VUYX, V30X               |
  | `y2xx`            | Y210 / Y212 / Y216                                       |
  | `v210`            | V210                                                     |
  | `rgb`             | Rgb24/Bgr24/Rgba/Bgra + 10-bit + 16-bit                  |
  | `rgb-float`       | Rgbf32 / Rgbf16 + Rgbaf16/f32                            |
  | `rgb-legacy`      | Rgb444 / 555 / 565 + Bgr counterparts                    |
  | `gbr`             | Gbrp / Gbrap + 9-16 bit + float                          |
  | `gray`            | Gray8 / 9-16 bit / f32, Ya8 / Ya16                       |
  | `bayer`           | Bayer 8 / 10 / 12 / 14 / 16-bit × 4 patterns             |
  | `xyz`             | Xyz12 (DCI-XYZ)                                          |
  | `mono`            | Monoblack / Monowhite / Pal8                             |
  | `frame`           | umbrella — enables every sub-feature above               |

  Deps pulled by family features:
  - `half` — `rgb-float`, `gbr`, `gray` (for `half::f16`)
  - `derive_more` `try_unwrap` / `unwrap` features — every
    per-family feature (so all `*FrameError` enums get the full
    unwrap accessor surface).

### `no_std`

Default-feature `std` is on. `--no-default-features` builds pure
no_std (enums + `Copy` types + marker ZSTs + frame primitives).
Add `alloc` for the small set of `Vec` / `String` helpers used
under `no_std + alloc`. The `extern crate alloc as std` aliasing
pattern keeps `std::vec::Vec` / `std::format!` resolving uniformly
across feature combos.

### Verification matrix

- Default features: 36 tests
- `--no-default-features --features alloc`: 32 tests
- `--features frame`: 656 tests
- All 15 individual per-family standalone builds compile
- `cargo xtask check` validates `PixelFormat` exhaustiveness
  against vendored FFmpeg `n8.1` slugs
