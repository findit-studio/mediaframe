//! `buffa::Message` implementations for the mediaframe wire-relevant
//! types, behind the `buffa` feature. Used via `extern_path` from
//! buffa-generated crates so a `.proto`-defined message can embed a
//! mediaframe type without redefining it.
//!
//! These are hand-written inherent-trait impls — there is **no**
//! codegen and **no** `.proto` in this crate (mirrors the
//! `mediatime` design). The module needs no re-export: the impls are
//! `impl Trait for crate::Type`.
//!
//! # Wire format (clean redesign — no compatibility with any prior
//! encoding is required)
//!
//! ## Enums (each is a standalone message = one field)
//!
//! ```text
//! ColorMatrix    { uint32 value = 1; }   // value = to_u32()
//! ColorPrimaries { uint32 value = 1; }
//! ColorTransfer  { uint32 value = 1; }
//! ColorRange     { uint32 value = 1; }
//! ChromaLocation { uint32 value = 1; }
//! DcpTargetGamut { uint32 value = 1; }
//! Rotation       { uint32 value = 1; }
//! FieldOrder     { uint32 value = 1; }   // value = to_u32() (FFmpeg AVFieldOrder code)
//! StereoMode     { uint32 value = 1; }   // value = to_u32() (FFmpeg AVStereo3DType code)
//! PixelFormat    { uint32 value = 1; }   // value = to_u32() (Unknown(n) → n)
//! ```
//!
//! Each enum encodes its stable `to_u32()` id as a single `uint32`
//! at field #1, decoded via `from_u32()`. The colour enums now use
//! the **FFmpeg code points** (e.g. `ColorMatrix::Unspecified` → 2,
//! `ColorMatrix::Rgb` → 0); an unrecognised id round-trips losslessly
//! as `Unknown(n)` (every enum, including `PixelFormat`, has a
//! data-carrying `Unknown(u32)`).
//!
//! **Default-elision (not proto3 zero-elision):** the field is
//! written iff `*self != <Ty>::default()`. The decoder seeds the
//! message from `Default` (= FFmpeg `UNSPECIFIED` for the colour
//! enums — code `2` for primaries/transfer/matrix, `0` for
//! range/chroma), so an absent field decodes back to `Default`. A
//! *present* field always carries the exact `to_u32()` code —
//! **including code `0`** (`ColorMatrix::Rgb`, FFmpeg
//! `AVCOL_SPC_RGB`), which is *non-default* and therefore explicitly
//! encoded, so it is never conflated with an absent field. Plain
//! proto3 zero-elision would be **unsound** here (it would drop the
//! non-default code-`0` `Rgb`); default-elision is exact for every
//! value and `Unknown(n)` is lossless. Wrong wire type on field #1 →
//! `DecodeError::WireTypeMismatch`; unknown fields are skipped via
//! `skip_field_depth`.
//!
//! ## Structs
//!
//! ```text
//! Dimensions        { uint32 width = 1; uint32 height = 2; }
//! Rect              { uint32 x = 1; uint32 y = 2; uint32 width = 3; uint32 height = 4; }
//! SampleAspectRatio { uint32 num = 1; uint32 den = 2; }          // both ALWAYS encoded
//! Rational          { uint32 num = 1; uint32 den = 2; }          // both ALWAYS encoded
//! FrameRate         { Rational rate = 1;                         // rate ALWAYS encoded
//!                     bool     is_vfr = 2; }                     // proto3 zero-elision
//! DolbyVisionConfig { uint32 profile = 1; uint32 level = 2;      // proto3 zero-elision
//!                     bool rpu_present = 3; bool el_present = 4; //   (all-zero default)
//!                     uint32 bl_signal_compat_id = 5; }
//! ColorInfo         { ColorPrimaries primaries = 1;              // all five ALWAYS
//!                     ColorTransfer  transfer  = 2;              //   encoded as the
//!                     ColorMatrix    matrix    = 3;              //   bare uint32 id
//!                     ColorRange     range     = 4;              //   (not nested msgs)
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
//! **always encoded** (the `mediatime::Timebase` reasoning):
//!
//! - `SampleAspectRatio` — `Default` is `1:1`. `num`'s default is
//!   `1` (≠ 0) and `den` is `NonZeroU32` (never 0), so eliding a
//!   zero would mis-decode. Both fields are always written; a
//!   malformed `den == 0` on the wire (never produced by this
//!   encoder) is clamped to `1` to keep decode total.
//! - `Rational` — same shape / reasoning as `SampleAspectRatio`
//!   (`Default` is `1/1`, `num` default `1`, `den` `NonZeroU32`):
//!   both fields always encoded, malformed `den == 0` clamped to
//!   `1`.
//! - `FrameRate` — `rate` is an always-encoded length-delimited
//!   `Rational` sub-message (its inner `Default` is `1/1` ≠
//!   proto-zero, so the nested-message-always-encoded
//!   `mediatime::Timebase` stance applies, like `MasteringDisplay`'s
//!   coords); `is_vfr` defaults to `false` == proto-zero so it uses
//!   proto3 zero-elision.
//! - `ColorInfo` — **all five enum fields are always encoded** as
//!   the bare FFmpeg-code `uint32` id (not a nested message); tags
//!   #1–#5 are single-byte. `ColorInfo`'s own seed is
//!   `ColorInfo::UNSPECIFIED` (every field FFmpeg `UNSPECIFIED`).
//!   Always-encoding keeps the round-trip exact regardless of which
//!   FFmpeg code a field holds — in particular `matrix ==
//!   ColorMatrix::Rgb` (FFmpeg code `0`) survives because the id is
//!   written unconditionally, never elided — the same defensive
//!   `mediatime::Timebase` always-encode stance.
//! - `MasteringDisplay` — the three primaries and the white point
//!   are always-encoded length-delimited sub-messages so presence
//!   is unambiguous and `decode(encode(x)) == x` holds regardless of
//!   `ChromaCoord` content (nested-message presence, like
//!   `mediatime`'s always-encoded `Timebase`).
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
//! AudioFormat          { uint32 value = 1; }   // value = to_u32() (FFmpeg AV_SAMPLE_FMT_* code, Other → u32::MAX)
//! AudioContainerFormat { string value = 1; }   // value = as_str()
//! ContainerFormat      { string value = 1; }   // value = as_str()
//!
//! Loudness         { float integrated_lufs = 1; float range_lu = 2;
//!                    float true_peak_dbtp = 3; float sample_peak_dbfs = 4; }
//! AudioFingerprint { string algorithm = 1; bytes value = 2; }     // algorithm ALWAYS encoded
//! AudioCoverArt    { string mime      = 1; bytes data  = 2; }     // both ALWAYS encoded
//! AudioTags        { string title        = 1; string artist        = 2;
//!                    string album_artist = 3; string album         = 4;
//!                    string composer     = 5; string genre         = 6;
//!                    string comment      = 7;
//!                    uint32 year         = 8; uint32 track_number  = 9;
//!                    uint32 track_total  = 10; uint32 disc_number   = 11;
//!                    uint32 disc_total   = 12; string language      = 13; }
//! ```
//!
//! - **String-bearing enums** (`ChannelLayout`, `AudioContainerFormat`,
//!   `ContainerFormat`) encode their `as_str()` slug. Default
//!   (where defined) elides; `Other(SmolStr)` round-trips losslessly.
//!   `BitRateMode` and `AudioFormat` encode the `to_u32()` id;
//!   `AudioFormat::Other(SmolStr)` collapses to `Unknown(u32::MAX)`
//!   on the wire (string content is not preserved by the numeric
//!   codec — `Other` is meant for the `FromStr` lossless escape, not
//!   the wire).
//! - **`Loudness`** — all four `f32` fields use proto3 zero-elision
//!   (`Default` is all-zero == proto-zero for `f32`). Each present
//!   field is wire-type `Fixed32` (4 bytes LE).
//! - **`AudioFingerprint`** — `algorithm` is ALWAYS encoded
//!   (`try_new` rejects empty, so a default-constructed wire-empty
//!   `algorithm` would not be a valid `AudioFingerprint` — encoding
//!   it explicitly preserves the invariant on the wire round-trip).
//!   `value` (bytes) uses proto3 zero-elision (an empty fingerprint
//!   is legal). The decoder seed is `try_new("default", []).unwrap()`
//!   so that an absent `algorithm` decodes to a synthetic non-empty
//!   placeholder rather than violating the type invariant.
//! - **`AudioCoverArt`** — both `mime` and `data` are ALWAYS encoded
//!   (`try_new` rejects empty in either, so default-constructed
//!   wire-empty fields would violate the invariant). Same
//!   placeholder-seed strategy as `AudioFingerprint`.
//! - **`AudioTags`** — string fields use proto3 zero-elision (the
//!   empty string is the canonical "absent" value); numeric `u16`
//!   fields are widened to `uint32` and use proto3 zero-elision —
//!   `Some(0)` (legal value) and `None` (absent) **cannot be
//!   distinguished on the wire** in this codec; both round-trip to
//!   `None`. A future codec revision can switch to wrapper messages
//!   if the distinction becomes load-bearing. `language` is the
//!   placeholder BCP-47 SmolStr; the `TODO(lang)` comment on the
//!   Rust type tracks the swap to `Option<Language>`.

