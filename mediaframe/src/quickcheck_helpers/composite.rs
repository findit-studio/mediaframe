//! Cluster C — audio composite metadata + capture + language.
//!
//! Validated `try_new` types build valid inputs FIRST (in-range floats via
//! `i32::arbitrary` clamped / scaled, non-empty strings via fallback like `"x"`
//! / `"application/octet-stream"`), THEN `try_new(...).expect(...)`. Never
//! pattern `try_new(arbitrary_float).unwrap()` — that would panic on input.
//!
//! Owned types:
//!   - audio::{ChannelSpec, ChannelLayoutDescription}
//!   - audio::{Loudness, Fingerprint, CoverArt, Tags}
//!   - capture::{Device, GeoLocation}
//!   - lang::{Language, ScriptSubtag, Region, LanguageId}

use ::quickcheck::Arbitrary;

/// `audio::ChannelSpec` — `new(index, raw_id)` + `with_label`.
///
/// Three independent fields, no invariant between them. The label is
/// free text rather than a slug, so an arbitrary string goes straight
/// through — nothing here folds or canonicalises.
pub(crate) fn channel_spec(g: &mut ::quickcheck::Gen) -> crate::audio::ChannelSpec {
  crate::audio::ChannelSpec::new(u32::arbitrary(g), u32::arbitrary(g)).with_label(
    ::smol_bytes::Utf8Bytes::from(<::std::string::String as Arbitrary>::arbitrary(g)),
  )
}

