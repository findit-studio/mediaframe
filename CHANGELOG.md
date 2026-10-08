# Changelog

All notable changes to this crate are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.0] — 2026-10-08

**Breaking:** the public dependency `mediatime` crosses 0.4 → 0.5.

### Changed

- **`mediatime` 0.4 → 0.5.** This crate re-exports no `mediatime` item, and
  `Timestamp` is the only `mediatime` type its public API carries —
  `frame::TimestampedFrame`'s `pts`/`duration` fields and its
  `pts`/`duration`/`with_pts`/`maybe_pts`/`set_pts`/`update_pts`/
  `with_duration`/`maybe_duration`/`set_duration`/`update_duration`
  accessors and builders. `Timestamp` itself did not change; a caller
  holding a `mediatime 0.4` `Timestamp` no longer type-checks against these
  signatures, which is Breaking as the 0.1 → 0.2, 0.2 → 0.3 and 0.3 → 0.4
  crossings were. What moved upstream (`mediatime`'s own changelog is the
  authority) reaches nothing here: `TimeRange`'s unchecked
  `with_start`/`with_end`/`set_start`/`set_end` gave way to checked moves,
  and `TimeRange` is not in this crate's API.
- **The `buffa` feature still forwards `mediatime/buffa`, which now brings
  `mediatime::wire`.** `mediatime` 0.5 no longer implements buffa's
  `Message` on its domain types: a buffa-generated crate holding a
  `.mediatime.v1` field beside a mediaframe type maps that package onto
  `::mediatime::wire` and converts at the edge (`TryFrom`/`From`). This
  crate's own encodings are unchanged, byte for byte. The comments that
  cited `mediatime`'s always-encode and clamp-on-decode stance as their
  precedent now cite it as of 0.4: `mediatime`'s `wire` types write
  proto3's canonical form and refuse a malformed value by name, so the
  clamp `SampleAspectRatio` and `Rational` apply on decode is this crate's
  own policy.

No other source line changed. Verified on the stable toolchain: `cargo test
--all-features`, `cargo clippy --all-features -- -D warnings`, `cargo hack
check --each-feature` and `cargo hack clippy --each-feature` over the
crate, `cargo xtask check`, and `cargo +nightly fmt --all -- --check`.

## [0.11.0] - 2026-09-12

### Changed

- **Every text seat in the crate now carries `smol_bytes::Utf8Bytes`;
  `smol_str` is gone.** One carrier for text, crate-wide — the same seat
  the `lang` household has held since 0.9.0, now also under the
  `Other(...)` escape arm of all twenty-two open vocabularies
  (`color::{Matrix, Primaries, Transfer, DynamicRange, ChromaLocation,
  DcpTargetGamut}`, `codec::{VideoCodec, AudioCodec, SubtitleCodec,
  DataCodec, AttachmentCodec}`, `container::Format`, `frame::{Rotation,
  FieldOrder, StereoMode}`, `pixel_format::PixelFormat`,
  `audio::{ChannelLayout, SampleFormat, ContainerFormat}`,
  `image::Format`, `subtitle::{Format, TrackOrigin}`), under the record
  text fields (`audio::Tags`' seven strings, `audio::ChannelSpec::label`,
  `audio::ChannelLayoutDescription::text`, `audio::CoverArt::mime`,
  `audio::Fingerprint::algorithm`, `capture::Device::{make, model}`), and
  in `capture::GeoLocationError::Iso6709Malformed`'s payload.

  **Breaking**, in exactly two shapes: code that names the escape arm's
  payload type (`Type::Other(SmolStr::new(s))`, a `let _: SmolStr` bound
  off a destructured `Other`) and code that hands a `SmolStr` to one of
  the text setters. Both are one-line fixes — `Utf8Bytes::from(s)`, and
  `&str` / `String` arguments keep working untouched, since every setter
  takes `impl Into<Utf8Bytes>` exactly as it took `impl Into<SmolStr>`.
  No method was added, removed or renamed; no getter changed its return
  type (they all still hand back `&str`).

  **One further break, found by this crate's own tests and worth calling
  out on its own: at the `alloc` / `std` tier a value of one of the
  twenty-two open vocabularies can no longer be created *and dropped*
  inside a `const` body.** `Utf8Bytes`' heap arm holds a `bytes::Bytes`,
  whose inline `AtomicPtr` makes the whole enum non-`freeze`; a const
  temporary of a non-`freeze` type cannot be promoted, so its drop lands
  in the const body and `rustc` rejects it with `E0493` ("destructor
  cannot be evaluated at compile-time"). `SmolStr`'s heap arm was an
  `Arc<str>`, which is `freeze`, so the same temporary used to promote.
  Every `const fn` on these types is still a `const fn` and still
  evaluates at compile time — what changed is where the value may live:

  ```rust
  // 0.10: compiled. 0.11 at the alloc tier: E0493.
  const W: Option<ChromaCoord> = Primaries::SmpteEg432.white_point();

  // 0.11: hold the value in a `&'static` const — nothing is dropped.
  const P3: &Primaries = &Primaries::SmpteEg432;
  const W: Option<ChromaCoord> = P3.white_point();
  ```

  Const items *of* these types (`const ROSTER: &'static [Self]`,
  `color::Info::UNSPECIFIED`) are unaffected, and so is the no-alloc
  tier, where the escape arm does not exist and the enums are plain
  fieldless vocabularies.

  **No wire or text form moved.** The open vocabularies serialize as
  their `as_str()` slug, as before; the record fields serialize as JSON
  strings, as before; `Display`, `FromStr` and the ignore-case parse
  tiers are untouched, and `GeoLocationError`'s `{0:?}` rendering is
  byte-identical (both carriers' `Debug` delegate to `str`'s). The
  existing serde, buffa and text-form tests pin all of it.

  **Why.** `SmolStr` stores up to 23 bytes inline and heap-allocates
  infallibly past that, so a caller that must not abort on a large or
  hostile string had no way to hand one over — a downstream decoder was
  dropping over-long channel labels and layout renderings as "absent"
  rather than risk the allocation. `Utf8Bytes::from(String)` takes
  ownership of a buffer the caller already built, so the fallible part
  (`Vec::try_reserve`, then the decode) happens on the caller's side and
  the value arrives here complete. mediaframe itself gains no fallible
  constructor: the seats store what they are given, and
  `ChannelSpec::with_label` / `ChannelLayoutDescription::with_text` now
  say so in their own docs.

  This reverses the note 0.9.0 attached to the `lang` seat, which kept
  `Utf8Bytes` for subtags and left `SmolStr` under the escape arms on the
  grounds that a subtag is a value with a grammar and an escape is a name
  carried verbatim. That distinction is real, and it was never about the
  *carrier*: both hold text the retrieval layer downstream addresses rows
  by. One carrier for all of it is what the crate settles on here.

### Removed

- **The `smol_str` dependency**, with its three feature rows
  (`alloc = [... "dep:smol_str" ...]`, `std = [... "smol_str?/std" ...]`,
  `serde = [... "smol_str?/serde" ...]`). `smol-bytes` — already a
  dependency since 0.9.0 — now carries all three: `alloc` enables it
  (`dep:smol-bytes`, `smol-bytes/alloc`), `std` forwards
  `smol-bytes?/std`, and `serde` forwards `smol-bytes?/serde` where
  `smol_str?/serde` stood. The capability tiers are unchanged: `Utf8Bytes`
  needs a heap exactly as `SmolStr` did, so every escape arm and every
  text field keeps its `any(feature = "std", feature = "alloc")` gate, and
  nothing that compiled at the no-alloc tier now requires `alloc`.

## [0.10.0] - 2026-09-02

### Added

- **A case-sensitivity axis for every vocabulary's parse table** (the
  0.10.0 axis, alongside the `other()` fix below). Case-sensitivity is
  now a per-household constitutional attribute: each of the crate's 22
  `Other(SmolStr)` households (listed in the `other()` entry below)
  declares its case-matching strategy explicitly, as the word passed to
  the one call that resolves its match key — `Insensitive` (fold, exactly
  how every household already matched) or `Sensitive` (exact bytes, no
  fold, no length cap). There is no default: a household that omits the
  word does not compile, so the choice is provable rather than assumed.

  **Zero behaviour change** — all 22 households declare `Insensitive`.
  Every existing roster is a lowercase-slug domain with no distinctly-cased
  member, so this PR flips nothing; it makes the crate's one matching gate
  (`fold`) and its (until now, implicit) "we always fold" assumption into
  a named, per-type choice instead. `Sensitive` has no
  production household yet — right for a **fourcc-shaped** domain
  (vendor codec tags, container FourCCs) where two different casings can
  legitimately name two different real values, so folding one onto the
  other would silently substitute a wrong roster member for the caller's
  actual value. It is proven end-to-end against a test-only table so the
  first real sensitive household inherits machinery already exercised,
  rather than being the first thing to exercise it.

  Internal (`pub(crate)`): no public API changes. `ingraph` / `mediagraph`
  citizen doors inherit each household's declared mode along with the
  household itself — no changes needed there.

### Changed

- **Every `other(...)` escape constructor now runs the ignore-case parse
  first.** Previously `Type::other(slug)` wrapped `slug` into
  `Type::Other(...)` unconditionally — it never consulted the type's own
  `FromStr` lookup, so e.g. `VideoCodec::other("h264") != VideoCodec::H264`
  despite naming the same codec: same meaning, two non-equal values. Now
  `other()` delegates to `FromStr` (the exact match table it already
  walks, never a duplicate), so a canonical spelling or a documented
  alias returns the **named** variant; only a genuine stranger still
  lands in `Other`. Applies to all twenty-two `Other(SmolStr)` households
  crate-wide: `color::{Matrix, Primaries, Transfer, DynamicRange,
  ChromaLocation, DcpTargetGamut}`, `codec::{VideoCodec, AudioCodec,
  SubtitleCodec, DataCodec, AttachmentCodec}`, `container::Format`,
  `frame::{Rotation, FieldOrder, StereoMode}`, `pixel_format::PixelFormat`,
  `audio::{ChannelLayout, SampleFormat, ContainerFormat}`,
  `image::Format`, `subtitle::{Format, TrackOrigin}`.

  **Behaviour change on the 0.x line** (0.10.0 axis): a stranger's
  spelling is now preserved **verbatim** in `Other` rather than
  ASCII-folded to lowercase — folding a name nobody claims discarded
  information (vendor fourccs and codec tags are routinely
  case-sensitive) for no benefit, now that the lookup above already
  catches every case-variant of a name this crate *does* recognise. The
  `Other(SmolStr)` wire form and every released-serde fixture are
  unaffected: only the in-memory value a mixed-case stranger produces
  changes.

  This closes the keyset defect ingraph#524 named as a precondition for
  the mirror-retirement wave: with `other()` and `FromStr` now provably
  sharing one match table, a value on a storage/cursor road can no
  longer carry two non-equal spellings of one named meaning.

## [0.9.2] - 2026-08-31

### Added