use core::num::NonZeroU32;

use ::buffa::{
  DecodeError, DefaultInstance, Message, SizeCache,
  bytes::{Buf, BufMut},
  encoding::{Tag, WireType, encode_varint, skip_field_depth, varint_len},
  types::{
    FIXED32_ENCODED_LEN, bytes_encoded_len, decode_bytes, decode_float, decode_string,
    decode_uint32, encode_bytes, encode_float, encode_string, encode_uint32, string_encoded_len,
    uint32_encoded_len,
  },
};
use smol_str::SmolStr;

use crate::{
  audio::{
    AudioContainerFormat, AudioCoverArt, AudioFingerprint, AudioFormat, AudioTags, BitRateMode,
    ChannelLayout, Loudness,
  },
  color::{
    ChromaCoord, ChromaLocation, ColorInfo, ColorMatrix, ColorPrimaries, ColorRange, ColorTransfer,
    ContentLightLevel, DcpTargetGamut, DolbyVisionConfig, HdrStaticMetadata, MasteringDisplay,
  },
  container::ContainerFormat,
  frame::{
    Dimensions, FieldOrder, FrameRate, Rational, Rect, Rotation, SampleAspectRatio, StereoMode,
  },
  pixel_format::PixelFormat,
};

const VARINT: u8 = WireType::Varint as u8;
const LEN: u8 = WireType::LengthDelimited as u8;

// ----------------------------------------------------------------------------
// Enum codec helper.
//
// A standalone enum is a one-field message: `uint32 value = 1`,
// where `value` is the stable `to_u32()` id (the FFmpeg code point
// for the colour enums). The field is encoded with **default-elision**
// (NOT proto3 zero-elision): written iff `*self != <Ty>::default()`.
// The decoder seeds from `Default` (= FFmpeg `UNSPECIFIED` for the
// colour enums), so an absent field decodes back to the default; a
// present field always carries the exact code — including code 0
// (`ColorMatrix::Rgb`), which is non-default and therefore explicitly
// encoded, so it is never conflated with an absent field. Plain
// zero-elision would drop that non-default code-0 value and is thus
// unsound here. `Unknown(n)` round-trips losslessly. Requires
// `Ty: PartialEq` (every enum derives it).
// ----------------------------------------------------------------------------

macro_rules! impl_enum_message {
  ($ty:ty, $to:expr, $from:expr) => {
    impl DefaultInstance for $ty {
      fn default_instance() -> &'static Self {
        static VALUE: buffa::__private::OnceBox<$ty> = buffa::__private::OnceBox::new();
        VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(<$ty>::default()))
      }
    }

    impl Message for $ty {
      fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
        // Default-elision (NOT proto3 zero-elision): the decoder
        // seeds from `Default`, so only a non-default value needs
        // its FFmpeg-code id written. Code 0 (`ColorMatrix::Rgb`)
        // is non-default and is therefore encoded.
        if *self != <$ty>::default() {
          let v: u32 = $to(self);
          1 + uint32_encoded_len(v) as u32
        } else {
          0
        }
      }

      fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
        // Default-elision (NOT proto3 zero-elision): see `compute_size`.
        if *self != <$ty>::default() {
          let v: u32 = $to(self);
          Tag::new(1, WireType::Varint).encode(buf);
          encode_uint32(v, buf);
        }
      }

      fn merge_field(
        &mut self,
        tag: Tag,
        buf: &mut impl Buf,
        depth: u32,
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
            *self = $from(v);
          }
          _ => skip_field_depth(tag, buf, depth)?,
        }
        Ok(())
      }

      fn clear(&mut self) {
        *self = <$ty>::default();
      }
    }
  };
}

impl_enum_message!(
  ColorMatrix,
  |s: &ColorMatrix| s.to_u32(),
  ColorMatrix::from_u32
);
impl_enum_message!(
  ColorPrimaries,
  |s: &ColorPrimaries| s.to_u32(),
  ColorPrimaries::from_u32
);
impl_enum_message!(
  ColorTransfer,
  |s: &ColorTransfer| s.to_u32(),
  ColorTransfer::from_u32
);
impl_enum_message!(
  ColorRange,
  |s: &ColorRange| s.to_u32(),
  ColorRange::from_u32
);
impl_enum_message!(
  ChromaLocation,
  |s: &ChromaLocation| s.to_u32(),
  ChromaLocation::from_u32
);
impl_enum_message!(
  DcpTargetGamut,
  |s: &DcpTargetGamut| s.to_u32(),
  DcpTargetGamut::from_u32
);
impl_enum_message!(Rotation, |s: &Rotation| s.to_u32(), Rotation::from_u32);
// `FieldOrder` default is `Unknown(0)` (FFmpeg `AV_FIELD_UNKNOWN`,
// code 0): `to_u32` of the default is `0`, so it elides; an absent
// field decodes via `from_u32(seed)` back to `Unknown(0)` (no
// canonical id is `0`), so the round-trip is lossless. Named
// variants and other `Unknown(n)` are non-default and explicitly
// encoded.
impl_enum_message!(
  FieldOrder,
  |s: &FieldOrder| s.to_u32(),
  FieldOrder::from_u32
);
// `StereoMode` default is `Mono` (FFmpeg `AV_STEREO3D_2D`, a *real*
// code, value 0). The default elides; an absent field decodes via
// `from_u32(0)` → `Mono` (the named variant for code 0), so the
// round-trip is exact. Non-default values (incl. `Unknown(n)`) are
// explicitly encoded; `Unknown` wrapping a canonical id
// canonicalises to its named variant on decode (the shared
// decoder-only `Unknown` convention, like `DcpTargetGamut`).
impl_enum_message!(
  StereoMode,
  |s: &StereoMode| s.to_u32(),
  StereoMode::from_u32
);
// `PixelFormat::to_u32` consumes `self` (it is `Copy`); `from_u32`
// maps unrecognised ids to `Unknown(n)` so the round-trip is
// lossless even for the elided-default case (`Unknown(0)` is the
// `Default`, so it elides and decodes back to `Unknown(0)`).
impl_enum_message!(
  PixelFormat,
  |s: &PixelFormat| s.to_u32(),
  PixelFormat::from_u32
);

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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
      _ => skip_field_depth(tag, buf, depth)?,
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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Rect::default();
  }
}

// ----------------------------------------------------------------------------
// SampleAspectRatio — { uint32 num = 1; uint32 den = 2; }
//
// `num`/`den` are encoded UNCONDITIONALLY — no proto3 zero elision.
// The decoder seeds from `SampleAspectRatio::default()` (1:1), NOT
// proto-zero. Eliding `num == 0` would decode back as `num == 1`;
// `den` is `NonZeroU32` and can never legitimately be 0. (Exactly
// the `mediatime::Timebase` reasoning.) Both tags are single-byte.
// ----------------------------------------------------------------------------

impl DefaultInstance for SampleAspectRatio {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<SampleAspectRatio> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(SampleAspectRatio::default()))
  }
}

