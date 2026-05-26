<div align="center">
<h1>mediaframe</h1>
</div>
<div align="center">

A common media-stream descriptor vocabulary for media processing pipelines — codec, pixel-format, colour, frame, audio, subtitle, container, track-disposition, EXIF capture, and BCP-47 language metadata.

[<img alt="github" src="https://img.shields.io/badge/github-findit--ai/mediaframe-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fmediaframe" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/findit-ai/mediaframe/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/findit-ai/mediaframe?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-mediaframe-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/mediaframe?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/mediaframe?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Overview

A common media-stream descriptor vocabulary for media processing
pipelines. The codec module covers video + audio + subtitle codec
identifiers. Frame-side modules — pixel-format / colour / frame /
source — cover the video pipeline. Stream-side modules cover audio
(channel layout, sample / container format, bit-rate mode, EBU R128
loudness, fingerprint, tags + cover art), subtitle (format + track
origin), top-level multimedia container format, FFmpeg track
dispositions, EXIF capture metadata (device + geolocation), and
BCP-47 language. Pure data types: no SIMD, no decoder, no codec
implementation, no math — just the shared spine that a color-
conversion library, a decoder backend, and a frame consumer can all
speak to without agreeing on anything heavier.

## What it provides

- **`codec`** — `VideoCodec`, `AudioCodec`, `SubtitleCodec` stream-
  descriptor enums covering **every** codec FFmpeg `n8.1` knows
  under each media type, plus an `Other(SmolStr)` lossless escape
  for codecs added upstream before the table is regenerated.
  Generated by `cargo xtask gen-codec` from the vendored FFmpeg
  table — see [xtask](#xtask). Requires `alloc` (gated behind
  `any(feature = "std", feature = "alloc")` for the `Other(SmolStr)`
  arm).
- **`color`** — ITU-T H.273 colour-metadata enums (`color::Matrix`,
  `color::Primaries`, `color::Transfer`, `color::DynamicRange`,
  `color::ChromaLocation`) bundled into `color::Info`, with
  FFmpeg-exact code points and a lossless `Unknown(u32)` arm on
  each. Plus `DcpTargetGamut`
  (DCI-XYZ target-gamut selection), `Rotation`, HDR static side-
  data (`ContentLightLevel`, `ChromaCoord`, `MasteringDisplay`,
  `HdrStaticMetadata` per SMPTE ST 2086 / FFmpeg HDR10), and
  `DolbyVisionConfig` (FFmpeg `AVDOVIDecoderConfigurationRecord`).
  Colour-enum numbering is CI-checked against the pinned FFmpeg
  header by `cargo xtask check`.
- **`pixel_format`** — single `PixelFormat` enum covering **every**
  pixel format in FFmpeg `n8.1`'s `AVPixelFormat` (254 variants
  excluding GPU-resident HW formats) plus cinema-RAW additions.
  Coverage is verified by `cargo xtask check` against vendored
  `pixfmt.h` slugs — see [xtask](#xtask).
- **`frame`** — structural primitives (`Dimensions`, `Rect`,
  `Plane<B>`), exact-ratio building blocks (`Rational`,
  `FrameRate`, `SampleAspectRatio` as a `Rational` newtype), stream-
  descriptor metadata (`FieldOrder`, `StereoMode` — both with
  lossless `Unknown(u32)`), `BayerPattern` for color-filter-array
  description, the runtime-tagged `VideoFrame<P, B>`, and the
  orthogonal `TimestampedFrame<F>` wrapper bundling
  `mediatime::Timestamp` PTS + duration around any inner frame
  shape. Plus per-format typed `*Frame<'a, BE>` zero-copy borrow
  views + `*FrameError` validation (feature-gated).
- **`source`** — per-format marker ZSTs (`Yuv420p`, `Nv12`, `Rgb24`,
  …), `*Row<'a>` borrows, `*Sink` subtraits, and `*_to` walker fns
  that iterate Frame → Row → `PixelSink`. The walker macro generates
  the marker / Row / Sink / walker quartet uniformly. Marker
  construction is `Foo::new()` (private `()` field locks shape
  evolution to additive changes).
- **`container`** — top-level multimedia `container::Format` enum
  (`Mov`, `Mp4`, `Mkv`, `Webm`, `Avi`, `MpegTs`, …) with an
  `Other(SmolStr)` lossless escape. Distinct from audio-only
  containers, which live on `audio::ContainerFormat`. Requires the
  `alloc` feature (`std` includes it).
- **`audio`** — audio-stream descriptor vocabulary:
  `ChannelLayout` (FFmpeg `AV_CHANNEL_LAYOUT_*` bitmask),
  `ContainerFormat` (audio-only containers: `mp3` / `flac` / `wav`
  / …), `SampleFormat`, `BitRateMode` (CBR / VBR / ABR), `Loudness`
  (EBU R128 integrated / true-peak / LRA), `Fingerprint`,
  embedded-metadata `Tags`, and `CoverArt` payload. Requires the
  `alloc` feature.
- **`subtitle`** — `subtitle::Format` (`Srt` / `WebVtt` / `Ass` /
  image-based `HdmvPgs` / …) + `TrackOrigin`
  (`Embedded` / `Sidecar` / `External`). Pure stream-descriptor
  vocabulary — no per-cue content. Requires the `alloc` feature.
- **`lang`** — validated BCP-47 `Language` tag (language subtag +
  optional script + optional region), wrapping `icu_locale_core`'s
  `TinyAsciiStr`-backed subtag types so the value is `Copy` /
  heap-free. Validates via `LanguageIdentifier::try_from_bytes`.
  Requires the `alloc` feature.
- **`disposition`** — `TrackDisposition` bitflags mirroring FFmpeg's
  `AV_DISPOSITION_*` (`DEFAULT`, `FORCED`, `HEARING_IMPAIRED`, …).
  Bit values are append-only, never renumbered; unknown bits
  round-trip losslessly through `to_u32` / `from_u32` via
  `bitflags::from_bits_retain` semantics. No alloc requirement —
  pure `Copy` bitflags.
- **`capture`** — EXIF / capture-metadata vocabulary: `Device`
  (capture device name / model / serial) and `GeoLocation`
  (latitude / longitude / altitude with ISO-6709 parse + format).
  Requires the `alloc` feature.
- **`buffa`** — optional `buffa` wire serialization (hand-written
  `Message` / `DefaultInstance` impls, no codegen) for the colour /
  frame / HDR vocabulary so downstream proto schemas can extern-map
  `.mediaframe.v1` → `::mediaframe`. Off by default — enable with
  `--features buffa`.
- **`serde`** — optional `serde::{Serialize, Deserialize}` for the whole
  descriptor vocabulary. Wire shape by type:
  - Open codec / format enums (`codec::*`, `container::Format`,
    `subtitle::Format`, `audio::{ChannelLayout, ContainerFormat}`) — the
    `as_str()` slug, unknown slugs ride `Other`.
  - FFmpeg-coded enums with an `Unknown(u32)` arm (colour, pixel-format,
    frame coded enums, `TrackDisposition`) — the `to_u32()` integer;
    unknown *codes* round-trip via `Unknown` (no slug form).
  - Strictly-closed coded enums (`subtitle::TrackOrigin`,
    `audio::BitRateMode`) — the `to_u32()` integer, but unknown codes are
    **rejected** as serde errors rather than collapsing to the default.
  - `audio::SampleFormat` (both `Unknown(u32)` and `Other(SmolStr)`) —
    bespoke: human-readable formats emit a string for named/`Other` and a
    number for `Unknown`; binary formats use a tagged `{Code, Slug}` wire.
  - `lang::Language` — its BCP-47 string. Validated structs (`GeoLocation`
    / `Fingerprint` / `CoverArt`) deserialize through their checking
    constructors.

  Orthogonal to the capability tiers (no-alloc Copy types included). Off
  by default — enable with `--features serde`.
- **`arbitrary`** — optional `arbitrary::Arbitrary` for the descriptor
  vocabulary (fuzzing / property tests), hand-written through each type's
  public constructors so `try_new`-validated types come out valid and
  open `Other(_)` arms are exercised. Generated values are wire-canonical
  (every value survives a serde round-trip). Std-only — the `arbitrary`
  crate itself is std-based — so this serves host-side fuzzing, not an
  embedded target. Off by default — enable with `--features arbitrary`.
- **`quickcheck`** — optional native `quickcheck::Arbitrary` for the same
  surface, via the [`quickcheck-richderive`] derive (each type carries
  `#[quickcheck(arbitrary = "…")]` pointing at a `fn(&mut Gen) -> Self`
  helper). Independent of `arbitrary`. Std-only. Off by default — enable
  with `--features quickcheck`.
- **`PixelSink`** + **`SourceFormat`** sealed traits re-exported at
  the crate root.

[`quickcheck-richderive`]: https://crates.io/crates/quickcheck-richderive

## Installation

```toml
[dependencies]
# Lean — codec + color + pixel_format + frame primitives.
# Adds `mediatime` + `derive_more` + `smol_str` (codec `Other` arm).
mediaframe = "0.1"
```

Opt into typed `*Frame<'a>` borrow views + the per-format
`source::*` walker quartet per family:

```toml
mediaframe = { version = "0.1", features = ["yuv-planar", "rgb"] }
```

Or take everything via the umbrella:

```toml
mediaframe = { version = "0.1", features = ["frame"] }
```

## Per-family feature flags

Enable only the families your pipeline actually consumes — each
flag pulls in just the matching `*Frame` validators, `*Row` borrow
types, marker ZSTs, walker fn, and Sink subtraits. The `frame`
umbrella enables all of them at once.

| Feature           | Formats                                                       |
|-------------------|---------------------------------------------------------------|
| `yuv-planar`      | `Yuv420p` / `422p` / `444p` / `440p` / `411p` / `410p` + 9-16 bit |
| `yuv-semi-planar` | `Nv12` / `16` / `21` / `24` / `42`, `P010` / `210` / `410` family |
| `yuva`            | YUVA planar 8-bit + 9-16 bit                                  |
| `yuv-packed`      | `Yuyv422`, `Uyvy422`, `Yvyu422`, `Uyyvyy411`                  |
| `yuv-444-packed`  | `V410`, `Xv30`, `Xv36`, `Ayuv64`, `Vuya`, `Vuyx`, `V30X`      |
| `y2xx`            | `Y210` / `Y212` / `Y216`                                      |
| `v210`            | `V210`                                                        |
| `rgb`             | `Rgb24` / `Bgr24` / `Rgba` / `Bgra` + 10-bit + 16-bit         |
| `rgb-float`       | `Rgbf32` / `Rgbf16` + `Rgbaf16`/`f32`                         |
| `rgb-legacy`      | `Rgb444` / `555` / `565` + Bgr counterparts                   |
| `gbr`             | `Gbrp` / `Gbrap` + 9-16 bit + float                           |
| `gray`            | `Gray8` / 9-16 bit / f32, `Ya8` / `Ya16`                      |
| `bayer`           | Bayer 8 / 10 / 12 / 14 / 16-bit × 4 patterns                  |
| `xyz`             | `Xyz12` (DCI-XYZ)                                             |
| `mono`            | `Monoblack` / `Monowhite` / `Pal8`                            |
| `frame`           | umbrella — enables every sub-feature above                    |

Deps pulled in by family features:

- `thiserror` — every per-family feature (for `*FrameError`).
- `half` — `rgb-float`, `gbr`, `gray` (for `half::f16`).
- `derive_more` `try_unwrap` / `unwrap` — `yuv-444-packed`, `y2xx`.

## `no_std`

```toml
# Pure no_std — just enums, marker ZSTs, structural primitives.
mediaframe = { version = "0.1", default-features = false }

# no_std + alloc — adds Vec-using helpers and tests.
mediaframe = { version = "0.1", default-features = false, features = ["alloc"] }
```

The `color`, `cfa`, and `pixel_format` modules work without `alloc`
(pure enums / `Copy` types). The `codec` module is gated on
`any(feature = "std", feature = "alloc")` because of its
`Other(SmolStr)` arm. Per-family `frame` / `source` features work
under `no_std + alloc`.

## xtask

`cargo xtask sync` fetches the pinned FFmpeg release tag (currently
`n8.1`) and refreshes the vendored tables under `xtask/vendor/`:

- `ffmpeg-pixfmts.txt` — every `AV_PIX_FMT_<NAME>` slug from
  `libavutil/pixfmt.h`.
- `ffmpeg-color.txt` — every colour-enum code point from
  `libavutil/pixfmt.h` (matrix / primaries / transfer / range /
  chroma location).
- `ffmpeg-codecs.txt` — every codec identifier under media types
  `video` / `audio` / `subtitle` from `libavcodec/codec_desc.c`.

`cargo xtask gen-codec` regenerates `src/codec.rs` from
`ffmpeg-codecs.txt` — one `VideoCodec` / `AudioCodec` /
`SubtitleCodec` enum variant per FFmpeg codec.

`cargo xtask check` diffs the vendored tables against the in-tree
enums and fails on any missing variant or numbering drift:
`PixelFormat::as_str()` vs `ffmpeg-pixfmts.txt`, colour-enum code
points vs `ffmpeg-color.txt`, and the codec enum variants vs
`ffmpeg-codecs.txt`. CI runs this so every enum stays exhaustive
against the pinned FFmpeg version.

Vendoring only the slug / code-point lists (not the LGPL FFmpeg
headers verbatim) sidesteps the license question.

#### License

`mediaframe` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2026 FinDIT Studio authors.

[Github-url]: https://github.com/findit-ai/mediaframe/
[CI-url]: https://github.com/findit-ai/mediaframe/actions/workflows/ci.yml
[doc-url]: https://docs.rs/mediaframe
[crates-url]: https://crates.io/crates/mediaframe
[codecov-url]: https://app.codecov.io/gh/findit-ai/mediaframe/
