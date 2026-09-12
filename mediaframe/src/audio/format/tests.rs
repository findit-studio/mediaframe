use super::*;
use ::std::string::ToString;

#[test]
fn audio_format_u32_round_trips_named_variants() {
  for v in [
    SampleFormat::U8,
    SampleFormat::S16,
    SampleFormat::S32,
    SampleFormat::Flt,
    SampleFormat::Dbl,
    SampleFormat::U8p,
    SampleFormat::S16p,
    SampleFormat::S32p,
    SampleFormat::Fltp,
    SampleFormat::Dblp,
    SampleFormat::S64,
    SampleFormat::S64p,
  ] {
    let back = SampleFormat::from_u32(v.to_u32().expect("named format has a code"));
    assert_eq!(
      back,
      Some(v.clone()),
      "round-trip mismatch for `{}`",
      v.as_str()
    );
  }
}

#[test]
fn audio_format_unnamed_code_is_rejected_and_the_escape_keeps_its_name() {
  assert_eq!(SampleFormat::from_u32(12_345), None);
  let vendor = SampleFormat::other("VENDOR_S24");
  assert_eq!(vendor.as_str(), "VENDOR_S24");
  assert_eq!(vendor.to_u32(), None);
}

#[test]
fn audio_format_from_str_named() {
  for slug in [
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
  ] {
    let v: SampleFormat = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn audio_format_unknown_slug_lands_in_other() {
  let v: SampleFormat = "weird_sample_fmt".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_sample_fmt");
}

#[test]
fn audio_format_is_planar_predicate() {
  assert!(SampleFormat::U8p.is_planar());
  assert!(SampleFormat::S16p.is_planar());
  assert!(SampleFormat::Fltp.is_planar());
  assert!(!SampleFormat::U8.is_planar());
  assert!(!SampleFormat::Flt.is_planar());
}

#[test]
fn audio_format_display_matches_as_str() {
  assert_eq!(SampleFormat::Flt.to_string(), "flt");
  assert_eq!(SampleFormat::Fltp.to_string(), "fltp");
}

#[test]
fn audio_container_round_trips_named_variants() {
  for slug in [
    "mp3", "aac", "flac", "ogg", "opus", "wav", "aiff", "aifc", "alac", "wma", "ape", "wv", "mka",
    "m4a", "caf",
  ] {
    let v: ContainerFormat = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn audio_container_unknown_lands_in_other() {
  let v: ContainerFormat = "weird_audio_container".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_audio_container");
}

#[test]
fn audio_container_display_matches_as_str() {
  assert_eq!(ContainerFormat::Mp3.to_string(), "mp3");
  assert_eq!(ContainerFormat::Flac.to_string(), "flac");
  assert_eq!(
    ContainerFormat::Other(Utf8Bytes::from("snd")).to_string(),
    "snd"
  );
}

#[test]
fn audio_container_unwrap_other_borrowed_view() {
  // `Other(Utf8Bytes)` carries data — golden-rule §2 mandates
  // unwrap/try_unwrap accessors for data-carrying variants.
  let v = ContainerFormat::Other(Utf8Bytes::from("custom_audio"));
  assert_eq!(v.unwrap_other_ref().as_str(), "custom_audio");
  assert!(v.try_unwrap_other_ref().is_ok());
  let named = ContainerFormat::Flac;
  assert!(named.try_unwrap_other_ref().is_err());
}

#[test]
fn audio_container_as_extension_matches_disk_form() {
  // Most variants: slug == extension.
  for (variant, ext) in [
    (ContainerFormat::Mp3, "mp3"),
    (ContainerFormat::Aac, "aac"),
    (ContainerFormat::Flac, "flac"),
    (ContainerFormat::Ogg, "ogg"),
    (ContainerFormat::Opus, "opus"),
    (ContainerFormat::Wav, "wav"),
    (ContainerFormat::Aiff, "aiff"),
    (ContainerFormat::Aifc, "aifc"),
    (ContainerFormat::Wma, "wma"),
    (ContainerFormat::Ape, "ape"),
    (ContainerFormat::Wv, "wv"),
    (ContainerFormat::Mka, "mka"),
    (ContainerFormat::M4a, "m4a"),
    (ContainerFormat::Caf, "caf"),
  ] {
    assert_eq!(variant.as_extension(), ext);
  }
  // ALAC has no standalone extension — rides in `.m4a`.
  assert_eq!(ContainerFormat::Alac.as_str(), "alac");
  assert_eq!(ContainerFormat::Alac.as_extension(), "m4a");
  // Other has no known extension.
  assert_eq!(
    ContainerFormat::Other(Utf8Bytes::from("weird")).as_extension(),
    ""
  );
}

/// The extension face's own contract: `as_extension()` is always
/// `extensions()[0]`, every entry in `extensions()` parses (ignore-case)
/// back to the same variant, and `Other` carries neither.
///
/// **One documented, pre-existing exception**: `Alac.as_extension()` is
/// `"m4a"` — ALAC has no standalone container, so it reports the shared
/// one — but `"m4a"` has always parsed to [`ContainerFormat::M4a`] (the
/// literal match arm `FromStr` picks), never back to `Alac`. This predates
/// the `extensions()` face entirely; `check` never saw it because it
/// round-trips through `as_str()` (`"alac"`), not `as_extension()`. Every
/// OTHER variant's every spelling is asserted to round-trip to itself with
/// no exception, so a *new* extension/parse drift still fails loudly here.
#[test]
fn audio_container_extensions_are_canonical_first_and_every_alias_parses() {
  for v in ContainerFormat::ROSTER {
    let exts = v.extensions();
    assert!(!exts.is_empty(), "{v:?}: extensions() is empty");
    assert_eq!(
      exts[0],
      v.as_extension(),
      "{v:?}: extensions()[0] must be as_extension()"
    );
    for ext in exts {
      let parsed: ContainerFormat = ext.parse().unwrap();
      if *v == ContainerFormat::Alac && *ext == "m4a" {
        assert_eq!(
          parsed,
          ContainerFormat::M4a,
          "the one documented Alac/M4a `.m4a` sharing changed shape"
        );
        continue;
      }
      assert_eq!(&parsed, v, "extension `{ext}` did not parse back to {v:?}");
      let shouted: ContainerFormat = ext.to_ascii_uppercase().parse().unwrap();
      assert_eq!(
        &shouted, v,
        "extension `{ext}` (uppercased) did not parse back to {v:?}"
      );
    }
  }
  assert_eq!(
    ContainerFormat::Other(Utf8Bytes::from("weird")).extensions(),
    &[] as &[&str]
  );

  // The specific multi-spelling groups the variant docs call out, spelled
  // out explicitly so a trimmed alias list fails loudly here rather than
  // only in the generic loop above.
  // R5: `.aifc` belongs to `Aifc`, not `Aiff` — see both variants' own
  // docs. `Aiff` carries only the true byte-identical alias, `.aif`.
  assert_eq!(ContainerFormat::Aiff.extensions(), &["aiff", "aif"]);
  assert_eq!(ContainerFormat::Aifc.extensions(), &["aifc"]);
  assert_eq!(ContainerFormat::Wv.extensions(), &["wv", "wvp"]);
  // RFC 5334 §10.3 registers `.ogg`/`.oga`/`.spx` together under
  // `audio/ogg` — see the variant's own doc.
  assert_eq!(ContainerFormat::Ogg.extensions(), &["ogg", "oga", "spx"]);
  // IANA `audio/aac` + ffmpeg's own `adts` muxer/demuxer both name
  // `.adts` — see the variant's own doc.
  assert_eq!(ContainerFormat::Aac.extensions(), &["aac", "adts"]);
  assert_eq!(
    "adts".parse::<ContainerFormat>().unwrap(),
    ContainerFormat::Aac
  );
  assert_eq!(
    "ADTS".parse::<ContainerFormat>().unwrap(),
    ContainerFormat::Aac
  );
  // ffmpeg's dedicated `ape` demuxer names `.mac` alongside `.ape` —
  // see the variant's own doc.
  assert_eq!(ContainerFormat::Ape.extensions(), &["ape", "mac"]);
  // `.apl` is deliberately NOT an Ape alias — see the variant's own doc
  // (R4: APE Link is a sidecar/track-split file, not the bitstream;
  // ffmpeg's shared demuxer listing was a hint to check, not proof).
  // Pinned so this cannot silently regress back: `.apl` must land on
  // the open escape, carrying its own name, not resolve to a variant.
  assert!(!ContainerFormat::Ape.extensions().contains(&"apl"));
  assert_eq!(
    "apl".parse::<ContainerFormat>().unwrap(),
    ContainerFormat::other("apl")
  );
  assert_eq!(
    "APL".parse::<ContainerFormat>().unwrap(),
    ContainerFormat::other("APL")
  );
  // A genuine stranger's spelling is not folded — the two cases are
  // distinct `Other` values now.
  assert_ne!(
    "apl".parse::<ContainerFormat>().unwrap(),
    "APL".parse::<ContainerFormat>().unwrap()
  );
  // `.caf` is deliberately NOT an Alac alias — see the variant's own doc.
  assert_eq!(ContainerFormat::Alac.extensions(), &["m4a"]);
  assert!(!ContainerFormat::Alac.extensions().contains(&"caf"));
}

/// R5 regression: `.aifc` was briefly (R2) misattributed to `Aiff` —
/// the same class of mistake `.apl` made on `Ape` one round later. It
/// now routes to its own promoted `Aifc` variant, ignore-case, and the
/// old `Aiff` route is gone — pinned explicitly so a future edit can't
/// silently fold it back in.
#[test]
fn r5_aifc_routes_to_its_own_variant_and_the_aiff_route_is_gone() {
  for ext in ["aifc", "AIFC", "Aifc"] {
    let v: ContainerFormat = ext.parse().unwrap();
    assert_eq!(v, ContainerFormat::Aifc, "`{ext}` must route to Aifc");
    assert_ne!(v, ContainerFormat::Aiff, "`{ext}` must NOT route to Aiff");
  }
  // And the survivors stay exactly where they were.
  assert_eq!(
    "aiff".parse::<ContainerFormat>().unwrap(),
    ContainerFormat::Aiff
  );
  assert_eq!(
    "aif".parse::<ContainerFormat>().unwrap(),
    ContainerFormat::Aiff
  );
}

/// Both audio vocabularies are lowercase-canonical, collision-free once
/// folded, and read case-insensitively — escape included.
#[test]
fn audio_slugs_are_lowercase_canonical_and_fold() {
  const SAMPLE: &[&str] = &[
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
  ];
  const CONTAINER: &[&str] = &[
    "mp3", "aac", "flac", "ogg", "opus", "wav", "aiff", "alac", "wma", "ape", "wv", "mka", "m4a",
    "caf",
  ];

  for (i, slug) in SAMPLE.iter().enumerate() {
    assert!(!slug.bytes().any(|b| b.is_ascii_uppercase()));
    for prior in &SAMPLE[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two formats fold onto {slug:?}"
      );
    }
    let v: SampleFormat = slug.parse().unwrap();
    assert!(!v.is_other());
    assert_eq!(v.as_str(), *slug);
  }
  for (i, slug) in CONTAINER.iter().enumerate() {
    assert!(!slug.bytes().any(|b| b.is_ascii_uppercase()));
    for prior in &CONTAINER[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two containers fold onto {slug:?}"
      );
    }
    let v: ContainerFormat = slug.parse().unwrap();
    assert!(!v.is_other());
    assert_eq!(v.as_str(), *slug);
  }

  assert_eq!("S16".parse(), Ok(SampleFormat::S16));
  assert_eq!("FLTP".parse(), Ok(SampleFormat::Fltp));
  assert_eq!("FLAC".parse(), Ok(ContainerFormat::Flac));

  // `other()` heals a canonical name to the named variant...
  assert_eq!(SampleFormat::other("S16"), SampleFormat::S16);
  assert_eq!(SampleFormat::other("s16"), SampleFormat::S16);
  assert_eq!(ContainerFormat::other("FLAC"), ContainerFormat::Flac);
  assert_eq!(ContainerFormat::other("flac"), ContainerFormat::Flac);

  // ...and a documented alias extension, same as `FromStr`.
  assert_eq!(ContainerFormat::other("adts"), ContainerFormat::Aac);
  assert_eq!(ContainerFormat::other("ADTS"), ContainerFormat::Aac);

  // ...but a genuine stranger keeps its own spelling verbatim.
  assert_eq!(SampleFormat::other("VENDOR_S24").as_str(), "VENDOR_S24");
  assert_eq!(ContainerFormat::other("SND").as_str(), "SND");
}
#[test]
fn sample_format_unwrap_other_borrowed_view() {
  // `Other(Utf8Bytes)` carries data — golden-rule §2 mandates
  // unwrap/try_unwrap accessors for data-carrying variants, and this
  // type's 12 variants are far under the compile-time threshold that
  // exempts the 200-plus codec enums.
  let v = SampleFormat::other("vendor_s24");
  assert_eq!(v.unwrap_other_ref().as_str(), "vendor_s24");
  assert!(v.try_unwrap_other_ref().is_ok());
  assert!(SampleFormat::S16.try_unwrap_other_ref().is_err());
}

/// The runtime half of the `ROSTER` contract for `SampleFormat` / `ContainerFormat` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(SampleFormat::ROSTER, "SampleFormat", SampleFormat::as_str);
  crate::roster_tests::check(
    ContainerFormat::ROSTER,
    "ContainerFormat",
    ContainerFormat::as_str,
  );
}
