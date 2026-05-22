// Cluster C — audio composite metadata, capture, language.
//
// Owned types (all hand-written so private fields stay encapsulated and the
// `try_new` validated types come out valid-by-construction — never feed
// attacker-controlled fuzz input into a fallible constructor + `.unwrap()`):
//
//   AUDIO COMPOSITE:
//     - audio::Loudness          (new(f32, f32, f32, f32) — plain ctor)
//     - audio::Fingerprint       (try_new(algo, value) — algo non-empty)
//     - audio::CoverArt          (try_new(mime, data) — both non-empty)
//     - audio::Tags              (new() + builder setters; representative
//                                 subset — title/artist/album_artist/album/
//                                 composer/genre/comment + year/track_number/
//                                 track_total/disc_number/disc_total)
//   CAPTURE:
//     - capture::Device          (new() + with_make / with_model)
//     - capture::GeoLocation     (try_new(lat, lon, altitude) — ranges built
//                                 with `int_in_range` then `.expect`)
//
//   LANGUAGE:
//     - lang::Language           (from_bcp47(<curated tag>) — `u.choose`)

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::Loudness {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // `f32::arbitrary` builds floats from raw bits — it can yield NaN / ±inf,
    // which JSON serializes as `null` and then fails to deserialize back
    // into `f32` (Codex round-5 finding). Generate FINITE values by mapping
    // a bounded integer: `[-10_000_000, 10_000_000] / 100` → finite f32 in
    // [-100_000.0, 100_000.0], comfortably covering every real EBU R128
    // scalar (LUFS / LU / dBTP / dBFS) while staying serde-round-trippable.
    fn finite(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<f32> {
      Ok(u.int_in_range(-10_000_000i32..=10_000_000)? as f32 / 100.0)
    }
    Ok(Self::new(finite(u)?, finite(u)?, finite(u)?, finite(u)?))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::Fingerprint {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // `try_new` rejects empty `algorithm`; ensure non-empty with a fallback
    // so the expect below is sound. Empty `value` is allowed.
    let algo_s = <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?;
    let algo: ::smol_str::SmolStr = if algo_s.is_empty() {
      ::smol_str::SmolStr::new_inline("x")
    } else {
      algo_s.into()
    };
    let value = ::bytes::Bytes::from(<::std::vec::Vec<u8> as ::arbitrary::Arbitrary>::arbitrary(
      u,
    )?);
    Ok(crate::audio::Fingerprint::try_new(algo, value).expect("algo non-empty by construction"))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::CoverArt {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // `try_new` rejects empty `mime` and empty `data`; supply both with
    // valid fallbacks so the expect below is sound.
    let mime_s = <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?;
    let mime: ::smol_str::SmolStr = if mime_s.is_empty() {
      ::smol_str::SmolStr::new_static("application/octet-stream")
    } else {
      mime_s.into()
    };
    let data_v = <::std::vec::Vec<u8> as ::arbitrary::Arbitrary>::arbitrary(u)?;
    let data = ::bytes::Bytes::from(if data_v.is_empty() {
      ::std::vec![0u8]
    } else {
      data_v
    });
    Ok(crate::audio::CoverArt::try_new(mime, data).expect("mime + data non-empty by construction"))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::Tags {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Every builder field: the seven `SmolStr` strings (empty = absent), the
    // five `Option<u16>` numerics (`None` ≠ `Some(0)`), and `language`
    // (`Option<SmolStr>` — Codex round-7 finding: it was previously omitted,
    // so the serialized buffa field 13 was outside the fuzzing surface).
    let t = crate::audio::Tags::new()
      .with_title(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_artist(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_album_artist(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_album(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_composer(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_genre(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_comment(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .maybe_year(<::core::option::Option<u16> as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .maybe_track_number(<::core::option::Option<u16> as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .maybe_track_total(<::core::option::Option<u16> as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .maybe_disc_number(<::core::option::Option<u16> as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .maybe_disc_total(<::core::option::Option<u16> as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .maybe_language(
        <::core::option::Option<::std::string::String> as ::arbitrary::Arbitrary>::arbitrary(u)?
          .map(::smol_str::SmolStr::from),
      );
    Ok(t)
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::capture::Device {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Both fields are `SmolStr` with empty-string-means-absent semantics;
    // pass arbitrary strings straight through.
    let d = crate::capture::Device::new()
      .with_make(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_model(::smol_str::SmolStr::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ));
    Ok(d)
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::capture::GeoLocation {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Build coordinates in-range using `int_in_range` (never panics).
    // Latitude ∈ [-90, 90], longitude ∈ [-180, 180]; both produced at
    // 1/100-degree resolution. Altitude is `Option<f32>`; we only ever
    // hand the constructor finite f32s, so it stays `Some(_)` when set.
    let lat = u.int_in_range(-9_000i32..=9_000)? as f64 / 100.0;
    let lon = u.int_in_range(-18_000i32..=18_000)? as f64 / 100.0;
    let altitude = if <bool as ::arbitrary::Arbitrary>::arbitrary(u)? {
      Some(u.int_in_range(-1_000i32..=100_000)? as f32)
    } else {
      None
    };
    Ok(
      crate::capture::GeoLocation::try_new(lat, lon, altitude)
        .expect("lat/lon in-range and altitude finite by construction"),
    )
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::lang::Language {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Curated BCP-47 tags that `Language::from_bcp47` accepts — covers
    // language-only, language+region, language+script+region, and the
    // `und` sentinel.
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
    let tag: &&str = u.choose(TAGS)?;
    Ok(crate::lang::Language::from_bcp47(tag).expect("curated BCP-47 tag must parse"))
  }
}
