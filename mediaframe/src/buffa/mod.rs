//! `buffa::Message` implementations for the mediaframe wire-relevant
//! types, behind the `buffa` feature. Used via `extern_path` from
//! buffa-generated crates so a `.proto`-defined message can embed a
//! mediaframe type without redefining it.
//!
//! These are hand-written inherent-trait impls — there is **no**
//! codegen and **no** `.proto` in this crate (the design `mediatime`'s
//! own impls had through 0.4; `mediatime` 0.5 maps `.mediatime.v1`
//! onto its `wire` types instead). The module needs no re-export: the
//! impls are `impl Trait for crate::Type`.
//!
//! # Wire format (clean redesign — no compatibility with any prior
//! encoding is required)
//!
//! ## Enums (each is a standalone message = one field)
//!
//! ```text
//! Matrix    { uint32 value = 1; }   // value = to_u32()
//! Primaries { string value = 1; }
//! Transfer  { string value = 1; }
//! DynamicRange     { string value = 1; }
//! ChromaLocation { string value = 1; }
//! DcpTargetGamut { string value = 1; }
//! Rotation       { string value = 1; }
//! FieldOrder     { string value = 1; }
//! StereoMode     { string value = 1; }
//! PixelFormat    { string value = 1; }
//! ```
//!
//! Each enum encodes its `as_str()` slug as a single `string` at
//! field #1, decoded via `FromStr` — the same shape the codec family
//! has always used. The slug is the spelling because `Other(Utf8Bytes)`
//! is the crate's one extension idiom: a value this build does not
//! name still has a *name*, and a number would not carry it.
//!
//! **Default-elision (not proto3 zero-elision):** the field is
//! written iff `*self != <Ty>::default()`. The decoder seeds the
//! message from `Default` (= FFmpeg `UNSPECIFIED` for the colour
//! enums — code `2` for primaries/transfer/matrix, `0` for
//! range/chroma), so an absent field decodes back to `Default`. A
//! *present* field always carries the exact `to_u32()` code —
//! **including code `0`** (`Matrix::Rgb`, FFmpeg
//! `AVCOL_SPC_RGB`), which is *non-default* and therefore explicitly
//! encoded, so it is never conflated with an absent field. Plain
//! proto3 zero-elision would be **unsound** here (it would drop the
//! non-default code-`0` `Rgb`); default-elision is exact for every
//! value. Wrong wire type on field #1 →
//! `DecodeError::WireTypeMismatch`; unknown fields are skipped via
//! `skip_field_depth`.
//!
//! ## Structs
//!
//! ```text
//! Dimensions        { uint32 width = 1; uint32 height = 2; }
//! Rect              { uint32 x = 1; uint32 y = 2; uint32 width = 3; uint32 height = 4; }
//! SampleAspectRatio { int64  num = 1; int64  den = 2; }          // both ALWAYS encoded
//! Rational          { int64  num = 1; int64  den = 2; }          // both ALWAYS encoded
//! FrameRate         { Rational rate = 1;                         // rate ALWAYS encoded
//!                     bool     is_vfr = 2; }                     // proto3 zero-elision
//! DolbyVisionConfig { uint32 profile = 1; uint32 level = 2;      // proto3 zero-elision
//!                     bool rpu_present = 3; bool el_present = 4; //   (all-zero default)
//!                     uint32 bl_signal_compat_id = 5; }
//! Info         { Primaries primaries = 1;              // all five ALWAYS
//!                     Transfer  transfer  = 2;              //   encoded as the
//!                     Matrix    matrix    = 3;              //   bare uint32 id
//!                     DynamicRange     range     = 4;              //   (not nested msgs)
//!                     ChromaLocation chroma    = 5; }
//! ContentLightLevel { uint32 max_cll = 1; uint32 max_fall = 2; }
//! ChromaCoord       { uint32 x = 1; uint32 y = 2; }              // u16 widened to uint32
//! MasteringDisplay  { ChromaCoord primary_r   = 1;               // ALWAYS encoded
//!                     ChromaCoord primary_g   = 2;               // ALWAYS encoded
//!                     ChromaCoord primary_b   = 3;               // ALWAYS encoded
//!                     ChromaCoord white_point = 4;               // ALWAYS encoded
//!                     uint32 max_luminance = 5;
//!                     uint32 min_luminance = 6; }
//! HdrStaticMetadata { MasteringDisplay  mastering     = 1;       // absent when None
//!                     ContentLightLevel content_light = 2; }     // absent when None
//! ```
//!
//! Field numbers follow declaration order. proto3 zero-elision is
//! used **only** where the decoder seed (`DefaultInstance`, i.e.
//! `Default`/`new`) is the proto-zero for that field
//! (`Dimensions`, `Rect`, `ContentLightLevel`, `ChromaCoord`, the
//! `*_luminance` scalars). Where `Default` ≠ proto-zero the field is
//! **always encoded** (the reasoning `mediatime::Timebase`'s own impl
//! followed through `mediatime` 0.4):
//!
//! - `SampleAspectRatio` — `Default` is `1:1`. `num`'s default is
//!   `1` (≠ 0) and `den` is `NonZeroI64` (never 0), so eliding a
//!   zero would mis-decode. Both fields are always written. On the
//!   way back in, both halves are clamped into the range
//!   `Rational::new` accepts, so decode stays total: a negative
//!   `num` becomes `0`, and a `den` that is zero *or negative*
//!   becomes `1`. Neither is producible by this encoder — a peer
//!   reaches them either directly or by writing a `uint64` above
//!   `i64::MAX`, which `decode_int64` reinterprets as negative.
//!   (Before `Rational` became signed, `den` was `NonZeroU32` and
//!   only the zero case existed.)
//! - `Rational` — same shape / reasoning as `SampleAspectRatio`
//!   (`Default` is `1/1`, `num` default `1`, `den` `NonZeroI64`):
//!   both fields always encoded, malformed `num`/`den` clamped the
//!   same way.
//! - `FrameRate` — `rate` is an always-encoded length-delimited
//!   `Rational` sub-message (its inner `Default` is `1/1` ≠
//!   proto-zero, so the nested-message-always-encoded stance
//!   `mediatime::Timebase` took through 0.4 applies, like `MasteringDisplay`'s
//!   coords); `is_vfr` defaults to `false` == proto-zero so it uses
//!   proto3 zero-elision.
//! - `Info` — **all five enum fields are always encoded** as
//!   the bare FFmpeg-code `uint32` id (not a nested message); tags
//!   #1–#5 are single-byte. `Info`'s own seed is
//!   `Info::UNSPECIFIED` (every field FFmpeg `UNSPECIFIED`).
//!   Always-encoding keeps the round-trip exact regardless of which
//!   FFmpeg code a field holds — in particular `matrix ==
//!   Matrix::Rgb` (FFmpeg code `0`) survives because the id is
//!   written unconditionally, never elided — the same defensive
//!   always-encode stance `mediatime::Timebase` took through 0.4.
//! - `MasteringDisplay` — the three primaries and the white point
//!   are always-encoded length-delimited sub-messages so presence
//!   is unambiguous and `decode(encode(x)) == x` holds regardless of
//!   `ChromaCoord` content (nested-message presence, like
//!   `mediatime`'s always-encoded `Timebase` through 0.4).
//! - `HdrStaticMetadata` — the two `Option` fields are
//!   presence-encoded length-delimited messages, omitted entirely
//!   when `None`.
//!
//! Every `merge_field` rejects a wrong wire type with
//! `DecodeError::WireTypeMismatch` and skips unknown fields with
//! `skip_field_depth`; `clear()` resets to `Default` / `new`.
//!
//! ## Audio + container types
//!
//! ```text
//! ChannelLayout        { string value = 1; }   // value = as_str()
//! BitRateMode          { uint32 value = 1; }   // value = to_u32() (Cbr=0)
//! ChannelOrder         { uint32 value = 1; }   // value = to_u32() (Unspecified=0)
//! SampleFormat          { uint32 value = 1; }   // value = to_u32() (FFmpeg AV_SAMPLE_FMT_* code, Other → u32::MAX)
//! ContainerFormat { string value = 1; }   // value = as_str()
//! Format      { string value = 1; }   // value = as_str()
//!
//! ChannelSpec { uint32 index = 1; uint32 raw_id = 2; string label = 3; }
//! ChannelLayoutDescription
//!                  { uint32 order       = 1;   // ChannelOrder::to_u32()
//!                    uint32 channels    = 2;
//!                    string known_kind  = 3;   // ChannelLayout::as_str()
//!                    optional uint64 native_mask = 4;
//!                    repeated ChannelSpec custom_channels = 5;
//!                    string text        = 6; }
//!
//! Loudness         { float integrated_lufs = 1; float range_lu = 2;
//!                    float true_peak_dbtp = 3; float sample_peak_dbfs = 4; }
//! ReplayGain       { float track_gain_db = 1; float track_peak = 2;
//!                    optional float album_gain_db = 3;
//!                    optional float album_peak = 4; }
//! Fingerprint { string algorithm = 1; bytes value = 2; }     // algorithm ALWAYS encoded
//! CoverArt    { string mime      = 1; bytes data  = 2; }     // both ALWAYS encoded
//! Tags        { string title        = 1; string artist        = 2;
//!                    string album_artist = 3; string album         = 4;
//!                    string composer     = 5; string genre         = 6;
//!                    string comment      = 7;
//!                    uint32 year         = 8; uint32 track_number  = 9;
//!                    uint32 track_total  = 10; uint32 disc_number   = 11;
//!                    uint32 disc_total   = 12; string language      = 13; }
//! ```
//!
//! - **String-bearing enums** (`ChannelLayout`, `ContainerFormat`,
//!   `Format`, `SampleFormat`) encode their `as_str()` slug. Default
//!   (where defined) elides; `Other(Utf8Bytes)` round-trips losslessly.
//!   `BitRateMode` and `ChannelOrder` are strictly closed and encode
//!   their `to_u32()` id. Zero-elision is sound for both: each one's
//!   `Default` (`Cbr` / `Unspecified`) *is* code `0`, so an absent field
//!   decodes back to it exactly — unlike the colour enums, where a
//!   non-default variant holds code `0` and eliding it would be a
//!   silent loss.
//! - **`ChannelSpec`** — `index` / `raw_id` (`uint32`) and `label`
//!   (`string`) all use proto3 zero-elision: the seed is
//!   `ChannelSpec::default()`, whose three fields are `0`, `0` and the
//!   empty string, which is proto-zero for each.
//! - **`ChannelLayoutDescription`** — `order`, `channels`,
//!   `known_kind` and `text` use proto3 zero-elision, sound for the
//!   same reason: the seed is the type's `Default`, whose order is code
//!   `0`, channel count `0`, name `ChannelLayout::default()` (the
//!   `Other("")` sentinel, so `as_str()` is `""`) and text empty.
//!   `known_kind` decodes through `ChannelLayout`'s total `FromStr`, so
//!   a name this build does not enumerate arrives as `Other(name)`
//!   rather than as a decode error — the open-vocabulary rule, applied
//!   to a field instead of a standalone message.
//!   `native_mask` is an `optional uint64` using presence encoding
//!   (emitted iff `Some`, including for `Some(0)`, so a layout that
//!   reports an all-zero mask stays distinct from one that reports
//!   none). `custom_channels` is the crate's first **repeated** field:
//!   one length-delimited `ChannelSpec` sub-message per element, in
//!   order, and an empty list writes nothing. The decoder appends each
//!   element as it arrives rather than reading the list out and writing
//!   it back, which would make decode quadratic in a length the peer
//!   chooses.
//! - **`Loudness`** — all four `f32` fields use proto3 zero-elision
//!   (`Default` is all-zero == proto-zero for `f32`). Each present
//!   field is wire-type `Fixed32` (4 bytes LE).
//! - **`ReplayGain`** — `track_gain_db` / `track_peak` use proto3
//!   zero-elision (`Default` is all-zero == proto-zero for `f32`);
//!   `album_gain_db` / `album_peak` are `optional float` so a
//!   distribution-absent album-level number round-trips as `None`
//!   (the wire field is absent rather than zero). Each present field
//!   is wire-type `Fixed32` (4 bytes LE).
//! - **`Fingerprint`** — `algorithm` is ALWAYS encoded
//!   (`try_new` rejects empty, so a default-constructed wire-empty
//!   `algorithm` would not be a valid `Fingerprint` — encoding
//!   it explicitly preserves the invariant on the wire round-trip).
//!   `value` (bytes) uses proto3 zero-elision (an empty fingerprint
//!   is legal). The decoder seed is `try_new("default", []).unwrap()`
//!   so that an absent `algorithm` decodes to a synthetic non-empty
//!   placeholder rather than violating the type invariant.
//! - **`CoverArt`** — both `mime` and `data` are ALWAYS encoded
//!   (`try_new` rejects empty in either, so default-constructed
//!   wire-empty fields would violate the invariant). Same
//!   placeholder-seed strategy as `Fingerprint`.
//! - **`Tags`** — string fields use proto3 zero-elision (the
//!   empty string is the canonical "absent" value); numeric `u16`
//!   fields are widened to `uint32` and use proto3 zero-elision —
//!   `Some(0)` (legal value) and `None` (absent) **cannot be
//!   distinguished on the wire** in this codec; both round-trip to
//!   `None`. A future codec revision can switch to wrapper messages
//!   if the distinction becomes load-bearing. `language` is the
//!   canonical BCP 47 tag as a string; the empty string means the
//!   `Option<LanguageId>` was `None`.
//!
//! ## Subtitle + disposition
//!
//! Three stream-vocab types from the `subtitle` + `disposition`
//! modules. All three are standalone one-field messages:
//!
//! ```text
//! Format      { string value = 1; }   // FFmpeg-style slug from `as_str()`
//! TrackOrigin { string value = 1; }   // value = as_str()
//! TrackDisposition    { uint32 bits  = 1; }   // bits = to_u32() (= raw bitflags bits)
//! ```
//!
//! - **`Format`** — a closed-ish enum with an `Other(Utf8Bytes)`
//!   escape arm has no stable numeric id, so it encodes the
//!   FFmpeg-style slug (`"srt"` / `"webvtt"` / `"hdmv_pgs_subtitle"` /
//!   …) as a `string`. The decoder funnels through `FromStr` (total —
//!   unknown slugs land in `Other`). Default-elision: the default is
//!   not proto-zero (`Srt` is the inhabited representative — though
//!   the encoder treats *every* value as non-default and always
//!   encodes, to side-step the issue entirely). In practice we
//!   always-encode the slug so an empty string can never be conflated
//!   with `Srt`; on decode an empty string maps to `Other("")`,
//!   matching `FromStr`.
//! - **`TrackOrigin`** — an open enum since 0.5.0 (`Other(Utf8Bytes)`),
//!   so its stable ids no longer span the value space and it encodes
//!   the slug as a `string`, like `Format` above. Always-encoded for
//!   the same reason: the empty string is `Other("")` on decode, a
//!   distinct legal value that eliding would conflate with an absent
//!   field. **Wire-incompatible with 0.4.x**, which wrote a varint id
//!   in this field.
//! - **`TrackDisposition`** — bitflags. Encoded as the raw `u32`
//!   bits at field #1, decoded via [`TrackDisposition::from_u32`]
//!   (`from_bits_retain` semantics — unknown bits round-trip
//!   losslessly). Default-elision is sound: the default is the
//!   empty flag set whose `bits()` is `0` (proto-zero).
//!
//! ## Capture + language
//!
//! Three alloc-gated types from the `capture` + `lang` modules:
//!
//! ```text
//! Device      { string make = 1; string model = 2; }     // proto3 zero-elision (empty == absent)
//! GeoLocation { double lat = 1; double lon = 2;          // lat/lon ALWAYS encoded
//!               float altitude = 3; }                     // altitude emitted iff Some
//! LanguageId  { string value = 1; }                      // BCP 47 canonical tag; never elides
//! ```
//!
//! - **`Device`** — two empty strings == proto-zero, so proto3
//!   zero-elision is sound. Empty string is the in-rust sentinel
//!   for "absent" (matches the same convention used by `Tags`).
//! - **`GeoLocation`** — the `(0.0, 0.0)` "Null Island" default is a
//!   real, legal coordinate, so proto3 zero-elision on `lat`/`lon`
//!   would be **unsound** (it would lose the Null-Island record).
//!   Both fields are always encoded; the optional `altitude` field
//!   uses presence encoding (emitted iff `Some`, including for
//!   `Some(0.0)` so sea-level distinct from absent altitude). Same
//!   defensive stance as `SampleAspectRatio`.
//! - **`LanguageId`** — encoded as the canonical BCP 47 tag, which is
//!   what `Display` writes and `LanguageId::new` reads back. Default
//!   is `"und"` (ISO 639-3 undetermined), the value a muxer writes
//!   when it looked and could not tell; the wire uses default-elision
//!   (an absent field decodes to `und`), and since `"und"` is not the
//!   empty string the encoder always writes it.
//!
//!   A wire string the door refuses coerces to `LanguageId::default()`
//!   (`und`) — `buffa::DecodeError` has no general "invalid value" arm,
//!   and the type's sentinel is the right fallback. That path is much
//!   NARROWER than it used to be: the door is WIDE IN, so an mkv's
//!   `ger` reaches `de`, a variant or private-use tail rides the
//!   fourth seat verbatim, and only a structurally impossible tag
//!   (`en-US-!!`, `日本語`) still falls back.

