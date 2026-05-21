# Changelog

All notable changes to this crate are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] May 19, 2026

Initial `mediaframe` release — this crate is a **rename** of the
`videoframe` crate. It was previously published as `videoframe`
(version line `0.1.x`–`0.3.x`); those `videoframe` crates.io versions
are being **yanked** and superseded by `mediaframe 0.1.0` (fresh crate
identity).

### Added

- **`audio` module** — first cut of the audio-stream descriptor
  vocabulary (audio + container cluster of the `0.1.0` stream-vocab
  expansion):
  - `audio::ChannelLayout` — `#[non_exhaustive]` closed enum of
    common FFmpeg `AV_CH_LAYOUT_*` shapes (`Mono`, `Stereo`,
    `_2_1` through `_7_1` with `*Back` side-vs-back variants,
    `Hexagonal`, `Octagonal`, `Ambisonic1`/`2`/`3`) plus
    `Other(SmolStr)` lossless escape; `as_str()` returns the
    FFmpeg-canonical slug, `FromStr` is total.
  - `audio::BitRateMode` — closed `Cbr` / `Vbr` / `Abr` trichotomy
    (default `Cbr`), `to_u32`/`from_u32` for the wire codec.
  - `audio::SampleFormat` — sample-format vocabulary mirroring
    FFmpeg `AVSampleFormat` (`U8`/`S16`/`S32`/`S64`/`Flt`/`Dbl`
    packed + their `*p` planar twins), lossless `Unknown(u32)` +
    `Other(SmolStr)` escapes, `to_u32`/`from_u32` per FFmpeg
    `AV_SAMPLE_FMT_*` enum indices, `is_planar()` predicate.
  - `audio::ContainerFormat` — audio-only container vocab
    (`Mp3`, `Aac`, `Flac`, `Ogg`, `Opus`, `Wav`, `Aiff`, `Alac`,
    `Wma`, `Ape`, `Wv`, `Mka`, `M4a`, `Caf`) plus `Other(SmolStr)`.
  - `audio::Loudness` — EBU R128 / ITU-R BS.1770 measurement
    value object (`integrated_lufs`, `range_lu`, `true_peak_dbtp`,
    `sample_peak_dbfs` — all `f32`; no `Eq`/`Hash`).
  - `audio::Fingerprint` — algorithm-tagged opaque bytes
    (`{ algorithm: SmolStr, value: Vec<u8> }`), `try_new` rejects
    empty algorithm.
  - `audio::CoverArt` — embedded picture
    (`{ mime: SmolStr, data: Vec<u8> }`), `try_new` rejects empty
    mime / empty data.
  - `audio::Tags` — FFmpeg / Vorbis-Comment / iTunes-atom
    metadata: title, artist, album_artist, album, composer,
    genre, comment (`SmolStr`, `""` = absent) + year, track / disc
    number + total (`Option<u16>`) + language (`Option<SmolStr>`,
    TODO(lang) — swap to `Option<crate::Language>` after the
    capture-lang cluster lands).
- **`container::Format`** — top-level multimedia container
  vocabulary (`Mov`, `Mp4`, `Mkv`, `Webm`, `Avi`, `Flv`, `MpegTs`,
  `Ogg`, `Asf`, `Rm`, `Wmv`, `Mxf`, `Gxf`, `Threegp` — `.3gp` digit-
  prefix-renamed) plus `Other(SmolStr)`; audio-only containers live
  on [`audio::ContainerFormat`].
