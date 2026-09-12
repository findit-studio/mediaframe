use crate::{
  audio::{ChannelLayout, CoverArt, Fingerprint, Tags},
  capture::GeoLocation,
  codec::VideoCodec,
  color::{self, Matrix},
  disposition::TrackDisposition,
  frame::{Dimensions, FrameRate, Rational, SampleAspectRatio},
  lang::{Language, LanguageId, Region, ScriptSubtag},
};

fn round_trip<T>(v: &T) -> T
where
  T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
{
  let json = serde_json::to_string(v).unwrap();
  let back: T = serde_json::from_str(&json).unwrap();
  assert_eq!(*v, back, "round-trip mismatch via {json}");
  back
}

#[test]
fn open_enum_serializes_as_slug() {
  assert_eq!(
    serde_json::to_string(&VideoCodec::H264).unwrap(),
    "\"h264\""
  );
  round_trip(&VideoCodec::H264);
  // Unknown slug rides the `Other` arm losslessly.
  let custom = VideoCodec::Other(smol_bytes::Utf8Bytes::from("zzcodec"));
  assert_eq!(serde_json::to_string(&custom).unwrap(), "\"zzcodec\"");
  round_trip(&custom);
  round_trip(&ChannelLayout::default());
}

#[test]
fn colour_enum_serializes_as_its_name() {
  assert_eq!(serde_json::to_string(&Matrix::Bt709).unwrap(), "\"bt709\"");
  round_trip(&Matrix::Bt709);
  // A name this build does not enumerate rides the `Other` arm, and
  // keeps its name across the wire — the old numeric shape handed the
  // reader a bare code with nothing to call it.
  let vendor = Matrix::other("acescct");
  assert_eq!(serde_json::to_string(&vendor).unwrap(), "\"acescct\"");
  round_trip(&vendor);
}

#[test]
fn structs_round_trip() {
  round_trip(&color::Info::default());
  round_trip(&Dimensions::new(1920, 1080));
  round_trip(&SampleAspectRatio::new(
    40,
    core::num::NonZeroI64::new(33).unwrap(),
  ));
  round_trip(&Tags::new().with_title("Song").with_year(2026));
  round_trip(&(TrackDisposition::DEFAULT | TrackDisposition::FORCED));
}