use core::num::NonZeroI64;
// `LanguageId`'s wire form is the canonical tag its `Display` writes, and the
// `ToString` that reaches it is not in the prelude on a `no_std` + `alloc`
// build — which the `buffa` feature is, since it implies `alloc` and not `std`.
use std::string::ToString;

use ::buffa::{
  DecodeContext, DecodeError, DefaultInstance, EncodeSink, Message, SizeCache,
  bytes::Buf,
  encoding::{Tag, WireType, encode_varint, skip_field_depth, varint_len},
  types::{
    FIXED32_ENCODED_LEN, bytes_encoded_len, decode_bytes, decode_double, decode_float,
    decode_int64, decode_string, decode_uint32, decode_uint64, encode_bytes, encode_double,
    encode_float, encode_int64, encode_string, encode_uint32, encode_uint64, int64_encoded_len,
    string_encoded_len, uint32_encoded_len, uint64_encoded_len,
  },
};
use smol_bytes::Utf8Bytes;

use crate::{
  audio::{
    BitRateMode, ChannelLayout, ChannelLayoutDescription, ChannelOrder, ChannelSpec,
    ContainerFormat, CoverArt, Fingerprint, Loudness, ReplayGain, SampleFormat, Tags,
  },
  capture::{Device, GeoLocation},
  color::{
    ChromaCoord, ChromaLocation, ContentLightLevel, DcpTargetGamut, DolbyVisionConfig,
    DynamicRange, HdrStaticMetadata, Info, MasteringDisplay, Matrix, Primaries, Transfer,
  },
  container::Format,
  disposition::TrackDisposition,
  frame::{
    DEN_ONE, Dimensions, FieldOrder, FrameRate, Rational, Rect, Rotation, SampleAspectRatio,
    StereoMode,
  },
  lang::LanguageId,
  pixel_format::PixelFormat,
};

