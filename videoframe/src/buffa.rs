//! `buffa::Message` implementations for the videoframe wire-relevant
//! types, behind the `buffa` feature. Used via `extern_path` from
//! buffa-generated crates so a `.proto`-defined message can embed a
//! videoframe type without redefining it.
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

use core::num::NonZeroU32;

use ::buffa::{
  DecodeError, DefaultInstance, Message, SizeCache,
  bytes::{Buf, BufMut},
  encoding::{Tag, WireType, encode_varint, skip_field_depth, varint_len},
  types::{decode_uint32, encode_uint32, uint32_encoded_len},
};

use crate::{
  color::{
    ChromaCoord, ChromaLocation, ColorInfo, ColorMatrix, ColorPrimaries, ColorRange,
    ColorTransfer, ContentLightLevel, DcpTargetGamut, HdrStaticMetadata, MasteringDisplay,
  },
  frame::{Dimensions, Rect, Rotation, SampleAspectRatio},
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

impl_enum_message!(ColorMatrix, |s: &ColorMatrix| s.to_u32(), ColorMatrix::from_u32);
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
impl_enum_message!(ColorRange, |s: &ColorRange| s.to_u32(), ColorRange::from_u32);
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
// `PixelFormat::to_u32` consumes `self` (it is `Copy`); `from_u32`
// maps unrecognised ids to `Unknown(n)` so the round-trip is
// lossless even for the elided-default case (`Unknown(0)` is the
// `Default`, so it elides and decodes back to `Unknown(0)`).
impl_enum_message!(PixelFormat, |s: &PixelFormat| s.to_u32(), PixelFormat::from_u32);

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
        // videoframe-only divergence.
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

#[cfg(test)]
mod tests {
  use super::*;
  // `videoframe` is `#![no_std]`; `Vec` is not in the core prelude. The
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
    // `ColorMatrix::Bt601` is a videoframe-domain id
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
    for r in [ColorRange::Unspecified, ColorRange::Limited, ColorRange::Full] {
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
    let md2 = MasteringDisplay::new(
      [cc(1, 2), cc(3, 4), cc(5, 6)],
      cc(7, 8),
      0,
      0,
    );
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
    let original = MasteringDisplay::new(
      [cc(9, 9), cc(8, 8), cc(7, 7)],
      cc(6, 6),
      123,
      4,
    );
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
      HdrStaticMetadata::default(),                       // None / None
      HdrStaticMetadata::new(Some(md), None),             // mastering only
      HdrStaticMetadata::new(None, Some(cll)),            // CLL only
      HdrStaticMetadata::new(Some(md), Some(cll)),        // both
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
    let original =
      HdrStaticMetadata::new(None, Some(ContentLightLevel::new(2000, 500)));
    let mut ok = original.encode_to_vec();
    Tag::new(7, WireType::Varint).encode(&mut ok);
    encode_varint(3, &mut ok);
    assert_eq!(
      <HdrStaticMetadata as Message>::decode_from_slice(&ok).unwrap(),
      original
    );
  }
}