- **`codec::DataCodec` and `codec::AttachmentCodec`** — the two
  remaining track-role codec vocabularies, in the same generated shape
  as `VideoCodec` / `AudioCodec` / `SubtitleCodec`: an `Other(SmolStr)`
  lossless escape, case-insensitive `FromStr`, lowercase FFmpeg-exact
  wire names, a `ROSTER` with a compile-time completeness witness, and
  — small enough to afford it — the `Unwrap` / `TryUnwrap` pair.

  `DataCodec` is vendored exactly like its three siblings: all 11
  `AVMEDIA_TYPE_DATA` codec ids in FFmpeg n9.0's
  `libavcodec/codec_desc.c` (`bin_data`, `dvd_nav_packet`, `epg`, `klv`,
  `mpegts`, `otf`, `scte_35`, `smpte_2038`, `smpte_436m_anc`,
  `timed_id3`, `ttf`) — `cargo xtask gen-codec` / `cargo xtask check`
  now cover four vendored media types instead of three.

  `AttachmentCodec` does not, because it cannot: FFmpeg's
  `codec_desc.c` carries **zero** descriptors typed
  `AVMEDIA_TYPE_ATTACHMENT`, in n9.0 or on current FFmpeg `master` —
  attachment is a demuxer-assigned *stream* role, not a
  codec-descriptor media type. The one place FFmpeg itself assigns a
  concrete codec id to an `AVMEDIA_TYPE_ATTACHMENT` stream is
  `libavformat/matroskadec.c`'s `mkv_mime_tags` table (the Matroska /
  WebM demuxer), which is what `AttachmentCodec`'s three variants
  (`Ttf`, `Otf`, `BinData`) transcribe. All three are also `DataCodec`
  variants of the same name — the same FFmpeg codec id wearing two
  different track-role hats, not an accidental duplication. `cargo
  xtask`'s generator carries this roster as a hand-curated
  `ATTACHMENT_CODECS` constant, documented in full and checked for
  drift the same way the four vendored enums are, just against that
  constant instead of a vendored FFmpeg table.

## [0.9.1] - 2026-08-30

### Changed

- **The generated lang registry's pair-table key columns move onto the
  family's inline `Ascii<N>` seat instead of `&str`** — `Language`'s
  (`LANGUAGES`, `LANGUAGE_PREFERRED`, `LANGUAGE_SUPPRESS_SCRIPT`, `ALPHA3`),
  `ScriptSubtag`'s (`SCRIPTS`), `Region`'s (`REGIONS`, `REGION_PREFERRED`),
  and `GRANDFATHERED` on its own existing 16-byte whole-tag buffer width.
  Every key already fit its family's seat; the comparison stays `key`'s own
  text order (`Ascii`'s derived `Ord` over a zero-padded buffer, PINNED to
  agree with `str` by the crate's own ordering sweep over all 8275 registered
  languages), so every lookup's binary search and every answer are unchanged.

  **Measured, not forecast.** Relocations across the eight compacted pair
  tables (9276 rows) drop from 18,552 to 9,276 — exactly half, since every
  row loses one `&str` fat pointer's worth of relocation regardless of its
  seat width. Static size drops only ~4.0 KiB (296,832 → 292,752 bytes,
  measured via `size_of` on the actual row types) — far short of an earlier
  ~90KB forecast: tuple alignment pads `(Ascii<8>, &str)` back up to the same
  32 bytes `(&str, &str)` already was, so the four language-seat tables (94%
  of all rows) round-trip at zero byte savings each, and `GRANDFATHERED`'s
  16-byte seat is a small net regression (+8 bytes/row, on the same padding
  rule). Only the narrower `SCRIPTS` / `REGIONS` / `REGION_PREFERRED` rows
  (531 of 9276) see the full 8 bytes/row the inline seat promises.

## [0.9.0] - 2026-08-30

### Added

