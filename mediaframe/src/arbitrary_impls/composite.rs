// Cluster C — audio composite metadata, capture, language.
//
// Owned types (all hand-written so private fields stay encapsulated and the
// `try_new` validated types come out valid-by-construction — never feed
// attacker-controlled fuzz input into a fallible constructor + `.unwrap()`):
//
//   AUDIO COMPOSITE:
//     - audio::ChannelSpec       (new(u32, u32) + with_label)
//     - audio::ChannelLayoutDescription
//                                (new(u32) + builder setters; every field drawn
//                                 independently — see the impl for why)
//     - audio::Loudness          (new(f32, f32, f32, f32) — plain ctor)
//     - audio::ReplayGain        (new(f32, f32, Option<f32>, Option<f32>) — plain ctor)
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
//     - lang::{Language, ScriptSubtag, Region, LanguageId}
//                                 (new(<curated subtag/tag>) — `u.choose`)

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::ChannelSpec {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Three independent fields, no invariant between them. The label is
    // free text rather than a slug, so an arbitrary string goes straight
    // through — nothing here folds or canonicalises.
    Ok(
      crate::audio::ChannelSpec::new(
        <u32 as ::arbitrary::Arbitrary>::arbitrary(u)?,
        <u32 as ::arbitrary::Arbitrary>::arbitrary(u)?,
      )
      .with_label(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      )),
    )
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::ChannelLayoutDescription {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Every field is drawn independently — including the combinations a
    // well-formed FFmpeg layout never shows (a `Custom` order with no
    // channel list, a `Native` order with no mask). That is deliberate:
    // the type enforces no relation between its fields, every one of
    // them has a public unchecked setter, and a generator that produced
    // only coherent descriptions would leave the incoherent ones — the
    // ones a consumer is most likely to mishandle — unreachable by the
    // fuzzer.
    Ok(
      crate::audio::ChannelLayoutDescription::new(<u32 as ::arbitrary::Arbitrary>::arbitrary(u)?)
        .with_order(::arbitrary::Arbitrary::arbitrary(u)?)
        .with_known_kind(::arbitrary::Arbitrary::arbitrary(u)?)
        .with_native_mask(<::core::option::Option<u64> as ::arbitrary::Arbitrary>::arbitrary(u)?)
        .with_custom_channels(
          <::std::vec::Vec<crate::audio::ChannelSpec> as ::arbitrary::Arbitrary>::arbitrary(u)?,
        )
        .with_text(::smol_bytes::Utf8Bytes::from(
          <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
        )),
    )
  }
}

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

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::ReplayGain {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Same finite-f32 generation as `Loudness` (see the rationale on
    // `Loudness::arbitrary` for the NaN / ±inf JSON-round-trip bug).
    fn finite(u: &mut ::arbitrary::Unstructured<'_>) -> ::arbitrary::Result<f32> {
      Ok(u.int_in_range(-10_000_000i32..=10_000_000)? as f32 / 100.0)
    }
    let track_gain = finite(u)?;
    let track_peak = finite(u)?;
    let album_gain = if bool::arbitrary(u)? {
      Some(finite(u)?)
    } else {
      None
    };
    let album_peak = if bool::arbitrary(u)? {
      Some(finite(u)?)
    } else {
      None
    };
    Ok(Self::new(track_gain, track_peak, album_gain, album_peak))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::audio::Fingerprint {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // `try_new` rejects empty `algorithm`; ensure non-empty with a fallback
    // so the expect below is sound. Empty `value` is allowed.
    let algo_s = <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?;
    let algo: ::smol_bytes::Utf8Bytes = if algo_s.is_empty() {
      ::smol_bytes::Utf8Bytes::from_static("x")
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
    let mime: ::smol_bytes::Utf8Bytes = if mime_s.is_empty() {
      ::smol_bytes::Utf8Bytes::from_static("application/octet-stream")
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
    // Every builder field: the seven `Utf8Bytes` strings (empty = absent), the
    // five bare-`u16` numerics (`0` = absent — generated freely, including
    // `0`, since type + buffa codec now agree), and `language`
    // (`Option<Language>`, from the curated BCP-47 generator).
    let t = crate::audio::Tags::new()
      .with_title(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_artist(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_album_artist(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_album(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_composer(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_genre(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_comment(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_year(<u16 as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .with_track_number(<u16 as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .with_track_total(<u16 as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .with_disc_number(<u16 as ::arbitrary::Arbitrary>::arbitrary(u)?)
      .with_disc_total(<u16 as ::arbitrary::Arbitrary>::arbitrary(u)?)
      // `language` is `Option<LanguageId>` — 50/50 `None` / `Some(<curated
      // BCP 47 tag>)`, reusing the `LanguageId` arbitrary impl in this module.
      .maybe_language(if <bool as ::arbitrary::Arbitrary>::arbitrary(u)? {
        Some(<crate::lang::LanguageId as ::arbitrary::Arbitrary>::arbitrary(u)?)
      } else {
        None
      });
    Ok(t)
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::capture::Device {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Both fields are `Utf8Bytes` with empty-string-means-absent semantics;
    // pass arbitrary strings straight through.
    let d = crate::capture::Device::new()
      .with_make(::smol_bytes::Utf8Bytes::from(
        <::std::string::String as ::arbitrary::Arbitrary>::arbitrary(u)?,
      ))
      .with_model(::smol_bytes::Utf8Bytes::from(
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

impl<'a> ::arbitrary::Arbitrary<'a> for crate::lang::LanguageId {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // Curated BCP 47 tags the whole-tag door accepts — language-only,
    // language+region, language+script+region, the `und` sentinel, and a
    // lossless tail, which is the seat this type has and its predecessor
    // did not.
    //
    // CANONICAL spellings only. `Arbitrary` feeds round-trip suites, and a
    // fold at the door would make a rendered-then-reparsed value differ from
    // the generated one — which is the fold working and would read as a codec
    // bug. The dirty spellings are pinned in the household's own tests.
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
    let tag: &&str = u.choose(TAGS)?;
    Ok(crate::lang::LanguageId::new(tag).expect("curated BCP 47 tag must parse"))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::lang::Language {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    const SUBTAGS: &[&str] = &[
      "und", "en", "de", "fr", "es", "zh", "ja", "yue", "ar", "qaa",
    ];
    let subtag: &&str = u.choose(SUBTAGS)?;
    Ok(crate::lang::Language::new(subtag).expect("curated language subtag must parse"))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::lang::ScriptSubtag {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    const SUBTAGS: &[&str] = &[
      "Latn", "Hans", "Hant", "Cyrl", "Arab", "Jpan", "Zxxx", "Zzzz",
    ];
    let subtag: &&str = u.choose(SUBTAGS)?;
    Ok(crate::lang::ScriptSubtag::new(subtag).expect("curated script subtag must parse"))
  }
}

impl<'a> ::arbitrary::Arbitrary<'a> for crate::lang::Region {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    // BOTH region grammars: ISO 3166-1 country codes and UN M.49 area codes,
    // the second of which a letters-only roster would never reach.
    const SUBTAGS: &[&str] = &["US", "DE", "TW", "CN", "BR", "ZZ", "419", "001", "150"];
    let subtag: &&str = u.choose(SUBTAGS)?;
    Ok(crate::lang::Region::new(subtag).expect("curated region subtag must parse"))
  }
}
