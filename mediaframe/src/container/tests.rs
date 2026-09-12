use super::*;
use ::std::string::ToString;

#[test]
fn every_named_variant_round_trips() {
  for slug in [
    "mov", "mp4", "mkv", "webm", "avi", "flv", "mpegts", "m2ts", "ogg", "asf", "rm", "wmv", "mxf",
    "gxf", "3gp", "3g2",
  ] {
    let v: Format = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn unknown_slug_lands_in_other() {
  let v: Format = "weird_container".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_container");
  assert_eq!(v.to_string(), "weird_container");
}

#[test]
fn display_matches_as_str() {
  assert_eq!(Format::Mp4.to_string(), "mp4");
  assert_eq!(Format::MpegTs.to_string(), "mpegts");
  assert_eq!(Format::M2ts.to_string(), "m2ts");
  assert_eq!(Format::Threegp.to_string(), "3gp");
  assert_eq!(Format::Threeg2.to_string(), "3g2");
  assert_eq!(
    Format::Other(Utf8Bytes::from("custom")).to_string(),
    "custom"
  );
}

#[test]
fn is_variant_predicates() {
  // Hand-written `is_mp4` (vs the auto-derived `is_mp_4` that the
  // `IsVariant` derive would otherwise produce) — see the
  // `#[is_variant(ignore)]` attribute on `Format::Mp4`.
  assert!(Format::Mp4.is_mp4());
  assert!(!Format::Mkv.is_mp4());
  assert!(Format::Threegp.is_threegp());
  // Hand-written overrides for the R5-promoted digit-bearing variants —
  // see their own `#[is_variant(ignore)]` attributes.
  assert!(Format::M2ts.is_m2ts());
  assert!(!Format::MpegTs.is_m2ts());
  assert!(Format::Threeg2.is_threeg2());
  assert!(!Format::Threegp.is_threeg2());
  assert!(Format::Other(Utf8Bytes::from("x")).is_other());
}

#[test]
fn unwrap_other_borrowed_view() {
  // `Other(Utf8Bytes)` carries data — golden-rule §2 mandates
  // unwrap/try_unwrap accessors for data-carrying variants.
  let v = Format::Other(Utf8Bytes::from("custom"));
  assert_eq!(v.unwrap_other_ref().as_str(), "custom");
  assert!(v.try_unwrap_other_ref().is_ok());
  let named = Format::Mp4;
  assert!(named.try_unwrap_other_ref().is_err());
}

#[test]
fn as_extension_matches_disk_form() {
  // Most variants: slug == extension.
  assert_eq!(Format::Mov.as_extension(), "mov");
  assert_eq!(Format::Mp4.as_extension(), "mp4");
  assert_eq!(Format::Mkv.as_extension(), "mkv");
  assert_eq!(Format::Webm.as_extension(), "webm");
  assert_eq!(Format::Avi.as_extension(), "avi");
  assert_eq!(Format::Flv.as_extension(), "flv");
  assert_eq!(Format::Threegp.as_extension(), "3gp");
  assert_eq!(Format::Threeg2.as_extension(), "3g2");
  assert_eq!(Format::M2ts.as_extension(), "m2ts");
  // Variants where extension differs from FFmpeg slug.
  assert_eq!(Format::MpegTs.as_str(), "mpegts");
  assert_eq!(Format::MpegTs.as_extension(), "ts");
  assert_eq!(Format::Ogg.as_str(), "ogg");
  assert_eq!(Format::Ogg.as_extension(), "ogv");
  // Other has no known extension.
  assert_eq!(Format::Other(Utf8Bytes::from("weird")).as_extension(), "");
}

/// The extension face's own contract: `as_extension()` is always
/// `extensions()[0]`, every entry in `extensions()` parses (ignore-case)
/// back to the same variant, and `Other` carries neither.
#[test]
fn extensions_are_canonical_first_and_every_alias_parses() {
  for v in Format::ROSTER {
    let exts = v.extensions();
    assert!(!exts.is_empty(), "{v:?}: extensions() is empty");
    assert_eq!(
      exts[0],
      v.as_extension(),
      "{v:?}: extensions()[0] must be as_extension()"
    );
    for ext in exts {
      let parsed: Format = ext.parse().unwrap();
      assert_eq!(&parsed, v, "extension `{ext}` did not parse back to {v:?}");
      let shouted: Format = ext.to_ascii_uppercase().parse().unwrap();
      assert_eq!(
        &shouted, v,
        "extension `{ext}` (uppercased) did not parse back to {v:?}"
      );
    }
  }
  assert_eq!(
    Format::Other(Utf8Bytes::from("weird")).extensions(),
    &[] as &[&str]
  );

  // The specific multi-spelling groups the type doc calls out, spelled
  // out explicitly so a trimmed alias list fails loudly here rather than
  // only in the generic loop above.
  assert_eq!(Format::Mov.extensions(), &["mov", "qt"]);
  assert_eq!(Format::Mp4.extensions(), &["mp4", "mpg4"]);
  // R5: `.m2ts`/`.mts` belong to `M2ts`, not `MpegTs` — see both
  // variants' own docs. R7: `.m2t` moved back to `MpegTs` — ExifTool's
  // own content detector (M2TS.pm) labels unprefixed 188-byte packets
  // `M2T`, reserving `M2TS` for the 192-byte prefixed form; `.m2t`
  // names the 188-byte (`MpegTs`) world, not the BDAV one.
  assert_eq!(Format::MpegTs.extensions(), &["ts", "m2t"]);
  assert_eq!(Format::M2ts.extensions(), &["m2ts", "mts"]);
  assert_eq!(Format::Ogg.extensions(), &["ogv", "ogx"]);
  // R5: `.3g2`/`.3gp2` belong to `Threeg2`, not `Threegp` — see both
  // variants' own docs.
  assert_eq!(Format::Threegp.extensions(), &["3gp", "3gpp"]);
  assert_eq!(Format::Threeg2.extensions(), &["3g2", "3gp2"]);
}

/// R5 regression: the extension groups that were briefly misattributed
/// (R1/R3) to `MpegTs`/`Threegp` now route to their own promoted
/// variants, ignore-case, and the *old* routes are gone — pinned
/// explicitly so a future edit can't silently fold them back in.
///
/// `.m2t` is deliberately **not** in the `M2ts` loop below — R5 put it
/// there, R7 moved it back to `MpegTs`; see
/// [`r7_m2t_routes_to_mpegts_not_m2ts`] for that pin specifically.
#[test]
fn r5_promoted_variants_route_correctly_and_old_routes_are_gone() {
  for ext in ["m2ts", "M2TS", "mts", "MTS"] {
    let v: Format = ext.parse().unwrap();
    assert_eq!(v, Format::M2ts, "`{ext}` must route to M2ts");
    assert_ne!(v, Format::MpegTs, "`{ext}` must NOT route to MpegTs");
  }
  for ext in ["3g2", "3G2", "3gp2", "3GP2"] {
    let v: Format = ext.parse().unwrap();
    assert_eq!(v, Format::Threeg2, "`{ext}` must route to Threeg2");
    assert_ne!(v, Format::Threegp, "`{ext}` must NOT route to Threegp");
  }
  // And the survivors stay exactly where they were.
  assert_eq!("ts".parse::<Format>().unwrap(), Format::MpegTs);
  assert_eq!("3gp".parse::<Format>().unwrap(), Format::Threegp);
  assert_eq!("3gpp".parse::<Format>().unwrap(), Format::Threegp);
}

/// R7 regression: `.m2t` was briefly (R5) misattributed to `M2ts`
/// alongside `.m2ts`/`.mts`, on ExifTool's *static* alias table alone.
/// ExifTool's own *content detector* (`M2TS.pm`'s `ProcessM2TS`) labels
/// unprefixed 188-byte packets `M2T`, reserving `M2TS` for the 192-byte
/// prefixed form — `.m2t` belongs to the 188-byte world (`MpegTs`), not
/// the BDAV one (`M2ts`). Pinned explicitly, both directions, so a
/// future edit can't silently swap it back.
#[test]
fn r7_m2t_routes_to_mpegts_not_m2ts() {
  for ext in ["m2t", "M2T", "M2t"] {
    let v: Format = ext.parse().unwrap();
    assert_eq!(v, Format::MpegTs, "`{ext}` must route to MpegTs");
    assert_ne!(v, Format::M2ts, "`{ext}` must NOT route to M2ts");
  }
  // `.m2ts`/`.mts` stay exactly where R5 put them.
  assert_eq!("m2ts".parse::<Format>().unwrap(), Format::M2ts);
  assert_eq!("mts".parse::<Format>().unwrap(), Format::M2ts);
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with `Self::other` running that same lookup, so
/// one **named** meaning is one value under the derived `Eq` / `Hash`.
#[test]
fn format_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &["mp4", "mkv", "webm", "mov", "avi", "mpegts", "flv", "ogg"];
  for (i, slug) in SLUGS.iter().enumerate() {
    assert!(
      !slug.bytes().any(|b| b.is_ascii_uppercase()),
      "slug {slug:?} is not lowercase-canonical"
    );
    for prior in &SLUGS[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two variants fold onto {slug:?}"
      );
    }
    let v: Format = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), *slug, "`{slug}` is not its own canonical form");
  }
  assert_eq!("mp4", "MP4".parse::<Format>().unwrap().as_str());

  // `other()` heals a canonical name to the named variant...
  assert_eq!(Format::other("MP4"), Format::Mp4);
  assert_eq!(Format::other("mp4"), Format::Mp4);

  // ...and a documented alias extension, same as `FromStr`.
  assert_eq!(Format::other("qt"), Format::Mov);
  assert_eq!(Format::other("QT"), Format::Mov);

  // ...but a genuine stranger keeps its own spelling verbatim — the
  // escape is a passthrough, not a fold target.
  let escaped: Format = "MP4_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "MP4_X");
  assert_eq!(Format::other("MP4_X"), escaped);
}

/// The runtime half of the `ROSTER` contract for `Format` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(Format::ROSTER, "Format", Format::as_str);
}