const VARINT: u8 = WireType::Varint as u8;
const LEN: u8 = WireType::LengthDelimited as u8;

// The colour / frame / pixel-format vocabularies carry names, not numbers:
// `Other(Utf8Bytes)` is the escape, so the slug is the only spelling that
// survives a value this build has never heard of. They therefore ride the
// same one-field `{ string value = 1; }` shape as the codec family — see
// `impl_string_enum_message!` below. The declarations sit there, beside it.

// ----------------------------------------------------------------------------
// Dimensions — { uint32 width = 1; uint32 height = 2; }
// Default is (0, 0) == proto-zero, so zero-elision is sound.
// ----------------------------------------------------------------------------

impl DefaultInstance for Dimensions {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Dimensions> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Dimensions::default()))
  }
}

impl Message for Dimensions {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — seed is Dimensions::default() = (0, 0).
    if self.width() != 0 {
      size += 1 + uint32_encoded_len(self.width()) as u32;
    }
    if self.height() != 0 {
      size += 1 + uint32_encoded_len(self.height()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    // proto3 zero-elision: sound — see `compute_size`.
    if self.width() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.width(), buf);
    }
    if self.height() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.height(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let w = decode_uint32(buf)?;
        self.set_width(w);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let h = decode_uint32(buf)?;
        self.set_height(h);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Dimensions::default();
  }
}

// ----------------------------------------------------------------------------
// Rect — { uint32 x = 1; uint32 y = 2; uint32 width = 3; uint32 height = 4; }
// Default is all-zero == proto-zero, so zero-elision is sound.
// ----------------------------------------------------------------------------

impl DefaultInstance for Rect {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Rect> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Rect::default()))
  }
}

impl Message for Rect {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — seed is Rect::default() = all-zero.
    if self.x() != 0 {
      size += 1 + uint32_encoded_len(self.x()) as u32;
    }
    if self.y() != 0 {
      size += 1 + uint32_encoded_len(self.y()) as u32;
    }
    if self.width() != 0 {
      size += 1 + uint32_encoded_len(self.width()) as u32;
    }
    if self.height() != 0 {
      size += 1 + uint32_encoded_len(self.height()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    // proto3 zero-elision: sound — see `compute_size`.
    if self.x() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.x(), buf);
    }
    if self.y() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.y(), buf);
    }
    if self.width() != 0 {
      Tag::new(3, WireType::Varint).encode(buf);
      encode_uint32(self.width(), buf);
    }
    if self.height() != 0 {
      Tag::new(4, WireType::Varint).encode(buf);
      encode_uint32(self.height(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_x(v);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_y(v);
      }
      3 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 3,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_width(v);
      }
      4 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 4,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_height(v);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Rect::default();
  }
}

// ----------------------------------------------------------------------------
// SampleAspectRatio — { int64 num = 1; int64 den = 2; }
//
// `num`/`den` are encoded UNCONDITIONALLY — no proto3 zero elision.
// The decoder seeds from `SampleAspectRatio::default()` (1:1), NOT
// proto-zero. Eliding `num == 0` would decode back as `num == 1`;
// `den` is `NonZeroI64` and can never legitimately be 0. (Exactly
// the reasoning `mediatime::Timebase`'s own impl followed through 0.4.)
// Both tags are single-byte.
//
// The fields were `uint32` before `Rational` became signed and 64-bit.
// Protobuf's `int64` and `uint32` are the same plain (non-ZigZag)
// varint over the values a `SampleAspectRatio` can hold — non-negative
// and, for anything a `uint32` peer wrote, at most `u32::MAX` — so the
// bytes are unchanged in both directions and previously-encoded
// payloads still decode. `sint64` would have been the silent break:
// ZigZag re-encodes every value. Note the widening is one-way at the
// edges: a value above `u32::MAX` is writable now and a `uint32`
// reader would truncate it.
// ----------------------------------------------------------------------------

impl DefaultInstance for SampleAspectRatio {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<SampleAspectRatio> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(SampleAspectRatio::default()))
  }
}

impl Message for SampleAspectRatio {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    2 + int64_encoded_len(self.num()) as u32 + int64_encoded_len(self.den().get()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::Varint).encode(buf);
    encode_int64(self.num(), buf);
    Tag::new(2, WireType::Varint).encode(buf);
    encode_int64(self.den().get(), buf);
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        // `Rational::new` panics on a negative numerator, so decode
        // must not hand it one. Our own encoder never writes a
        // negative; a peer can, either directly or by writing a
        // `uint64` above `i64::MAX` that `decode_int64` reinterprets
        // into the negative half. Clamp to the smallest legal
        // numerator so decode stays total, as `den` does.
        let num = decode_int64(buf)?.max(0);
        self.set_num(num);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        // `den` is NonZeroI64; a malformed den — zero, or negative by
        // the same route as `num` above — is clamped to 1. Since
        // `NonZeroI64::MIN` is `i64::MIN`, the clamp target is spelled
        // out as `DEN_ONE`. This was the decode policy of
        // `mediatime::Timebase`'s own impl through `mediatime` 0.4,
        // which SAR mirrors, and it upholds the codec family's
        // total-scalar-decode invariant (scalar values never raise
        // decode errors; only structural errors do). Codex
        // adversarial-review F6 resolved it as a coordinated
        // mediatime/buffa policy; `mediatime` 0.5 moved its own mapping
        // to `mediatime::wire`, which keeps what it reads and refuses a
        // malformed value by name at the conversion, so the clamp is
        // this crate's own policy now.
        let den = NonZeroI64::new(decode_int64(buf)?)
          .filter(|d| d.get() > 0)
          .unwrap_or(DEN_ONE);
        self.set_den(den);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = SampleAspectRatio::default();
  }
}

// ----------------------------------------------------------------------------
// Rational — { int64 num = 1; int64 den = 2; }
//
// Same shape and reasoning as `SampleAspectRatio`, including the
// `uint32` → `int64` widening being byte-compatible in both
// directions: `num`/`den` are encoded UNCONDITIONALLY (no proto3
// zero-elision). The decoder seeds from `Rational::default()` (1/1),
// NOT proto-zero; eliding `num == 0` would decode back as `num == 1`.
// `den` is `NonZeroI64` and can never legitimately be 0; a malformed
// wire `den` — zero or negative — is clamped to 1 to keep decode
// total, as is a negative `num`. Both tags are single-byte.
// ----------------------------------------------------------------------------

impl DefaultInstance for Rational {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Rational> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Rational::default()))
  }
}

impl Message for Rational {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    2 + int64_encoded_len(self.num()) as u32 + int64_encoded_len(self.den().get()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::Varint).encode(buf);
    encode_int64(self.num(), buf);
    Tag::new(2, WireType::Varint).encode(buf);
    encode_int64(self.den().get(), buf);
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        // A negative numerator would trip `Rational::new`'s assert;
        // clamp as `SampleAspectRatio` does to keep decode total.
        let num = decode_int64(buf)?.max(0);
        self.set_num(num);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        // `den` is NonZeroI64; a malformed 0 or negative on the wire
        // (never produced by our own encoder) is clamped to 1 —
        // identical to `SampleAspectRatio`'s decode, upholding the
        // codec family's total-scalar-decode invariant.
        let den = NonZeroI64::new(decode_int64(buf)?)
          .filter(|d| d.get() > 0)
          .unwrap_or(DEN_ONE);
        self.set_den(den);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Rational::default();
  }
}

// ----------------------------------------------------------------------------
// FrameRate — { Rational rate = 1; bool is_vfr = 2; }
//
// `rate` is an always-encoded length-delimited `Rational`
// sub-message: its inner `Default` is `1/1` ≠ proto-zero, so the
// nested-message-always-encoded stance `mediatime::Timebase` took
// through 0.4 applies (like `MasteringDisplay`'s coords) — presence is
// unambiguous and `decode(encode(x)) == x` holds regardless of the inner
// ratio.
// `is_vfr` defaults to `false` == proto-zero, so it uses sound proto3
// zero-elision (only `true` is written).
// ----------------------------------------------------------------------------

impl DefaultInstance for FrameRate {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<FrameRate> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(FrameRate::default()))
  }
}

impl Message for FrameRate {
  fn compute_size(&self, cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // rate (field 1) — always-encoded nested message.
    {
      let slot = cache.reserve();
      let inner = self.rate().compute_size(cache);
      cache.set(slot, inner);
      size += 1 + varint_len(inner as u64) as u32 + inner;
    }
    // proto3 zero-elision: sound — seed `is_vfr` is `false`.
    if self.is_vfr() {
      size += 1 + 1; // tag + single-byte bool varint
    }
    size
  }

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_varint(cache.consume_next() as u64, buf);
    self.rate().write_to(cache, buf);
    // proto3 zero-elision: sound — see `compute_size`.
    if self.is_vfr() {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_varint(1, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let mut rate = self.rate();
        buffa::Message::merge_length_delimited(&mut rate, buf, ctx)?;
        self.set_rate(rate);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.update_is_vfr(decode_uint32(buf)? != 0);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = FrameRate::default();
  }
}

// ----------------------------------------------------------------------------
// DolbyVisionConfig — { uint32 profile = 1; uint32 level = 2;
//                       bool rpu_present = 3; bool el_present = 4;
//                       uint32 bl_signal_compat_id = 5; }
//
// `Default` is all-zero == proto-zero for every field, so proto3
// zero-elision is sound throughout. `u8` fields widen to the `uint32`
// wire scalar; bools are 0/1 varints.
// ----------------------------------------------------------------------------

impl DefaultInstance for DolbyVisionConfig {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<DolbyVisionConfig> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(DolbyVisionConfig::default()))
  }
}