impl Message for SampleAspectRatio {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    2 + uint32_encoded_len(self.num()) as u32 + uint32_encoded_len(self.den().get()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    Tag::new(1, WireType::Varint).encode(buf);
    encode_uint32(self.num(), buf);
    Tag::new(2, WireType::Varint).encode(buf);
    encode_uint32(self.den().get(), buf);
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let num = decode_uint32(buf)?;
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
        // `den` is NonZeroU32; a malformed 0 on the wire (never
        // produced by our own encoder) is clamped to 1. This is
        // byte-identical to `mediatime::Timebase`'s decode in the
        // published `mediatime` extern (>= 0.1.6) that SAR mirrors,
        // and upholds the codec family's total-scalar-decode
        // invariant (scalar values never raise decode errors; only
        // structural errors do). Codex adversarial-review F6:
        // resolved as a coordinated mediatime/buffa policy, NOT a
        // mediaframe-only divergence.
        let den = NonZeroU32::new(decode_uint32(buf)?).unwrap_or(NonZeroU32::MIN);
        self.set_den(den);
      }
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = SampleAspectRatio::default();
  }
}

// ----------------------------------------------------------------------------
// Rational — { uint32 num = 1; uint32 den = 2; }
//
// Same shape and reasoning as `SampleAspectRatio`: `num`/`den` are
// encoded UNCONDITIONALLY (no proto3 zero-elision). The decoder
// seeds from `Rational::default()` (1/1), NOT proto-zero; eliding
// `num == 0` would decode back as `num == 1`. `den` is `NonZeroU32`
// and can never legitimately be 0; a malformed wire `den == 0` (never
// produced by this encoder) is clamped to 1 to keep decode total.
// Both tags are single-byte.
// ----------------------------------------------------------------------------

impl DefaultInstance for Rational {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<Rational> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(Rational::default()))
  }
}

impl Message for Rational {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    2 + uint32_encoded_len(self.num()) as u32 + uint32_encoded_len(self.den().get()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    Tag::new(1, WireType::Varint).encode(buf);
    encode_uint32(self.num(), buf);
    Tag::new(2, WireType::Varint).encode(buf);
    encode_uint32(self.den().get(), buf);
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        let num = decode_uint32(buf)?;
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
        // `den` is NonZeroU32; a malformed 0 on the wire (never
        // produced by our own encoder) is clamped to 1 — identical
        // to `SampleAspectRatio`'s decode, upholding the codec
        // family's total-scalar-decode invariant.
        let den = NonZeroU32::new(decode_uint32(buf)?).unwrap_or(NonZeroU32::MIN);
        self.set_den(den);
      }
      _ => skip_field_depth(tag, buf, depth)?,
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
// nested-message-always-encoded `mediatime::Timebase` stance applies
// (like `MasteringDisplay`'s coords) — presence is unambiguous and
// `decode(encode(x)) == x` holds regardless of the inner ratio.
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

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl BufMut) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_varint(cache.consume_next() as u64, buf);
    self.rate().write_to(cache, buf);
    // proto3 zero-elision: sound — see `compute_size`.
    if self.is_vfr() {
      Tag::new(2, WireType::Varint).encode(buf);
      encode_varint(1, buf);
    }
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
        buffa::Message::merge_length_delimited(&mut rate, buf, depth)?;
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
        self.set_is_vfr(decode_uint32(buf)? != 0);
      }
      _ => skip_field_depth(tag, buf, depth)?,
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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
        self.set_rpu_present(decode_uint32(buf)? != 0);
      }
      4 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 4,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        self.set_el_present(decode_uint32(buf)? != 0);
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
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = DolbyVisionConfig::default();
  }
}

// ----------------------------------------------------------------------------
// ColorInfo — five enum ids, each a bare `uint32`, ALL always
// encoded. See the module doc: always-encoding (esp. `matrix`, whose
// semantic default is `Bt709`) decouples the wire round-trip from
// the `ColorMatrix` discriminant assignment — the `mediatime`
// always-encode-nontrivial-default stance. Tags #1–#5 single-byte.
// ----------------------------------------------------------------------------

impl DefaultInstance for ColorInfo {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<ColorInfo> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(ColorInfo::UNSPECIFIED))
  }
}

impl Message for ColorInfo {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    // All five are unconditionally encoded (presence-independent).
    5 + uint32_encoded_len(self.primaries().to_u32()) as u32
      + uint32_encoded_len(self.transfer().to_u32()) as u32
      + uint32_encoded_len(self.matrix().to_u32()) as u32
      + uint32_encoded_len(self.range().to_u32()) as u32
      + uint32_encoded_len(self.chroma_location().to_u32()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    Tag::new(1, WireType::Varint).encode(buf);
    encode_uint32(self.primaries().to_u32(), buf);
    Tag::new(2, WireType::Varint).encode(buf);
    encode_uint32(self.transfer().to_u32(), buf);
    Tag::new(3, WireType::Varint).encode(buf);
    encode_uint32(self.matrix().to_u32(), buf);
    Tag::new(4, WireType::Varint).encode(buf);
    encode_uint32(self.range().to_u32(), buf);
    Tag::new(5, WireType::Varint).encode(buf);
    encode_uint32(self.chroma_location().to_u32(), buf);
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
        self.set_primaries(ColorPrimaries::from_u32(v));
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
        self.set_transfer(ColorTransfer::from_u32(v));
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
        self.set_matrix(ColorMatrix::from_u32(v));
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
        self.set_range(ColorRange::from_u32(v));
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
        self.set_chroma_location(ChromaLocation::from_u32(v));
      }
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = ColorInfo::UNSPECIFIED;
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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
      _ => skip_field_depth(tag, buf, depth)?,
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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
      _ => skip_field_depth(tag, buf, depth)?,
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
// content (the `mediatime` always-encoded-nested-message stance).
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

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
        buffa::Message::merge_length_delimited(&mut cc, buf, depth)?;
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
        buffa::Message::merge_length_delimited(&mut wp, buf, depth)?;
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
      _ => skip_field_depth(tag, buf, depth)?,
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

  fn write_to(&self, cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
        buffa::Message::merge_length_delimited(&mut md, buf, depth)?;
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
        buffa::Message::merge_length_delimited(&mut cll, buf, depth)?;
        self.set_content_light(Some(cll));
      }
      _ => skip_field_depth(tag, buf, depth)?,
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
// `as_str()` slug. Default-elision: written iff
// `*self != <Ty>::default()`. For enums without `Default`, the
// "default" is the wire-zero state (empty string → `Other("")`),
// and we encode every non-zero value.
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
        // Always-encode: every value (including the `Other("")`
        // wire-zero) round-trips losslessly. The decoder seed is the
        // wire-zero state, so an absent field decodes to that exact
        // state — there is no information loss.
        let s = self.as_str();
        if !s.is_empty() {
          1 + string_encoded_len(s) as u32
        } else {
          0
        }
      }

      fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
        let s = self.as_str();
        if !s.is_empty() {
          Tag::new(1, WireType::LengthDelimited).encode(buf);
          encode_string(s, buf);
        }
      }

      fn merge_field(
        &mut self,
        tag: Tag,
        buf: &mut impl Buf,
        depth: u32,
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
            *self = <$ty as core::str::FromStr>::from_str(&s).unwrap_or_else(|_| unreachable!());
          }
          _ => skip_field_depth(tag, buf, depth)?,
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
impl_string_enum_message!(ChannelLayout, ChannelLayout::Other(SmolStr::new_inline("")));
impl_string_enum_message!(
  AudioContainerFormat,
  AudioContainerFormat::Other(SmolStr::new_inline(""))
);
impl_string_enum_message!(
  ContainerFormat,
  ContainerFormat::Other(SmolStr::new_inline(""))
);

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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    let v = self.to_u32();
    if v != 0 {
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(v, buf);
    }
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = BitRateMode::default();
  }
}

// ----------------------------------------------------------------------------
// AudioFormat — { uint32 value = 1; }
//
// FFmpeg-coded; `AudioFormat::default() == Unknown(u32::MAX)` (the
// `AV_SAMPLE_FMT_NONE` sentinel), whose `to_u32() == u32::MAX`.
// That is NOT proto-zero, so default-elision (NOT proto3
// zero-elision) is used: written iff `*self != default()`. `U8`
// (code `0`) is non-default and therefore explicitly encoded —
// never conflated with the absent / default sentinel.
// `Other(SmolStr)` collapses to `Unknown(u32::MAX)` on the wire
// (numeric codec; the slug is preserved only on the `FromStr` path).
// ----------------------------------------------------------------------------

impl DefaultInstance for AudioFormat {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<AudioFormat> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(AudioFormat::default()))
  }
}