- **`lang` is a household, not a type** — four public types where there was
  one, and a vendored registry behind every fold they apply:

  - **`lang::Language`** — a primary language subtag in the shortest
    spelling BCP 47 has for it (`de`, `zh`, `yue`, `und`), WIDE IN: any
    ASCII case, and either ISO 639-2 alphabet. An mkv writes the
    bibliographic `ger` and an mp4 the terminological `deu`; both are `de`
    here, in one hop, and `iw` is `he` by the registry's own
    `Preferred-Value`. Publishes `name()`, `is_registered()`,
    `is_deprecated()`, `is_private_use()` and `suppressed_script()`, plus
    the `UND` constant and `is_undetermined()`.
  - **`lang::ScriptSubtag`** — an ISO 15924 subtag in the registry's own
    Titlecase (`Latn`, `Hans`, `Hant`), with `ZXXX` / `ZZZZ` named. `Hans`
    and `Hant` are two values and nothing folds them: this is the metadata
    layer, and its job is to carry what the file declared.
  - **`lang::Region`** — the one type with TWO grammars, because BCP 47
    gives a region two: ISO 3166-1 country codes and UN M.49 area codes
    (`419` is *Latin America and the Caribbean*, and its leading zeros are
    part of the code). `BU` folds onto `MM`; the five deprecated regions
    the registry names no successor for keep their own spelling, because a
    state that dissolved into several has none to fold onto.
  - **`lang::LanguageId`** — the whole identity, in FOUR seats: the three
    subtags plus everything past the region held VERBATIM. `zh_Hans_CN` is
    `zh-Hans-CN`, `i-klingon` is `tlh`, `GER-latn-de` is `de-DE`, and
    `en-Latn` composes as `en` while `zh-Hans` composes as itself — the
    registry's `Suppress-Script` column doing the work a hand-written fold
    would have had to guess at for all 134 languages that have one.

  **Zero language knowledge is written in the crate.** Two authority files
  are vendored under `xtask/vendor/` — the IANA language-subtag-registry
  and the ISO 639-2 registrar's own table — and `cargo xtask gen-lang`
  turns them into `lang::registry::table` (8275 languages, 224 scripts,
  303 regions, 26 grandfathered tags and four fold tables). `cargo xtask
  check` renders the same text in memory and diffs it byte for byte,
  beside the pixel-format, colour and codec checks it already ran, so a
  stale table or a hand-edited one fails the same gate. The generator
  audits its own premises before emitting — that every fold is one hop,
  that the two files agree, that every `Suppress-Script` names a
  registered script, and that a region is still the only subtag kind with
  individually registered private-use rows.

  The second file exists because the first cannot answer the question:
  BCP 47 takes a language's two-letter code where one exists and never
  registers the three-letter one beside it, so the whole ISO 639-2 alpha-3
  space for a major language is ABSENT from the IANA registry — and that
  is exactly the space a container writes.

  `lang::registry` is public, and what it is for is the question the
  types' own methods cannot answer: **why** a value came out the way it
  did. A tag that arrived as `ger-Latn-DE` and canonicalised to `de-DE`
  took two folds and a suppression, and `alpha3`, `language_preferred` and
  `language_suppress_script` are the three rows that performed them.

  **The three subtag types are `Copy`.** BCP 47 bounds each of them — a
  language at eight ASCII letters, a script at four, a region at two
  letters or three digits — so each is stored as a fixed byte buffer with
  a length and nothing else: `Language` is 9 bytes, `ScriptSubtag` 5,
  `Region` 4, against the 64 a heap-backed text seat costs. A clone is a
  register move, equality is a fixed-width comparison, and none of the
  three can allocate. `LanguageId` measures 88 bytes, 64 of which are its
  one remaining heap-backed seat.

  That seat is the tail, and it is the reason the exception exists:
  variants, extensions and the private-use sequence have no width the
  grammar bounds, so `rest` is a `smol_bytes::Utf8Bytes` and `LanguageId`
  is `Clone` rather than `Copy`. Because the other three seats are `Copy`,
  `LanguageId::language()`, `::script()` and `::region()` hand back
  **values**; only `::rest()` hands back a borrow. All four are `const fn`.

  The derived `Ord` is byte-for-byte the text's order, padding included:
  every byte a subtag can hold is `0x30` or above, so a shorter subtag's
  first unused byte sorts below anything a longer one could have there —
  which is what `str` does when one operand runs out. Asserted across the
  whole registry rather than sampled, since the pairs that could break it
  are the ones where one spelling prefixes the other.

  All four types carry `serde` (as their canonical text, read back through
  the type's own door so it CANONICALISES rather than merely validating),
  `arbitrary` and `quickcheck`; `LanguageId` additionally carries the
  `buffa` message row its predecessor had. All four also carry
  `TryFrom<&str>` and `TryFrom<Utf8Bytes>`, each a delegation to the same
  door `FromStr` walks.

  The family arrived from `ingraph::primitives::lang` under that crate's
  #428, whole and with no behaviour changed — its 68 tests came with it
  and pass unchanged. What did NOT come is the SCORING composite
  (`DetectedLanguage`, an identity with a confidence beside it): identity
  is a vocabulary question and belongs here, scoring is a retrieval
  question and stays where the retrieval framework is.

### Removed

**Breaking.** `lang::Language` used to be a wrapper over three
`icu_locale_core` subtag types, and it is gone. The name is now the
primary language SUBTAG (see Added); the whole tag is `lang::LanguageId`.

The old type was **lossy by construction**, which is why it is retired
outright rather than deprecated: it validated a full BCP 47 identifier and
then kept only language/script/region, discarding every variant, extension
and private-use subtag. `de-CH-1901` and `de-CH` were one value, and
`en-US-x-lorem` came back as `en-US`. It also had no registry behind it,
so a container's `ger` was not German — it was a language subtag the
type happily held and nothing downstream could match against `de`.

Gone with it: `Language::try_new`, `Language::from_bcp47`,
`Language::to_bcp47`, `Language::new`, `Language::default`,
`Language::language/script/region` (as `&str` accessors),
`Language::is_undetermined` (the whole-tag reading), and `LanguageError`.

| was | now |
|---|---|
| `Language::from_bcp47(s)` | `LanguageId::new(s)` |
| `s.parse::<Language>()` | `s.parse::<LanguageId>()` |
| `Language::try_new(l, s, r)` | `LanguageId::compose(Language::new(l)?, …)` |
| `Language::to_bcp47()` | `LanguageId::to_string()` (`Display`) |
| `Language::default()` | `LanguageId::default()` — still the `und` tag |
| `.language()` → `&str` | `.language()` → `&Language`, then `.as_str()` |
| `.script()` → `Option<&str>` | `.script()` → `Option<&ScriptSubtag>` |
| `.region()` → `Option<&str>` | `.region()` → `Option<&Region>` |
| `.is_undetermined()` | `.language().is_undetermined()` |
| `LanguageError` | `ParseLanguageIdError` (and the three seats' own) |

`LanguageId` is **`Clone`, not `Copy`** — its lossless tail is heap-backed
— which moves `audio::Tags` with it:

- `Tags::language()` returns `Option<&LanguageId>` where it returned
  `Option<Language>` by value. It is still `const`.
- `Tags::with_language`, `maybe_language`, `set_language`,
  `update_language` and `clear_language` are **no longer `const`**. The
  boundary is exactly *does this overwrite drop a `Utf8Bytes`*: assigning
  over the field drops the `Option<LanguageId>` that was there, whose tail
  may be heap-backed, so the drop glue is real and cannot run at compile
  time (`E0493`). The type's `u16` setters overwrite a value with no
  destructor and stay `const`; its `SmolStr` setters never were. Their
  signatures otherwise only swap `Language` for `LanguageId`.

There is **no public by-parts constructor**. `LanguageId::compose` is
`pub(crate)`, because it trusts its tail — a bare text seat carries nothing
that says it went through the envelope — and a public constructor with that
hole could mint an identity whose rendering does not parse back. The public
road from parts is to spell the seats and walk the standard door, which
validates all four in one pass.

The `buffa` wire form is unchanged in SHAPE — `Tags` field 13 and the
standalone message are still one canonical tag string, and an absent field
still decodes to `und`. What narrowed is the fallback: the door is wide
in, so the tags that used to coerce silently to `und` (a variant, a
private-use sequence, an alpha-3 language code) now decode to the value
they name, and only a structurally impossible tag still falls back.

The serde form is unchanged for every tag the old type could represent,
and is now lossless for the ones it could not.

### Changed

- **Dependencies**: `icu_locale_core` is dropped and `smol-bytes = "0.1"`
  takes its place at the `alloc` tier, which is a net reduction — the icu
  crate pulled `tinystr`, `zerovec`, `writeable`, `litemap` and
  `potential_utf` behind it. `smol-bytes` supplies `Utf8Bytes`, the text
  seat each subtag holds: short subtags ride its inline representation, so
  no subtag this crate constructs allocates.

  It is deliberately not `SmolStr`, which is what the rest of the crate's
  open vocabularies spell their `Other` escape with. A subtag is not an
  escape — it is a value with a grammar — and what it is measured against
  is the retrieval layer that stores it, where `Utf8Bytes` is the text
  seat a row is addressed by.

- **`simdutf8 = "0.1"`** joins at the `alloc` tier (zero dependencies of its
  own, `no_std` with `default-features = false`), and with it a standing law
  for byte-to-text conversion in this crate:

  1. at a **validation boundary** — bytes whose origin proves nothing —
     validation runs through `simdutf8`, not `core::str::from_utf8`;
  2. where UTF-8 validity is **proven by provenance**, the conversion is
     `from_utf8_unchecked` and the `unsafe` block carries a `SAFETY` note
     naming that provenance exactly.

  A census found **no validation boundary anywhere in the library**: every
  door in this crate takes `&str`, so the only byte-to-text conversions are
  the two in the `lang` household, and both are provenance-proven (a case
  fold's output buffer, copied from a `&str` the door had already refused
  unless it was ASCII alphanumeric). Both are now `unsafe` — **the first two
  `unsafe` blocks in this crate** — and `simdutf8` is what checks their
  claim: each sits under a `debug_assert!`, so the provenance argument is
  re-verified on every debug build, every `cargo test`, and every `miri` and
  sanitizer lane, over all 8275 registered languages the suite walks.

- **`cargo xtask` grows a fourth vendored authority.** `check` now runs a
  byte-for-byte freshness diff on the generated language table beside its
  three FFmpeg checks; `sync` re-fetches the two BCP 47 registries beside
  `pixfmt.h` and `codec_desc.c`; and `gen-lang` regenerates the table. No
  workflow change was needed — the `xtask-check` job already runs
  `cargo xtask check`.

## [0.8.0] - 2026-08-29

### Added

- **`image::Format`** — a new still-image vocabulary household, the same
  shape as `container::Format` and `audio::ContainerFormat`: standard
  photo formats (`Jpeg`, `Png`, `Heif`, `Avif`, `Tiff`, `Webp`, `Gif`,
  `Bmp`) plus a curated camera-RAW family (`Dng`, `Cr2`, `Cr3`, `Nef`,
  `Nrw`, `Arw`, `Orf`, `Rw2`, `Raf`, `Pef`, `Srw`, `Rwl`, `Iiq`,
  `Threefr`, `X3f`, `Mrw`, `Gpr`), with the usual `Other(SmolStr)`
  lossless escape. `serde` / `arbitrary` / `quickcheck` coverage matches
  every other open vocabulary in the crate.

  No still-image format existed anywhere in mediaframe before this —
  container/audio-container-only filtering meant photographs never
  entered a directory walk built on this crate's rosters. See the
  module's own doc for the full RAW inclusion/exclusion census and the
  ExifTool / ffmpeg sources it was checked against.

  **New face**, alongside the usual singular `as_extension()`: `Format`
  carries a plural `extensions() -> &'static [&'static str]` (canonical
  spelling first, then every alias) — still-image formats have more
  genuine multi-spelling extensions (`jpg`/`jpeg`/`jpe`, `tif`/`tiff`,
  `bmp`/`dib`, `orf`/`ori`) than the video/audio container rosters do,
  and `FromStr` accepts every one of them, ignore-case. (`Heic`/`Heif`
  are not such a group — see their own entries below; each is a
  single-spelling variant, not two aliases of one.)

- **`container::Format` and `audio::ContainerFormat` gain the same
  `extensions()` face**, for the same reason: an adversarial review of
  `image::Format`'s new cross-roster disjointness test (below) found it
  only reached `image::Format`'s aliases, not the two siblings' — several
  of which already had aliases sitting in doc prose that no code could
  see (`container::Format::MpegTs`'s `.m2ts`, `::Ogg`'s `.ogx`,
  `::Threegp`'s `.3g2`, `audio::ContainerFormat::Aiff`'s `.aif`). Both
  now carry `extensions()`, censused against the same ExifTool table:
  `container::Format` adds `.qt` (`Mov`), `.mts`/`.m2t` (`MpegTs`,
  alongside the already-documented `.m2ts`); `audio::ContainerFormat`
  adds `.aifc` (`Aiff`) and `.wvp` (`Wv`). Full per-variant provenance is
  in each module's doc comments.

  **`FromStr` behaviour change on these two existing types**: both
  parsers previously accepted only each variant's `as_str()` slug — for
  `MpegTs`/`Ogg`/`Threegp` that slug is not even the same spelling as
  `as_extension()` (`"mpegts"` vs `"ts"`, `"ogg"` vs `"ogv"`), so those
  variants' own primary on-disk extension, and every alias, previously
  parsed to `Other(..)` rather than the named variant. Both `FromStr`
  impls now accept every `extensions()` entry, ignore-case, matching the
  new face. This is additive in the sense every open vocabulary in this
  crate is designed to be additive (a value that used to ride the escape
  now has a name — the crate's stated forward-compatible evolution path,
  see `lib.rs`), but it is a real, observable parsing-result change on
  two already-published types; flagged here explicitly rather than
  folded silently into the new-module bullet above.

  **R2 addendum**: a second adversarial pass caught one more missing
  alias in the same census — `audio::ContainerFormat::Ogg` gains `.oga`
  and `.spx` (`extensions()` is now `["ogg", "oga", "spx"]`), both
  registered alongside the already-canonical `.ogg` by RFC 5334 §10.3
  under `audio/ogg`. This is the first alias in this crate sourced from
  an IETF RFC rather than ExifTool's file-type table — a provenance
  widening applied crate-wide during the same review: IANA's
  `audio/mpeg` registration also lists `.mp1`/`.mp2` beside `.mp3`, but
  those are genuinely different MPEG audio layers (I/II vs III), not
  spelling variants, so `ContainerFormat::Mp3` is unchanged. No other
  variant in either roster turned up a further citable RFC/IANA alias
  beyond what R1 already found via ExifTool.

  **R3 addendum**: a third pass found the same class recurring
  (`audio::ContainerFormat::Aac` was missing IANA/ffmpeg-registered
  `.adts`) and escalated to an exhaustive per-variant sweep — all 53
  variants across all three rosters, checked against IANA's media-types
  registry, ffmpeg's own muxer/demuxer extension tables (queried
  directly, not from memory), and ExifTool's file-type table. Landed:
  `ContainerFormat::Aac` gains `.adts`; `container::Format::Mp4` gains
  `.mpg4` (IANA: `"mp4 and mpg4 are both declared"`);
  `container::Format::Threegp` gains `.3gpp`/`.3gp2`, reversing an R2
  exclusion that turned out to be a subjective "unrealistic spelling"
  call rather than a source-grounded one (IANA's `video/3gpp`
  registration independently confirms `.3gpp`); `ContainerFormat::Ape`
  gains `.apl`/`.mac` (ffmpeg's dedicated `ape` demuxer — the one alias
  pair in this crate sourced purely from ffmpeg, not IANA or ExifTool).
  Every other variant's sweep result — alias found and landed, or
  checked-and-none-found, with the source(s) consulted — is recorded
  per-variant in the module docs and the originating PR's discussion,
  not asserted in bulk.

  **R4 correction**: a fourth pass caught that `.apl` (landed in R3,
  above) does not actually pass this crate's own alias test. An APE
  Link file is Monkey's Audio's own per-track *sidecar* — split points
  derived from a CUE sheet against a companion `.ape` image — not the
  compressed bitstream; ffprobe rejects APL content even forced through
  ffmpeg's `ape` demuxer, which only *lists* `.apl` alongside `.ape` as
  a matter of demuxer convenience, not identical bytes.
  `ContainerFormat::Ape.extensions()` is now `["ape", "mac"]` — `.mac`
  is untouched, it names the same bitstream and was independently
  unaffected by this finding. A regression test pins `"apl"` (any case)
  to `Other("apl")` so this cannot silently re-land.

  **Verification pass**: `.mac` carried the identical evidence profile
  `.apl` had before it failed — a single ffmpeg extension-table listing,
  unverified at the byte level — so it was checked the same way before
  R5 dispatch, rather than trusted by the coincidence of surviving R4.
  Verdict: **keep**, now on stronger grounds than ffmpeg's table alone.
  Monkey's Audio's own official version history (v3.00: "now uses the
  extension .APE instead of .MAC") documents `.mac` → `.ape` as a pure
  developer rename, not a format change, with `.mac`-file support
  explicitly kept working afterward (v3.40: "Made the Winamp plugin also
  support the old .MAC extension"). Empirically re-verified too: a
  synthetic 332-byte file built from the real `APE_DESCRIPTOR` +
  `APE_HEADER` layout produced byte-identical `ffprobe` behaviour under
  three different extensions (`.mac`, `.ape`, and one ffmpeg has no APE
  association for at all), auto-detected and forced alike — the
  signature decided it every time, extension irrelevant, exactly the
  test `.apl` failed. No code change; the variant doc now carries this
  evidence directly rather than a bare source citation.

- **`container::Format::M2ts`** — MPEG-2 Transport Stream in Blu-ray
  Disc / AVCHD's BDAV framing: 192-byte packets (a 4-byte
  `TP_extra_header` prepended to every 188-byte MPEG-TS packet), a
  genuinely different on-disk byte layout from plain `.ts`
  (`MpegTs`). `extensions()` was `["m2ts", "mts", "m2t"]` at the time
  this bullet was written — **R7 moved `.m2t` back to `MpegTs`** (see
  the R7 bullet below); current value is `["m2ts", "mts"]`. **R5
  correction** (Codex R5 HIGH finding, user-ruled 甲): `.m2ts`/`.mts`/
  `.m2t` briefly (R1, widened R3) lived in `MpegTs.extensions()` on
  ExifTool's alias table alone — ExifTool's own table actually names a
  *fourth*, separate file type (`M2TS`) that those three alias to, not
  `MpegTs`/plain-`TS`; the R1 reading of that table was the error, not
  the table itself. `MpegTs.extensions()` was `["ts"]` immediately
  after this correction — **R7 added `.m2t` back**, so the current
  value is `["ts", "m2t"]`; re-swept honestly at R5, no alias survived
  the identical-bytes test once the misattributed three were removed,
  and `.m2t` only returned once R7's content-detector evidence showed
  it belonged to `MpegTs`'s 188-byte world after all.

- **`container::Format::Threeg2`** — 3GPP2 (the CDMA2000-lineage
  sibling standard to 3GPP), a distinct ISOBMFF `major_brand` (`3g2a`)
  from `Threegp`'s 3GPP-family brands, and a separately-named,
  dedicated ffmpeg muxer (`3g2`, vs `Threegp`'s own dedicated `3gp`
  muxer — not a shared implementation the way `Mov`/`Mp4` share one).
  `extensions()` is `["3g2", "3gp2"]`. **R5 correction**: `.3g2`/
  `.3gp2` briefly (R1, widened R3) lived in `Threegp.extensions()`;
  `Threegp.extensions()` is now `["3gp", "3gpp"]` only.

- **`audio::ContainerFormat::Aifc`** — AIFF-Compressed: a different
  IFF `formType` at the fixed header offset (`AIFC` vs plain `AIFF`'s
  `AIFF`), a mandatory `FVER` chunk plain AIFF never carries, and a
  `COMM` chunk with two extra required fields (`compressionType`,
  `compressionName`) plain AIFF's `COMM` has no room for — a different
  required byte layout, not a filename convention, per Apple's own
  AIFF-C specification. `extensions()` is `["aifc"]`. **R5
  correction**: `.aifc` briefly (R2) lived in `Aiff.extensions()`
  despite this crate's own R2 census already noting ExifTool keeps
  `AIFC` as its own file-type entry (not a pure alias like `AIF`) —
  the census had the right fact and drew the wrong conclusion from it.
  `Aiff.extensions()` is now `["aiff", "aif"]` only — the one true
  byte-identical alias remains.

  **The pattern underneath all three**: each was originally justified
  by a real, correctly-sourced citation (ExifTool's alias table,
  ffmpeg's shared demuxer, IANA's own registration text) that named a
  *related* format, not a *spelling* of one — the same "shared
  tooling/registration is a hint to check, never itself the
  identical-bytes proof" lesson `.apl` (`Ape`, R4) already taught one
  round earlier, recurring because the sweep that produced these three
  predates that lesson. Promoted to their own variants rather than
  patched as exclusions, per the user's ruling, because they *are* real
  formats — just not spellings of the ones they were attached to.

- **`image::Format::Heic`** — High Efficiency Image Format, HEVC-coded:
  requires one of the `heic`/`heix`/`heim`/`heis` ISOBMFF major/
  compatible brands per IANA's own `image/heic` registration, distinct
  from `Heif`'s generic `mif1`-brand (any coding) requirement per
  `image/heif` — the same structurally-signaled-subtype shape already
  used to keep `Avif` separate. `extensions()` is `["heic"]`.
  **R6 correction, same class as the R5 promotions** (per the user's R5
  甲 ruling, applied directly): R3 had collapsed `HEIC`/`HEIF`/`HIF`
  onto one `Heif` variant on ExifTool's "nearly identical file types"
  reasoning, without applying the module's own identical-bytes test.
  `Heif.extensions()` is now `["heif"]` only.

  **`.hif` is excluded from both `Heic` and `Heif`** (R8 correction to
  R6's own work, directly above — same round-later self-correction
  pattern as `.m2t` below). R6 routed `.hif` to `Heic` on the strength
  of Canon — the dominant real-world `.hif` producer — writing
  `major_brand = 'heix'` (one of the four HEIC-qualifying brands) with
  HEVC-coded tile data, per independent reverse-engineering
  documentation of the real byte layout. **That evidence is real, but
  it proves frequency, not totality**: IANA's own extension field says
  `.hif` names *either* subtype (`"hif (for subtypes heif and
  heic)"`) — Canon's dominance means most `.hif` files in the wild are
  probably `heix`-branded, not that `.hif` *is* `heix`-branded by
  extension alone. A `FromStr` total mapping from `.hif` to one variant
  would claim the latter, which the spec itself contradicts — the same
  identical-bytes reasoning that already keeps `Avif`'s IANA-listed
  `heif`/`hif` spellings off `Avif`. `.hif` now parses to `Other`,
  carrying its own name, any case. The Canon evidence stays recorded in
  `Heic`'s own doc as exactly what a future content-aware door (real
  `ftyp`-box brand inspection, rather than extension text) would need —
  mediaframe has no such tier today, so that door is named, not built.

- **`container::Format::MpegTs` / `::M2ts`: `.m2t` moved back to
  `MpegTs`** (R7 correction to R5's own work — same identical-bytes
  discipline applied a round later). `MpegTs.extensions()` is now
  `["ts", "m2t"]`; `M2ts.extensions()` is now `["m2ts", "mts"]`. R5 put
  `.m2t` on `M2ts` alongside `.m2ts`/`.mts` on ExifTool's *static* alias
  table alone (`M2T` aliases to the `M2TS` file-type name). ExifTool's
  actual **content detector** (`M2TS.pm`'s `ProcessM2TS`, read directly
  from the real Perl source) measures packet stride and picks the
  FileType from *that*: `$et->SetFileType($tcLen ? 'M2TS' : 'M2T')` —
  unprefixed 188-byte packets are labelled `M2T`, not `M2TS`, which is
  reserved for the 4-byte-prefixed 192-byte BDAV form. `.m2t` names the
  same 188-byte world `.ts` does; it was never `M2ts`'s to begin with.

## [0.7.0] - 2026-08-28

**Breaking:** the public dependency `mediatime` crosses 0.3 → 0.4.

### Changed

- **`mediatime` 0.3 → 0.4.** Upstream is additive only — one commit, adding
  `Duration` (the unsigned counterpart to `SignedDuration`) plus its
  `core::time::Duration` and `SignedDuration` conversions; `mediatime`'s own
  changelog says plainly that `Timebase`'s public surface is unchanged, and
  neither `SignedDuration` nor the new `Duration` exists in this crate's
  dependency graph at all. `Timestamp` is the *only* `mediatime` type this
  crate's public API carries — `frame::TimestampedFrame`'s `pts`/`duration`
  fields and its `pts`/`duration`/`with_pts`/`maybe_pts`/`set_pts`/
  `update_pts`/`with_duration`/`maybe_duration`/`set_duration`/
  `update_duration` accessors and builders — and `Timestamp` itself did not
  change. Still marked **Breaking**, matching how this crate treated
  `mediatime` 0.1 → 0.2 and 0.2 → 0.3: a public dependency crossing an
  incompatible Cargo SemVer class (0.x's minor digit is the compatibility
  boundary) breaks a caller holding a `mediatime 0.3` value against this
  crate's `0.4`-typed signatures, regardless of how much of `mediatime`'s own
  surface actually moved — the 0.1 → 0.2 release proved the same point in the
  other direction, staying **Breaking** despite touching only one test's
  literal. Verified empirically here, not just argued: `cargo check` (default
  features, `--all-features`, and both no-alloc/`alloc` no_std tier floors),
  `cargo test --all-features`, and `cargo clippy --all-features -- -D
  warnings` all pass with no source change required — zero fallout in this
  crate's own source.

## [0.6.0] - 2026-08-21

**Breaking**, on two counts: `audio::ChannelLayout`'s twelve numeric
variants are renamed, and `audio::BitRateMode`'s `serde` wire changes
shape in human-readable formats.

- **`audio::BitRateMode` carries its name where a human will read it.**
  Its `serde` representation was the `u32` code in every format; it is
  now the canonical slug wherever `Serializer::is_human_readable()` — so
  a JSON value written as `1` is now written as `"vbr"`. **Stored JSON
  from 0.5.0 does not read back**: the slug leg refuses a bare integer
  outright, so a persisted `0` is a deserialization error rather than a
  silent `Cbr`. Binary formats are **unaffected** — postcard, bincode and
  anything else declaring itself non-human-readable still carry the
  `to_u32()` code, byte for byte as before. `buffa` is a separate codec
  and is untouched.

  This is a law, not a special case, and it applies to every
  strictly-closed coded enum the crate has:

  | leg | shape | read side |
  |---|---|---|
  | `is_human_readable()` | the `as_str()` slug | `FromStr` — an unrecognised **name** is an error, and a *number* is refused: a number is not a name |
  | binary | the `to_u32()` code | `try_from_u32` — an out-of-range **code** is an error |

  Both legs stay strict in the same sense they were: an input this
  vocabulary cannot name is refused, never collapsed onto the default
  variant the way `from_u32` would collapse it. The slug leg folds ASCII
  case (`"CBR"` is `"cbr"`), because folding a *spelling* is not
  inventing a *value*.

  **Open vocabularies are unchanged** and stay on their single slug wire
  under every format. The asymmetry is not an oversight: an open
  vocabulary's `Other(SmolStr)` holds a name with no code behind it, so a
  numeric leg would have nothing to write for the one value the escape
  exists to carry. A closed vocabulary has no such value — every member
  has both spellings — so the format gets to choose, and a binary format
  has no reason to pay for a string it cannot read anyway.
- **`audio::ChannelLayout`'s numeric variants take a `Ch` prefix and drop
  `Point`.** `N5Point1Back` becomes `Ch5_1Back`, `N2Point1` becomes
  `Ch2_1` — all twelve of them. Slugs, wire form, serde, `buffa` and
  every stored value are **unchanged**; this renames Rust identifiers
  only.

  | was | is | was | is |
  |---|---|---|---|
  | `N2Point1` | `Ch2_1` | `N5Point1` | `Ch5_1` |
  | `N3Point0` | `Ch3_0` | `N5Point1Back` | `Ch5_1Back` |
  | `N3Point0Back` | `Ch3_0Back` | `N6Point0` | `Ch6_0` |
  | `N3Point1` | `Ch3_1` | `N6Point1` | `Ch6_1` |
  | `N5Point0` | `Ch5_0` | `N7Point0` | `Ch7_0` |
  | `N5Point0Back` | `Ch5_0Back` | `N7Point1` | `Ch7_1` |

  That is the whole rename. The other `Ch`-prefixed idents in this
  release — `Ch3_1_2`, `Ch4_0`, `Ch4_1`, `Ch5_1_2`, `Ch5_1_2Back`,
  `Ch5_1_4Back`, `Ch6_0Front`, `Ch6_1Back`, `Ch6_1Front`, `Ch7_0Front`,
  `Ch7_1Wide`, `Ch7_1WideBack`, `Ch7_1_2`, `Ch7_1_4Back`, `Ch7_2_3`,
  `Ch9_1_4Back`, `Ch9_1_6`, `Ch22_2` — are **new variants**, listed
  under Added. They never had an `N` spelling to be renamed from, so
  there is nothing in existing code to search for.

  The `N` was a lexical dodge — `5Point1Back` is not an identifier, so a
  letter had to go in front, and `N` said nothing. `Ch` says *channels*,
  and `Ch5_1Back` reads as its layout where `N5Point1Back` reads as a
  spelling exercise. It is also the prefix `mediadecode` already uses, so
  the two vocabularies converge on names as well as on slugs.

  **The letter-named variants keep their names**: `Mono`, `Stereo`,
  `Quad`, `Hexagonal`, `Octagonal`, `Ambisonic1`/`2`/`3` and `Other` are
  untouched, and the letter-named layouts added this release
  (`StereoDownmix`, `Binaural`, `QuadSide`, `Hexadecagonal`, `Cube`)
  arrive without a prefix for the same reason. The prefix exists to make
  a leading digit legal and these have no leading digit; `ChMono` would
  stutter against the type name, and renaming `Other` would move the
  escape arm that `Unwrap`, `TryUnwrap`, `IsVariant`, the roster macro
  and every `is_other()` call site depend on.

  The `IsVariant` predicates move with the idents: `is_n_5_point_1()` is
  now `is_ch_5_1()`.

### Added

- **`audio::ChannelLayout` names twenty more layouts**, growing the roster
  from 20 to 40 so there is one channel-layout vocabulary for this
  ecosystem instead of two that disagree. `mediadecode`'s
  `ChannelLayoutKind` named a different set of 38 and read `"5.0"` /
  `"5.1"` as the *side* layouts, where this crate reads them as FFmpeg
  does — the back ones. The union settles that in favour of FFmpeg.

  The new layouts, spelled as FFmpeg's `channel_layout_map[]` spells
  them:

  | slug | FFmpeg constant | slug | FFmpeg constant |
  |---|---|---|---|
  | `downmix` | `STEREO_DOWNMIX` | `7.0(front)` | `7POINT0_FRONT` |
  | `quad(side)` | `2_2` | `7.1(wide)` | `7POINT1_WIDE_BACK` |
  | `3.1.2` | `3POINT1POINT2` | `7.1(wide-side)` | `7POINT1_WIDE` |
  | `4.0` | `4POINT0` | `7.1.2` | `7POINT1POINT2` |
  | `4.1` | `4POINT1` | `7.1.4` | `7POINT1POINT4_BACK` |
  | `5.1.2(back)` | `5POINT1POINT2_BACK` | `7.2.3` | `7POINT2POINT3` |
  | `5.1.4` | `5POINT1POINT4_BACK` | `9.1.4` | `9POINT1POINT4_BACK` |
  | `6.0(front)` | `6POINT0_FRONT` | `22.2` | `22POINT2` |
  | `6.1(back)` | `6POINT1_BACK` | `hexadecagonal` | `HEXADECAGONAL` |
  | `6.1(front)` | `6POINT1_FRONT` | `cube` | `CUBE` |

  Slugs are FFmpeg's, not `mediadecode`'s. Taking `mediadecode`'s would
  have imported its ambiguity along with its coverage: it spells the
  side 5.1 `"5.1"` and the back one `"5.1-back"`, inverting FFmpeg, and
  it crosses the 7.1-wide pair the same way. Every slug here is what
  `av_channel_layout_describe` prints for that constant in libavutil
  9.0.1, and the full 37-row map is transcribed into the tests.

  Three things make these arms look wrong and are not:

  * The unqualified name belongs to the **back** layout for 5.0, 5.1 and
    7.1(wide) — but to the **side** layout for 5.1.2, where the back one
    is qualified `"5.1.2(back)"`.
  * `_BACK` in `5POINT1POINT4_BACK`, `7POINT1POINT4_BACK` and
    `9POINT1POINT4_BACK` marks a top-back *height* pair, not surround
    placement, and never reaches the slug: they are `"5.1.4"`, `"7.1.4"`
    and `"9.1.4"`.
  * `2_1` and `2_2` are named `"3.0(back)"` and `"quad(side)"`, and
    their idents follow the slug rather than the constant — `2_1` as an
    ident is the real `2POINT1`'s.

  `AV_CH_LAYOUT_7POINT1_TOP_BACK` gets no variant: it is a deprecated
  alias of `5POINT1POINT2_BACK`, one mask under two names.

  **Growing the roster shrinks the escape.** A value held as
  `Other("22.2")` now reads back as the named variant. The wire form is
  the slug, so nothing serialized changes and nothing on disk needs
  migrating — but the two values are not `Eq`, so code matching
  `Other(s) if s == "22.2"` (or any of the other nineteen new slugs)
  stops matching.

- **`audio::ChannelLayout` names the last three**, closing the roster at
  40 named layouts plus `Ambisonic1`/`2`/`3` — **43** entries in
  `ROSTER`.

  | slug | FFmpeg constant | layout |
  |---|---|---|
  | `5.1.2` | `5POINT1POINT2` | FL+FR+FC+LFE+SL+SR+TFL+TFR |
  | `9.1.6` | `9POINT1POINT6` | 9.1.4 plus the top *side* pair (16ch) |
  | `binaural` | `BINAURAL` | BIL+BIR |

  These three were in neither this crate's roster nor `mediadecode`'s, so
  the union alone left them out — and a vocabulary carrying
  `"5.1.2(back)"` but not `"5.1.2"`, or `"9.1.4"` but not `"9.1.6"`,
  reads as an accident rather than as a decision.

  **`Binaural` is not a stereo pair.** Binaural audio is rendered for
  headphones with the head-related transfer function already applied, so
  each channel is what one *ear* receives rather than what one *speaker*
  emits; playing it over loudspeakers, or folding it down to `Mono`,
  destroys the spatial cue it exists to carry. FFmpeg gives it channel
  ids of its own (`BIL`/`BIR`) instead of reusing `FL`/`FR`, and this
  vocabulary keeps that distinction rather than treating it as another
  two-channel layout.

  With these, **every entry in FFmpeg n9.0's `channel_layout_map[]` is
  named** — all forty, pinned by `the_map_is_transcribed_in_full`, which
  turns "the whole map is transcribed" from a comment into a checked
  claim and makes a future FFmpeg's new layout arrive as a red test
  rather than as silence. `Other` now carries only what the map cannot
  name at all: custom channel orderings, ambisonic groupings beyond
  third order, and whatever a later release adds.

  The escape-shrinking note above applies to these three as well —
  `Other("binaural")`, `Other("9.1.6")` and `Other("5.1.2")` now read
  back as named variants.

- **The channel household moves in from `mediadecode`** — three new
  types, one module each, describing *how* a layout is arranged where
  `audio::ChannelLayout` names *which* layout it is.

  | new | was, in `mediadecode` |
  |---|---|
  | `audio::channel_order::ChannelOrder` | `AudioChannelOrderKind` |
  | `audio::channel_spec::ChannelSpec` | `AudioChannelSpec` |
  | `audio::channel_layout_description::ChannelLayoutDescription` | `AudioChannelLayout` |
  | `audio::channel_order::ParseChannelOrderError` | `ParseAudioChannelOrderKindError` |

  `mediadecode` carried a four-layer channel model. Its first layer,
  `ChannelLayoutKind`, named a different 38 layouts and read `"5.0"` /
  `"5.1"` as the *side* ones — the duplicate this release's roster
  settled, and it is not migrated. The other three layers have no
  duplicate here and belong where the vocabulary they describe already
  lives. `mediadecode` 0.6 will depend on these types rather than define
  them; its FFmpeg interop functions stay there.

  `Kind` was a redundancy that only read as a distinction while a
  `ChannelLayoutKind` stood beside it. `ChannelLayoutDescription` is
  long deliberately: the enum is the layout's *name* and this record is
  its full *description*, which is `av_channel_layout_describe`'s own
  distinction.

  Two seats changed meaning rather than only spelling:

  * `description` is now **`text`** — `description()` on a type called
    `…Description` says nothing, while `text()` names what the field
    holds: the backend's own rendering, verbatim, folded and parsed by
    nothing. `known_kind` is the parsed name; the two are separate seats
    and a description may carry either, both or neither.
  * `known_kind` held the dead `ChannelLayoutKind` and now holds this
    crate's `ChannelLayout`. Its "no well-known shape matches" state is
    `ChannelLayout::default()` — the `Other("")` absent sentinel —
    rather than a named `Unknown` variant, which is the same statement
    in the vocabulary that survived.

  Semantics, slugs and the `u32` wire integers are unchanged:
  `"unspecified"` / `"native"` / `"custom"` / `"ambisonic"` at codes
  `0`–`3`, with `from_u32` absorbing an unrecognised code into
  `Unspecified`. Three adaptations to this crate's house style, none of
  them semantic: `as_u32` is spelled **`to_u32`** as every other coded
  vocabulary here spells it and `try_from_u32` joins it (the strict door
  the wire uses); `FromStr` folds through the crate's shared ASCII gate;
  and `ROSTER` comes from the roster macro, so completeness is a compile
  error rather than a counted test.

  `ChannelOrder` joins `audio::BitRateMode` in the **strictly-closed**
  serde group, so it takes both legs described under Breaking above:
  `"native"` in JSON, the code `1` in a binary format. `mediadecode`
  sent the slug in every format, and the human-readable leg keeps that
  spelling exactly — what changes is that a binary peer no longer pays
  for a string it cannot read.

  What puts it in that group at all is that its code space really is
  closed: it mirrors FFmpeg's `AVChannelOrder`, four members with no
  vendor range, so an integer outside `0..=3` is a corrupt read rather
  than a name this build has not heard of, and it is refused instead of
  decoded as `Unspecified`.

  The `buffa` codec gains its first **repeated** field
  (`custom_channels`), one length-delimited `ChannelSpec` per element.
  `native_mask` uses presence encoding, so a layout reporting an
  all-zero mask stays distinct from one reporting no mask at all.

  These three live at the `alloc` tier, with the rest of `audio`. In
  `mediadecode` the enum reached the no-alloc tier and only the two
  records needed an allocator; nothing in `ChannelOrder` needs a heap,
  so that split is recoverable, but a household cannot be less gated
  than the module holding it.
## [0.5.0] - 2026-08-20

A vocabulary window: two closed enums stop pretending they might grow, one
closed enum opens, every open vocabulary publishes its roster, and the nine
that are compiled at every tier stop declaring an error their `alloc` build
cannot return.

**Breaking**, on three counts.

1. **`FromStr::Err` is now `Infallible` at the `alloc` / `std` tier** for
   the ten vocabularies compiled at every tier: `color::Matrix`,
   `color::Primaries`, `color::Transfer`, `color::DynamicRange`,
   `color::ChromaLocation`, `color::DcpTargetGamut`,
   `pixel_format::PixelFormat`, `frame::Rotation`, `frame::FieldOrder` and
   `frame::StereoMode`. Behaviour does not change at any tier — these
   parses have always been total wherever `Other` exists, and their docs
   have said so for releases. What changes is that the type now says it
   too. The `Parse*Error` types are untouched and still exported; the
   no-alloc tier still returns them.

   Downstream breaks are the places that *name* the old error: a `match`
   arm on it, a `From` impl or `?` conversion into a local error enum, and
   annotated or turbofished bindings that spell it. Code that merely
   propagated the error usually just deletes the arm. Where a value is
   wanted, the impossible error is discharged by an irrefutable binding —
   `let Ok(m) = s.parse::<Matrix>();` — which is stable today and is what
   this crate uses internally (`Result::into_ok` is the same thing once it
   stabilises).

2. **`subtitle::TrackOrigin` gained an `Other(SmolStr)` escape**, which
   moves it onto the crate-wide open-vocabulary shape and changes four
   surfaces: it is no longer `Copy`, `as_str` is no longer
   `const fn -> &'static str`, `to_u32` returns `Option<u32>`, and
   `FromStr::Err` is `Infallible` — **unconditionally**, with no `cfg`
   fork, because the `subtitle` module is compiled only at the `alloc`
   tier and so has no build in which the vocabulary closes.
   `ParseTrackOriginError` is kept and exported on the same policy as the
   ten above, and its doc states the one way it differs: their lean build
   still returns theirs, nothing returns this one today.

   **Both of its wire forms change**, so persisted 0.4.x values do not
   read back: `serde` moves from an integer code to the canonical slug
   string (so `0` no longer deserializes), and `buffa` field 1 moves from
   `Varint` to `LengthDelimited` (so a 0.4.x payload fails to decode
   rather than decoding wrongly). This is the same wire every other name
   vocabulary in the crate already uses.

3. **`subtitle::Format::PgsSub` is removed**, merged into
   `subtitle::Format::HdmvPgs`. **Stored data is unaffected** — the slug
   on disk is unchanged and parses to the surviving variant. Rust code
   naming `PgsSub` renames to `HdmvPgs`, which is the same value it
   already meant.

### Added

- **`ROSTER` on all eighteen open vocabularies.** `pub const ROSTER:
  &'static [Self]` lists the named variants in declaration order:
  `VideoCodec`, `AudioCodec`, `SubtitleCodec`, `audio::SampleFormat`,
  `audio::ContainerFormat`, `audio::ChannelLayout`, `container::Format`,
  `subtitle::Format`, the five `color` enums, `pixel_format::PixelFormat`,
  the three `frame` orientation enums, and `subtitle::TrackOrigin`.

  `#[non_exhaustive]` denies a downstream the `match` it would need to
  enumerate these itself, so every consumer that mirrors one of these
  vocabularies was going to hand-copy a list that silently rots at the
  next release. This publishes the list instead. It is a **slice**, not an
  array, so the count stays out of the type and a later addition remains a
  minor change, and it excludes the `Other` escape — the roster answers
  which names a build knows, and the escape is the arm carrying one it
  does not.

  Completeness is proved here rather than promised: an exhaustive `match`
  sits beside each type (`#[non_exhaustive]` does not bind the defining
  crate), so adding a variant without rostering it is `E0004` with the
  compiler naming it. Roster and witness are generated from one list per
  type, so there is no second list to drift.
- `subtitle::TrackOrigin::Derived` (slug `"derived"`, wire id `3`) — a
  track produced by a pass over the media (ASR transcript, machine
  translation, OCR) rather than obtained as subtitle text. `External` had
  been carrying that case in its doc while also meaning "downloaded";
  the two are now separate. Ids stay append-only.

### Changed

- **`frame::BayerPattern` is closed** — `#[non_exhaustive]` removed. The
  four standard arrangements are a geometric closure, not a snapshot of
  today's cameras: a 2×2 tile with one red, one blue and two greens admits
  exactly four top-left phases. Every CFA family that would want a fifth
  (Quad Bayer, X-Trans, RGBW, Foveon, monochrome) is a different tile shape
  and already leaves via a different type, as the type's own scope note has
  always said. Downstream matches keep compiling and gain a completeness
  proof.
- **`audio::BitRateMode` is closed** — `#[non_exhaustive]` removed. CBR /
  VBR / ABR is the whole of the *reporting* domain and has been stable for
  twenty-five years. The near misses are not members: CVBR is a shape of
  VBR, and CRF / CQP are encoder knobs describing how a file was produced,
  not a property the stream reports.
- **`subtitle::TrackOrigin` is an open vocabulary.** See breaking note 2.
  mediaframe is a shared library, not one pipeline's private enum: the set
  of provenances worth distinguishing belongs to whoever does the
  classifying, so a downstream tracking an origin this crate has not heard
  of now keeps its *name* rather than losing it to a nearby variant.
  `#[non_exhaustive]` is retained, so promoting a slug that rides `Other`
  today into a named variant tomorrow stays minor.
- **The ten all-tier vocabularies tell the truth about their parse.** See
  breaking note 1. The `cfg` predicate on the split is the same one that
  gates each type's `Other` arm, so the error type and the escape cannot
  drift apart, and each type carries an irrefutable-`let` proof that stops
  compiling if the error is narrowed back.

  With the set closed, the `buffa` string-enum codec stops guessing: its
  shared decoder replaced `from_str(..).unwrap_or_else(|_| unreachable!())`
  — unreachable only by argument — with an irrefutable binding, so all
  fourteen vocabularies it serves are now *proved* to parse totally at that
  tier, and adding one that cannot is a compile error.
- **`subtitle::Format::PgsSub` merged into `HdmvPgs`.** See breaking note
  3. One format wore two variant names rendering the same slug, so
  `FromStr` could return only one of them and `Display` was not invertible
  for the other. The survivor is the one whose name matches the canonical
  slug — the name the type's doc already crowned as FFmpeg-canonical.

## [0.4.0] - 2026-08-19

The FFmpeg pin moves `n8.1` → `n9.0` and every provenance label in the crate
is re-verified against it rather than re-typed.

**Breaking**: FFmpeg 9 drops the three `FF_API_V408_CODECID` codecs, so
`VideoCodec::{V308, V408, V410}` go with them — the strings still round-trip
through `VideoCodec::Other`.

### Added

- **FFmpeg synonyms on the parse side.** Where mediaframe's canonical slug
  and FFmpeg's own name for the same thing differ, `FromStr` now accepts
  both. Emission is unchanged and still injective: `as_str` / `Display` /
  serde render one canonical slug per variant and never a synonym, so
  `parse(display(x)) == x` and the `display ∘ parse` idempotence both still
  hold. The ten pairs, all read off FFmpeg's own name tables:
  `PixelFormat` — `gray` → `gray8`, `monob` → `monoblack`, `monow` →
  `monowhite` (the descriptor names `ffprobe` prints, against the
  `AV_PIX_FMT_<NAME>` identifiers this vocabulary is spelled after);
  `color::Matrix` — `gbr` → `rgb`, `unknown` → `unspecified`;
  `color::Primaries` and `color::DynamicRange` — `unknown` → `unspecified`;
  `color::Transfer` — `unknown` → `unspecified`, `bt470m` → `gamma22`,
  `bt470bg` → `gamma28`. A name copied off `ffprobe` now lands on the named
  variant and keeps its H.273 code instead of riding the `to_u32`-less
  escape. Nail tests prove no synonym shadows a canonical slug.
- `VideoCodec::WebpAnim` (`webp_anim`) and `AudioCodec::AppleApac`
  (`apple_apac`), both new in FFmpeg 9.

### Changed

- **`*Row::new` is `pub(crate)`; a hidden test door replaces the public
  promise.** Breaking. With the walkers no longer taking a selector, a public
  row constructor was the last way to build a row beside the description that
  chose its matrix — so it is gone from the public surface, and the one-source
  rule now holds at row grain for everything outside this crate. Covers all
  132 walker-generated row types plus `Pal8Row`, `BayerRow` and `BayerRow16`
  — 135 in all; the ten hand-written source rows that were already
  `pub(crate)` are unchanged, because there was no public promise there to
  replace. Nine of those ten also get no door for the same reason.
  `Xyz12Row` is the tenth and does get one, for a different reason: `xyz`
  was the only one of the fifteen format features with no door anywhere
  behind it, so the format had no way in for a kernel-parity suite at all.
  That is a hole in the coverage rather than a promise to replace, and it
  is now closed — every format feature owns at least one door, which is
  what the `row_test_door_doc` gate now says.

  The named exception is `#[doc(hidden)] *Row::for_tests`, emitted beside
  `new` with the identical parameter list — selector included — and
  forwarding to it, so the two cannot drift. It exists for one reason: a
  kernel-parity suite drives a single row kernel without materialising a
  frame, and there is no other way to reach one from outside. That is
  measured, not assumed — a census on 2026-08-19 found **493 such
  constructions across 85 files and 52 row types** in `pixon` alone, all of
  them test code, and every one of those 52 types has a door. It carries no
  stability promise and its doc says so.

  Nothing in this crate needed migrating: every in-tree row already came from
  a walker, which is why the door is exercised by a test of its own rather
  than by existing callers. **pixon's suites are the breakage**, and their
  migration (`Row::new` → `Row::for_tests`) rides pixon's own 0.4 bump.
- **The walkers stop taking a colour selector; the sink supplies it.**
  Breaking, across every `{fmt}_to` / `{fmt}_to_endian` entry point. The
  `matrix: KernelMatrix` parameter is **gone** — no deprecation — and
  `xyz12_to`'s `target_gamut: KernelGamut` with it. The value now travels on
  the sink contract:
  - `PixelSink::kernel_matrix(&self) -> KernelMatrix`, defaulting to
    `Unspecified` (the kernels' documented BT.709 posture).
  - `Xyz12Sink::target_gamut(&self) -> KernelGamut`, defaulting to `DciP3`
    (what `DcpTargetGamut` already documents for a caller who does not
    re-target). It sits on the XYZ12 subtrait rather than `PixelSink`
    because a gamut is an **output** axis with exactly one consumer, and a
    knob every other sink carries but no walker reads would be the same
    second door in a new place.

  Each walker asks once, after `begin_frame` and before the row loop, and
  stamps the answer on every row of that frame. Why: the sink already held a
  colour description, so passing a selector beside it meant two doors onto
  one fact — name one matrix, build the sink from another, and the picture
  is quietly wrong with nothing to fail on. Migration is deleting the
  argument; a sink that wants a specific matrix or gamut overrides the
  method. `full_range` stays a parameter — it is a quantisation fact about
  the frame, not a colour intent the sink owns.

  The kernels are untouched: rows still carry `matrix()` / `target_gamut()`,
  and the closed `KernelMatrix` / `KernelGamut` selectors are unchanged. Only
  *where the value enters* moved.
- **FFmpeg pin `n8.1` → `n9.0`** (`xtask/src/main.rs`), vendored tables
  regenerated by `cargo xtask sync` and `mediaframe::codec` by
  `cargo xtask gen-codec`. `AVPixelFormat` did not move between the two
  releases — `libavutil/pixfmt.h` is byte-identical — so the 254 vendored
  slugs, the 56 colour code points and `PixelFormat` itself need **no new
  variants**; the same holds for `AV_DISPOSITION_*` (19 flags),
  `AVSampleFormat` (12) and `AV_CODEC_PROP_*` (8). Only `codec_desc.c` moved.
- Provenance labels across the crate now read `n9.0`, each re-verified
  against the n9.0 headers rather than relabelled on faith. The generated
  codec module interpolates the pin and its own variant counts instead of
  carrying them as hand-typed strings — both had gone stale (`(281)` /
  `(221)` for enums that are now 279 / 222). Historical `CHANGELOG` entries
  and the "new in n8.1" note on the 96/128-bit packed RGB formats keep their
  tags: those are statements about the past, and still true.
- `"unknown"` still names nothing any colour enum renders, but on the four
  enums whose FFmpeg table spells `UNSPECIFIED` that way it now parses to
  `Unspecified` instead of riding the escape. `ChromaLocation` and
  `DcpTargetGamut` are unaffected — FFmpeg already agrees with them.

### Fixed

- **`ChannelLayout`'s 5.x slugs were inverted against FFmpeg** — breaking, and
  the wire form moves with them. FFmpeg's `channel_layout_map[]` gives the
  unqualified name to the **back**-speaker layouts (`"5.0"` →
  `AV_CH_LAYOUT_5POINT0_BACK` = `SURROUND|BACK_LEFT|BACK_RIGHT`, `"5.1"` →
  `AV_CH_LAYOUT_5POINT1_BACK`) and qualifies the side ones (`"5.0(side)"` →
  `AV_CH_LAYOUT_5POINT0` = `SURROUND|SIDE_LEFT|SIDE_RIGHT`, `"5.1(side)"` →
  `AV_CH_LAYOUT_5POINT1`). This crate had the four the other way round, so an
  FFmpeg- or `ffprobe`-sourced `"5.1"` parsed to `N5Point1`, whose docs promise
  side speakers, when FFmpeg meant back. The strings round-tripped, so nothing
  caught it; anything keying off the variant's documented speaker set was
  quietly wrong. The four slugs are swapped to match:
  `N5Point0` → `"5.0(side)"`, `N5Point0Back` → `"5.0"`,
  `N5Point1` → `"5.1(side)"`, `N5Point1Back` → `"5.1"`. `as_str`, `Display`,
  `FromStr` and the serde wire form move together. A transcribed
  `channel_layout_map[]` table now pins every named layout, so the next
  inversion fails a test instead of shipping.
- `ChannelLayout::Quad`'s doc claimed it was "L+R+SL+SR **or** L+R+BL+BR". It
  is `AV_CH_LAYOUT_QUAD` = `STEREO|BACK_LEFT|BACK_RIGHT` — back only. The side
  four-channel layout is FFmpeg's `AV_CH_LAYOUT_2_2`, named `"quad(side)"`,
  which this vocabulary does not enumerate and which rides `Other`. Doc only;
  the slug was already right.
- The hardware-exclusion roster no longer lies: `xvmc` had outlived
  `AV_PIX_FMT_XVMC` (already gone at n8.1) and excluded nothing.
  `cargo xtask sync` now proves every roster entry against the pinned header
  and refuses to write a table built from a stale one.

## [0.3.0]

**Breaking**, on three counts: the numeric escape (`Unknown(u32)`) is struck
from every coded vocabulary and `Other(SmolStr)` becomes the one extension
idiom; the YUV/RGB kernel door takes a closed selector instead of the open
`Matrix`; and the public dependency `mediatime` crosses 0.2 → 0.3 (its rescale
ladder was renamed and its rounding corrected — see mediatime's own notes;
`mediatime::Timestamp` is in this crate's public API).

### Added

- `FromStr` for the pixel-format, bayer, subtitle, audio and colour
  vocabularies — eighteen parse twins, each generated from its own `as_str`
  table and each with its **own** parse error type (the shared
  `parse::ParseError` is gone).
- The five RAW types (`BayerPattern`, `BayerDemosaic`, `WbChannel`,
  `WhiteBalance`, `ColorCorrectionMatrix`) join serde, arbitrary and
  quickcheck; the two float carriers deserialize through `try_new` and refuse
  invalid values.
- `Unwrap`/`TryUnwrap` reach `SampleFormat`, `ChannelLayout` and
  `SubtitleCodec`; the size threshold that keeps the two 200-plus codec enums
  out is now written down.
- `KernelMatrix` — a closed `Copy` selector of the ten matrices the
  conversion kernels actually have coefficients for — and `KernelGamut`
  (which deletes `xyz12_to`'s documented panic). Kernel entries take them
  directly; the other eight named matrices now refuse loudly
  (`UnsupportedKernelMatrixError`) where they used to convert silently as
  BT.709. `Unspecified` keeps its documented BT.709 default.
- Geometry projections: `Dimensions::aspect_ratio`, `Rect::aspect_ratio`
  (`Option<Rational>` — zero extents are ordinary), and
  `Dimensions::display_size(SampleAspectRatio)` with FFmpeg's
  `AV_ROUND_NEAR_INF` rounding; `Dimensions::contains(&Rect)` for crop
  validation.
- `FieldOrder::Unknown` and `PixelFormat::None` as **named** members (FFmpeg's
  own `AV_FIELD_UNKNOWN` / `AV_PIX_FMT_NONE` code points — a file saying
  "unknown" is a value, not an escape).

### Changed

- **Breaking:** `Unknown(u32)` struck from eleven types. `from_u32` returns
  `Option<Self>` and `to_u32` returns `Option<u32>` (the FFmpeg-interop
  boundary); serde **and** buffa wire shapes move from number to slug for the
  coded enums; `Copy` leaves the ten enums, `color::Info` and
  `frame::VideoFrame` (the per-row walker types get it back through
  `KernelMatrix`); `as_str` returns `&str` and is no longer `const` on the
  ten.
- **Breaking:** the canonical text form is lowercase (`"Bilinear"` →
  `"bilinear"`), every name door ASCII-case-folds its input (`FromStr` and the
  `other()` constructors), and folding is allocation-free — the parse tables
  compare bytes, which also made the biggest table ~2.5× faster.
- **Breaking:** at the no-alloc tier the coded enums are closed vocabularies —
  `Other` lives behind `any(feature = "alloc", feature = "std")`, and the tier
  law is documented: no name available means an error at the boundary, never a
  wrong value.
- **Breaking:** public dependency `mediatime` 0.2 → 0.3.
- Unit tests moved beside their modules (`foo/mod.rs` + `foo/tests.rs`) across
  the crate; test counts verified identical by name. Internal only.

**Breaking**, on two independent counts. `frame::Rational` widens to
`i64`/`NonZeroI64` and its constructor becomes checked (see **Changed**
below). And two public dependencies cross a major: `mediatime` 0.1 → 0.2
(`mediatime::Timestamp` appears in `frame::TimestampedFrame`'s public
signatures, so a caller holding a `mediatime 0.1` value no longer type-checks)
and `buffa` 0.8 → 0.9 (`Message` is implemented for public types, so a
downstream on 0.8 no longer sees those impls). **No wire byte changes** — every
entry below carries its own proof.

### Changed

- **`frame::Rational` is now `i64` / `NonZeroI64`** (was `u32` /
  `NonZeroU32`), and [`Rational::new`] is checked rather than total:
  it panics on `num < 0` or `den < 0`, with a new
  `Rational::try_new -> Option<Self>` as the fallible form.
  `SampleAspectRatio` (a newtype over `Rational`) and `FrameRate`
  (which composes it) follow automatically and carry no width of
  their own; `SampleAspectRatio::new` panics the same way, and its
  fallible route is the existing
  `Rational::try_new(..).map(SampleAspectRatio::from)`.

  *Why `i64`, and why `mediatime::Timebase` stays `i32`.* `mediaframe`
  is a pure **receiver** — nothing here is handed back to a decoder
  SDK — so "must round-trip into an `AVRational`", the reason
  `Timebase` went to `i32`, does not apply. What does apply is
  storage (`sqlx` has no `Type<Postgres>` for `u32`, so a `u32`
  widens to `i64` to be stored regardless) and ingest (R3D metadata
  returns `unsigned int`, ISO BMFF `pasp` is `unsigned int(32)` —
  values `i32` would have to *reject*). `Timebase` is additionally an
  arithmetic operand whose rescale overflow proofs need `num < 2^32`;
  `Rational` never multiplies against a PTS and carries no such
  proof. **The two types differ deliberately** — this is not an
  inconsistency to reconcile.

  The four setters (`with_num`/`with_den`/`set_num`/`set_den`) now
  route through `new`, so the sign invariants have exactly one
  enforcement site rather than a mutator hole. `Deserialize` was the
  other unguarded construction path — the derive assigns fields
  directly, and the field types no longer carry the invariant — so
  each field gained a `deserialize_with` guard; `{"num": -5}` is now
  a deserialization error instead of a value the constructor would
  refuse. The constructor deliberately does **not** reduce to lowest
  terms: a stream declaring `2/4` reads back as `2/4`.

  **The wire format does not change.** `SampleAspectRatio` and
  `Rational` move from `uint32 num/den` to `int64 num/den`, which is
  the same plain non-ZigZag varint over every value the old
  representation could hold. Proven, not inferred: 680 payloads
  across `Rational`, `SampleAspectRatio` and `FrameRate` — spanning
  every varint continuation boundary and `u32::MAX` — encode to
  identical bytes under both representations, and the `i64` build
  cross-decodes all 680 `uint32`-era payloads back to the same values
  and the same bytes. (`sint32`/`sint64` would have been the silent
  break, since ZigZag re-encodes every value; this crate uses neither.)
  Decode stays total in the newly reachable directions: a negative
  numerator clamps to `0` and a zero-or-negative denominator to `1`,
  matching `mediatime::Timebase`'s decode policy.

- **`xtask`: `syn` 2 → 3, `prettyplease` 0.2 → 0.3** — a coupled bump
  (`prettyplease` 0.3 requires `syn ^3`, so neither moves alone).
  Dev-only: `xtask` is `publish = false`, so nothing here reaches the
  published `mediaframe` artifact. `syn` 3's breaking change is
  `Signature::unsafety: Option<Token![unsafe]>` → the tri-state
  `Signature::safety: Safety` (Rust 2024 `unsafe extern`); `xtask`
  names only `syn::Ident` and `syn::parse2::<syn::File>` and never
  inspects a signature, so it compiles unchanged. `prettyplease` 0.3
  emits **byte-identical** output to 0.2 for the generated
  `mediaframe/src/codec.rs` (89,303 bytes pre-`rustfmt`), so
  `cargo xtask check`'s byte-for-byte freshness diff stays green and
  the committed file needs no regeneration.
- **`quickcheck-richderive` 0.3 → 0.4** (`quickcheck` feature) — upstream
  is a dependency-only release (its own `syn` 2 → 3 migration); the
  derive, the accepted attribute keys, and the emitted impls are
  unchanged. Re-verified against *this* crate rather than inherited:
  `-Zunpretty=expanded` over `--features quickcheck,frame,buffa,serde,arbitrary`
  is byte-identical across the bump (263,301 lines). All 40 derive sites
  keep their `#[quickcheck(arbitrary = "…")]` attributes as-is —
  that key names a **function**, and every value here points at a
  `pub(crate) fn(&mut Gen) -> T` in `quickcheck_helpers`, so none of
  them is the sibling `with = "…"` key (which names a *module* supplying
  both `arbitrary` and `shrink`). No consumer-visible change.
- **`buffa` 0.8 → 0.9** (`buffa` feature) — `Message::write_to` now takes
  `&mut impl EncodeSink` in place of `&mut impl BufMut`, so all 26
  `write_to` signatures in `src/buffa.rs` move (the trait method's
  parameter type is what changed, so keeping `BufMut` is an `E0276`
  "impl has stricter requirements"). Nothing else in the module changes:
  no body touches a `BufMut` method directly — every byte goes through
  `buffa`'s `encode_*` helpers, whose bodies are unchanged — and
  `buffa` carries a blanket `impl<T: BufMut + ?Sized> EncodeSink for T`,
  so every existing caller still passes a `Vec<u8>` / `BytesMut`.
  **The wire format does not change.** Established on this crate's own
  types rather than inherited: all 37 `Message` impls were driven over
  400 deterministic `arbitrary` values each (14,800 encodings) under
  0.8.1 and 0.9.1, and the encoded bytes are identical in every case —
  so bytes written by a 0.8-linked peer still decode here. The 112
  non-identity round-trips are `audio::SampleFormat` only, are present
  identically in both runs, and are the documented `Other(SmolStr)` →
  `Unknown(u32::MAX)` collapse, not a regression.
  `EncodeSink`'s segmented `Rope` sink is **not** adopted here.
- **`mediatime` 0.1 → 0.2** — `mediatime::Timebase`'s `num`/`den` became
  `i32`/`NonZeroI32` (matching ffmpeg's `AVRational`, which is
  `{int num; int den;}`), `Timebase::new` now panics on a negative
  numerator or denominator with `try_new` returning `Option`, and its
  `Deserialize` gained a range guard. The surface this crate touches is
  small: `mediatime::Timestamp` — not `Timebase` — is what
  `frame::TimestampedFrame` carries, and `Timestamp::new(i64, Timebase)`
  is unchanged, so the single site that moves is one test's
  `NonZeroU32` → `NonZeroI32` denominator literal. Every other
  `Timebase` mention in this crate is prose, and each statement it
  makes (non-proto-zero `1/1` default; a frame rate is deliberately not
  a PTS timebase) is still true of 0.2.
  Also collapses the transient duplicate from the previous commit:
  `mediatime` 0.2 requires `buffa` 0.9, so the graph carries one
  `buffa` again.

## [0.1.7]

### Added

- **`Primaries::chromaticities()` / `Primaries::white_point()`** —
  `const fn`s exposing the per-standard CIE 1931 `xy` reference data for
  each defined `Primaries` variant: the R, G, B primaries as
  `Option<[ChromaCoord; 3]>` (index `0` = red, `1` = green, `2` = blue)
  and the reference white point as `Option<ChromaCoord>`, both in
  `ChromaCoord`'s SMPTE ST 2086 fixed-point units (floating value =
  `raw / 50000.0`, so BT.709 red `(0.640, 0.330)` is `(32000, 16500)`).
  Values track FFmpeg's `av_csp_primaries_desc` (`libavutil/csp.c`)
  across BT.709 / sRGB, BT.470 M/BG, SMPTE 170M/240M, Film, BT.2020,
  SMPTE ST 428, DCI-P3 (RP 431-2), Display-P3 (EG 432-1), and EBU
  3213-E, with white points D65 / CIE C / DCI / equal-energy E as each
  standard dictates. `Unknown` and `Unspecified` return `None` (no
  defined primaries); the within-crate match is exhaustive without a
  wildcard, so a future primaries variant cannot silently fall through.
  Puts the colorimetric reference data in the format authority so
  downstream crates (e.g. `colconv`) consume one table instead of
  re-hardcoding chromaticities, and unblocks chromaticity-derived matrix
  work. Note that SMPTE ST 428 mirrors FFmpeg's tabulated D-Cinema
  primaries (white point E), **not** the CIE XYZ identity that ITU-T
  H.273 Table 2 lists for ST 428-1. Additive and non-breaking.

## [0.1.6]

### Added

- **`PixelFormat::V410Be`** — first-class big-endian counterpart of
  `V410Le` for the packed YUV 4:4:4 10-bit `V410` layout (one 32-bit
  word per sample). The big-endian decode path already existed — the
  `V410Frame<'a, true>` / `V410BeFrame` borrow view, the `V410<true>`
  source marker, and the endian-generic `v410_to::<true>` walker — and
  is now exposed as a wire-stable enum variant (`as_str()` slug
  `"v410be"`, discriminant `435`). Additive and non-breaking.
- **`PixelFormat::canonical()`** — `const fn` resolving a deprecated /
  aliased pixel format to `(canonical_format, Option<DynamicRange>)`:
  the non-deprecated format describing the same bytes, plus the dynamic
  range the alias *pins* (or `None` when the range is stream-driven).
  Centralises the alias table in the format authority so downstream
  crates (e.g. `colconv`) consume one mapping instead of each
  re-deriving it. Resolves the `yuvj{411,420,422,440,444}p` full-range
  aliases → their `yuv*p` base + `DynamicRange::Full`, `Gray8a` /
  `Y400a` → `Ya8`, and the `XV30` byte-order pair onto its matching
  `V410` variant — `Xv30Le` → `V410Le` and `Xv30Be` → `V410Be` (`XV30`
  is the FFmpeg rename of the identical-bit-pattern `V410`; both endians
  resolve while preserving byte order). The match is exhaustive without
  a wildcard, so a future alias variant cannot silently fall through.
  Additive and non-breaking — every other format (including `Unknown`)
  returns `(self, None)`.

## [0.1.5]

### Added

- **Pixel-format source coverage** — frame types, source markers, and
  `{fmt}_to` walkers for a large batch of additional formats, each wired
  through its per-family feature flag:
  - **NV20** (`yuv-semi-planar`) — 10-bit low-bit-packed semi-planar
    4:2:2; the low-bit-aligned twin of `P210` (one `u16` per sample with
    the 10 active bits in the low positions).
  - **Gray family** (`gray`) — `Gray32` (32-bit), `Grayf16` (`f16`),
    `Yaf16` / `Yaf32` (`f16` / `f32` gray + alpha).
  - **GBR family** (`gbr`) — `Gbrap32` (32-bit GBRA), `Gbrp10Msb` /
    `Gbrp12Msb` (MSB-packed — samples in the high bits).
  - **RGB family** (`rgb` / `rgb-float`) — `Rgb96` / `Rgba128`
    (32-bit-per-channel integer), `Rgbaf16` / `Rgbaf32` (`f16` / `f32`
    RGBA).
  - **YUV 4:4:4 MSB** (`yuv-planar`) — `Yuv444p10Msb` / `Yuv444p12Msb`
    (MSB-packed planar 4:4:4).
  - **Packed 4:4:4** (`yuv-444-packed`) — `Ayuv`, `Uyva`, `Vyu444`.
  - **Legacy bit-packed RGB** (`rgb-legacy`) — `Rgb4` / `Rgb4Byte` /
    `Rgb8` and `Bgr4` / `Bgr4Byte` / `Bgr8`.
  - **`Xv48`** (`yuv-444-packed`) — 16-bit packed YUV 4:4:4 (FFmpeg
    `AV_PIX_FMT_XV48LE` / `BE`); the full-16-bit sibling of `Xv36`.
  - **`Yuva420p12`** (`yuva`) — 12-bit low-bit-packed planar YUVA 4:2:0;
    a mediaframe extension (no FFmpeg pixel format) that non-FFmpeg
    decoders / WebCodecs emit.

### Changed

- **High-bit Bayer is now endian-aware** (`bayer`) — the Bayer source
  marker gains a trailing `const BE: bool = false` (source-compatible
  default), mirroring the `Y2xx` family, so the 10 / 12 / 14 / 16-bit
  Bayer formats (all four CFA patterns) support both little- and
  big-endian planes. The `&[u16]` plane is interpreted as wire bytes
  (LE for `BE = false`, BE for `BE = true`); FFmpeg defines the Bayer
  LE/BE split only at 16-bit, so the 10 / 12 / 14-bit forms are
  mediaframe extensions. Little-endian behavior is byte-identical on
  little-endian hosts.

## [0.1.4]

### Added

- **`audio::ReplayGain`** — value object for container-tagged loudness-
  normalization recommendations (FFmpeg `AV_PKT_DATA_REPLAYGAIN` side
  data or the `REPLAYGAIN_TRACK_*` / `REPLAYGAIN_ALBUM_*` `AVDictionary`
  keys). Carries `track_gain_db`, `track_peak`, and the optional
  album-level `album_gain_db` / `album_peak`. Distinct from
  [`audio::Loudness`]: `Loudness` is the EBU R128 measurement of the
  signal; `ReplayGain` is the normalization recommendation a tagger
  wrote into the container (the delta from a −18 LUFS reference).
  Album-level numbers cannot be computed from a single track's loudness
  alone, so both are independently useful and not redundant. Buffa wire
  bridge: `{ float track_gain_db = 1; float track_peak = 2; optional
  float album_gain_db = 3; optional float album_peak = 4; }`. Test
  helpers wired through `quickcheck_helpers::composite::replay_gain` +
  `arbitrary_impls::composite`.

## [0.1.1] May 21, 2026

### Added

- **`serde` feature** — optional `serde::{Serialize, Deserialize}` for the
  whole descriptor vocabulary, gated behind `--features serde` (off by
  default). The wire shape mirrors what storage backends already use, so a
  serde-`json` value matches their representation:
  - **Open** codec / format enums (`codec::{Video,Audio,Subtitle}Codec`,
    `container::Format`, `subtitle::Format`,
    `audio::{ChannelLayout, ContainerFormat}`) serialize as their canonical
    `as_str()` slug — `VideoCodec::H264` ⇄ `"h264"`, `Other("x265")` ⇄
    `"x265"` (no `{"Other": …}` wrapper).
  - **`audio::SampleFormat`** — has BOTH an `Unknown(u32)` numeric escape
    AND an `Other(SmolStr)` string escape, so it gets a bespoke impl rather
    than the slug-only path. On **human-readable** formats (JSON / YAML /
    …): named + `Other` values serialize as their `as_str()` string,
    `Unknown(v)` as the bare numeric code `v`. On **non-human-readable**
    binary formats (bincode / postcard / …): an explicit tagged
    `{Code(u32), Slug(Cow<str>)}` wire enum, since `deserialize_any` is
    unavailable there. All three arms round-trip losslessly on both.
  - **Closed FFmpeg-coded enums with a lossless `Unknown(u32)` escape**
    (`color::{Matrix, Primaries, Transfer, DynamicRange, ChromaLocation,
    DcpTargetGamut}`, `pixel_format::PixelFormat`,
    `frame::{Rotation, FieldOrder, StereoMode}`) and
    `disposition::TrackDisposition` serialize as their `to_u32()` integer.
    Round-trip is total: an unrecognised *code* deserializes to `Unknown(v)`.
    These accept only integers — there is no slug form.
  - **Strictly-closed coded enums (no `Unknown` arm)** —
    `subtitle::TrackOrigin` (`Embedded`/`Sidecar`/`External`) and
    `audio::BitRateMode` (`Cbr`/`Vbr`/`Abr`) — serialize as their `to_u32()`
    integer but **reject unrecognised wire codes** as serde errors instead
    of silently collapsing them to the default variant. Both expose a
    `try_from_u32(v: u32) -> Option<Self>` method backing this behavior.
  - **Plain structs** (`color::Info` and its HDR/mastering sub-structs,
    `frame::{Dimensions, Rect, Rational, SampleAspectRatio, FrameRate}`,
    `audio::{Loudness, Tags, Device}`… ) derive serde directly.
  - **Validated structs** (`capture::GeoLocation`, `audio::Fingerprint`,
    `audio::CoverArt`) route deserialize through their checking
    constructors, so out-of-range / invariant-violating values are rejected
    rather than materialised.
  - **`lang::Language`** serializes as its canonical BCP-47 string
    (`"en-US"`, `"zh-Hant-TW"`, `"und"`).
  - Works at every capability tier: the no-alloc Copy types gain serde
    under bare `--features serde`; the heap-tier types (codecs, formats,
    audio metadata, capture, language) when paired with `alloc` / `std`
    (forwarding `serde` to `smol_str` / `bytes`).

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
    (`{ algorithm: SmolStr, value: bytes::Bytes }` — O(1) clone),
    `try_new` rejects empty algorithm.
  - `audio::CoverArt` — embedded picture
    (`{ mime: SmolStr, data: bytes::Bytes }` — O(1) clone), `try_new`
    rejects empty mime / empty data.
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
  wrapping `icu_locale_core` `Language`/`Script`/`Region` subtags (`Copy`,
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
- **Deps** — adds `icu_locale_core = "2"` and `bytes = "1"` (both
  optional, gated on the `alloc` feature; both `no_std`-friendly).
  `bytes::Bytes` backs the `audio::CoverArt` / `audio::Fingerprint`
  payloads so large blobs clone in O(1).

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

## videoframe 0.3.1 — May 19, 2026

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

## videoframe 0.3.0 — May 19, 2026

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

## videoframe 0.2.0 — May 12, 2026

### Added

- Add bayer structures

### Breakage

- **`cfa`** - remove cfa mod

### Changes

- Make all error enums follows tuple enum errors

## videoframe 0.1.0 — May 11, 2026

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