- **`subtitle` module** — `Format` (file / demuxer-tag axis,
  `#[non_exhaustive]` + `Other(SmolStr)`; named variants for the
  common text- and image-based formats — `Srt` / `WebVtt` / `Ass` /
  `Ssa` / `Sub` (MicroDVD) / `Mpl2` / `Lrc` / `Smi` / `Stl` / `Sbv` /
  `Ttml` / `MovText` / `DvdSub` / `PgsSub` / `HdmvPgs` / `DvbSub` /
  `XSub`; `as_str` / total `FromStr` round-trip; `is_image_based`
  helper for mediaschema's `REQUIRES_OCR` derivation) and
  `TrackOrigin` (closed unit-only enum — `Embedded` /
  `Sidecar` / `External`; stable `to_u32` / `from_u32` ids
  `0` / `1` / `2`; `Default == Embedded`). The module is gated on
  the `alloc` feature for the `Other(SmolStr)` escape.
- **`disposition::TrackDisposition`** — FFmpeg `AV_DISPOSITION_*`
  bitflags from `libavformat/avformat.h` n8.1 (`u32` backing).
  Shared across video / audio / subtitle tracks; ports the
  placeholder that used to live in `mediaschema::domain::bitflags`.
  `to_u32` / `from_u32` aliases for `bits` / `from_bits_retain` so
  unknown bits round-trip losslessly.
- **`capture` module** (alloc-gated) — EXIF / capture-metadata
  vocabulary.
  - `Device { make, model }` (private `SmolStr` fields; empty string
    means absent, never `Option<SmolStr>`; builders / setters /
    `is_empty`).
  - `GeoLocation { lat: f64, lon: f64, altitude: Option<f32> }` with
    range-validating `try_new`, ISO-6709 degrees-only
    parse/format (`from_iso6709` + `to_iso6709`, `FromStr` +
    `Display`, hand-rolled <200-line parser — no regex / no chrono).
    `(0, 0)` "Null Island" is accepted (it is a real, legal
    coordinate); only out-of-range lat/lon and structurally bad
    strings are rejected via `GeoLocationError::{LatOutOfRange,
    LonOutOfRange, Iso6709Malformed}`.
- **`lang::Language`** (alloc-gated) — validated BCP-47 language tag
  wrapping `icu_locid` `Language`/`Script`/`Region` subtags (`Copy`,
  heap-free in-rust representation; the `to_bcp47() -> String` /
  `Display` surface needs the allocator). `try_new(lang, script,
  region)` + `from_bcp47` / `Default = "und"` (ISO 639-3
  undetermined) + `is_undetermined` + `FromStr`.
  `LanguageError::{InvalidLanguage, InvalidScript, InvalidRegion,
  MalformedBcp47}`.
- **`buffa`** — hand-written `Message` / `DefaultInstance` wire
  support for every new type (see the `## Audio + container types`,
  `## Subtitle + disposition`, and `## Capture + language` sub-
  sections of the `buffa.rs` module doc). `GeoLocation` always-encodes
  `lat`/`lon` (the `(0, 0)` "Null Island" default is a real
  coordinate — proto3 zero-elision would be unsound, same defensive
  stance as `SampleAspectRatio`); `altitude` is presence-encoded
  (field emitted iff `Some`, including for `Some(0.0)`). The `buffa`
  feature now implies `alloc` (string-bearing wire codecs pull in
  `smol_str`).
- **Deps** — adds `icu_locid = "1.5"` (optional, gated on the
  `alloc` feature; itself `no_std`-friendly).

### Changes

- **Crate rename** — `videoframe` → `mediaframe`, version reset to
  `0.1.0`. The contents are carried over **verbatim**: the
  pixel-format / colour / frame vocabulary plus `Rational`,
  `FrameRate`, `FieldOrder`, `StereoMode`, `DolbyVisionConfig`, and
  `SampleAspectRatio` represented via `Rational`. No types, logic, or
  API changed other than the crate name (and the `buffa` proto
  package identifier `videoframe.v1` → `mediaframe.v1`).
- **Charter broadened** — the crate is now a *media-stream descriptor
  vocabulary* for video **+ audio + subtitle**, not video-only. Only
  the existing video vocabulary ships in `0.1.0`; audio/subtitle
  descriptor types will be added incrementally in later releases.

---

— the following entries are from the crate's `videoframe` history —

## [0.3.1] May 19, 2026

### Added

- **`frame`** — `Rational` (generic exact `num/den` ratio,
  `NonZeroU32` denominator, `1/1` default), `FrameRate` (exact fps
  `Rational` + `is_vfr` marker; deliberately not
  `mediatime::Timebase`), `FieldOrder` (FFmpeg `AVFieldOrder`,
  lossless `Unknown(u32)`, `Unknown(0)` default), `StereoMode`
  (FFmpeg `AVStereo3DType`, lossless `Unknown(u32)`, `Mono` default).
- **`color`** — `DolbyVisionConfig` (FFmpeg
  `AVDOVIDecoderConfigurationRecord`; distinct from the HDR10 static
  `HdrStaticMetadata`).
- **`buffa`** — hand-written `Message`/`DefaultInstance` wire support
  for `Rational`, `FrameRate`, `FieldOrder`, `StereoMode`,
  `DolbyVisionConfig`.
- **`frame`** — `SampleAspectRatio` → `Rational` interop
  (`SampleAspectRatio::rational`/`as_rational`,
  `From<SampleAspectRatio> for Rational`, `From<Rational> for
  SampleAspectRatio`).

### Breakage

- **`frame::SampleAspectRatio`** — now represented as a newtype over
  `Rational` (`pub struct SampleAspectRatio(Rational)`) instead of
  its own `{ num, den }` fields, making `Rational` the single source
  of truth for "exact ratio with a non-zero denominator". The public
  *method* API (`new`/`num`/`den`/`is_square`/`with_*`/`set_*`/
  `Default`/`Display`/derives) and the `buffa` wire format are
  **byte-for-byte unchanged**; only the internal representation and
  the `From` surface (added `From<Rational> for SampleAspectRatio`,
  added `rational()` alongside `as_rational()`) changed.

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
- **`color`** — `DOMAIN_EXT_BASE` + `Matrix::Bt601`
  (videoframe-domain superset id, disjoint from FFmpeg/H.273 codes).
- **`color`/`frame`** — `ContentLightLevel`, `ChromaCoord`,
  `MasteringDisplay`, `HdrStaticMetadata` (SMPTE ST 2086 / FFmpeg
  HDR10 static side-data); `Rotation`; `SampleAspectRatio`.
- **xtask** — `check` verifies colour-enum numbering against the
  pinned FFmpeg n8.1 header (vendored `ffmpeg-color.txt`).

### Breakage

- **`color`** — `Primaries`/`Transfer`/`Matrix`/
  `DynamicRange`/`ChromaLocation` renumbered to exact FFmpeg n8.1 /
  ITU-T H.273 code points; `to_u32`/`from_u32` now lossless.
- **`color::Transfer`** — `Bt470M`/`Bt470Bg` renamed to
  `Gamma22`/`Gamma28` (FFmpeg-canonical names for the identical
  transfer code 4/5; slugs / `Display` unchanged).
- **`color::Matrix`** — `Default` changed `Bt709` →
  `Unspecified` (FFmpeg `AVCOL_SPC_UNSPECIFIED`); `Info`
  default/`UNSPECIFIED` `matrix` likewise.
- **`color::ChromaCoord`** — `x`/`y` widened `u16` → `u32` so
  out-of-range wire values are preserved losslessly (no saturation).
- **`frame::Rotation`** — no longer `#[repr(u32)]`; gains
  `Unknown(u32)`.

### Changes

- **`buffa`** — standalone-enum codec elides on the type's `Default`
  (FFmpeg `UNSPECIFIED`), not proto3 wire-zero, so code `0` (e.g.
  `Matrix::Rgb`) is no longer conflated with "absent".
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

- **`color`** — ITU-T H.273 enums (`Matrix`, `Primaries`,
  `Transfer`, `DynamicRange`, `ChromaLocation`) bundled into
  `Info`. Plus `DcpTargetGamut` for DCI-XYZ target-gamut
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
  `Info`. **No timestamp**, no backend extras — pure pixel
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