impl Message for AudioFormat {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    if *self != AudioFormat::default() {
      let v = self.to_u32();
      1 + uint32_encoded_len(v) as u32
    } else {
      0
    }
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    if *self != AudioFormat::default() {
      let v = self.to_u32();
      Tag::new(1, WireType::Varint).encode(buf);
      encode_uint32(v, buf);
    }
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
    match tag.field_number() {
      1 => {
        if tag.wire_type() != WireType::Varint {
          return Err(DecodeError::WireTypeMismatch {
            field_number: 1,
            expected: VARINT,
            actual: tag.wire_type() as u8,
          });
        }
        *self = AudioFormat::from_u32(decode_uint32(buf)?);
      }
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = AudioFormat::default();
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

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = Loudness::default();
  }
}

// ----------------------------------------------------------------------------
// AudioFingerprint — { string algorithm = 1; bytes value = 2; }
//
// `try_new` rejects empty `algorithm`, so the type has no
// natural-zero `Default`. The decoder seed is a synthetic
// `AudioFingerprint { algorithm: "default", value: [] }` (the
// always-encoded `algorithm` overwrites it on decode). `algorithm`
// is encoded UNCONDITIONALLY; `value` (bytes) uses proto3
// zero-elision (empty fingerprint is a legal value).
// ----------------------------------------------------------------------------

fn audio_fingerprint_seed() -> AudioFingerprint {
  // Safety: the literal is non-empty so `try_new` cannot fail.
  AudioFingerprint::try_new(SmolStr::new_inline("default"), std::vec::Vec::new())
    .unwrap_or_else(|_| unreachable!())
}

impl DefaultInstance for AudioFingerprint {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<AudioFingerprint> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(audio_fingerprint_seed()))
  }
}

impl Message for AudioFingerprint {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    let mut size = 1 + string_encoded_len(self.algorithm()) as u32;
    if !self.value().is_empty() {
      size += 1 + bytes_encoded_len(self.value()) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_string(self.algorithm(), buf);
    if !self.value().is_empty() {
      Tag::new(2, WireType::LengthDelimited).encode(buf);
      encode_bytes(self.value(), buf);
    }
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
          SmolStr::new_inline("default")
        } else {
          SmolStr::new(&algo)
        };
        // Preserve existing `value`, swap `algorithm`. `try_new`
        // moves the bytes back in unchanged.
        let value = self.value().to_vec();
        *self = AudioFingerprint::try_new(algo, value).unwrap_or_else(|_| audio_fingerprint_seed());
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
        let algo = SmolStr::from(self.algorithm());
        *self = AudioFingerprint::try_new(algo, bytes).unwrap_or_else(|_| audio_fingerprint_seed());
      }
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = audio_fingerprint_seed();
  }
}

// ----------------------------------------------------------------------------
// AudioCoverArt — { string mime = 1; bytes data = 2; }
//
// `try_new` rejects empty mime / empty data, so the type has no
// natural-zero `Default`. Decoder seed is a synthetic
// `AudioCoverArt { mime: "application/octet-stream", data: [0u8] }`
// (sentinel that gets overwritten on decode; both fields are
// ALWAYS encoded on the write path).
// ----------------------------------------------------------------------------

fn audio_cover_art_seed() -> AudioCoverArt {
  AudioCoverArt::try_new(
    SmolStr::new_static("application/octet-stream"),
    std::vec![0u8],
  )
  .unwrap_or_else(|_| unreachable!())
}

impl DefaultInstance for AudioCoverArt {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<AudioCoverArt> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(audio_cover_art_seed()))
  }
}

impl Message for AudioCoverArt {
  fn compute_size(&self, _cache: &mut SizeCache) -> u32 {
    2 + string_encoded_len(self.mime()) as u32 + bytes_encoded_len(self.data()) as u32
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
    Tag::new(1, WireType::LengthDelimited).encode(buf);
    encode_string(self.mime(), buf);
    Tag::new(2, WireType::LengthDelimited).encode(buf);
    encode_bytes(self.data(), buf);
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
          SmolStr::new_static("application/octet-stream")
        } else {
          SmolStr::new(&mime)
        };
        let data = self.data().to_vec();
        let data = if data.is_empty() {
          std::vec![0u8]
        } else {
          data
        };
        *self = AudioCoverArt::try_new(mime, data).unwrap_or_else(|_| audio_cover_art_seed());
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
        let mime = SmolStr::from(self.mime());
        *self = AudioCoverArt::try_new(mime, data).unwrap_or_else(|_| audio_cover_art_seed());
      }
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = audio_cover_art_seed();
  }
}

// ----------------------------------------------------------------------------
// AudioTags — every string field uses proto3 zero-elision ("" ==
// "absent" by the type's own convention); every numeric Option<u16>
// is widened to uint32 and uses proto3 zero-elision. `Some(0)`
// (theoretically legal) and `None` (absent) round-trip identically
// to `None` — documented limitation.
// ----------------------------------------------------------------------------

impl DefaultInstance for AudioTags {
  fn default_instance() -> &'static Self {
    static VALUE: buffa::__private::OnceBox<AudioTags> = buffa::__private::OnceBox::new();
    VALUE.get_or_init(|| buffa::alloc::boxed::Box::new(AudioTags::default()))
  }
}