impl Message for DolbyVisionConfig {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — seed is all-zero default.
    if self.profile() != 0 {
      size += 1 + uint32_encoded_len(self.profile() as u32) as u32;
    }
    if self.level() != 0 {
      size += 1 + uint32_encoded_len(self.level() as u32) as u32;
    }
    if self.rpu_present() {
      size += 1 + 1;
    }
    if self.el_present() {
      size += 1 + 1;
    }
    if self.bl_signal_compat_id() != 0 {
      size += 1 + uint32_encoded_len(self.bl_signal_compat_id() as u32) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    // proto3 zero-elision: sound — see `compute_size`.
    if self.profile() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.profile() as u32, buf);
    }
    if self.level() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.level() as u32, buf);
    }
    if self.rpu_present() {
      Tag::new(3, WireType::Varint).encode(buf);
      encode_varint(1, buf);
    }
    if self.el_present() {
      Tag::new(4, WireType::Varint).encode(buf);
      encode_varint(1, buf);
    }
    if self.bl_signal_compat_id() != 0 {
      Tag::new(5, WireType::Varint).encode(buf);
      encode_uint32(self.bl_signal_compat_id() as u32, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.set_profile(decode_uint32(buf)? as u8);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.set_level(decode_uint32(buf)? as u8);
      }
      3 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 3,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.update_rpu_present(decode_uint32(buf)? != 0);
      }
      4 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 4,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.update_el_present(decode_uint32(buf)? != 0);
      }
      5 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 5,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.set_bl_signal_compat_id(decode_uint32(buf)? as u8);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = DolbyVisionConfig::default();
  }
}

// ----------------------------------------------------------------------------
// Info — five enum slugs, each a bare `string`, ALL always encoded.
// See the module doc: always-encoding (esp. `matrix`, whose semantic
// default is `Bt709`) decouples the wire round-trip from the field's own
// default — the always-encode-nontrivial-default stance `mediatime`
// took through 0.4.
// Tags #1–#5 single-byte. The slug is the spelling because the member
// enums' only escape is `Other(Utf8Bytes)`.
// ----------------------------------------------------------------------------

impl DefaultInstance for Info {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Info> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Info::UNSPECIFIED))
  }
}

impl Message for Info {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    // All five are unconditionally encoded (presence-independent).
    5 + string_encoded_len(self.primaries().as_str()) as u32
      + string_encoded_len(self.transfer().as_str()) as u32
      + string_encoded_len(self.matrix().as_str()) as u32
      + string_encoded_len(self.range().as_str()) as u32
      + string_encoded_len(self.chroma_location().as_str()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_string(self.primaries().as_str(), buf);
    Tag::new(2, WireType::LengthDelimited).encode(buf);
    encode_string(self.transfer().as_str(), buf);
    Tag::new(3, WireType::LengthDelimited).encode(buf);
    encode_string(self.matrix().as_str(), buf);
    Tag::new(4, WireType::LengthDelimited).encode(buf);
    encode_string(self.range().as_str(), buf);
    Tag::new(5, WireType::LengthDelimited).encode(buf);
    encode_string(self.chroma_location().as_str(), buf);
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_primaries(s.parse().unwrap_or_else(|_| unreachable!()));
      }
      2 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_transfer(s.parse().unwrap_or_else(|_| unreachable!()));
      }
      3 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 3,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_matrix(s.parse().unwrap_or_else(|_| unreachable!()));
      }
      4 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 4,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_range(s.parse().unwrap_or_else(|_| unreachable!()));
      }
      5 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 5,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_chroma_location(s.parse().unwrap_or_else(|_| unreachable!()));
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Info::UNSPECIFIED;
  }
}

// ----------------------------------------------------------------------------
// ContentLightLevel — { uint32 max_cll = 1; uint32 max_fall = 2; }
// Default is (0, 0) == proto-zero, so zero-elision is sound.
// ----------------------------------------------------------------------------

impl DefaultInstance for ContentLightLevel {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ContentLightLevel> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ContentLightLevel::default()))
  }
}

impl Message for ContentLightLevel {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — seed is ContentLightLevel::default() = (0, 0).
    if self.max_cll() != 0 {
      size += 1 + uint32_encoded_len(self.max_cll()) as u32;
    }
    if self.max_fall() != 0 {
      size += 1 + uint32_encoded_len(self.max_fall()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    // proto3 zero-elision: sound — see `compute_size`.
    if self.max_cll() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.max_cll(), buf);
    }
    if self.max_fall() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.max_fall(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_max_cll(v);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_max_fall(v);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ContentLightLevel::default();
  }
}

// ----------------------------------------------------------------------------
// ChromaCoord — { uint32 x = 1; uint32 y = 2; }
// `x`/`y` are `u32` storage == the wire scalar; every value (incl.
// out-of-range / future / corrupt) round-trips losslessly — no
// saturation (Codex adversarial-review F3).
// Default is (0, 0) == proto-zero, so zero-elision is sound.
// ----------------------------------------------------------------------------

impl DefaultInstance for ChromaCoord {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ChromaCoord> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ChromaCoord::default()))
  }
}

impl Message for ChromaCoord {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — seed is ChromaCoord::default() = (0, 0).
    if self.x() != 0 {
      size += 1 + uint32_encoded_len(self.x()) as u32;
    }
    if self.y() != 0 {
      size += 1 + uint32_encoded_len(self.y()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    // proto3 zero-elision: sound — see `compute_size`.
    if self.x() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.x(), buf);
    }
    if self.y() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.y(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        // u32 storage == wire scalar: preserved verbatim, no
        // saturation (Codex F3).
        self.set_x(decode_uint32(buf)?);
      }
      2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.set_y(decode_uint32(buf)?);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ChromaCoord::default();
  }
}

// ----------------------------------------------------------------------------
// MasteringDisplay — { ChromaCoord primary_r = 1; primary_g = 2;
//                      primary_b = 3; white_point = 4;
//                      uint32 max_luminance = 5; uint32 min_luminance = 6; }
//
// The four nested ChromaCoords are ALWAYS encoded (length-delimited)
// so presence is unambiguous and round-trip holds regardless of
// content (the always-encoded-nested-message stance `mediatime` took
// through 0.4).
// The two luminance scalars default to 0 == proto-zero so they use
// proto3 zero-elision.
// ----------------------------------------------------------------------------

impl DefaultInstance for MasteringDisplay {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<MasteringDisplay> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(MasteringDisplay::default()))
  }
}

impl Message for MasteringDisplay {
  fn compute_size(&self, cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    let primaries = self.display_primaries();
    // primary_r / g / b (fields 1..=3) — always encoded.
    for cc in &primaries {
      let slot = cache.reserve();
      let inner = cc.compute_size(cache);
      cache.set(slot, inner);
      size += 1 + varint_len(inner as u64) as u32 + inner;
    }
    // white_point (field 4) — always encoded.
    {
      let slot = cache.reserve();
      let inner = self.white_point().compute_size(cache);
      cache.set(slot, inner);
      size += 1 + varint_len(inner as u64) as u32 + inner;
    }
    // proto3 zero-elision: sound — seed is MasteringDisplay::default(),
    // whose luminances are 0.
    if self.max_luminance() != 0 {
      size += 1 + uint32_encoded_len(self.max_luminance()) as u32;
    }
    if self.min_luminance() != 0 {
      size += 1 + uint32_encoded_len(self.min_luminance()) as u32;
    }
    size
  }

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    let primaries = self.display_primaries();
    for (i, cc) in primaries.iter().enumerate() {
      Tag::new(1 + i as u32, WireType::LengthDelimited).encode(buf);
      encode_varint(cache.consume_next() as u64, buf);
      cc.write_to(cache, buf);
    }
    Tag::new(4, WireType::LengthDelimited).encode(buf);
    encode_varint(cache.consume_next() as u64, buf);
    self.white_point().write_to(cache, buf);
    // proto3 zero-elision: sound — see `compute_size`.
    if self.max_luminance() != 0 {
      Tag::new(5, WireType::Varint).encode(buf);
      encode_uint32(self.max_luminance(), buf);
    }
    if self.min_luminance() != 0 {
      Tag::new(6, WireType::Varint).encode(buf);
      encode_uint32(self.min_luminance(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      n @ 1..=3 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let mut primaries = self.display_primaries();
        let mut cc = primaries[(n - 1) as usize];
        buffa::Message::merge_length_delimited(&mut cc, buf, ctx)?;
        primaries[(n - 1) as usize] = cc;
        self.set_display_primaries(primaries);
      }
      4 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 4,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let mut wp = self.white_point();
        buffa::Message::merge_length_delimited(&mut wp, buf, ctx)?;
        self.set_white_point(wp);
      }
      5 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 5,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_max_luminance(v);
      }
      6 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 6,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        self.set_min_luminance(v);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = MasteringDisplay::default();
  }
}

// ----------------------------------------------------------------------------
// HdrStaticMetadata — { MasteringDisplay mastering = 1;
//                       ContentLightLevel content_light = 2; }
//
// Both fields are `Option`: presence-encoded length-delimited
// sub-messages, omitted entirely when `None`. (A present-but-default
// inner message still round-trips because each inner type's own
// codec is round-trip-safe and presence is carried by the tag.)
// ----------------------------------------------------------------------------

