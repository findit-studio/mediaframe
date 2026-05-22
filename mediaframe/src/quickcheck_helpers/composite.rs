//! Cluster C — audio composite metadata + capture + language.
//!
//! Validated `try_new` types build valid inputs FIRST (in-range floats via
//! `i32::arbitrary` clamped / scaled, non-empty strings via fallback like `"x"`
//! / `"application/octet-stream"`), THEN `try_new(...).expect(...)`. Never
//! pattern `try_new(arbitrary_float).unwrap()` — that would panic on input.
//!
//! Owned types:
//!   - audio::{Loudness, Fingerprint, CoverArt, Tags}
//!   - capture::{Device, GeoLocation}
//!   - lang::Language

use ::quickcheck::Arbitrary;

/// `audio::Loudness` — plain `new(f32, f32, f32, f32)` constructor; no
/// validation, so the four `f32` fields pass straight through.
pub(crate) fn loudness(g: &mut ::quickcheck::Gen) -> crate::audio::Loudness {
  crate::audio::Loudness::new(
    f32::arbitrary(g),
    f32::arbitrary(g),
    f32::arbitrary(g),
    f32::arbitrary(g),
  )
}

/// `audio::Fingerprint` — `try_new(algo, value)` rejects empty `algo`; fall
/// back to `"x"` so the `expect` is sound. Empty `value` is allowed.
pub(crate) fn fingerprint(g: &mut ::quickcheck::Gen) -> crate::audio::Fingerprint {
  let algo_s = <::std::string::String as Arbitrary>::arbitrary(g);
  let algo: ::smol_str::SmolStr = if algo_s.is_empty() {
    ::smol_str::SmolStr::new_inline("x")
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
  let mime: ::smol_str::SmolStr = if mime_s.is_empty() {
    ::smol_str::SmolStr::new_static("application/octet-stream")
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

/// `audio::Tags` — `new()` + every builder setter: the seven string fields
/// (title/artist/album_artist/album/composer/genre/comment), the five
/// `Option<u16>` numeric fields (year/track_number/track_total/disc_number/
/// disc_total), and `language` (Codex round-4 finding — it was previously
/// omitted, so every generated `Tags` had `language == None`).
pub(crate) fn tags(g: &mut ::quickcheck::Gen) -> crate::audio::Tags {
  crate::audio::Tags::new()
    .with_title(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_artist(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_album_artist(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_album(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_composer(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_genre(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_comment(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .maybe_year(<::core::option::Option<u16> as Arbitrary>::arbitrary(g))
    .maybe_track_number(<::core::option::Option<u16> as Arbitrary>::arbitrary(g))
    .maybe_track_total(<::core::option::Option<u16> as Arbitrary>::arbitrary(g))
    .maybe_disc_number(<::core::option::Option<u16> as Arbitrary>::arbitrary(g))
    .maybe_disc_total(<::core::option::Option<u16> as Arbitrary>::arbitrary(g))
    // `language` is `Option<SmolStr>`; generate both `None` and
    // `Some(<arbitrary string>)` so the absent / present halves are covered.
    .maybe_language(if bool::arbitrary(g) {
      Some(::smol_str::SmolStr::from(
        <::std::string::String as Arbitrary>::arbitrary(g),
      ))
    } else {
      None
    })
}

/// `capture::Device` — `new()` + `with_make` / `with_model`. Both fields are
/// `SmolStr` with empty-string-means-absent semantics; pass arbitrary strings
/// straight through.
pub(crate) fn capture_device(g: &mut ::quickcheck::Gen) -> crate::capture::Device {
  crate::capture::Device::new()
    .with_make(::smol_str::SmolStr::from(
      <::std::string::String as Arbitrary>::arbitrary(g),
    ))
    .with_model(::smol_str::SmolStr::from(
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

/// `lang::Language` — curated BCP-47 tags that `from_bcp47` accepts: covers
/// language-only, language+region, language+script+region, and the `und`
/// sentinel.
pub(crate) fn language(g: &mut ::quickcheck::Gen) -> crate::lang::Language {
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
  ];
  let tag: &&str = g.choose(TAGS).expect("non-empty curated TAGS slice");
  crate::lang::Language::from_bcp47(tag).expect("curated BCP-47 tag must parse")
}