impl Message for AudioTags {
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
    if let Some(v) = self.year()
      && v != 0
    {
      size += 1 + uint32_encoded_len(v as u32) as u32;
    }
    if let Some(v) = self.track_number()
      && v != 0
    {
      size += 1 + uint32_encoded_len(v as u32) as u32;
    }
    if let Some(v) = self.track_total()
      && v != 0
    {
      size += 1 + uint32_encoded_len(v as u32) as u32;
    }
    if let Some(v) = self.disc_number()
      && v != 0
    {
      size += 1 + uint32_encoded_len(v as u32) as u32;
    }
    if let Some(v) = self.disc_total()
      && v != 0
    {
      size += 1 + uint32_encoded_len(v as u32) as u32;
    }
    if let Some(s) = self.language()
      && !s.is_empty()
    {
      size += 1 + string_encoded_len(s) as u32;
    }
    size
  }

  fn write_to(&self, _cache: &mut SizeCache, buf: &mut impl BufMut) {
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
    if let Some(v) = self.year()
      && v != 0
    {
      Tag::new(8, WireType::Varint).encode(buf);
      encode_uint32(v as u32, buf);
    }
    if let Some(v) = self.track_number()
      && v != 0
    {
      Tag::new(9, WireType::Varint).encode(buf);
      encode_uint32(v as u32, buf);
    }
    if let Some(v) = self.track_total()
      && v != 0
    {
      Tag::new(10, WireType::Varint).encode(buf);
      encode_uint32(v as u32, buf);
    }
    if let Some(v) = self.disc_number()
      && v != 0
    {
      Tag::new(11, WireType::Varint).encode(buf);
      encode_uint32(v as u32, buf);
    }
    if let Some(v) = self.disc_total()
      && v != 0
    {
      Tag::new(12, WireType::Varint).encode(buf);
      encode_uint32(v as u32, buf);
    }
    if let Some(s) = self.language()
      && !s.is_empty()
    {
      Tag::new(13, WireType::LengthDelimited).encode(buf);
      encode_string(s, buf);
    }
  }

  fn merge_field(&mut self, tag: Tag, buf: &mut impl Buf, depth: u32) -> Result<(), DecodeError> {
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
        let s = SmolStr::new(&s);
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
            self.set_language(if s.is_empty() { None } else { Some(s) });
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
        let v = decode_uint32(buf)? as u16;
        let opt = if v == 0 { None } else { Some(v) };
        match n {
          8 => {
            self.set_year(opt);
          }
          9 => {
            self.set_track_number(opt);
          }
          10 => {
            self.set_track_total(opt);
          }
          11 => {
            self.set_disc_number(opt);
          }
          12 => {
            self.set_disc_total(opt);
          }
          _ => unreachable!(),
        }
      }
      _ => skip_field_depth(tag, buf, depth)?,
    }
    Ok(())
  }

  fn clear(&mut self) {
    *self = AudioTags::default();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // `mediaframe` is `#![no_std]`; `Vec` is not in the core prelude. The
  // non-test impls above reach `alloc` through the always-present `buffa`
  // crate (`buffa::alloc::*`); the test module does the same so it builds
  // under `--no-default-features --features buffa`.
  use ::buffa::alloc::vec::Vec;

  fn nz(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).unwrap()
  }

  fn cc(x: u32, y: u32) -> ChromaCoord {
    ChromaCoord::new(x, y)
  }

  // ---- enums: default-elision codec (FFmpeg code points) ----
  //
  // For every enum: (a) `default()` encodes to ZERO bytes and
  // decodes back to `default()`; (b) a non-default value whose
  // `to_u32() == 0` (`ColorMatrix::Rgb`, FFmpeg `AVCOL_SPC_RGB`)
  // encodes to NON-zero bytes and round-trips — proving an absent
  // field is never conflated with code-0 `Rgb`; (c) `Unknown(12345)`
  // round-trips losslessly; (d) a normal non-default value
  // round-trips.

  #[test]
  fn enum_default_elides_to_zero_bytes() {
    // (a) Default value → empty wire → decodes back to default.
    assert!(ColorMatrix::default().encode_to_vec().is_empty());
    assert!(ColorPrimaries::default().encode_to_vec().is_empty());
    assert!(ColorTransfer::default().encode_to_vec().is_empty());
    assert!(ColorRange::default().encode_to_vec().is_empty());
    assert!(ChromaLocation::default().encode_to_vec().is_empty());
    assert!(DcpTargetGamut::default().encode_to_vec().is_empty());
    assert_eq!(
      ColorMatrix::decode_from_slice(&[]).unwrap(),
      ColorMatrix::default()
    );
    assert_eq!(
      ColorPrimaries::decode_from_slice(&[]).unwrap(),
      ColorPrimaries::default()
    );
    assert_eq!(
      ColorTransfer::decode_from_slice(&[]).unwrap(),
      ColorTransfer::default()
    );
    assert_eq!(
      ColorRange::decode_from_slice(&[]).unwrap(),
      ColorRange::default()
    );
    assert_eq!(
      ChromaLocation::decode_from_slice(&[]).unwrap(),
      ChromaLocation::default()
    );
    assert_eq!(
      DcpTargetGamut::decode_from_slice(&[]).unwrap(),
      DcpTargetGamut::default()
    );
  }

  #[test]
  fn enum_non_default_code_zero_is_encoded_not_conflated() {
    // (b) `ColorMatrix::Rgb` is FFmpeg code 0 but is NON-default, so
    // it must be explicitly encoded (non-empty) and round-trip to
    // `Rgb` — never decoded as the absent/default `Unspecified`.
    let b = ColorMatrix::Rgb.encode_to_vec();
    assert!(!b.is_empty(), "non-default code-0 Rgb must be encoded");
    let back = ColorMatrix::decode_from_slice(&b).unwrap();
    assert_eq!(back, ColorMatrix::Rgb);
    assert!(back.is_rgb());
    assert_ne!(back, ColorMatrix::default());
  }

  #[test]
  fn enum_unknown_round_trips_losslessly() {
    // (c) `Unknown(12345)` survives encode/decode for every enum.
    macro_rules! rt_unknown {
      ($ty:ty) => {{
        let v = <$ty>::Unknown(12_345);
        let b = v.encode_to_vec();
        assert_eq!(<$ty>::decode_from_slice(&b).unwrap(), v);
      }};
    }
    rt_unknown!(ColorMatrix);
    rt_unknown!(ColorPrimaries);
    rt_unknown!(ColorTransfer);
    rt_unknown!(ColorRange);
    rt_unknown!(ChromaLocation);
    rt_unknown!(DcpTargetGamut);
    rt_unknown!(PixelFormat);
  }

  #[test]
  fn enum_non_default_round_trips() {
    // (d) A normal non-default value round-trips for every enum.
    let cm = ColorMatrix::Bt2020Ncl.encode_to_vec();
    assert_eq!(
      ColorMatrix::decode_from_slice(&cm).unwrap(),
      ColorMatrix::Bt2020Ncl
    );
    let cp = ColorPrimaries::Bt2020.encode_to_vec();
    assert_eq!(
      ColorPrimaries::decode_from_slice(&cp).unwrap(),
      ColorPrimaries::Bt2020
    );
    let ct = ColorTransfer::AribStdB67Hlg.encode_to_vec();
    assert_eq!(
      ColorTransfer::decode_from_slice(&ct).unwrap(),
      ColorTransfer::AribStdB67Hlg
    );
    let dg = DcpTargetGamut::Rec2020.encode_to_vec();
    assert_eq!(
      DcpTargetGamut::decode_from_slice(&dg).unwrap(),
      DcpTargetGamut::Rec2020
    );
  }

  #[test]
  fn dcp_target_gamut_unknown_canonicalization() {
    // Codex adversarial-review F8. `Unknown` is decoder-only: the
    // decoder never emits `Unknown(0..=2)` (`from_u32` maps the
    // canonical ids to their named variants), so a *decoded* value
    // always round-trips. Manually wrapping a canonical id in
    // `Unknown` is a misuse; it canonicalises to the named variant
    // on a buffa round-trip (correct — the id *is* that gamut),
    // never silent data loss.
    for (misuse, named) in [
      (DcpTargetGamut::Unknown(0), DcpTargetGamut::DciP3),
      (DcpTargetGamut::Unknown(1), DcpTargetGamut::Rec709),
      (DcpTargetGamut::Unknown(2), DcpTargetGamut::Rec2020),
    ] {
      let b = misuse.encode_to_vec();
      assert_eq!(DcpTargetGamut::decode_from_slice(&b).unwrap(), named);
    }
    // Non-canonical ids are preserved losslessly and the decoder
    // yields `Unknown` (still F7-rejected by `xyz12_to`).
    for v in [3u32, 4242, u32::MAX] {
      let u = DcpTargetGamut::Unknown(v);
      let b = u.encode_to_vec();
      assert_eq!(DcpTargetGamut::decode_from_slice(&b).unwrap(), u);
      assert_eq!(DcpTargetGamut::from_u32(v), DcpTargetGamut::Unknown(v));
    }
  }

  #[test]
  fn color_matrix_bt601_domain_variant_round_trips() {
    // `ColorMatrix::Bt601` is a mediaframe-domain id
    // (`DOMAIN_EXT_BASE` = 0x8000_0000), non-default, so it must be
    // explicitly encoded to NON-zero bytes and round-trip losslessly
    // via the `Message` impl (uint32 carrying 0x8000_0000).
    let b = ColorMatrix::Bt601.encode_to_vec();
    assert!(!b.is_empty(), "non-default domain Bt601 must be encoded");
    let back = ColorMatrix::decode_from_slice(&b).unwrap();
    assert_eq!(back, ColorMatrix::Bt601);
    assert!(back.is_bt_601());
    assert_ne!(back, ColorMatrix::default());
    // Default `Unspecified` still elides to zero bytes.
    assert!(ColorMatrix::default().encode_to_vec().is_empty());
    assert_eq!(
      ColorMatrix::decode_from_slice(&[]).unwrap(),
      ColorMatrix::default()
    );
  }

  #[test]
  fn color_matrix_default_instance_and_clear() {
    assert_eq!(
      *<ColorMatrix as DefaultInstance>::default_instance(),
      ColorMatrix::default()
    );
    let mut m = ColorMatrix::YCgCo;
    Message::clear(&mut m);
    assert_eq!(m, ColorMatrix::default());
  }

  #[test]
  fn color_range_round_trip() {
    for r in [
      ColorRange::Unspecified,
      ColorRange::Limited,
      ColorRange::Full,
    ] {
      let b = r.encode_to_vec();
      assert_eq!(ColorRange::decode_from_slice(&b).unwrap(), r);
    }
  }

  #[test]
  fn rotation_round_trip() {
    // `D0` is the default (wire id 0) so it elides. `Unknown(n)`
    // preserves unrecognised / corrupt / future wire ids losslessly
    // through the shared enum codec — no silent collapse to `D0`
    // (Codex adversarial-review F1).
    for r in [
      Rotation::D0,
      Rotation::D90,
      Rotation::D180,
      Rotation::D270,
      Rotation::Unknown(7),
      Rotation::Unknown(4242),
    ] {
      let b = r.encode_to_vec();
      assert_eq!(Rotation::decode_from_slice(&b).unwrap(), r);
    }
  }

  #[test]
  fn enum_wrong_wire_type_errors() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    let err = <ColorMatrix as Message>::decode_from_slice(&buf).unwrap_err();
    assert!(
      matches!(err, DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == VARINT && actual == LEN),
      "got {err:?}"
    );
  }

  #[test]
  fn enum_unknown_field_is_skipped() {
    let mut buf = ColorRange::Full.encode_to_vec();
    Tag::new(7, WireType::Varint).encode(&mut buf); // unknown → skip
    encode_varint(123, &mut buf);
    assert_eq!(
      <ColorRange as Message>::decode_from_slice(&buf).unwrap(),
      ColorRange::Full
    );
  }

  #[test]
  fn enum_unknown_id_decodes_losslessly() {
    // An unrecognised on-wire id now decodes to `Unknown(n)` (no
    // silent collapse to `default()`), preserving the value.
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_uint32(9_999, &mut buf);
    assert_eq!(
      <ColorTransfer as Message>::decode_from_slice(&buf).unwrap(),
      ColorTransfer::Unknown(9_999)
    );
  }

  #[test]
  fn pixel_format_round_trip_including_unknown() {
    for p in [
      PixelFormat::Yuv420p,
      PixelFormat::default(), // Unknown(0) → elided → Unknown(0)
      PixelFormat::Unknown(77),
    ] {
      let b = p.encode_to_vec();
      assert_eq!(PixelFormat::decode_from_slice(&b).unwrap(), p);
    }
  }

  // ---- Dimensions ----

  #[test]
  fn dimensions_round_trip_and_default() {
    for d in [
      Dimensions::default(),
      Dimensions::new(1920, 1080),
      Dimensions::new(0, 720),
    ] {
      let b = d.encode_to_vec();
      assert_eq!(Dimensions::decode_from_slice(&b).unwrap(), d);
    }
  }

  #[test]
  fn dimensions_wrong_wire_type_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(2, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <Dimensions as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 2, expected, actual }
        if expected == VARINT && actual == LEN
    ));
    let mut ok = Dimensions::new(64, 48).encode_to_vec();
    Tag::new(9, WireType::Varint).encode(&mut ok);
    encode_varint(5, &mut ok);
    assert_eq!(
      <Dimensions as Message>::decode_from_slice(&ok).unwrap(),
      Dimensions::new(64, 48)
    );
  }

  // ---- Rect ----

  #[test]
  fn rect_round_trip_and_default() {
    for r in [
      Rect::default(),
      Rect::new(10, 20, 1280, 720),
      Rect::new(0, 0, 0, 480),
    ] {
      let b = r.encode_to_vec();
      assert_eq!(Rect::decode_from_slice(&b).unwrap(), r);
    }
  }

  #[test]
  fn rect_wrong_wire_type_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(3, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <Rect as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 3, expected, actual }
        if expected == VARINT && actual == LEN
    ));
    let mut ok = Rect::new(1, 2, 3, 4).encode_to_vec();
    Tag::new(8, WireType::Varint).encode(&mut ok);
    encode_varint(1, &mut ok);
    assert_eq!(
      <Rect as Message>::decode_from_slice(&ok).unwrap(),
      Rect::new(1, 2, 3, 4)
    );
  }

  // ---- SampleAspectRatio ----

  #[test]
  fn sar_round_trip_default_and_nondefault() {
    for s in [
      SampleAspectRatio::default(),       // 1:1
      SampleAspectRatio::new(40, nz(33)), // NTSC SAR
      SampleAspectRatio::new(0, nz(1)),   // num == 0 must survive
    ] {
      let b = s.encode_to_vec();
      assert_eq!(SampleAspectRatio::decode_from_slice(&b).unwrap(), s);
    }
  }

  // Byte-for-byte wire-stability guard. `SampleAspectRatio` is a
  // `buffa` extern target whose representation changed (newtype over
  // `Rational` in 0.3.1); the wire encoding MUST stay identical to
  // 0.3.0: `{ uint32 num = 1; uint32 den = 2; }`, both always encoded.
  // For `new(40, 33)`: tag1 varint `0x08`, value `40` (`0x28`),
  // tag2 varint `0x10`, value `33` (`0x21`).
  #[test]
  fn sar_wire_is_byte_stable() {
    let bytes = SampleAspectRatio::new(40, nz(33)).encode_to_vec();
    let expected: Vec<u8> = [0x08u8, 0x28, 0x10, 0x21].into_iter().collect();
    assert_eq!(bytes, expected);
    // …and decodes back unchanged.
    assert_eq!(
      SampleAspectRatio::decode_from_slice(&bytes).unwrap(),
      SampleAspectRatio::new(40, nz(33))
    );
  }

  #[test]
  fn sar_field2_wrong_wire_type_errors() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_uint32(4, &mut buf);
    Tag::new(2, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <SampleAspectRatio as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 2, expected, actual }
        if expected == VARINT && actual == LEN
    ));
  }

  #[test]
  fn sar_den_zero_clamped_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_uint32(16, &mut buf);
    Tag::new(2, WireType::Varint).encode(&mut buf);
    encode_uint32(0, &mut buf); // malformed den == 0
    Tag::new(6, WireType::Varint).encode(&mut buf); // unknown → skip
    encode_varint(42, &mut buf);
    let s = <SampleAspectRatio as Message>::decode_from_slice(&buf).unwrap();
    assert_eq!(s.num(), 16);
    assert_eq!(s.den().get(), 1);
  }

  // ---- ColorInfo ----

  #[test]
  fn color_info_round_trip_default_and_nondefault() {
    let default = ColorInfo::UNSPECIFIED;
    let b = default.encode_to_vec();
    assert_eq!(ColorInfo::decode_from_slice(&b).unwrap(), default);

    let ci = ColorInfo::UNSPECIFIED
      .with_primaries(ColorPrimaries::Bt2020)
      .with_transfer(ColorTransfer::SmpteSt2084Pq)
      .with_matrix(ColorMatrix::Bt2020Ncl)
      .with_range(ColorRange::Limited)
      .with_chroma_location(ChromaLocation::Left);
    let b2 = ci.encode_to_vec();
    assert_eq!(ColorInfo::decode_from_slice(&b2).unwrap(), ci);
  }

  #[test]
  fn color_info_matrix_always_encoded_round_trips_code_zero_matrix() {
    // `ColorMatrix::Rgb` is FFmpeg code 0; `ColorInfo` always-encodes
    // all five ids as bare uint32, so a code-0 matrix survives and is
    // never conflated with an unset field.
    let ci = ColorInfo::new(
      ColorPrimaries::Unspecified,
      ColorTransfer::Unspecified,
      ColorMatrix::Rgb,
      ColorRange::Unspecified,
      ChromaLocation::Unspecified,
    );
    let b = ci.encode_to_vec();
    let back = ColorInfo::decode_from_slice(&b).unwrap();
    assert_eq!(back, ci);
    assert!(back.matrix().is_rgb());
  }

  #[test]
  fn color_info_wrong_wire_type_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(3, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <ColorInfo as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 3, expected, actual }
        if expected == VARINT && actual == LEN
    ));
    let mut ok = ColorInfo::UNSPECIFIED
      .with_range(ColorRange::Full)
      .encode_to_vec();
    Tag::new(9, WireType::Varint).encode(&mut ok);
    encode_varint(1, &mut ok);
    assert_eq!(
      <ColorInfo as Message>::decode_from_slice(&ok).unwrap(),
      ColorInfo::UNSPECIFIED.with_range(ColorRange::Full)
    );
  }

  // ---- ContentLightLevel ----

  #[test]
  fn content_light_round_trip_and_default() {
    for c in [
      ContentLightLevel::default(),
      ContentLightLevel::new(1000, 400),
      ContentLightLevel::new(0, 250),
    ] {
      let b = c.encode_to_vec();
      assert_eq!(ContentLightLevel::decode_from_slice(&b).unwrap(), c);
    }
  }

  #[test]
  fn content_light_wrong_wire_type_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <ContentLightLevel as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == VARINT && actual == LEN
    ));
    let mut ok = ContentLightLevel::new(4000, 1000).encode_to_vec();
    Tag::new(5, WireType::Varint).encode(&mut ok);
    encode_varint(9, &mut ok);
    assert_eq!(
      <ContentLightLevel as Message>::decode_from_slice(&ok).unwrap(),
      ContentLightLevel::new(4000, 1000)
    );
  }

  // ---- ChromaCoord ----

  #[test]
  fn chroma_coord_round_trip_and_default() {
    for c in [
      ChromaCoord::default(),
      cc(34000, 16000),
      cc(0, 3000),
      cc(u16::MAX as u32, u16::MAX as u32),
      // Out-of-ST 2086-range / corrupt / future producer values are
      // preserved verbatim, NOT saturated (Codex F3).
      cc(70_000, 100_000),
      cc(u32::MAX, u32::MAX - 1),
    ] {
      let b = c.encode_to_vec();
      assert_eq!(ChromaCoord::decode_from_slice(&b).unwrap(), c);
    }
  }

  // ---- MasteringDisplay ----

  #[test]
  fn mastering_display_round_trip_default_and_nondefault() {
    let default = MasteringDisplay::default();
    let b = default.encode_to_vec();
    assert_eq!(MasteringDisplay::decode_from_slice(&b).unwrap(), default);

    let md = MasteringDisplay::new(
      [cc(34000, 16000), cc(13250, 34500), cc(7500, 3000)],
      cc(15635, 16450),
      10_000_000,
      50,
    );
    let b2 = md.encode_to_vec();
    let back = MasteringDisplay::decode_from_slice(&b2).unwrap();
    assert_eq!(back, md);
    assert_eq!(back.display_primaries()[1], cc(13250, 34500));

    // Zeroed luminances elide but the always-encoded coords keep
    // round-trip exact.
    let md2 = MasteringDisplay::new([cc(1, 2), cc(3, 4), cc(5, 6)], cc(7, 8), 0, 0);
    let b3 = md2.encode_to_vec();
    assert_eq!(MasteringDisplay::decode_from_slice(&b3).unwrap(), md2);
  }

  #[test]
  fn mastering_display_wrong_wire_type_and_unknown_skip() {
    // Field 2 (primary_g) must be length-delimited.
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(2, WireType::Varint).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <MasteringDisplay as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 2, expected, actual }
        if expected == LEN && actual == VARINT
    ));
    // Field 5 (max_luminance) must be varint.
    let mut buf5: Vec<u8> = Vec::new();
    Tag::new(5, WireType::LengthDelimited).encode(&mut buf5);
    encode_varint(0, &mut buf5);
    assert!(matches!(
      <MasteringDisplay as Message>::decode_from_slice(&buf5).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 5, expected, actual }
        if expected == VARINT && actual == LEN
    ));
    let original = MasteringDisplay::new([cc(9, 9), cc(8, 8), cc(7, 7)], cc(6, 6), 123, 4);
    let mut ok = original.encode_to_vec();
    Tag::new(12, WireType::Varint).encode(&mut ok);
    encode_varint(99, &mut ok);
    assert_eq!(
      <MasteringDisplay as Message>::decode_from_slice(&ok).unwrap(),
      original
    );
  }

  // ---- HdrStaticMetadata ----

  #[test]
  fn hdr_static_metadata_round_trip_all_presence_combos() {
    let cll = ContentLightLevel::new(1000, 400);
    let md = MasteringDisplay::new(
      [cc(34000, 16000), cc(13250, 34500), cc(7500, 3000)],
      cc(15635, 16450),
      10_000_000,
      50,
    );
    for h in [
      HdrStaticMetadata::default(),                // None / None
      HdrStaticMetadata::new(Some(md), None),      // mastering only
      HdrStaticMetadata::new(None, Some(cll)),     // CLL only
      HdrStaticMetadata::new(Some(md), Some(cll)), // both
    ] {
      let b = h.encode_to_vec();
      assert_eq!(HdrStaticMetadata::decode_from_slice(&b).unwrap(), h);
    }
  }

  #[test]
  fn hdr_static_metadata_wrong_wire_type_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <HdrStaticMetadata as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == LEN && actual == VARINT
    ));
    let original = HdrStaticMetadata::new(None, Some(ContentLightLevel::new(2000, 500)));
    let mut ok = original.encode_to_vec();
    Tag::new(7, WireType::Varint).encode(&mut ok);
    encode_varint(3, &mut ok);
    assert_eq!(
      <HdrStaticMetadata as Message>::decode_from_slice(&ok).unwrap(),
      original
    );
  }

  // ---- FieldOrder ----

  #[test]
  fn field_order_round_trip() {
    // `Unknown(0)` is the default (FFmpeg `AV_FIELD_UNKNOWN`, code 0)
    // so it elides and decodes back to `Unknown(0)`. Named variants
    // and other `Unknown(n)` are non-default and round-trip via the
    // shared enum codec — lossless, no silent collapse.
    for f in [
      FieldOrder::Unknown(0),
      FieldOrder::Progressive,
      FieldOrder::Tt,
      FieldOrder::Bb,
      FieldOrder::Tb,
      FieldOrder::Bt,
      FieldOrder::Unknown(7),
      FieldOrder::Unknown(4242),
    ] {
      let b = f.encode_to_vec();
      assert_eq!(FieldOrder::decode_from_slice(&b).unwrap(), f);
    }
    // Default elides to zero bytes; empty wire decodes to default.
    assert!(FieldOrder::default().encode_to_vec().is_empty());
    assert_eq!(
      FieldOrder::decode_from_slice(&[]).unwrap(),
      FieldOrder::default()
    );
  }

  #[test]
  fn field_order_wrong_wire_type_errors() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    let err = <FieldOrder as Message>::decode_from_slice(&buf).unwrap_err();
    assert!(
      matches!(err, DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == VARINT && actual == LEN),
      "got {err:?}"
    );
  }

  // ---- StereoMode ----

  #[test]
  fn stereo_mode_round_trip() {
    // `Mono` is the default (FFmpeg `AV_STEREO3D_2D`, code 0) so it
    // elides and decodes back via `from_u32(0)` → `Mono`. Other
    // named variants and `Unknown(n)` round-trip losslessly.
    for s in [
      StereoMode::Mono,
      StereoMode::SideBySide,
      StereoMode::TopBottom,
      StereoMode::FrameSequence,
      StereoMode::Checkerboard,
      StereoMode::SideBySideQuincunx,
      StereoMode::Lines,
      StereoMode::Columns,
      StereoMode::Unknown(99),
      StereoMode::Unknown(4242),
    ] {
      let b = s.encode_to_vec();
      assert_eq!(StereoMode::decode_from_slice(&b).unwrap(), s);
    }
    // Default `Mono` elides to zero bytes; empty wire → default.
    assert!(StereoMode::default().encode_to_vec().is_empty());
    assert_eq!(
      StereoMode::decode_from_slice(&[]).unwrap(),
      StereoMode::default()
    );
  }

  #[test]
  fn stereo_mode_unknown_canonicalization() {
    // `Unknown` is decoder-only: the decoder never emits
    // `Unknown(0..=7)` (`from_u32` maps the canonical codes to their
    // named variants). Manually wrapping a canonical id in `Unknown`
    // is a misuse; it canonicalises to the named variant on a buffa
    // round-trip (correct — the id *is* that mode), never silent
    // data loss. Mirrors `dcp_target_gamut_unknown_canonicalization`.
    for (misuse, named) in [
      (StereoMode::Unknown(0), StereoMode::Mono),
      (StereoMode::Unknown(1), StereoMode::SideBySide),
      (StereoMode::Unknown(7), StereoMode::Columns),
    ] {
      let b = misuse.encode_to_vec();
      assert_eq!(StereoMode::decode_from_slice(&b).unwrap(), named);
    }
    // Non-canonical ids are preserved losslessly as `Unknown`.
    for v in [8u32, 4242, u32::MAX] {
      let u = StereoMode::Unknown(v);
      let b = u.encode_to_vec();
      assert_eq!(StereoMode::decode_from_slice(&b).unwrap(), u);
      assert_eq!(StereoMode::from_u32(v), StereoMode::Unknown(v));
    }
  }

  #[test]
  fn stereo_mode_wrong_wire_type_errors() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    let err = <StereoMode as Message>::decode_from_slice(&buf).unwrap_err();
    assert!(
      matches!(err, DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == VARINT && actual == LEN),
      "got {err:?}"
    );
  }

  // ---- Rational ----

  #[test]
  fn rational_round_trip_default_and_nondefault() {
    for r in [
      Rational::default(),            // 1/1
      Rational::new(30000, nz(1001)), // NTSC fps
      Rational::new(0, nz(1)),        // num == 0 must survive
    ] {
      let b = r.encode_to_vec();
      assert_eq!(Rational::decode_from_slice(&b).unwrap(), r);
    }
  }

  #[test]
  fn rational_field2_wrong_wire_type_errors() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_uint32(4, &mut buf);
    Tag::new(2, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <Rational as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 2, expected, actual }
        if expected == VARINT && actual == LEN
    ));
  }

  #[test]
  fn rational_den_zero_clamped_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_uint32(24, &mut buf);
    Tag::new(2, WireType::Varint).encode(&mut buf);
    encode_uint32(0, &mut buf); // malformed den == 0
    Tag::new(6, WireType::Varint).encode(&mut buf); // unknown → skip
    encode_varint(42, &mut buf);
    let r = <Rational as Message>::decode_from_slice(&buf).unwrap();
    assert_eq!(r.num(), 24);
    assert_eq!(r.den().get(), 1);
  }

  // ---- FrameRate ----

  #[test]
  fn frame_rate_round_trip_default_and_nondefault() {
    for fr in [
      FrameRate::default(),                                  // 1/1, CFR
      FrameRate::new(Rational::new(30000, nz(1001)), false), // NTSC CFR
      FrameRate::new(Rational::new(60, nz(1)), true),        // VFR avg
      FrameRate::new(Rational::new(0, nz(1)), true),         // zero rate
    ] {
      let b = fr.encode_to_vec();
      assert_eq!(FrameRate::decode_from_slice(&b).unwrap(), fr);
    }
  }

  #[test]
  fn frame_rate_wrong_wire_type_and_unknown_skip() {
    // Field 1 (rate) must be length-delimited.
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::Varint).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <FrameRate as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == LEN && actual == VARINT
    ));
    let original = FrameRate::new(Rational::new(25, nz(1)), true);
    let mut ok = original.encode_to_vec();
    Tag::new(9, WireType::Varint).encode(&mut ok);
    encode_varint(7, &mut ok);
    assert_eq!(
      <FrameRate as Message>::decode_from_slice(&ok).unwrap(),
      original
    );
  }

  // ---- DolbyVisionConfig ----

  #[test]
  fn dolby_vision_config_round_trip_default_and_nondefault() {
    for d in [
      DolbyVisionConfig::default(),
      DolbyVisionConfig::new(8, 9, true, false, 1),
      DolbyVisionConfig::new(5, 6, true, true, 2),
      DolbyVisionConfig::new(0, 0, false, true, 0), // single non-zero bool
      DolbyVisionConfig::new(255, 255, true, true, 255),
    ] {
      let b = d.encode_to_vec();
      assert_eq!(DolbyVisionConfig::decode_from_slice(&b).unwrap(), d);
    }
  }

  #[test]
  fn dolby_vision_config_wrong_wire_type_and_unknown_skip() {
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(0, &mut buf);
    assert!(matches!(
      <DolbyVisionConfig as Message>::decode_from_slice(&buf).unwrap_err(),
      DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
        if expected == VARINT && actual == LEN
    ));
    let original = DolbyVisionConfig::new(7, 4, true, false, 4);
    let mut ok = original.encode_to_vec();
    Tag::new(11, WireType::Varint).encode(&mut ok);
    encode_varint(9, &mut ok);
    assert_eq!(
      <DolbyVisionConfig as Message>::decode_from_slice(&ok).unwrap(),
      original
    );
  }

  // ---- audio + container types ----

  #[test]
  fn channel_layout_round_trip_named_and_other() {
    let v = ChannelLayout::Stereo;
    assert_eq!(
      ChannelLayout::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
    let v = ChannelLayout::N5Point1;
    assert_eq!(
      ChannelLayout::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
    let v = ChannelLayout::Other(SmolStr::new("22.2"));
    assert_eq!(
      ChannelLayout::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
    // Default (Other("")) elides to empty bytes.
    assert!(ChannelLayout::default().encode_to_vec().is_empty());
    assert_eq!(
      ChannelLayout::decode_from_slice(&[]).unwrap(),
      ChannelLayout::default()
    );
  }

  #[test]
  fn audio_container_round_trip() {
    let v = AudioContainerFormat::Mp3;
    assert_eq!(
      AudioContainerFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
    let v = AudioContainerFormat::Other(SmolStr::new("snd"));
    assert_eq!(
      AudioContainerFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
  }

  #[test]
  fn container_format_round_trip() {
    let v = ContainerFormat::Mp4;
    assert_eq!(
      ContainerFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
    let v = ContainerFormat::Threegp;
    assert_eq!(
      ContainerFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
  }

  #[test]
  fn bit_rate_mode_round_trip() {
    // Default Cbr elides to empty.
    assert!(BitRateMode::Cbr.encode_to_vec().is_empty());
    assert_eq!(
      BitRateMode::decode_from_slice(&[]).unwrap(),
      BitRateMode::Cbr
    );
    for v in [BitRateMode::Vbr, BitRateMode::Abr] {
      assert_eq!(
        BitRateMode::decode_from_slice(&v.encode_to_vec()).unwrap(),
        v
      );
    }
  }

  #[test]
  fn audio_format_round_trip_named_and_unknown() {
    // Default = Unknown(u32::MAX) (AV_SAMPLE_FMT_NONE-ish sentinel),
    // non-zero `to_u32` — but default-elision means it encodes to empty.
    assert!(AudioFormat::default().encode_to_vec().is_empty());
    assert_eq!(
      AudioFormat::decode_from_slice(&[]).unwrap(),
      AudioFormat::default()
    );
    // U8 is FFmpeg code 0 but NON-default — must be explicitly encoded.
    let b = AudioFormat::U8.encode_to_vec();
    assert!(!b.is_empty(), "non-default code-0 U8 must be encoded");
    assert_eq!(AudioFormat::decode_from_slice(&b).unwrap(), AudioFormat::U8);
    // A normal named variant.
    let b = AudioFormat::Fltp.encode_to_vec();
    assert_eq!(
      AudioFormat::decode_from_slice(&b).unwrap(),
      AudioFormat::Fltp
    );
    // Unknown(n) round-trips.
    let v = AudioFormat::Unknown(12_345);
    assert_eq!(
      AudioFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
  }

  #[test]
  fn loudness_round_trip_with_zero_elision() {
    // Default (all-zero) elides to empty.
    assert!(Loudness::default().encode_to_vec().is_empty());
    assert_eq!(
      Loudness::decode_from_slice(&[]).unwrap(),
      Loudness::default()
    );
    let l = Loudness::new(-23.0, 7.5, -1.25, -3.5);
    let b = l.encode_to_vec();
    assert_eq!(Loudness::decode_from_slice(&b).unwrap(), l);
    // Single-field set.
    let l = Loudness::default().with_true_peak_dbtp(-1.0);
    assert_eq!(Loudness::decode_from_slice(&l.encode_to_vec()).unwrap(), l);
  }

  #[test]
  fn audio_fingerprint_round_trip() {
    let fp =
      AudioFingerprint::try_new("chromaprint", ::buffa::alloc::vec![0xAA, 0xBB, 0xCC]).unwrap();
    let b = fp.encode_to_vec();
    assert_eq!(AudioFingerprint::decode_from_slice(&b).unwrap(), fp);
    // Empty value (legal) round-trips.
    let fp = AudioFingerprint::try_new("acoustid", ::buffa::alloc::vec::Vec::new()).unwrap();
    let b = fp.encode_to_vec();
    assert_eq!(AudioFingerprint::decode_from_slice(&b).unwrap(), fp);
  }

  #[test]
  fn audio_cover_art_round_trip() {
    let art = AudioCoverArt::try_new("image/jpeg", ::buffa::alloc::vec![0xFF, 0xD8, 0xFF]).unwrap();
    let b = art.encode_to_vec();
    assert_eq!(AudioCoverArt::decode_from_slice(&b).unwrap(), art);
  }

  #[test]
  fn audio_tags_round_trip() {
    let t = AudioTags::new()
      .with_title("Song")
      .with_artist("Band")
      .with_album("Album")
      .with_year(Some(1999))
      .with_track_number(Some(3))
      .with_track_total(Some(12))
      .with_language(Some(SmolStr::new("en-US")));
    let b = t.encode_to_vec();
    assert_eq!(AudioTags::decode_from_slice(&b).unwrap(), t);
    // Default round-trips.
    let t0 = AudioTags::default();
    assert!(t0.encode_to_vec().is_empty());
    assert_eq!(AudioTags::decode_from_slice(&[]).unwrap(), t0);
  }
}