impl DefaultInstance for HdrStaticMetadata {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<HdrStaticMetadata> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(HdrStaticMetadata::default()))
  }
}

impl Message for HdrStaticMetadata {
  fn compute_size(&self, cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    if let Some(md) = self.mastering() {
      let slot = cache.reserve();
      let inner = md.compute_size(cache);
      cache.set(slot, inner);
      size += 1 + varint_len(inner as u64) as u32 + inner;
    }
    if let Some(cll) = self.content_light() {
      let slot = cache.reserve();
      let inner = cll.compute_size(cache);
      cache.set(slot, inner);
      size += 1 + varint_len(inner as u64) as u32 + inner;
    }
    size
  }

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if let Some(md) = self.mastering() {
      Tag::new(1, WireType::LengthDelimited).encode(buf);
      encode_varint(cache.consume_next() as u64, buf);
      md.write_to(cache, buf);
    }
    if let Some(cll) = self.content_light() {
      Tag::new(2, WireType::LengthDelimited).encode(buf);
      encode_varint(cache.consume_next() as u64, buf);
      cll.write_to(cache, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let mut md = self.mastering().unwrap_or_default();
        buffa::Message::merge_length_delimited(&mut md, buf, ctx)?;
        self.set_mastering(Some(md));
      }
      2 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let mut cll = self.content_light().unwrap_or_default();
        buffa::Message::merge_length_delimited(&mut cll, buf, ctx)?;
        self.set_content_light(Some(cll));
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = HdrStaticMetadata::default();
  }
}

// ============================================================================
// Audio + container types — see the `## Audio + container types`
// sub-section of the module doc block at the top of this file for
// the full wire layout.
// ============================================================================

// ----------------------------------------------------------------------------
// String-bearing enum codec helper.
//
// One-field message `{ string value = 1; }` where `value` is the
// `as_str()` slug — the crate's one wire shape for a name vocabulary.
// Default-elision: written iff `*self != $default_expr`. For enums with
// a `Default` that is a named variant, that default elides and an absent
// field decodes back to it; for enums without one, the "default" is the
// wire-zero state (empty string → `Other("")`).
// ----------------------------------------------------------------------------

macro_rules! impl_string_enum_message {
  ($ty:ty, $default_expr:expr) => {
    impl DefaultInstance for $ty {
      fn default_instance() -> &'static Self {
        static VALUE: buffa::__private::OnceBox<$ty> = buffa::__private::OnceBox::new();
        VALUE.get_or_init(|| buffa::alloc::boxed::Box::new($default_expr))
      }
    }

    impl Message for $ty {
      fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
        // Default-elision: the decoder seeds from the same default, so an
        // absent field decodes back to it exactly. Every other value —
        // including the empty slug where that is not the default —
        // writes its name.
        if *self != $default_expr {
          1 + string_encoded_len(self.as_str()) as u32
        } else {
          0
        }
      }

      fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
        if *self != $default_expr {
          Tag::new(1, WireType::LengthDelimited).encode(buf);
          encode_string(self.as_str(), buf);
        }
      }

      fn merge_field(
        &mut self,
        tag: Tag,
        buf: &mut impl Buf,
        ctx: DecodeContext<'_>,
      ) -> Result<(), DecodeError> {
        match tag.field_number() {
          1 => {
            if tag.wire_type() != WireType::LengthDelimited {
              return Err(DecodeError::WireTypeMismatch {
                field_number: 1,
                expected: LEN,
                actual: tag.wire_type() as u8,
              });
            }
            let s = decode_string(buf)?;
            // Every vocabulary routed through this macro parses totally at
            // the `buffa` tier (`buffa` implies `alloc`, which is exactly
            // where each one's `Other` arm lives), so `FromStr::Err` is
            // `Infallible` and this binding is irrefutable. That is the
            // point of spelling it this way: add a type here whose parse
            // can refuse and this line stops compiling (`E0005`) instead
            // of silently acquiring an `unreachable!()` that is only
            // unreachable by argument.
            let Ok(parsed) = <$ty as core::str::FromStr>::from_str(&s);
            *self = parsed;
          }
          _ => skip_field_depth(tag, buf, ctx.depth())?,
        }
        Ok(())
      }

      fn clear(&mut self) {
        *self = $default_expr;
      }
    }
  };
}

// Closed-vocabulary string-bearing enums. They don't have a
// `Default` impl, so the decoder seed is the wire-zero `Other("")`
// (round-trips losslessly through the slug codec).
impl_string_enum_message!(ChannelLayout, ChannelLayout::Other(Utf8Bytes::new()));
impl_string_enum_message!(ContainerFormat, ContainerFormat::Other(Utf8Bytes::new()));
impl_string_enum_message!(Format, Format::Other(Utf8Bytes::new()));

// Name vocabularies with a real `Default`: the seed is that default, and
// every value writes its slug. `Unknown(u32)` is gone, so a number is no
// longer a spelling any of these has.
impl_string_enum_message!(Matrix, Matrix::default());
impl_string_enum_message!(Primaries, Primaries::default());
impl_string_enum_message!(Transfer, Transfer::default());
impl_string_enum_message!(DynamicRange, DynamicRange::default());
impl_string_enum_message!(ChromaLocation, ChromaLocation::default());
impl_string_enum_message!(DcpTargetGamut, DcpTargetGamut::default());
impl_string_enum_message!(Rotation, Rotation::default());
impl_string_enum_message!(FieldOrder, FieldOrder::default());
impl_string_enum_message!(StereoMode, StereoMode::default());
impl_string_enum_message!(PixelFormat, PixelFormat::default());
impl_string_enum_message!(SampleFormat, SampleFormat::default());

// ----------------------------------------------------------------------------
// BitRateMode — { uint32 value = 1; }
//
// `BitRateMode::default() == Cbr` whose `to_u32() == 0`, so proto3
// zero-elision is sound: an absent field decodes via
// `from_u32(0) == Cbr`.
// ----------------------------------------------------------------------------

impl DefaultInstance for BitRateMode {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<BitRateMode> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(BitRateMode::default()))
  }
}

impl Message for BitRateMode {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let v = self.to_u32();
    if v != 0 {
      1 + uint32_encoded_len(v) as u32
    } else {
      0
    }
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    let v = self.to_u32();
    if v != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(v, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        *self = BitRateMode::from_u32(decode_uint32(buf)?);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = BitRateMode::default();
  }
}

// ----------------------------------------------------------------------------
// ChannelOrder — { uint32 value = 1; }
//
// Same shape and same reasoning as `BitRateMode`:
// `ChannelOrder::default() == Unspecified` whose `to_u32() == 0`, so
// proto3 zero-elision is sound — an absent field decodes via
// `from_u32(0) == Unspecified`.
// ----------------------------------------------------------------------------

impl DefaultInstance for ChannelOrder {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ChannelOrder> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ChannelOrder::default()))
  }
}

impl Message for ChannelOrder {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let v = self.to_u32();
    if v != 0 {
      1 + uint32_encoded_len(v) as u32
    } else {
      0
    }
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    let v = self.to_u32();
    if v != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(v, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        *self = ChannelOrder::from_u32(decode_uint32(buf)?);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ChannelOrder::default();
  }
}

// ----------------------------------------------------------------------------
// ChannelSpec — { uint32 index = 1; uint32 raw_id = 2; string label = 3; }
//
// Default is `(0, 0, "")`, which is proto-zero for all three, so proto3
// zero-elision is sound throughout.
// ----------------------------------------------------------------------------

impl DefaultInstance for ChannelSpec {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ChannelSpec> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ChannelSpec::default()))
  }
}

impl Message for ChannelSpec {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    if self.index() != 0 {
      size += 1 + uint32_encoded_len(self.index()) as u32;
    }
    if self.raw_id() != 0 {
      size += 1 + uint32_encoded_len(self.raw_id()) as u32;
    }
    if !self.label().is_empty() {
      size += 1 + string_encoded_len(self.label()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if self.index() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.index(), buf);
    }
    if self.raw_id() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.raw_id(), buf);
    }
    if !self.label().is_empty() {
      Tag::new(3, WireType::LengthDelimited).encode(buf);
      encode_string(self.label(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      n @ 1..=2 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        match n {
          1 => {
            self.set_index(v);
          }
          2 => {
            self.set_raw_id(v);
          }
          _ => unreachable!(),
        }
      }
      3 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 3,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_label(Utf8Bytes::from(s));
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ChannelSpec::default();
  }
}

// ----------------------------------------------------------------------------
// ChannelLayoutDescription —
//   { uint32 order = 1; uint32 channels = 2; string known_kind = 3;
//     optional uint64 native_mask = 4;
//     repeated ChannelSpec custom_channels = 5; string text = 6; }
//
// Fields 1/2/3/6 use proto3 zero-elision: the seed is the type's
// `Default`, whose order is code 0, channel count 0, name
// `ChannelLayout::default()` (the `Other("")` sentinel, rendering `""`)
// and text empty — proto-zero in each case.
//
// `native_mask` is `optional`: emitted iff `Some`, including for
// `Some(0)`, so a reported all-zero mask stays distinct from no mask at
// all (the `ReplayGain` album-scalar stance).
//
// `custom_channels` is the crate's first repeated field — one
// length-delimited sub-message per element, in order. The decoder
// appends as elements arrive; reading the list out and writing it back
// per element would make decode quadratic in a length an untrusted peer
// chooses.
// ----------------------------------------------------------------------------

