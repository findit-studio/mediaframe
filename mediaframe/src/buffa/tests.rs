use super::*;
// `mediaframe` is `#![no_std]`; `Vec` is not in the core prelude. The
// non-test impls above reach `alloc` through the always-present `buffa`
// crate (`buffa::alloc::*`); the test module does the same so it builds
// under `--no-default-features --features buffa`.
use ::buffa::alloc::vec::Vec;

fn nz(n: i64) -> NonZeroI64 {
  NonZeroI64::new(n).unwrap()
}

fn cc(x: u32, y: u32) -> ChromaCoord {
  ChromaCoord::new(x, y)
}

// ---- enums: default-elision codec (FFmpeg code points) ----
//
// For every enum: (a) `default()` encodes to ZERO bytes and
// decodes back to `default()`; (b) a non-default value whose
// `to_u32() == 0` (`Matrix::Rgb`, FFmpeg `AVCOL_SPC_RGB`)
// encodes to NON-zero bytes and round-trips — proving an absent
// field is never conflated with code-0 `Rgb`; (c) the `Other(name)`
// escape round-trips with its name intact; (d) a normal non-default
// value round-trips.

#[test]
fn enum_default_elides_to_zero_bytes() {
  // (a) Default value → empty wire → decodes back to default.
  assert!(Matrix::default().encode_to_vec().is_empty());
  assert!(Primaries::default().encode_to_vec().is_empty());
  assert!(Transfer::default().encode_to_vec().is_empty());
  assert!(DynamicRange::default().encode_to_vec().is_empty());
  assert!(ChromaLocation::default().encode_to_vec().is_empty());
  assert!(DcpTargetGamut::default().encode_to_vec().is_empty());
  assert_eq!(Matrix::decode_from_slice(&[]).unwrap(), Matrix::default());
  assert_eq!(
    Primaries::decode_from_slice(&[]).unwrap(),
    Primaries::default()
  );
  assert_eq!(
    Transfer::decode_from_slice(&[]).unwrap(),
    Transfer::default()
  );
  assert_eq!(
    DynamicRange::decode_from_slice(&[]).unwrap(),
    DynamicRange::default()
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
  // (b) `Matrix::Rgb` is FFmpeg code 0 but is NON-default, so
  // it must be explicitly encoded (non-empty) and round-trip to
  // `Rgb` — never decoded as the absent/default `Unspecified`.
  let b = Matrix::Rgb.encode_to_vec();
  assert!(!b.is_empty(), "non-default code-0 Rgb must be encoded");
  let back = Matrix::decode_from_slice(&b).unwrap();
  assert_eq!(back, Matrix::Rgb);
  assert!(back.is_rgb());
  assert_ne!(back, Matrix::default());
}

#[test]
fn enum_escape_round_trips_with_its_name() {
  // (c) A name this build does not enumerate survives encode/decode
  // for every enum — the whole point of moving the wire to the slug.
  macro_rules! rt_other {
    ($ty:ty) => {{
      let v = <$ty>::other("vendor-value-12345");
      let b = v.encode_to_vec();
      let back = <$ty>::decode_from_slice(&b).unwrap();
      assert_eq!(back, v);
      assert_eq!(back.as_str(), "vendor-value-12345");
    }};
  }
  rt_other!(Matrix);
  rt_other!(Primaries);
  rt_other!(Transfer);
  rt_other!(DynamicRange);
  rt_other!(ChromaLocation);
  rt_other!(DcpTargetGamut);
  rt_other!(PixelFormat);
}

#[test]
fn enum_non_default_round_trips() {
  // (d) A normal non-default value round-trips for every enum.
  let cm = Matrix::Bt2020Ncl.encode_to_vec();
  assert_eq!(Matrix::decode_from_slice(&cm).unwrap(), Matrix::Bt2020Ncl);
  let cp = Primaries::Bt2020.encode_to_vec();
  assert_eq!(
    Primaries::decode_from_slice(&cp).unwrap(),
    Primaries::Bt2020
  );
  let ct = Transfer::AribStdB67Hlg.encode_to_vec();
  assert_eq!(
    Transfer::decode_from_slice(&ct).unwrap(),
    Transfer::AribStdB67Hlg
  );
  let dg = DcpTargetGamut::Rec2020.encode_to_vec();
  assert_eq!(
    DcpTargetGamut::decode_from_slice(&dg).unwrap(),
    DcpTargetGamut::Rec2020
  );
}

#[test]
fn dcp_target_gamut_escape_canonicalization() {
  // Codex adversarial-review F8, restated for the name-shaped wire.
  // Spelling a *named* gamut through the escape is a misuse; `Self::other`
  // now heals it to the named variant directly (no wire round-trip
  // needed — the equality-heals fixture), and it still canonicalises on
  // a round-trip regardless (correct — the name *is* that gamut), never
  // silent data loss.
  for (misuse, named) in [
    (DcpTargetGamut::other("dci-p3"), DcpTargetGamut::DciP3),
    (DcpTargetGamut::other("rec709"), DcpTargetGamut::Rec709),
    (DcpTargetGamut::other("rec2020"), DcpTargetGamut::Rec2020),
  ] {
    assert_eq!(
      misuse, named,
      "other() must heal to the named variant directly"
    );
    let b = misuse.encode_to_vec();
    assert_eq!(DcpTargetGamut::decode_from_slice(&b).unwrap(), named);
  }
  // A name this build does not enumerate is preserved verbatim (and
  // still F7-rejected by `xyz12_to`).
  for name in ["aces-ap0", "vendor-gamut", "rec2100"] {
    let u = DcpTargetGamut::other(name);
    let b = u.encode_to_vec();
    assert_eq!(DcpTargetGamut::decode_from_slice(&b).unwrap(), u);
  }
}

#[test]
fn color_matrix_bt601_domain_variant_round_trips() {
  // `Matrix::Bt601` is a mediaframe-domain id
  // (`DOMAIN_EXT_BASE` = 0x8000_0000), non-default, so it must be
  // explicitly encoded to NON-zero bytes and round-trip losslessly
  // via the `Message` impl (string carrying `"bt601"`).
  let b = Matrix::Bt601.encode_to_vec();
  assert!(!b.is_empty(), "non-default domain Bt601 must be encoded");
  let back = Matrix::decode_from_slice(&b).unwrap();
  assert_eq!(back, Matrix::Bt601);
  assert!(back.is_bt_601());
  assert_ne!(back, Matrix::default());
  // Default `Unspecified` still elides to zero bytes.
  assert!(Matrix::default().encode_to_vec().is_empty());
  assert_eq!(Matrix::decode_from_slice(&[]).unwrap(), Matrix::default());
}

#[test]
fn color_matrix_default_instance_and_clear() {
  assert_eq!(
    *<Matrix as DefaultInstance>::default_instance(),
    Matrix::default()
  );
  let mut m = Matrix::YCgCo;
  Message::clear(&mut m);
  assert_eq!(m, Matrix::default());
}

#[test]
fn color_range_round_trip() {
  for r in [
    DynamicRange::Unspecified,
    DynamicRange::Limited,
    DynamicRange::Full,
  ] {
    let b = r.encode_to_vec();
    assert_eq!(DynamicRange::decode_from_slice(&b).unwrap(), r);
  }
}

#[test]
fn rotation_round_trip() {
  // `D0` is the default so it elides. `Other(name)` preserves an
  // unrecognised / future rotation name losslessly through the shared
  // enum codec — no silent collapse to `D0` (Codex adversarial-review
  // F1).
  for r in [
    Rotation::D0,
    Rotation::D90,
    Rotation::D180,
    Rotation::D270,
    Rotation::other("45"),
    Rotation::other("vendor-tilt"),
  ] {
    let b = r.encode_to_vec();
    assert_eq!(Rotation::decode_from_slice(&b).unwrap(), r);
  }
}

#[test]
fn enum_wrong_wire_type_errors() {
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::Varint).encode(&mut buf);
  encode_varint(0, &mut buf);
  let err = <Matrix as Message>::decode_from_slice(&buf).unwrap_err();
  assert!(
    matches!(err, DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
      if expected == LEN && actual == VARINT),
    "got {err:?}"
  );
}