/// `audio::ChannelLayoutDescription` — `new(channels)` + every builder
/// setter, each field drawn independently.
///
/// Mirrors the `arbitrary_impls` half field for field and for the same
/// reason: the type enforces no relation between its fields, so the
/// incoherent combinations a well-formed FFmpeg layout never shows (a
/// `Custom` order with no channel list, a `Native` order with no mask)
/// must stay reachable — they are the ones a consumer is most likely to
/// mishandle.
pub(crate) fn channel_layout_description(
  g: &mut ::quickcheck::Gen,
) -> crate::audio::ChannelLayoutDescription {
  crate::audio::ChannelLayoutDescription::new(u32::arbitrary(g))
    .with_order(Arbitrary::arbitrary(g))
    .with_known_kind(Arbitrary::arbitrary(g))
    .with_native_mask(<::core::option::Option<u64> as Arbitrary>::arbitrary(g))
    .with_custom_channels(<::std::vec::Vec<crate::audio::ChannelSpec> as Arbitrary>::arbitrary(g))
    .with_text(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
}

/// `audio::Loudness` — plain `new(f32, f32, f32, f32)` constructor.
///
/// The four fields are generated FINITE (Codex round-5 finding): raw
/// `f32::arbitrary` yields NaN / ±inf, which JSON serializes as `null`
/// and then fails to deserialize back into `f32`. A bounded integer
/// `[-10_000_000, 10_000_000]` mapped to `f32 / 100.0` gives a finite
/// value in [-100_000, 100_000] — covers every real EBU R128 scalar
/// (LUFS / LU / dBTP / dBFS) while staying serde-round-trippable.
pub(crate) fn loudness(g: &mut ::quickcheck::Gen) -> crate::audio::Loudness {
  fn finite(g: &mut ::quickcheck::Gen) -> f32 {
    // `Gen` has no `int_in_range`; bound via `rem_euclid` on an arbitrary
    // `i32` (always non-negative remainder), then shift into range.
    (i32::arbitrary(g).rem_euclid(20_000_001) - 10_000_000) as f32 / 100.0
  }
  crate::audio::Loudness::new(finite(g), finite(g), finite(g), finite(g))
}

/// `audio::ReplayGain` — same finite-f32 generation strategy as
/// [`loudness`]; album scalars are independently `Some`/`None`.
pub(crate) fn replay_gain(g: &mut ::quickcheck::Gen) -> crate::audio::ReplayGain {
  fn finite(g: &mut ::quickcheck::Gen) -> f32 {
    (i32::arbitrary(g).rem_euclid(20_000_001) - 10_000_000) as f32 / 100.0
  }
  let album_gain = if bool::arbitrary(g) {
    Some(finite(g))
  } else {
    None
  };
  let album_peak = if bool::arbitrary(g) {
    Some(finite(g))
  } else {
    None
  };
  crate::audio::ReplayGain::new(finite(g), finite(g), album_gain, album_peak)
}

/// `audio::Fingerprint` — `try_new(algo, value)` rejects empty `algo`; fall
/// back to `"x"` so the `expect` is sound. Empty `value` is allowed.
pub(crate) fn fingerprint(g: &mut ::quickcheck::Gen) -> crate::audio::Fingerprint {
  let algo_s = <::std::string::String as Arbitrary>::arbitrary(g);
  let algo: ::smol_bytes::Utf8Bytes = if algo_s.is_empty() {
    ::smol_bytes::Utf8Bytes::from_static("x")
  } else {
    algo_s.into()
  };
  let value = ::bytes::Bytes::from(<::std::vec::Vec<u8> as Arbitrary>::arbitrary(g));
  crate::audio::Fingerprint::try_new(algo, value).expect("algo non-empty by construction")
}

/// `audio::CoverArt` — `try_new(mime, data)` rejects empty `mime` *and*
/// empty `data`; supply both with valid fallbacks so the `expect` is sound.
pub(crate) fn cover_art(g: &mut ::quickcheck::Gen) -> crate::audio::CoverArt {
  let mime_s = <::std::string::String as Arbitrary>::arbitrary(g);
  let mime: ::smol_bytes::Utf8Bytes = if mime_s.is_empty() {
    ::smol_bytes::Utf8Bytes::from_static("application/octet-stream")
  } else {
    mime_s.into()
  };
  let data_v = <::std::vec::Vec<u8> as Arbitrary>::arbitrary(g);
  let data = ::bytes::Bytes::from(if data_v.is_empty() {
    ::std::vec![0u8]
  } else {
    data_v
  });
  crate::audio::CoverArt::try_new(mime, data).expect("mime + data non-empty by construction")
}

/// `audio::Tags` — `new()` + every builder setter: the seven `Utf8Bytes`
/// string fields, the five bare-`u16` numeric fields (`0` = absent —
/// generated freely incl. `0`, since the type + buffa codec agree), and
/// `language` (`Option<Language>`, from the curated BCP-47 `language`
/// helper).
pub(crate) fn tags(g: &mut ::quickcheck::Gen) -> crate::audio::Tags {
  crate::audio::Tags::new()
    .with_title(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_artist(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_album_artist(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_album(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_composer(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_genre(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_comment(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_year(<u16 as Arbitrary>::arbitrary(g))
    .with_track_number(<u16 as Arbitrary>::arbitrary(g))
    .with_track_total(<u16 as Arbitrary>::arbitrary(g))
    .with_disc_number(<u16 as Arbitrary>::arbitrary(g))
    .with_disc_total(<u16 as Arbitrary>::arbitrary(g))
    // `language` is `Option<Language>` — 50/50 `None` / `Some(<curated
    // BCP-47 tag>)`, reusing this module's `language` helper.
    .maybe_language(if bool::arbitrary(g) {
      Some(language(g))
    } else {
      None
    })
}

/// `capture::Device` — `new()` + `with_make` / `with_model`. Both fields are
/// `Utf8Bytes` with empty-string-means-absent semantics; pass arbitrary strings
/// straight through.
pub(crate) fn capture_device(g: &mut ::quickcheck::Gen) -> crate::capture::Device {
  crate::capture::Device::new()
    .with_make(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_model(::smol_bytes::Utf8Bytes::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
}

/// `capture::GeoLocation` — `try_new(lat, lon, altitude)` validates ranges
/// (`lat ∈ [-90, 90]`, `lon ∈ [-180, 180]`, altitude must be finite when
/// `Some`). `quickcheck::Gen` has no `int_in_range`; we compute valid
/// coordinates via `rem_euclid` on an arbitrary `i32`, which always returns a
/// non-negative remainder, then shift into range. Same 1/100° resolution and
/// `-1_000..=100_000` altitude band as the `arbitrary_impls` cluster.
pub(crate) fn geo_location(g: &mut ::quickcheck::Gen) -> crate::capture::GeoLocation {
  let lat = (i32::arbitrary(g).rem_euclid(18_001) - 9_000) as f64 / 100.0;
  let lon = (i32::arbitrary(g).rem_euclid(36_001) - 18_000) as f64 / 100.0;
  let altitude = if bool::arbitrary(g) {
    Some((i32::arbitrary(g).rem_euclid(101_001) - 1_000) as f32)
  } else {
    None
  };
  crate::capture::GeoLocation::try_new(lat, lon, altitude)
    .expect("lat/lon in-range and altitude finite by construction")
}

/// `lang::LanguageId` — curated BCP 47 tags the whole-tag door accepts.
///
/// The tags are CANONICAL, and deliberately: `Arbitrary` feeds round-trip
/// suites, and a fold at the door would make `from(tag) != from(render(from(tag)))`
/// look like a serde bug when it is the fold working. The dirty spellings
/// (`ger`, `iw`, `en-Latn`) are pinned in the household's own tests, where
/// what is being asserted is the fold itself.
///
/// Covers language-only, language+region, language+script+region, the `und`
/// sentinel, and a lossless tail — the fourth seat being the whole reason
/// this type replaced the icu triple, a generator that never filled it would
/// leave that seat unexercised by every property test in the crate.
pub(crate) fn language(g: &mut ::quickcheck::Gen) -> crate::lang::LanguageId {
  const TAGS: &[&str] = &[
    "und",
    "en",
    "en-US",
    "es",
    "fr",
    "de",
    "ja",
    "zh-Hant-TW",
    "pt-BR",
    "ar",
    "ru",
    "ko",
    "de-CH-1901",
    "en-US-x-lorem",
  ];
  let tag: &&str = g.choose(TAGS).expect("non-empty curated TAGS slice");
  crate::lang::LanguageId::new(tag).expect("curated BCP 47 tag must parse")
}

/// `lang::Language` — curated primary language subtags, canonical, for
/// [`language`]'s reason.
pub(crate) fn language_subtag(g: &mut ::quickcheck::Gen) -> crate::lang::Language {
  const SUBTAGS: &[&str] = &[
    "und", "en", "de", "fr", "es", "zh", "ja", "yue", "ar", "qaa",
  ];
  let subtag: &&str = g.choose(SUBTAGS).expect("non-empty curated slice");
  crate::lang::Language::new(subtag).expect("curated language subtag must parse")
}

/// `lang::ScriptSubtag` — curated ISO 15924 subtags, in the registry's own
/// Titlecase.
pub(crate) fn script_subtag(g: &mut ::quickcheck::Gen) -> crate::lang::ScriptSubtag {
  const SUBTAGS: &[&str] = &[
    "Latn", "Hans", "Hant", "Cyrl", "Arab", "Jpan", "Zxxx", "Zzzz",
  ];
  let subtag: &&str = g.choose(SUBTAGS).expect("non-empty curated slice");
  crate::lang::ScriptSubtag::new(subtag).expect("curated script subtag must parse")
}

/// `lang::Region` — curated subtags from BOTH region grammars: ISO 3166-1
/// country codes and UN M.49 area codes, the second of which is the arm a
/// letters-only roster would never reach.
pub(crate) fn region(g: &mut ::quickcheck::Gen) -> crate::lang::Region {
  const SUBTAGS: &[&str] = &["US", "DE", "TW", "CN", "BR", "ZZ", "419", "001", "150"];
  let subtag: &&str = g.choose(SUBTAGS).expect("non-empty curated slice");
  crate::lang::Region::new(subtag).expect("curated region subtag must parse")
}