impl DefaultInstance for ChannelLayoutDescription {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ChannelLayoutDescription> =
      buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ChannelLayoutDescription::default()))
  }
}

impl Message for ChannelLayoutDescription {
  fn compute_size(&self, cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    if self.order().to_u32() != 0 {
      size += 1 + uint32_encoded_len(self.order().to_u32()) as u32;
    }
    if self.channels() != 0 {
      size += 1 + uint32_encoded_len(self.channels()) as u32;
    }
    if !self.known_kind().as_str().is_empty() {
      size += 1 + string_encoded_len(self.known_kind().as_str()) as u32;
    }
    if let Some(mask) = self.native_mask() {
      size += 1 + uint64_encoded_len(mask) as u32;
    }
    for spec in self.custom_channels() {
      let slot = cache.reserve();
      let inner = spec.compute_size(cache);
      cache.set(slot, inner);
      size += 1 + varint_len(inner as u64) as u32 + inner;
    }
    if !self.text().is_empty() {
      size += 1 + string_encoded_len(self.text()) as u32;
    }
    size
  }

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if self.order().to_u32() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.order().to_u32(), buf);
    }
    if self.channels() != 0 {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_uint32(self.channels(), buf);
    }
    if !self.known_kind().as_str().is_empty() {
      Tag::new(3, WireType::LengthDelimited).encode(buf);
      encode_string(self.known_kind().as_str(), buf);
    }
    if let Some(mask) = self.native_mask() {
      Tag::new(4, WireType::Varint).encode(buf);
      encode_uint64(mask, buf);
    }
    for spec in self.custom_channels() {
      Tag::new(5, WireType::LengthDelimited).encode(buf);
      encode_varint(cache.consume_next() as u64, buf);
      spec.write_to(cache, buf);
    }
    if !self.text().is_empty() {
      Tag::new(6, WireType::LengthDelimited).encode(buf);
      encode_string(self.text(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      n @ (1 | 2 | 4) => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        match n {
          1 => {
            self.set_order(ChannelOrder::from_u32(decode_uint32(buf)?));
          }
          2 => {
            self.set_channels(decode_uint32(buf)?);
          }
          4 => {
            self.set_native_mask(Some(decode_uint64(buf)?));
          }
          _ => unreachable!(),
        }
      }
      n @ (3 | 5 | 6) => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        match n {
          3 => {
            let s = decode_string(buf)?;
            // `ChannelLayout`'s parse is total at this tier (`buffa`
            // implies `alloc`, which is where its `Other` arm lives), so
            // this binding is irrefutable. Spelled this way on purpose:
            // a future refusal stops it compiling (`E0005`) instead of
            // silently acquiring an `unreachable!()`.
            let Ok(parsed) = <ChannelLayout as core::str::FromStr>::from_str(&s);
            self.set_known_kind(parsed);
          }
          5 => {
            let mut spec = ChannelSpec::default();
            buffa::Message::merge_length_delimited(&mut spec, buf, ctx)?;
            self.push_custom_channel(spec);
          }
          6 => {
            let s = decode_string(buf)?;
            self.set_text(Utf8Bytes::from(s));
          }
          _ => unreachable!(),
        }
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ChannelLayoutDescription::default();
  }
}

// ----------------------------------------------------------------------------
// Loudness — four `float` fields (Fixed32 wire). Default is
// all-zero, which is proto-zero for `f32`, so proto3 zero-elision is
// sound throughout.
// ----------------------------------------------------------------------------

const FIXED32: u8 = WireType::Fixed32 as u8;

impl DefaultInstance for Loudness {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Loudness> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Loudness::default()))
  }
}