#[test]
fn enum_unknown_field_is_skipped() {
  let mut buf = DynamicRange::Full.encode_to_vec();
  Tag::new(7, WireType::Varint).encode(&mut buf); // unknown → skip
  encode_varint(123, &mut buf);
  assert_eq!(
    <DynamicRange as Message>::decode_from_slice(&buf).unwrap(),
    DynamicRange::Full
  );
}

#[test]
fn enum_unknown_name_decodes_losslessly() {
  // An unrecognised on-wire name decodes to `Other(name)` (no silent
  // collapse to `default()`), preserving the value a newer producer
  // wrote.
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
  encode_string("st-2065-1", &mut buf);
  assert_eq!(
    <Transfer as Message>::decode_from_slice(&buf).unwrap(),
    Transfer::other("st-2065-1")
  );
}

#[test]
fn pixel_format_round_trip_including_escape() {
  for p in [
    PixelFormat::Yuv420p,
    PixelFormat::default(), // `None` → elided → `None`
    PixelFormat::other("vendor_raw12"),
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
// `buffa` extern target whose representation has changed twice —
// newtype over `Rational` in 0.3.1, then `Rational`'s halves going
// `u32`/`NonZeroU32` → `i64`/`NonZeroI64` in 0.2.0 — and the wire
// encoding MUST stay identical to 0.3.0 through both. These exact
// bytes were written when the schema said `uint32`; they are what
// `int64` must still produce, which holds because protobuf's
// `int64` and `uint32` are the same plain non-ZigZag varint over
// the non-negative range a SAR can hold. (`sint64` would have been
// the silent break — ZigZag re-encodes every value.)
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

// The same guard from the reader's side: bytes a `uint32`-era peer
// produced must still decode to the same value under `int64`.
#[test]
fn sar_decodes_uint32_era_bytes() {
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::Varint).encode(&mut buf);
  encode_uint32(u32::MAX, &mut buf); // widest a uint32 peer can write
  Tag::new(2, WireType::Varint).encode(&mut buf);
  encode_uint32(1001, &mut buf);
  let s = <SampleAspectRatio as Message>::decode_from_slice(&buf).unwrap();
  assert_eq!(s.num(), i64::from(u32::MAX));
  assert_eq!(s.den().get(), 1001);
  // And re-encoding those values reproduces the same bytes.
  assert_eq!(s.encode_to_vec(), buf);
}

#[test]
fn sar_negative_fields_clamped() {
  // Not producible by this encoder — a peer reaches the negative
  // half either directly or by writing a `uint64` above `i64::MAX`.
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::Varint).encode(&mut buf);
  encode_int64(-7, &mut buf);
  Tag::new(2, WireType::Varint).encode(&mut buf);
  encode_int64(-9, &mut buf);
  let s = <SampleAspectRatio as Message>::decode_from_slice(&buf).unwrap();
  assert_eq!(s.num(), 0);
  assert_eq!(s.den().get(), 1);
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

// ---- Info ----

#[test]
fn color_info_round_trip_default_and_nondefault() {
  let default = Info::UNSPECIFIED;
  let b = default.encode_to_vec();
  assert_eq!(Info::decode_from_slice(&b).unwrap(), default);

  let ci = Info::UNSPECIFIED
    .with_primaries(Primaries::Bt2020)
    .with_transfer(Transfer::SmpteSt2084Pq)
    .with_matrix(Matrix::Bt2020Ncl)
    .with_range(DynamicRange::Limited)
    .with_chroma_location(ChromaLocation::Left);
  let b2 = ci.encode_to_vec();
  assert_eq!(Info::decode_from_slice(&b2).unwrap(), ci);
}

#[test]
fn color_info_matrix_always_encoded_round_trips_code_zero_matrix() {
  // `Matrix::Rgb` is FFmpeg code 0; `Info` always-encodes
  // all five ids as bare uint32, so a code-0 matrix survives and is
  // never conflated with an unset field.
  let ci = Info::new(
    Primaries::Unspecified,
    Transfer::Unspecified,
    Matrix::Rgb,
    DynamicRange::Unspecified,
    ChromaLocation::Unspecified,
  );
  let b = ci.encode_to_vec();
  let back = Info::decode_from_slice(&b).unwrap();
  assert_eq!(back, ci);
  assert!(back.matrix().is_rgb());
}

#[test]
fn color_info_wrong_wire_type_and_unknown_skip() {
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(3, WireType::Varint).encode(&mut buf);
  encode_varint(0, &mut buf);
  assert!(matches!(
    <Info as Message>::decode_from_slice(&buf).unwrap_err(),
    DecodeError::WireTypeMismatch { field_number: 3, expected, actual }
      if expected == LEN && actual == VARINT
  ));
  let mut ok = Info::UNSPECIFIED
    .with_range(DynamicRange::Full)
    .encode_to_vec();
  Tag::new(9, WireType::Varint).encode(&mut ok);
  encode_varint(1, &mut ok);
  assert_eq!(
    <Info as Message>::decode_from_slice(&ok).unwrap(),
    Info::UNSPECIFIED.with_range(DynamicRange::Full)
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
  // `Unknown` is the default (FFmpeg's own `AV_FIELD_UNKNOWN`) so it
  // elides and decodes back to itself. Named variants and the
  // `Other(name)` escape are non-default and round-trip via the shared
  // enum codec — lossless, no silent collapse.
  for f in [
    FieldOrder::Unknown,
    FieldOrder::Progressive,
    FieldOrder::Tt,
    FieldOrder::Bb,
    FieldOrder::Tb,
    FieldOrder::Bt,
    FieldOrder::other("segmented-frame"),
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
  Tag::new(1, WireType::Varint).encode(&mut buf);
  encode_varint(0, &mut buf);
  let err = <FieldOrder as Message>::decode_from_slice(&buf).unwrap_err();
  assert!(
    matches!(err, DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
      if expected == LEN && actual == VARINT),
    "got {err:?}"
  );
}

// ---- StereoMode ----

#[test]
fn stereo_mode_round_trip() {
  // `Mono` is the default (FFmpeg `AV_STEREO3D_2D`) so it elides and
  // decodes back to itself. Other named variants and the
  // `Other(name)` escape round-trip losslessly.
  for s in [
    StereoMode::Mono,
    StereoMode::SideBySide,
    StereoMode::TopBottom,
    StereoMode::FrameSequence,
    StereoMode::Checkerboard,
    StereoMode::SideBySideQuincunx,
    StereoMode::Lines,
    StereoMode::Columns,
    StereoMode::other("anaglyph"),
    StereoMode::other("vendor-packing"),
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
fn stereo_mode_escape_canonicalization() {
  // Spelling a *named* mode through the escape is a misuse; `Self::other`
  // now heals it to the named variant directly (no wire round-trip
  // needed — the equality-heals fixture), and it still canonicalises on
  // a round-trip regardless (correct — the name *is* that mode), never
  // silent data loss. Mirrors `dcp_target_gamut_escape_canonicalization`.
  for (misuse, named) in [
    (StereoMode::other("mono"), StereoMode::Mono),
    (StereoMode::other("side-by-side"), StereoMode::SideBySide),
    (StereoMode::other("columns"), StereoMode::Columns),
  ] {
    assert_eq!(
      misuse, named,
      "other() must heal to the named variant directly"
    );
    let b = misuse.encode_to_vec();
    assert_eq!(StereoMode::decode_from_slice(&b).unwrap(), named);
  }
  // A name this build does not enumerate is preserved verbatim.
  for name in ["anaglyph", "vendor-packing", "interleaved-quincunx"] {
    let u = StereoMode::other(name);
    let b = u.encode_to_vec();
    assert_eq!(StereoMode::decode_from_slice(&b).unwrap(), u);
  }
}

#[test]
fn stereo_mode_wrong_wire_type_errors() {
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::Varint).encode(&mut buf);
  encode_varint(0, &mut buf);
  let err = <StereoMode as Message>::decode_from_slice(&buf).unwrap_err();
  assert!(
    matches!(err, DecodeError::WireTypeMismatch { field_number: 1, expected, actual }
      if expected == LEN && actual == VARINT),
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

#[test]
fn rational_negative_fields_clamped() {
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::Varint).encode(&mut buf);
  encode_int64(-1, &mut buf);
  Tag::new(2, WireType::Varint).encode(&mut buf);
  encode_int64(i64::MIN, &mut buf);
  let r = <Rational as Message>::decode_from_slice(&buf).unwrap();
  assert_eq!(r.num(), 0);
  assert_eq!(r.den().get(), 1);
}

// The width the `i64` change buys, end to end on the wire.
#[test]
fn rational_round_trips_above_u32_max() {
  let big = i64::from(u32::MAX) + 1;
  let r = Rational::new(big, nz(i64::MAX));
  let b = r.encode_to_vec();
  assert_eq!(Rational::decode_from_slice(&b).unwrap(), r);
  let fr = FrameRate::new(r, true);
  let b = fr.encode_to_vec();
  assert_eq!(FrameRate::decode_from_slice(&b).unwrap(), fr);
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
  let v = ChannelLayout::Ch5_1;
  assert_eq!(
    ChannelLayout::decode_from_slice(&v.encode_to_vec()).unwrap(),
    v
  );
  // A slug outside the roster. `"22.2"` stood here until 0.6.0 named
  // it: the wire form is the slug, so an escape carrying a slug the
  // reader now recognises comes back as the *named* variant, not as the
  // `Other` it was encoded from. Lossless on the wire, a different value
  // in memory. This one is FFmpeg's rendering of a layout its map has no
  // name for, so it cannot be promoted out of the escape by a later
  // release the way `"22.2"` was.
  let v = ChannelLayout::Other(Utf8Bytes::from("fl+fr+tfl"));
  assert_eq!(
    ChannelLayout::decode_from_slice(&v.encode_to_vec()).unwrap(),
    v
  );
  assert_eq!(
    ChannelLayout::decode_from_slice(
      &ChannelLayout::Other(Utf8Bytes::from("22.2")).encode_to_vec()
    )
    .unwrap(),
    ChannelLayout::Ch22_2
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
  let v = ContainerFormat::Mp3;
  assert_eq!(
    ContainerFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
    v
  );
  let v = ContainerFormat::Other(Utf8Bytes::from("snd"));
  assert_eq!(
    ContainerFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
    v
  );
}

#[test]
fn container_format_round_trip() {
  let v = Format::Mp4;
  assert_eq!(Format::decode_from_slice(&v.encode_to_vec()).unwrap(), v);
  let v = Format::Threegp;
  assert_eq!(Format::decode_from_slice(&v.encode_to_vec()).unwrap(), v);
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
fn channel_order_round_trip() {
  // Default `Unspecified` is code 0, so zero-elision is exact: it
  // encodes to nothing and an absent field decodes back to it.
  assert!(ChannelOrder::Unspecified.encode_to_vec().is_empty());
  assert_eq!(
    ChannelOrder::decode_from_slice(&[]).unwrap(),
    ChannelOrder::Unspecified
  );
  for &v in ChannelOrder::ROSTER {
    assert_eq!(
      ChannelOrder::decode_from_slice(&v.encode_to_vec()).unwrap(),
      v
    );
  }
}

#[test]
fn channel_order_wrong_wire_type_and_unknown_skip() {
  // Field 1 is a varint; a length-delimited payload there is a
  // mismatch, not something to guess at.
  let mut bad = Vec::new();
  Tag::new(1, WireType::LengthDelimited).encode(&mut bad);
  encode_string("native", &mut bad);
  assert!(ChannelOrder::decode_from_slice(&bad).is_err());

  // An unknown field is skipped, leaving the known one intact.
  let mut buf = ChannelOrder::Custom.encode_to_vec();
  Tag::new(9, WireType::Varint).encode(&mut buf);
  encode_uint32(7, &mut buf);
  assert_eq!(
    ChannelOrder::decode_from_slice(&buf).unwrap(),
    ChannelOrder::Custom
  );
}

#[test]
fn channel_spec_round_trip_with_zero_elision() {
  // Default (0, 0, "") is proto-zero throughout → empty wire.
  assert!(ChannelSpec::default().encode_to_vec().is_empty());
  assert_eq!(
    ChannelSpec::decode_from_slice(&[]).unwrap(),
    ChannelSpec::default()
  );
  for spec in [
    ChannelSpec::new(0, 1),
    ChannelSpec::new(3, 0).with_label("LFE"),
    ChannelSpec::new(7, 11).with_label("TBC"),
  ] {
    assert_eq!(
      ChannelSpec::decode_from_slice(&spec.encode_to_vec()).unwrap(),
      spec
    );
  }
}

#[test]
fn channel_layout_description_round_trip_default_and_populated() {
  // Every seat at its absent value is proto-zero, and `native_mask` is
  // `None` and so simply absent → empty wire.
  assert!(
    ChannelLayoutDescription::default()
      .encode_to_vec()
      .is_empty()
  );
  assert_eq!(
    ChannelLayoutDescription::decode_from_slice(&[]).unwrap(),
    ChannelLayoutDescription::default()
  );

  let native = ChannelLayoutDescription::new(6)
    .with_order(ChannelOrder::Native)
    .with_known_kind(ChannelLayout::Ch5_1Back)
    .with_native_mask(Some(0x3F))
    .with_text("5.1");
  assert_eq!(
    ChannelLayoutDescription::decode_from_slice(&native.encode_to_vec()).unwrap(),
    native
  );

  let custom = ChannelLayoutDescription::new(3)
    .with_order(ChannelOrder::Custom)
    .with_custom_channels(::std::vec![
      ChannelSpec::new(0, 1).with_label("FL"),
      ChannelSpec::new(1, 2).with_label("FR"),
      ChannelSpec::new(2, 3).with_label("LFE"),
    ])
    .with_text("3 channels (FL+FR+LFE)");
  assert_eq!(
    ChannelLayoutDescription::decode_from_slice(&custom.encode_to_vec()).unwrap(),
    custom
  );
}

/// A repeated field is the crate's first, so the two properties that
/// distinguish it from a scalar are pinned directly: order is preserved,
/// and an element that is itself all-default still occupies a slot
/// rather than vanishing into the elision that would swallow a scalar.
#[test]
fn channel_layout_description_repeated_field_keeps_order_and_empty_elements() {
  let d = ChannelLayoutDescription::new(4).with_custom_channels(::std::vec![
    ChannelSpec::new(0, 9).with_label("a"),
    ChannelSpec::default(),
    ChannelSpec::new(2, 0).with_label("c"),
    ChannelSpec::default(),
  ]);
  let back = ChannelLayoutDescription::decode_from_slice(&d.encode_to_vec()).unwrap();
  assert_eq!(back, d);
  assert_eq!(back.custom_channels().len(), 4);
  assert_eq!(back.custom_channels()[1], ChannelSpec::default());
  assert_eq!(back.custom_channels()[2].label(), "c");
}

/// `Some(0)` and `None` are different facts — a layout that reports an
/// all-zero mask against one that reports no mask at all — so the
/// optional field uses presence encoding rather than zero-elision.
#[test]
fn channel_layout_description_distinguishes_a_zero_mask_from_no_mask() {
  let zero = ChannelLayoutDescription::new(2).with_native_mask(Some(0));
  let none = ChannelLayoutDescription::new(2);
  assert_ne!(zero, none);
  assert_ne!(zero.encode_to_vec(), none.encode_to_vec());
  assert_eq!(
    ChannelLayoutDescription::decode_from_slice(&zero.encode_to_vec()).unwrap(),
    zero
  );
  assert_eq!(
    ChannelLayoutDescription::decode_from_slice(&none.encode_to_vec()).unwrap(),
    none
  );
}

/// The name field carries an open vocabulary, so a slug this build does
/// not enumerate arrives as `Other(name)` rather than as a decode error
/// — the same rule the standalone `ChannelLayout` message follows,
/// applied to a field.
#[test]
fn channel_layout_description_name_field_keeps_an_unknown_slug() {
  let d = ChannelLayoutDescription::new(64).with_known_kind(ChannelLayout::other("64.4.8"));
  let back = ChannelLayoutDescription::decode_from_slice(&d.encode_to_vec()).unwrap();
  assert_eq!(back, d);
  assert_eq!(back.known_kind().as_str(), "64.4.8");
}

#[test]
fn channel_layout_description_wrong_wire_type_and_unknown_skip() {
  // Field 2 (`channels`) is a varint; a length-delimited payload there
  // is a mismatch.
  let mut bad = Vec::new();
  Tag::new(2, WireType::LengthDelimited).encode(&mut bad);
  encode_string("6", &mut bad);
  assert!(ChannelLayoutDescription::decode_from_slice(&bad).is_err());

  // Field 5 (`custom_channels`) is length-delimited; a varint there is
  // a mismatch too.
  let mut bad = Vec::new();
  Tag::new(5, WireType::Varint).encode(&mut bad);
  encode_uint32(1, &mut bad);
  assert!(ChannelLayoutDescription::decode_from_slice(&bad).is_err());

  // An unknown field is skipped, leaving the known ones intact.
  let d = ChannelLayoutDescription::new(6).with_order(ChannelOrder::Native);
  let mut buf = d.encode_to_vec();
  Tag::new(11, WireType::Varint).encode(&mut buf);
  encode_uint32(7, &mut buf);
  assert_eq!(
    ChannelLayoutDescription::decode_from_slice(&buf).unwrap(),
    d
  );
}

#[test]
fn audio_format_round_trip_named_and_unknown() {
  // Default = Unknown(u32::MAX) (AV_SAMPLE_FMT_NONE-ish sentinel),
  // non-zero `to_u32` — but default-elision means it encodes to empty.
  assert!(SampleFormat::default().encode_to_vec().is_empty());
  assert_eq!(
    SampleFormat::decode_from_slice(&[]).unwrap(),
    SampleFormat::default()
  );
  // U8 is FFmpeg code 0 but NON-default — must be explicitly encoded.
  let b = SampleFormat::U8.encode_to_vec();
  assert!(!b.is_empty(), "non-default code-0 U8 must be encoded");
  assert_eq!(
    SampleFormat::decode_from_slice(&b).unwrap(),
    SampleFormat::U8
  );
  // A normal named variant.
  let b = SampleFormat::Fltp.encode_to_vec();
  assert_eq!(
    SampleFormat::decode_from_slice(&b).unwrap(),
    SampleFormat::Fltp
  );
  // The escape round-trips with its name.
  let v = SampleFormat::other("vendor_s24");
  assert_eq!(
    SampleFormat::decode_from_slice(&v.encode_to_vec()).unwrap(),
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
  let fp = Fingerprint::try_new("chromaprint", ::buffa::alloc::vec![0xAA, 0xBB, 0xCC]).unwrap();
  let b = fp.encode_to_vec();
  assert_eq!(Fingerprint::decode_from_slice(&b).unwrap(), fp);
  // Empty value (legal) round-trips.
  let fp = Fingerprint::try_new("acoustid", ::buffa::alloc::vec::Vec::new()).unwrap();
  let b = fp.encode_to_vec();
  assert_eq!(Fingerprint::decode_from_slice(&b).unwrap(), fp);
}

#[test]
fn audio_cover_art_round_trip() {
  let art = CoverArt::try_new("image/jpeg", ::buffa::alloc::vec![0xFF, 0xD8, 0xFF]).unwrap();
  let b = art.encode_to_vec();
  assert_eq!(CoverArt::decode_from_slice(&b).unwrap(), art);
}

#[test]
fn audio_tags_round_trip() {
  let t = Tags::new()
    .with_title("Song")
    .with_artist("Band")
    .with_album("Album")
    .with_year(1999)
    .with_track_number(3)
    .with_track_total(12)
    .with_language(crate::lang::LanguageId::new("en-US").unwrap());
  let b = t.encode_to_vec();
  assert_eq!(Tags::decode_from_slice(&b).unwrap(), t);
  // Default round-trips.
  let t0 = Tags::default();
  assert!(t0.encode_to_vec().is_empty());
  assert_eq!(Tags::decode_from_slice(&[]).unwrap(), t0);
  // A numeric `0` is the absent sentinel — `with_year(0)` is identical to
  // never-set, encodes to nothing, and decodes back identically (proto3
  // zero-elision; type + codec now agree).
  let z = Tags::new().with_year(0).with_track_number(0);
  assert_eq!(z, Tags::default());
  assert_eq!(Tags::decode_from_slice(&z.encode_to_vec()).unwrap(), z);
}

// ---- Capture + language wire round-trips ----
//
// These types live behind the `alloc` gate; the tests do too.

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn device_round_trip_empty_and_populated() {
  use crate::capture::Device;

  // Default = both empty → zero bytes encoded → decodes back to default.
  assert!(Device::default().encode_to_vec().is_empty());
  assert_eq!(Device::decode_from_slice(&[]).unwrap(), Device::default());

  // Populated round-trip.
  let d = Device::new().with_make("Apple").with_model("iPhone 15 Pro");
  let b = d.encode_to_vec();
  assert_eq!(Device::decode_from_slice(&b).unwrap(), d);

  // Make-only / model-only round-trips.
  let m = Device::new().with_make("Sony");
  assert_eq!(Device::decode_from_slice(&m.encode_to_vec()).unwrap(), m);
  let n = Device::new().with_model("ILCE-7M4");
  assert_eq!(Device::decode_from_slice(&n.encode_to_vec()).unwrap(), n);
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn geo_location_round_trip_null_island_and_populated() {
  use crate::capture::GeoLocation;

  // Null Island (the default) — always-encoded lat+lon means it does
  // NOT collapse to zero bytes (defensive non-zero-elision stance).
  let null = GeoLocation::default();
  let b = null.encode_to_vec();
  assert!(!b.is_empty());
  assert_eq!(GeoLocation::decode_from_slice(&b).unwrap(), null);

  // Paris, no altitude.
  let paris = GeoLocation::try_new(48.8566, 2.3522, None).unwrap();
  let b = paris.encode_to_vec();
  let back = GeoLocation::decode_from_slice(&b).unwrap();
  assert!((back.lat() - paris.lat()).abs() < 1e-9);
  assert!((back.lon() - paris.lon()).abs() < 1e-9);
  assert!(back.altitude().is_none());

  // Paris with explicit Some(0.0) altitude — sea level — must round
  // trip distinct from None.
  let sea = GeoLocation::try_new(48.8566, 2.3522, Some(0.0)).unwrap();
  let b = sea.encode_to_vec();
  let back = GeoLocation::decode_from_slice(&b).unwrap();
  assert_eq!(back.altitude(), Some(0.0));

  // São Paulo with +760 m altitude.
  let sp = GeoLocation::try_new(-23.5505, -46.6333, Some(760.0)).unwrap();
  let b = sp.encode_to_vec();
  let back = GeoLocation::decode_from_slice(&b).unwrap();
  assert!((back.lat() - sp.lat()).abs() < 1e-9);
  assert!((back.lon() - sp.lon()).abs() < 1e-9);
  assert_eq!(back.altitude(), Some(760.0));
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn language_round_trip_und_and_populated() {
  use crate::lang::LanguageId;

  // Default = "und"; a non-empty tag, so it IS encoded.
  let und = LanguageId::default();
  let b = und.encode_to_vec();
  assert!(!b.is_empty());
  assert_eq!(LanguageId::decode_from_slice(&b).unwrap(), und);
  // An absent field on the wire (empty buffer) re-seeds to default.
  assert_eq!(LanguageId::decode_from_slice(&[]).unwrap(), und);

  // The last two are the seats the retired triple could not carry: a variant
  // and a private-use sequence, both held verbatim through the round trip.
  for tag in ["en", "en-US", "zh-Hant-TW", "de-CH-1901", "en-US-x-lorem"] {
    let l = LanguageId::new(tag).unwrap();
    let b = l.encode_to_vec();
    assert_eq!(LanguageId::decode_from_slice(&b).unwrap(), l);
    assert_eq!(l.to_string(), tag, "the wire form is the canonical tag");
  }
}

/// **A FOLD THAT WOULD NOT REPARSE NEVER FIRES**, which this codec is the
/// other surface for: the encode is the rendering and the decode is the door,
/// so a canonicalisation whose text read back as a DIFFERENT value would
/// rewrite a stored identity on every read.
///
/// `en-Latn-Cyrl` holds `Latn` on the script seat and `Cyrl` on the lossless
/// tail. Suppressing `Latn` would put `en-Cyrl` on the wire, and the decode
/// door reads that as the SCRIPT `Cyrl` with an empty tail — so the
/// suppression is skipped and the wire form keeps the script.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn language_wire_carries_a_tag_the_suppression_must_not_fold() {
  use crate::lang::LanguageId;

  for tag in ["en-Latn-Cyrl", "en-Latn-Latn"] {
    let l = LanguageId::new(tag).unwrap();
    assert_eq!(
      l.to_string(),
      tag,
      "the script is retained in the wire form"
    );

    let b = l.encode_to_vec();
    assert_eq!(LanguageId::decode_from_slice(&b).unwrap(), l);
  }

  // …and the fold still fires wherever it is reversible.
  let folded = LanguageId::new("en-Latn-US").unwrap();
  assert_eq!(folded.to_string(), "en-US");
  assert_eq!(
    LanguageId::decode_from_slice(&folded.encode_to_vec()).unwrap(),
    folded
  );
}

/// **A TAG THAT BECOMES GRANDFATHERED THROUGH ANOTHER FOLD IS FOLDED ANYWAY**,
/// and this codec is the surface where stopping short would cost most: the
/// encode is the rendering and the decode is the door, so a value whose text
/// the whole-tag table still folds would come back a DIFFERENT identity on its
/// first read and be stable ever after.
///
/// None of the four is grandfathered as written — `en-Latn-GB-oed` becomes one
/// when the suppression drops `Latn`, the other three when the alpha-3 fold
/// rewrites their language subtag — so the wire form is the tag the fold
/// reaches, and a wire carrying the MIDDLE spelling decodes to the same value.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn language_wire_folds_a_tag_that_becomes_grandfathered_through_another_fold() {
  use crate::lang::LanguageId;

  for (sent, canonical) in [
    ("en-Latn-GB-oed", "en-GB-oxendict"),
    ("eng-GB-oed", "en-GB-oxendict"),
    ("nor-bok", "nb"),
    ("zho-guoyu", "cmn"),
  ] {
    let held = LanguageId::new(sent).unwrap();
    assert_eq!(held.to_string(), canonical, "`{sent}` — the wire form");
    assert_eq!(
      LanguageId::decode_from_slice(&held.encode_to_vec()).unwrap(),
      held,
      "`{sent}`"
    );

    // …and the middle spelling, which another writer's wire can carry, decodes
    // to that same identity rather than to one the next read would move.
    let mut buf: Vec<u8> = Vec::new();
    Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
    encode_varint(sent.len() as u64, &mut buf);
    buf.extend_from_slice(sent.as_bytes());
    assert_eq!(
      LanguageId::decode_from_slice(&buf).unwrap(),
      held,
      "`{sent}` — the decode door stopped short"
    );
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn language_wire_garbage_falls_back_to_und() {
  use crate::lang::LanguageId;

  // A wire string the whole-tag door REFUSES silently coerces to
  // `LanguageId::default()` (= "und") — see the rationale in the
  // module-level doc.
  //
  // The refusal has to be STRUCTURAL, which is the fallback path narrowing
  // with the door widening: `xx-yy-zz-bogus` was this test's garbage under
  // the retired icu triple and is a perfectly good tag now (an unregistered
  // language `xx`, the region `YY`, and `zz-bogus` on the lossless tail).
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
  encode_varint("en-US-!!".len() as u64, &mut buf);
  buf.extend_from_slice("en-US-!!".as_bytes());
  assert_eq!(
    LanguageId::decode_from_slice(&buf).unwrap(),
    LanguageId::default()
  );

  // …and the tag that used to be garbage now survives the wire whole.
  let mut buf: Vec<u8> = Vec::new();
  Tag::new(1, WireType::LengthDelimited).encode(&mut buf);
  encode_varint("xx-yy-zz-bogus".len() as u64, &mut buf);
  buf.extend_from_slice("xx-yy-zz-bogus".as_bytes());
  assert_eq!(
    LanguageId::decode_from_slice(&buf).unwrap(),
    LanguageId::new("xx-YY-zz-bogus").unwrap()
  );
}