/// The channel household's two records carry several wire shapes in one
/// map, and each is its field type's own: `order` through the closed
/// law, `known_kind` through the open one, `custom_channels` as an array
/// of maps. A derive inherits the field types' impls — including their
/// leg split, since the derive hands the same serializer down — and this
/// is what proves it did.
#[test]
fn the_channel_records_carry_each_field_in_its_own_shape() {
  use crate::audio::{ChannelLayoutDescription, ChannelOrder, ChannelSpec};

  let spec = ChannelSpec::new(2, 5).with_label("FL");
  assert_eq!(
    serde_json::to_string(&spec).unwrap(),
    r#"{"index":2,"raw_id":5,"label":"FL"}"#
  );
  round_trip(&spec);

  let described = ChannelLayoutDescription::new(6)
    .with_order(ChannelOrder::Native)
    .with_known_kind(ChannelLayout::Ch5_1)
    .with_native_mask(Some(0x3F))
    .with_text("5.1(side)");
  assert_eq!(
    serde_json::to_string(&described).unwrap(),
    r#"{"order":"native","channels":6,"known_kind":"5.1(side)","native_mask":63,"custom_channels":[],"text":"5.1(side)"}"#
  );
  round_trip(&described);

  // The leg split reaches a *nested* field: the same record is a name in
  // JSON and a code in postcard, because the derive passes the format's
  // own `is_human_readable` down to `order`'s impl rather than picking a
  // shape at the record's level.
  let binary = postcard::to_allocvec(&described).unwrap();
  assert_eq!(binary[0], ChannelOrder::Native.to_u32() as u8);
  assert_eq!(
    postcard::from_bytes::<ChannelLayoutDescription>(&binary).unwrap(),
    described
  );

  let custom = ChannelLayoutDescription::new(2)
    .with_order(ChannelOrder::Custom)
    .with_custom_channels(::std::vec![
      ChannelSpec::new(0, 1).with_label("FL"),
      ChannelSpec::new(1, 2).with_label("FR"),
    ]);
  round_trip(&custom);

  // The record inherits its fields' refusals: `known_kind` is an open
  // vocabulary and absorbs an unknown slug, but `order` is closed and an
  // unrecognised name fails the whole map rather than softening to
  // `Unspecified`. A bare code fails here too — this document is
  // human-readable, so `order` is on the slug leg.
  assert!(
    serde_json::from_str::<ChannelLayoutDescription>(
      r#"{"order":"interleaved","channels":2,"known_kind":"unknown","native_mask":null,"custom_channels":[],"text":""}"#
    )
    .is_err()
  );
  assert!(
    serde_json::from_str::<ChannelLayoutDescription>(
      r#"{"order":1,"channels":2,"known_kind":"unknown","native_mask":null,"custom_channels":[],"text":""}"#
    )
    .is_err()
  );
  let odd: ChannelLayoutDescription = serde_json::from_str(
    r#"{"order":"unspecified","channels":2,"known_kind":"zzlayout","native_mask":null,"custom_channels":[],"text":""}"#,
  )
  .unwrap();
  assert_eq!(odd.known_kind(), &ChannelLayout::other("zzlayout"));

  // `serde(default)` keeps a sparse document readable — an omitted field
  // falls back to the type's own absent value.
  let sparse: ChannelLayoutDescription = serde_json::from_str(r#"{"channels":2}"#).unwrap();
  assert_eq!(sparse, ChannelLayoutDescription::new(2));
  let sparse_spec: ChannelSpec = serde_json::from_str(r#"{"raw_id":7}"#).unwrap();
  assert_eq!(sparse_spec, ChannelSpec::new(0, 7));
}

#[test]
fn rational_deserialize_rejects_out_of_range_fields() {
  // The derived `Deserialize` assigns fields directly, so it is a
  // second construction path; under `i64`/`NonZeroI64` the types no
  // longer carry the sign invariant, and the `deserialize_with`
  // guards are what stop it minting a value `Rational::new` rejects.
  assert!(serde_json::from_str::<Rational>(r#"{"num":2,"den":4}"#).is_ok());
  assert!(serde_json::from_str::<Rational>(r#"{"num":-5,"den":4}"#).is_err());
  assert!(serde_json::from_str::<Rational>(r#"{"num":5,"den":-4}"#).is_err());
  // `NonZeroI64`'s own deserializer rejects zero before the guard runs.
  assert!(serde_json::from_str::<Rational>(r#"{"num":5,"den":0}"#).is_err());
  // The wrappers derive through `Rational`, so guarding it guards them.
  assert!(serde_json::from_str::<SampleAspectRatio>(r#"{"num":-1,"den":1}"#).is_err());
  assert!(
    serde_json::from_str::<FrameRate>(r#"{"rate":{"num":-1,"den":1},"is_vfr":false}"#).is_err()
  );
}

#[test]
fn rational_serde_survives_above_u32_max() {
  let big = i64::from(u32::MAX) + 1;
  let r = Rational::new(big, core::num::NonZeroI64::new(big).unwrap());
  round_trip(&r);
}

#[test]
fn language_round_trips_as_bcp47() {
  let l = LanguageId::new("zh-Hant-TW").unwrap();
  assert_eq!(serde_json::to_string(&l).unwrap(), "\"zh-Hant-TW\"");
  round_trip(&l);
  round_trip(&LanguageId::default());

  // The lossless tail is the seat this type has and its predecessor did not,
  // so it is the one a wire form has to be shown carrying.
  let tailed = LanguageId::new("de-CH-1901").unwrap();
  assert_eq!(serde_json::to_string(&tailed).unwrap(), "\"de-CH-1901\"");
  round_trip(&tailed);

  // The three subtag types cross as their own canonical word.
  let language = Language::new("de").unwrap();
  assert_eq!(serde_json::to_string(&language).unwrap(), "\"de\"");
  round_trip(&language);
  let script = ScriptSubtag::new("Hans").unwrap();
  assert_eq!(serde_json::to_string(&script).unwrap(), "\"Hans\"");
  round_trip(&script);
  let region = Region::new("419").unwrap();
  assert_eq!(
    serde_json::to_string(&region).unwrap(),
    "\"419\"",
    "a numeric region is a STRING and never a JSON number — the leading zero of `001` is part of \
     the M.49 code"
  );
  round_trip(&region);
}

/// **A FOLD THAT WOULD NOT REPARSE NEVER FIRES**, and serde is the surface
/// that makes that matter: the write is the rendering and the read is the
/// door, so a canonicalisation whose text read back as a DIFFERENT value would
/// mutate a stored identity on every load rather than merely look untidy.
///
/// `en-Latn-Cyrl` is the case. `Latn` sits on the script seat and `Cyrl` on the
/// lossless tail; suppressing `Latn` would write `"en-Cyrl"`, which the read
/// door takes as the SCRIPT `Cyrl` with an empty tail. So the suppression is
/// skipped and the document keeps every seat it was given.
#[test]
fn a_tag_the_suppression_must_not_fold_survives_the_document() {
  for (tag, document, tail) in [
    ("en-Latn-Cyrl", r#""en-Latn-Cyrl""#, "Cyrl"),
    ("en-Latn-Latn", r#""en-Latn-Latn""#, "Latn"),
  ] {
    let held = LanguageId::new(tag).unwrap();
    assert_eq!(serde_json::to_string(&held).unwrap(), document);
    round_trip(&held);

    // …and the read of that document is the same value, seat for seat.
    let read: LanguageId = serde_json::from_str(document).unwrap();
    assert_eq!(read, held);
    assert_eq!(
      read.script().as_ref().map(ScriptSubtag::as_str),
      Some("Latn"),
      "`{tag}` — the script seat"
    );
    assert_eq!(
      read.rest().map(smol_bytes::Utf8Bytes::as_str),
      Some(tail),
      "`{tag}` — the tail seat"
    );
  }

  // The fold still fires wherever its text reads back as the same value, which
  // is the row this guard must not have cost.
  let folded = LanguageId::new("en-Latn-US").unwrap();
  assert_eq!(serde_json::to_string(&folded).unwrap(), "\"en-US\"");
  round_trip(&folded);
}

/// **A TAG THAT BECOMES GRANDFATHERED THROUGH ANOTHER FOLD IS FOLDED ANYWAY**,
/// and serde is a surface for that rather than a bystander: the write is the
/// rendering and the read is the door, so a canonicalisation that stopped one
/// fold short would write a document that loads as a DIFFERENT identity.
///
/// None of the four below is grandfathered as written. `en-Latn-GB-oed` becomes
/// one when the suppression drops `Latn`; the other three when the alpha-3 fold
/// rewrites their language subtag. A single pass would have stored the middle
/// spelling, and the middle spelling reads back as the right one — so the value
/// would move on its first reload and on no read after it, which is the worst
/// shape a storage defect comes in.
#[test]
fn a_tag_that_becomes_grandfathered_through_a_fold_is_folded_in_the_document() {
  for (sent, document) in [
    ("en-Latn-GB-oed", r#""en-GB-oxendict""#),
    ("eng-GB-oed", r#""en-GB-oxendict""#),
    ("nor-bok", r#""nb""#),
    ("zho-guoyu", r#""cmn""#),
  ] {
    let held = LanguageId::new(sent).unwrap();
    assert_eq!(serde_json::to_string(&held).unwrap(), document, "`{sent}`");
    round_trip(&held);

    // The READ of a document another writer left — the half a round trip cannot
    // show, since this crate never writes the middle spelling.
    let read: LanguageId = serde_json::from_str(&format!("\"{sent}\"")).unwrap();
    assert_eq!(read, held, "`{sent}` — the read door stopped short");
    assert_eq!(serde_json::to_string(&read).unwrap(), document, "`{sent}`");
  }
}

/// **THE READ GOES THROUGH THE DOOR, SO IT FOLDS** — the half a round trip
/// cannot show, because this crate never WRITES a non-canonical spelling.
///
/// The only way to ask what the read does with an mkv's `ger` is to hand it
/// one by hand, which is what a document written by another writer, or before
/// a registry bump, actually is.
#[test]
fn a_hand_written_language_document_folds_at_the_read_door() {
  let folded: LanguageId = serde_json::from_str("\"GER-latn-de\"").unwrap();
  assert_eq!(folded, LanguageId::new("de-DE").unwrap());
  assert_eq!(serde_json::to_string(&folded).unwrap(), "\"de-DE\"");

  let subtag: Language = serde_json::from_str("\"iw\"").unwrap();
  assert_eq!(subtag, Language::new("he").unwrap());

  // …and a text that names no tag is an ERROR rather than a silent sentinel.
  assert!(serde_json::from_str::<LanguageId>("\"en-US-!!\"").is_err());
  assert!(serde_json::from_str::<Region>("\"DEU\"").is_err());
}

#[test]
fn validated_structs_check_on_deserialize() {
  let g = GeoLocation::try_new(48.8584, 2.2945, Some(330.0)).unwrap();
  round_trip(&g);
  // Out-of-range latitude is rejected, not silently materialised.
  assert!(
    serde_json::from_str::<GeoLocation>(r#"{"lat":999.0,"lon":0.0,"altitude":null}"#).is_err()
  );

  let fp = Fingerprint::try_new("chromaprint", &b"\x01\x02\x03"[..]).unwrap();
  round_trip(&fp);
  // Empty algorithm violates the invariant and must be rejected.
  assert!(serde_json::from_str::<Fingerprint>(r#"{"algorithm":"","value":[1,2,3]}"#).is_err());

  let art = CoverArt::try_new("image/png", &b"\x89PNG"[..]).unwrap();
  round_trip(&art);
  // Empty mime violates the invariant and must be rejected.
  assert!(serde_json::from_str::<CoverArt>(r#"{"mime":"","data":[1]}"#).is_err());
}

// ── Codex round 1 findings ──

/// `SampleFormat` is a plain name vocabulary now: the numeric arm that
/// forced a bespoke two-shape codec is gone, so it rides the same slug
/// wire as every other vocabulary, on both human-readable and binary
/// formats.
#[test]
fn sample_format_rides_the_slug_wire() {
  use crate::audio::SampleFormat;
  assert_eq!(
    serde_json::to_string(&SampleFormat::S16).unwrap(),
    "\"s16\""
  );
  round_trip(&SampleFormat::S16);
  let other = SampleFormat::other("custom");
  assert_eq!(serde_json::to_string(&other).unwrap(), "\"custom\"");
  round_trip(&other);
}

/// The **slug leg** of the closed law: a human-readable format carries
/// the `as_str()` name, reads it back through `FromStr`, and is strict
/// at that door — an unrecognised name is a serde error, never collapsed
/// onto the default the way `from_u32` would collapse a code.
#[test]
fn closed_coded_enums_ride_the_slug_leg_where_a_human_reads_it() {
  use crate::audio::{BitRateMode, ChannelOrder};

  assert_eq!(serde_json::to_string(&BitRateMode::Cbr).unwrap(), "\"cbr\"");
  assert_eq!(serde_json::to_string(&BitRateMode::Abr).unwrap(), "\"abr\"");
  for m in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
    round_trip(&m);
  }

  assert_eq!(
    serde_json::to_string(&ChannelOrder::Unspecified).unwrap(),
    "\"unspecified\""
  );
  assert_eq!(
    serde_json::to_string(&ChannelOrder::Ambisonic).unwrap(),
    "\"ambisonic\""
  );
  for &o in ChannelOrder::ROSTER {
    round_trip(&o);
  }

  // Case folds — one name per value, not one spelling. Folding a
  // spelling is not inventing a value, which is the line strictness is
  // drawn on.
  assert_eq!(
    serde_json::from_str::<BitRateMode>("\"CBR\"").unwrap(),
    BitRateMode::Cbr
  );
  assert_eq!(
    serde_json::from_str::<ChannelOrder>("\"Native\"").unwrap(),
    ChannelOrder::Native
  );
}

/// Both halves of "strict" on the slug leg: a name this vocabulary
/// cannot spell is refused, and a *number* is refused outright. The two
/// legs are alternatives, not a chain of fallbacks — a human-readable
/// document carrying a bare code here is malformed, not merely terse.
#[test]
fn the_slug_leg_refuses_an_unknown_name_and_refuses_a_number() {
  use crate::audio::{BitRateMode, ChannelOrder};

  assert!(serde_json::from_str::<BitRateMode>("\"constant\"").is_err());
  assert!(serde_json::from_str::<BitRateMode>("\"\"").is_err());
  assert!(serde_json::from_str::<ChannelOrder>("\"interleaved\"").is_err());
  assert!(serde_json::from_str::<ChannelOrder>("\"\"").is_err());

  // A number is not a name. `1` is `Vbr`'s code and `Native`'s code, and
  // the slug leg refuses both — the numeric door is the binary leg, not
  // this one.
  assert!(serde_json::from_str::<BitRateMode>("1").is_err());
  assert!(serde_json::from_str::<ChannelOrder>("1").is_err());
  assert!(serde_json::from_str::<ChannelOrder>("0").is_err());

  // Neither door collapses onto the default, which is exactly what the
  // lenient `from_u32` would have done with the same code.
  assert_eq!(BitRateMode::from_u32(999), BitRateMode::Cbr);
  assert_eq!(ChannelOrder::from_u32(999), ChannelOrder::Unspecified);
}

/// The **code leg**: a format that is not `is_human_readable` carries
/// the `to_u32()` code instead, reads it back through `try_from_u32`,
/// and is strict there too — an out-of-range code is a serde error.
///
/// `postcard` is the binary format under test. It is already a
/// dev-dependency (`sample_format_postcard_binary_roundtrip` below uses
/// it) and it declares `is_human_readable() == false` on *both* its
/// serializer and its deserializer, which is the whole of what this leg
/// keys on. A hand-rolled stub serializer would test the macro against
/// this crate's own idea of a binary format; a real one tests it against
/// a format that exists.
#[test]
fn closed_coded_enums_ride_the_code_leg_where_only_a_machine_reads_it() {
  use crate::audio::{BitRateMode, ChannelOrder};

  // The wire is the varint code, not the name.
  assert_eq!(
    postcard::to_allocvec(&BitRateMode::Abr).unwrap(),
    ::std::vec![2u8]
  );
  assert_eq!(
    postcard::to_allocvec(&ChannelOrder::Ambisonic).unwrap(),
    ::std::vec![3u8]
  );
  assert_eq!(
    postcard::to_allocvec(&ChannelOrder::Unspecified).unwrap(),
    ::std::vec![0u8]
  );

  for m in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
    let bytes = postcard::to_allocvec(&m).unwrap();
    assert_eq!(postcard::from_bytes::<BitRateMode>(&bytes).unwrap(), m);
  }
  for &o in ChannelOrder::ROSTER {
    let bytes = postcard::to_allocvec(&o).unwrap();
    assert_eq!(postcard::from_bytes::<ChannelOrder>(&bytes).unwrap(), o);
  }

  // Out-of-range codes are refused — not canonicalised to the default.
  assert!(postcard::from_bytes::<BitRateMode>(&[3u8]).is_err());
  assert!(postcard::from_bytes::<ChannelOrder>(&[4u8]).is_err());

  // And the two legs really are different shapes for one value: what
  // JSON writes as a name, postcard writes as a code.
  assert_eq!(
    serde_json::to_string(&ChannelOrder::Custom).unwrap(),
    "\"custom\""
  );
  assert_eq!(
    postcard::to_allocvec(&ChannelOrder::Custom).unwrap(),
    ::std::vec![2u8]
  );
}

/// `TrackOrigin` left the coded wire in 0.5.0: it is an open vocabulary
/// now, so it rides the same slug wire as every other name enum and an
/// unnamed slug survives the round trip instead of being refused.
#[test]
fn track_origin_rides_the_slug_wire() {
  use crate::subtitle::TrackOrigin;

  for o in [
    TrackOrigin::Embedded,
    TrackOrigin::Sidecar,
    TrackOrigin::External,
    TrackOrigin::Derived,
    TrackOrigin::other("broadcast"),
  ] {
    round_trip(&o);
  }

  assert_eq!(
    serde_json::to_string(&TrackOrigin::Derived).unwrap(),
    "\"derived\""
  );
  assert_eq!(
    serde_json::from_str::<TrackOrigin>("\"broadcast\"").unwrap(),
    TrackOrigin::other("broadcast")
  );
  // The old numeric wire is not silently accepted.
  assert!(serde_json::from_str::<TrackOrigin>("0").is_err());
}

// ── Codex round 2 findings ──

/// The slug wire has to survive a non-self-describing binary format
/// too — the earlier bespoke codec branched on `is_human_readable()`
/// precisely because a bare `deserialize_any` does not work there.
#[test]
fn sample_format_postcard_binary_roundtrip() {
  use crate::audio::SampleFormat;

  fn binary_round_trip(v: &SampleFormat) -> SampleFormat {
    let bytes = postcard::to_allocvec(v).expect("postcard serialize");
    postcard::from_bytes::<SampleFormat>(&bytes).expect("postcard deserialize")
  }

  assert_eq!(binary_round_trip(&SampleFormat::S16), SampleFormat::S16);
  let other = SampleFormat::other("custom");
  assert_eq!(binary_round_trip(&other), other);
}

/// Default-backed metadata structs must accept sparse JSON — missing
/// fields default rather than failing — so older / partial records
/// remain readable as the schema evolves. `serde(default)` at the
/// container level routes missing fields through `Default`.
#[test]
fn sparse_json_uses_serde_default_on_default_backed_structs() {
  use crate::{
    audio::{Loudness, Tags},
    capture::Device,
  };

  // Tags: only `title` present; the rest fall back to absent sentinels.
  let t: Tags = serde_json::from_str(r#"{"title":"hello"}"#).unwrap();
  let expected = Tags::new().with_title(smol_bytes::Utf8Bytes::from("hello"));
  assert_eq!(t, expected);

  // Tags: completely empty object → fully-default value (no missing-field error).
  let empty: Tags = serde_json::from_str("{}").unwrap();
  assert_eq!(empty, Tags::default());

  // Device: only `make` present.
  let d: Device = serde_json::from_str(r#"{"make":"Apple"}"#).unwrap();
  let expected = Device::new().with_make(smol_bytes::Utf8Bytes::from("Apple"));
  assert_eq!(d, expected);

  // Loudness: partial measurement.
  let l: Loudness = serde_json::from_str(r#"{"integrated_lufs":-23.0}"#).unwrap();
  assert_eq!(l, Loudness::new(-23.0, 0.0, 0.0, 0.0));
}

/// golden-rule §9: an absent `Option` field serializes to an *omitted
/// key*, never `null`. Verified for every `Option`-bearing serde type.
#[test]
fn absent_option_fields_are_omitted_not_null() {
  use crate::{audio::Tags, capture::GeoLocation, color::HdrStaticMetadata};

  // `Tags.language` absent.
  let j = serde_json::to_string(&Tags::default()).unwrap();
  assert!(!j.contains("null"), "Tags emitted `null`: {j}");
  assert!(
    !j.contains("language"),
    "Tags emitted absent `language`: {j}"
  );

  // `HdrStaticMetadata` — both `mastering` / `content_light` absent.
  let j = serde_json::to_string(&HdrStaticMetadata::default()).unwrap();
  assert_eq!(
    j, "{}",
    "empty HdrStaticMetadata should serialize to `{{}}`"
  );

  // `GeoLocation.altitude` absent (hand-written `Serialize`).
  let g = GeoLocation::try_new(0.0, 0.0, None).unwrap();
  let j = serde_json::to_string(&g).unwrap();
  assert!(!j.contains("null"), "GeoLocation emitted `null`: {j}");
  assert!(
    !j.contains("altitude"),
    "GeoLocation emitted absent `altitude`: {j}"
  );
  // …and present `altitude` still round-trips.
  round_trip(&GeoLocation::try_new(0.0, 0.0, Some(12.5)).unwrap());
}