impl Message for Loudness {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — every field defaults to 0.0
    // (proto-zero for `f32`).
    if self.integrated_lufs() != 0.0 {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    if self.range_lu() != 0.0 {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    if self.true_peak_dbtp() != 0.0 {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    if self.sample_peak_dbfs() != 0.0 {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if self.integrated_lufs() != 0.0 {
      Tag::new(1, WireType::Fixed32).encode(buf);
      encode_float(self.integrated_lufs(), buf);
    }
    if self.range_lu() != 0.0 {
      Tag::new(2, WireType::Fixed32).encode(buf);
      encode_float(self.range_lu(), buf);
    }
    if self.true_peak_dbtp() != 0.0 {
      Tag::new(3, WireType::Fixed32).encode(buf);
      encode_float(self.true_peak_dbtp(), buf);
    }
    if self.sample_peak_dbfs() != 0.0 {
      Tag::new(4, WireType::Fixed32).encode(buf);
      encode_float(self.sample_peak_dbfs(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      n @ 1..=4 => {
        if tag.wire_type() != WireType::Fixed32 {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: FIXED32,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_float(buf)?;
        match n {
          1 => {
            self.set_integrated_lufs(v);
          }
          2 => {
            self.set_range_lu(v);
          }
          3 => {
            self.set_true_peak_dbtp(v);
          }
          4 => {
            self.set_sample_peak_dbfs(v);
          }
          _ => unreachable!(),
        }
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Loudness::default();
  }
}

// ----------------------------------------------------------------------------
// ReplayGain — `track_gain_db` / `track_peak` are `float` with proto3
// zero-elision (Default is all-zero == proto-zero for f32). The two
// album-level scalars are `optional float` so a distribution-absent
// album-level number round-trips as `None` (wire field absent rather
// than zero). All four are wire-type `Fixed32` (4 bytes LE).
// ----------------------------------------------------------------------------

impl DefaultInstance for ReplayGain {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ReplayGain> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ReplayGain::default()))
  }
}

impl Message for ReplayGain {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision on the two `float` fields.
    if self.track_gain_db() != 0.0 {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    if self.track_peak() != 0.0 {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    // `optional float` — present iff `Some` (independent of value).
    if self.album_gain_db().is_some() {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    if self.album_peak().is_some() {
      size += 1 + FIXED32_ENCODED_LEN as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if self.track_gain_db() != 0.0 {
      Tag::new(1, WireType::Fixed32).encode(buf);
      encode_float(self.track_gain_db(), buf);
    }
    if self.track_peak() != 0.0 {
      Tag::new(2, WireType::Fixed32).encode(buf);
      encode_float(self.track_peak(), buf);
    }
    if let Some(v) = self.album_gain_db() {
      Tag::new(3, WireType::Fixed32).encode(buf);
      encode_float(v, buf);
    }
    if let Some(v) = self.album_peak() {
      Tag::new(4, WireType::Fixed32).encode(buf);
      encode_float(v, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      n @ 1..=4 => {
        if tag.wire_type() != WireType::Fixed32 {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: FIXED32,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_float(buf)?;
        match n {
          1 => {
            self.set_track_gain_db(v);
          }
          2 => {
            self.set_track_peak(v);
          }
          3 => {
            self.set_album_gain_db(Some(v));
          }
          4 => {
            self.set_album_peak(Some(v));
          }
          _ => unreachable!(),
        }
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ReplayGain::default();
  }
}

// ----------------------------------------------------------------------------
// Fingerprint — { string algorithm = 1; bytes value = 2; }
//
// `try_new` rejects empty `algorithm`, so the type has no
// natural-zero `Default`. The decoder seed is a synthetic
// `Fingerprint { algorithm: "default", value: [] }` (the
// always-encoded `algorithm` overwrites it on decode). `algorithm`
// is encoded UNCONDITIONALLY; `value` (bytes) uses proto3
// zero-elision (empty fingerprint is a legal value).
// ----------------------------------------------------------------------------

fn audio_fingerprint_seed() -> Fingerprint {
  // Safety: the literal is non-empty so `try_new` cannot fail.
  Fingerprint::try_new(Utf8Bytes::from_static("default"), std::vec::Vec::new())
    .unwrap_or_else(|_| unreachable!())
}

impl DefaultInstance for Fingerprint {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Fingerprint> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(audio_fingerprint_seed()))
  }
}

impl Message for Fingerprint {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 1 + string_encoded_len(self.algorithm()) as u32;
    if !self.value().is_empty() {
      size += 1 + bytes_encoded_len(self.value()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_string(self.algorithm(), buf);
    if !self.value().is_empty() {
      Tag::new(2, WireType::LengthDelimited).encode(buf);
      encode_bytes(self.value(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let algo = decode_string(buf)?;
        // Empty algorithm on the wire is malformed (the type
        // invariant forbids it); clamp to the seed's `"default"`
        // sentinel to keep decode total.
        let algo = if algo.is_empty() {
          Utf8Bytes::from_static("default")
        } else {
          Utf8Bytes::from(algo)
        };
        // Preserve existing `value`, swap `algorithm`. `try_new`
        // moves the bytes back in unchanged.
        let value = self.value().to_vec();
        *self = Fingerprint::try_new(algo, value).unwrap_or_else(|_| audio_fingerprint_seed());
      }
      2 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let bytes = decode_bytes(buf)?;
        // Preserve `algorithm`, replace `value`.
        let algo = Utf8Bytes::from(self.algorithm());
        *self = Fingerprint::try_new(algo, bytes).unwrap_or_else(|_| audio_fingerprint_seed());
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = audio_fingerprint_seed();
  }
}

// ----------------------------------------------------------------------------
// CoverArt — { string mime = 1; bytes data = 2; }
//
// `try_new` rejects empty mime / empty data, so the type has no
// natural-zero `Default`. Decoder seed is a synthetic
// `CoverArt { mime: "application/octet-stream", data: [0u8] }`
// (sentinel that gets overwritten on decode; both fields are
// ALWAYS encoded on the write path).
// ----------------------------------------------------------------------------

fn audio_cover_art_seed() -> CoverArt {
  CoverArt::try_new(
    Utf8Bytes::from_static("application/octet-stream"),
    std::vec![0u8],
  )
  .unwrap_or_else(|_| unreachable!())
}

impl DefaultInstance for CoverArt {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<CoverArt> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(audio_cover_art_seed()))
  }
}

impl Message for CoverArt {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    2 + string_encoded_len(self.mime()) as u32 + bytes_encoded_len(self.data()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_string(self.mime(), buf);
    Tag::new(2, WireType::LengthDelimited).encode(buf);
    encode_bytes(self.data(), buf);
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let mime = decode_string(buf)?;
        // Empty mime on the wire violates the invariant; clamp to
        // the sentinel to keep decode total.
        let mime = if mime.is_empty() {
          Utf8Bytes::from_static("application/octet-stream")
        } else {
          Utf8Bytes::from(mime)
        };
        let data = self.data().to_vec();
        let data = if data.is_empty() {
          std::vec![0u8]
        } else {
          data
        };
        *self = CoverArt::try_new(mime, data).unwrap_or_else(|_| audio_cover_art_seed());
      }
      2 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let data = decode_bytes(buf)?;
        // Empty data on the wire violates the invariant; clamp to
        // the single-byte sentinel.
        let data = if data.is_empty() {
          std::vec![0u8]
        } else {
          data
        };
        let mime = Utf8Bytes::from(self.mime());
        *self = CoverArt::try_new(mime, data).unwrap_or_else(|_| audio_cover_art_seed());
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = audio_cover_art_seed();
  }
}

// ----------------------------------------------------------------------------
// Tags — every string field uses proto3 zero-elision ("" ==
// "absent" by the type's own convention); every numeric Option<u16>
// is widened to uint32 and uses proto3 zero-elision. `Some(0)`
// (theoretically legal) and `None` (absent) round-trip identically
// to `None` — documented limitation.
// ----------------------------------------------------------------------------

impl DefaultInstance for Tags {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Tags> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Tags::default()))
  }
}

impl Message for Tags {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    if !self.title().is_empty() {
      size += 1 + string_encoded_len(self.title()) as u32;
    }
    if !self.artist().is_empty() {
      size += 1 + string_encoded_len(self.artist()) as u32;
    }
    if !self.album_artist().is_empty() {
      size += 1 + string_encoded_len(self.album_artist()) as u32;
    }
    if !self.album().is_empty() {
      size += 1 + string_encoded_len(self.album()) as u32;
    }
    if !self.composer().is_empty() {
      size += 1 + string_encoded_len(self.composer()) as u32;
    }
    if !self.genre().is_empty() {
      size += 1 + string_encoded_len(self.genre()) as u32;
    }
    if !self.comment().is_empty() {
      size += 1 + string_encoded_len(self.comment()) as u32;
    }
    // Numeric fields are bare `u16` with `0` = absent — proto3 zero-elision
    // applies directly (no `Option` to unwrap).
    if self.year() != 0 {
      size += 1 + uint32_encoded_len(self.year() as u32) as u32;
    }
    if self.track_number() != 0 {
      size += 1 + uint32_encoded_len(self.track_number() as u32) as u32;
    }
    if self.track_total() != 0 {
      size += 1 + uint32_encoded_len(self.track_total() as u32) as u32;
    }
    if self.disc_number() != 0 {
      size += 1 + uint32_encoded_len(self.disc_number() as u32) as u32;
    }
    if self.disc_total() != 0 {
      size += 1 + uint32_encoded_len(self.disc_total() as u32) as u32;
    }
    if let Some(lang) = self.language() {
      size += 1 + string_encoded_len(&lang.to_string()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if !self.title().is_empty() {
      Tag::new(1, WireType::LengthDelimited).encode(buf);
      encode_string(self.title(), buf);
    }
    if !self.artist().is_empty() {
      Tag::new(2, WireType::LengthDelimited).encode(buf);
      encode_string(self.artist(), buf);
    }
    if !self.album_artist().is_empty() {
      Tag::new(3, WireType::LengthDelimited).encode(buf);
      encode_string(self.album_artist(), buf);
    }
    if !self.album().is_empty() {
      Tag::new(4, WireType::LengthDelimited).encode(buf);
      encode_string(self.album(), buf);
    }
    if !self.composer().is_empty() {
      Tag::new(5, WireType::LengthDelimited).encode(buf);
      encode_string(self.composer(), buf);
    }
    if !self.genre().is_empty() {
      Tag::new(6, WireType::LengthDelimited).encode(buf);
      encode_string(self.genre(), buf);
    }
    if !self.comment().is_empty() {
      Tag::new(7, WireType::LengthDelimited).encode(buf);
      encode_string(self.comment(), buf);
    }
    if self.year() != 0 {
      Tag::new(8, WireType::Varint).encode(buf);
      encode_uint32(self.year() as u32, buf);
    }
    if self.track_number() != 0 {
      Tag::new(9, WireType::Varint).encode(buf);
      encode_uint32(self.track_number() as u32, buf);
    }
    if self.track_total() != 0 {
      Tag::new(10, WireType::Varint).encode(buf);
      encode_uint32(self.track_total() as u32, buf);
    }
    if self.disc_number() != 0 {
      Tag::new(11, WireType::Varint).encode(buf);
      encode_uint32(self.disc_number() as u32, buf);
    }
    if self.disc_total() != 0 {
      Tag::new(12, WireType::Varint).encode(buf);
      encode_uint32(self.disc_total() as u32, buf);
    }
    if let Some(lang) = self.language() {
      Tag::new(13, WireType::LengthDelimited).encode(buf);
      encode_string(&lang.to_string(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    let n = tag.field_number();
    match n {
      1..=7 | 13 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        let s = Utf8Bytes::from(s);
        match n {
          1 => {
            self.set_title(s);
          }
          2 => {
            self.set_artist(s);
          }
          3 => {
            self.set_album_artist(s);
          }
          4 => {
            self.set_album(s);
          }
          5 => {
            self.set_composer(s);
          }
          6 => {
            self.set_genre(s);
          }
          7 => {
            self.set_comment(s);
          }
          13 => {
            // An empty field-13 string means "no language tag" (`None`);
            // a non-empty value goes through the whole-tag door, coercing a
            // tag it refuses to `LanguageId::default()` (`und`) — the same
            // lenient semantics the standalone `LanguageId` codec uses
            // (`buffa::DecodeError` has no general "invalid value" arm).
            self.update_language(if s.is_empty() {
              None
            } else {
              Some(LanguageId::new(&s).unwrap_or_default())
            });
          }
          _ => unreachable!(),
        }
      }
      8..=12 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: n,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        // Numeric fields are bare `u16` with `0` = absent — a decoded `0`
        // (or an elided, never-written field) is simply `0`.
        let v = decode_uint32(buf)? as u16;
        match n {
          8 => {
            self.set_year(v);
          }
          9 => {
            self.set_track_number(v);
          }
          10 => {
            self.set_track_total(v);
          }
          11 => {
            self.set_disc_number(v);
          }
          12 => {
            self.set_disc_total(v);
          }
          _ => unreachable!(),
        }
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Tags::default();
  }
}

// ----------------------------------------------------------------------------
// TrackDisposition — { uint32 bits = 1; }
// Default is the empty flag set (`bits() == 0`), so proto3 zero-elision is
// sound: an absent field decodes back to `TrackDisposition::empty()`. The
// `from_u32` (= `from_bits_retain`) decoder preserves every bit, so unknown
// bits introduced in a future FFmpeg release round-trip losslessly.
// ----------------------------------------------------------------------------

impl DefaultInstance for TrackDisposition {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<TrackDisposition> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(TrackDisposition::default()))
  }
}

impl Message for TrackDisposition {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    // proto3 zero-elision: sound — default is the empty flag set
    // (`bits() == 0`).
    if self.to_u32() != 0 {
      1 + uint32_encoded_len(self.to_u32()) as u32
    } else {
      0
    }
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    if self.to_u32() != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(self.to_u32(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_uint32(buf)?;
        *self = TrackDisposition::from_u32(v);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = TrackDisposition::default();
  }
}

// ----------------------------------------------------------------------------
// TrackOrigin + Format live in `crate::subtitle`, which is
// `cfg`-gated on the `alloc` feature (the `Other(Utf8Bytes)` escape on
// `Format`). Mirror that gate on the wire impls so a
// `--no-default-features --features buffa` build (no `alloc`) still compiles.
// ----------------------------------------------------------------------------

#[cfg(any(feature = "std", feature = "alloc"))]
mod subtitle_impls {
  use super::*;
  use ::buffa::types::{decode_string, encode_string, string_encoded_len};
  use core::str::FromStr;

  use crate::subtitle::{Format, TrackOrigin};

  // ----------------------------------------------------------------------------
  // TrackOrigin — { string value = 1; }
  // Open since 0.5.0 (`Other(Utf8Bytes)`), so there is no total numeric id;
  // encodes the slug from `as_str()`, exactly like `Format` below.
  // Always-encoded (NOT default-elision): the empty string decodes to
  // `Other("")`, a distinct legal value, so eliding would conflate it with
  // an absent field.
  // ----------------------------------------------------------------------------

  impl DefaultInstance for TrackOrigin {
    fn default_instance() -> &'static Self {
      static VALUE: buffa::__private::OnceBox<TrackOrigin> = buffa::__private::OnceBox::new();
      VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(TrackOrigin::default()))
    }
  }

  impl Message for TrackOrigin {
    fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
      // Always-encode the slug — see the wire-format note above.
      1 + string_encoded_len(self.as_str()) as u32
    }

    fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
      Tag::new(1, WireType::LengthDelimited).encode(buf);
      encode_string(self.as_str(), buf);
    }

    fn merge_field(
      &mut self,
      tag: Tag,
      buf: &mut impl Buf,
      ctx: DecodeContext<'_>,
    ) -> Result<(), DecodeError> {
      match tag.field_number() {
        1 => {
          if tag.wire_type() != WireType::LengthDelimited {
            return Err(DecodeError::WireTypeMismatch {
              field_number: 1,
              expected: LEN,
              actual: tag.wire_type() as u8,
            });
          }
          let s = decode_string(buf)?;
          // `FromStr for TrackOrigin` is `Infallible` (total — every
          // string decodes either to a named variant or to `Other(_)`),
          // so there is no failure branch to write.
          let Ok(parsed) = TrackOrigin::from_str(&s);
          *self = parsed;
        }
        _ => skip_field_depth(tag, buf, ctx.depth())?,
      }
      Ok(())
    }

    fn clear(&mut self) {
      *self = TrackOrigin::default();
    }
  }

  // ----------------------------------------------------------------------------
  // Format — { string value = 1; }
  // No stable numeric id; encodes the FFmpeg-style slug from `as_str()`.
  // Always-encoded (NOT proto3 zero-elision and NOT default-elision): the
  // empty string is `Other("")` on decode (per the total `FromStr`), which
  // is a distinct, legal value — eliding the field would conflate it with
  // an absent field. Writing the slug unconditionally side-steps the
  // ambiguity; on decode an absent field stays at the encoder's seed
  // (`Default::default()` = `Other("")`, defined ungated in
  // `crate::subtitle::format` — available regardless of the `buffa`
  // feature).
  // ----------------------------------------------------------------------------

  impl DefaultInstance for Format {
    fn default_instance() -> &'static Self {
      static VALUE: buffa::__private::OnceBox<Format> = buffa::__private::OnceBox::new();
      VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Format::default()))
    }
  }

  impl Message for Format {
    fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
      // Always-encode the slug — see the module-level wire-format note
      // for the rationale (empty slug is `Other("")` ≠ absent).
      let slug = self.as_str();
      1 + string_encoded_len(slug) as u32
    }

    fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
      let slug = self.as_str();
      Tag::new(1, WireType::LengthDelimited).encode(buf);
      encode_string(slug, buf);
    }

    fn merge_field(
      &mut self,
      tag: Tag,
      buf: &mut impl Buf,
      ctx: DecodeContext<'_>,
    ) -> Result<(), DecodeError> {
      match tag.field_number() {
        1 => {
          if tag.wire_type() != WireType::LengthDelimited {
            return Err(DecodeError::WireTypeMismatch {
              field_number: 1,
              expected: LEN,
              actual: tag.wire_type() as u8,
            });
          }
          let s = decode_string(buf)?;
          // `FromStr for Format` is `Infallible` (total — every
          // string decodes either to a named variant or to `Other(_)`).
          let Ok(parsed) = Format::from_str(&s);
          *self = parsed;
        }
        _ => skip_field_depth(tag, buf, ctx.depth())?,
      }
      Ok(())
    }

    fn clear(&mut self) {
      *self = Format::default();
    }
  }
}

// ============================================================================
// Capture + language — `alloc`-gated wire impls. See the
// "## Capture + language" section in the module-level doc for the
// wire-format spec.
// ============================================================================

// ----------------------------------------------------------------------------
// Device — { string make = 1; string model = 2; }
// Default is two empty strings == proto-zero, so proto3 zero-elision
// is sound. Empty string is the in-rust sentinel for "absent".
// ----------------------------------------------------------------------------

#[cfg(any(feature = "std", feature = "alloc"))]
impl DefaultInstance for Device {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Device> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Device::default()))
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Message for Device {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 0u32;
    // proto3 zero-elision: sound — seed is two empty strings.
    if !self.make().is_empty() {
      size += 1 + string_encoded_len(self.make()) as u32;
    }
    if !self.model().is_empty() {
      size += 1 + string_encoded_len(self.model()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    // proto3 zero-elision: sound — see `compute_size`.
    if !self.make().is_empty() {
      Tag::new(1, WireType::LengthDelimited).encode(buf);
      encode_string(self.make(), buf);
    }
    if !self.model().is_empty() {
      Tag::new(2, WireType::LengthDelimited).encode(buf);
      encode_string(self.model(), buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_make(s.as_str());
      }
      2 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        self.set_model(s.as_str());
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Device::default();
  }
}

// ----------------------------------------------------------------------------
// GeoLocation — { double lat = 1; double lon = 2; float altitude = 3; }
//
// `lat` and `lon` are always encoded: the default `(0.0, 0.0)` is
// "Null Island" — a real, legal coordinate. Proto3 zero-elision would
// conflate it with an absent field, which is unsound (same defensive
// stance as `SampleAspectRatio`, `mediatime::Timebase`'s through 0.4).
//
// `altitude` is presence-encoded: field #3 is written iff
// `Some(_)`, including for an explicit `Some(0.0)` (sea level); an
// absent field #3 on the wire decodes back to `None`. No companion
// presence bit is needed because the encoder is the sole writer.
// ----------------------------------------------------------------------------

#[cfg(any(feature = "std", feature = "alloc"))]
impl DefaultInstance for GeoLocation {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<GeoLocation> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| {
      buffa::alloc::boxed::Box::new(GeoLocation::try_new(0.0, 0.0, None).expect("0,0 is valid"))
    })
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Message for GeoLocation {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    // lat (Fixed64) + lon (Fixed64), always encoded: 1-byte tag + 8 bytes each.
    let mut size = (1 + 8) + (1 + 8);
    if self.altitude().is_some() {
      // altitude (Fixed32): 1-byte tag + 4 bytes.
      size += 1 + 4;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    Tag::new(1, WireType::Fixed64).encode(buf);
    encode_double(self.lat(), buf);
    Tag::new(2, WireType::Fixed64).encode(buf);
    encode_double(self.lon(), buf);
    if let Some(alt) = self.altitude() {
      Tag::new(3, WireType::Fixed32).encode(buf);
      encode_float(alt, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Fixed64 {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: WireType::Fixed64 as u8,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_double(buf)?;
        let prev = *self;
        // Range-clamp at the boundary: a malformed wire value
        // outside [-90, 90] is replaced with the closest valid
        // extreme so decode is total (mirrors the
        // `SampleAspectRatio` `den == 0` → `1` defensive clamp).
        let lat = if v.is_finite() {
          v.clamp(-90.0, 90.0)
        } else {
          0.0
        };
        *self =
          GeoLocation::try_new(lat, prev.lon(), prev.altitude()).expect("clamped lat is in range");
      }
      2 => {
        if tag.wire_type() != WireType::Fixed64 {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 2,
            expected: WireType::Fixed64 as u8,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_double(buf)?;
        let prev = *self;
        let lon = if v.is_finite() {
          v.clamp(-180.0, 180.0)
        } else {
          0.0
        };
        *self =
          GeoLocation::try_new(prev.lat(), lon, prev.altitude()).expect("clamped lon is in range");
      }
      3 => {
        if tag.wire_type() != WireType::Fixed32 {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 3,
            expected: WireType::Fixed32 as u8,
            actual: tag.wire_type() as u8,
          });
        }
        let v = decode_float(buf)?;
        self.set_altitude(v);
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = GeoLocation::try_new(0.0, 0.0, None).expect("0,0 is valid");
  }
}

// ----------------------------------------------------------------------------
// LanguageId — { string value = 1; }
//
// Encodes the canonical BCP 47 tag at field #1. proto3 zero-elision
// applies to the empty string; since `LanguageId::default()` is the
// `"und"` tag (non-empty), the encoder always writes it. On decode, an
// absent field (empty buffer / unknown-field skip) seeds back to
// `Default` = `"und"`.
// ----------------------------------------------------------------------------

#[cfg(any(feature = "std", feature = "alloc"))]
impl DefaultInstance for LanguageId {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<LanguageId> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(LanguageId::default()))
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Message for LanguageId {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let tag = self.to_string();
    if tag.is_empty() {
      0
    } else {
      1 + string_encoded_len(&tag) as u32
    }
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl EncodeSink) {
    let tag = self.to_string();
    if !tag.is_empty() {
      Tag::new(1, WireType::LengthDelimited).encode(buf);
      encode_string(&tag, buf);
    }
  }

  fn merge_field(
    &mut self,
    tag: Tag,
    buf: &mut impl Buf,
    ctx: DecodeContext<'_>,
  ) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::LengthDelimited {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: LEN,
            actual: tag.wire_type() as u8,
          });
        }
        let s = decode_string(buf)?;
        // A wire value the whole-tag door refuses is mapped to
        // `LanguageId::default()` (the ISO 639-3 `"und"` "undetermined"
        // sentinel) rather than failing the decode — that is the same
        // semantics the type uses in-rust for "no usable language tag",
        // and keeps the decoder total. `buffa::DecodeError` has no
        // general "invalid value" arm, so silent coercion to the
        // already-existing sentinel is the least-bad choice.
        *self = LanguageId::new(&s).unwrap_or_default();
      }
      _ => skip_field_depth(tag, buf, ctx.depth())?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = LanguageId::default();
  }
}

#[cfg(test)]
mod tests;
