use super::*;
use ::std::string::ToString;

#[test]
fn every_named_variant_round_trips() {
  for slug in [
    "jpeg", "png", "heif", "heic", "avif", "tiff", "webp", "gif", "bmp", "dng", "cr2", "cr3",
    "nef", "nrw", "arw", "orf", "rw2", "raf", "pef", "srw", "rwl", "iiq", "3fr", "x3f", "mrw",
    "gpr",
  ] {
    let v: Format = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn unknown_slug_lands_in_other() {
  let v: Format = "weird_image".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_image");
  assert_eq!(v.to_string(), "weird_image");
}

#[test]
fn display_matches_as_str() {
  assert_eq!(Format::Jpeg.to_string(), "jpeg");
  assert_eq!(Format::Threefr.to_string(), "3fr");
  assert_eq!(
    Format::Other(Utf8Bytes::from("custom")).to_string(),
    "custom"
  );
}

#[test]
fn is_variant_predicates() {
  // Auto-derived predicates for plain-letter variants.
  assert!(Format::Jpeg.is_jpeg());
  assert!(!Format::Png.is_jpeg());
  assert!(Format::Threefr.is_threefr());
  assert!(Format::Other(Utf8Bytes::from("x")).is_other());
  // Hand-written overrides (see the `#[is_variant(ignore)]` attributes on
  // the digit-suffixed variants — the auto-derived name would be
  // digit-snake-case).
  assert!(Format::Cr2.is_cr2());
  assert!(!Format::Cr3.is_cr2());
  assert!(Format::Cr3.is_cr3());
  assert!(Format::Rw2.is_rw2());
  assert!(Format::X3f.is_x3f());
}

#[test]
fn unwrap_other_borrowed_view() {
  // `Other(Utf8Bytes)` carries data — golden-rule §2 mandates unwrap /
  // try_unwrap accessors for data-carrying variants.
  let v = Format::Other(Utf8Bytes::from("custom"));
  assert_eq!(v.unwrap_other_ref().as_str(), "custom");
  assert!(v.try_unwrap_other_ref().is_ok());
  let named = Format::Jpeg;
  assert!(named.try_unwrap_other_ref().is_err());
}

#[test]
fn as_extension_matches_disk_form() {
  for (variant, ext) in [
    (Format::Jpeg, "jpg"),
    (Format::Png, "png"),
    (Format::Heif, "heif"),
    (Format::Heic, "heic"),
    (Format::Avif, "avif"),
    (Format::Tiff, "tif"),
    (Format::Webp, "webp"),
    (Format::Gif, "gif"),
    (Format::Bmp, "bmp"),
    (Format::Dng, "dng"),
    (Format::Cr2, "cr2"),
    (Format::Cr3, "cr3"),
    (Format::Nef, "nef"),
    (Format::Nrw, "nrw"),
    (Format::Arw, "arw"),
    (Format::Orf, "orf"),
    (Format::Rw2, "rw2"),
    (Format::Raf, "raf"),
    (Format::Pef, "pef"),
    (Format::Srw, "srw"),
    (Format::Rwl, "rwl"),
    (Format::Iiq, "iiq"),
    (Format::Threefr, "3fr"),
    (Format::X3f, "x3f"),
    (Format::Mrw, "mrw"),
    (Format::Gpr, "gpr"),
  ] {
    assert_eq!(variant.as_extension(), ext, "{variant:?}");
  }
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

  // The specific multi-spelling groups the module doc calls out, spelled
  // out explicitly so a trimmed alias list fails loudly here rather than
  // only in the generic loop above.
  assert_eq!(Format::Jpeg.extensions(), &["jpg", "jpeg", "jpe"]);
  // R6: `Heif` (generic, `mif1`-brand) and `Heic` (HEVC-brand) are
  // separate variants — see both variants' own docs. R8: `.hif` is
  // deliberately excluded from both — IANA permits it for either
  // subtype, so an extension-only mapping can't pick one; see `Heic`'s
  // own doc and `r8_hif_routes_to_other_not_heic_or_heif` below.
  assert_eq!(Format::Heif.extensions(), &["heif"]);
  assert_eq!(Format::Heic.extensions(), &["heic"]);
  assert!(!Format::Heif.extensions().contains(&"hif"));
  assert!(!Format::Heic.extensions().contains(&"hif"));
  assert_eq!(Format::Tiff.extensions(), &["tif", "tiff"]);
  assert_eq!(Format::Bmp.extensions(), &["bmp", "dib"]);
  assert_eq!(Format::Orf.extensions(), &["orf", "ori"]);
}

/// R6 regression: `.heic` was briefly (R3) merged onto `Heif` under one
/// collapsed variant. It now routes to its own promoted `Heic` variant,
/// ignore-case, and the old `Heif` route for that spelling is gone —
/// pinned explicitly so a future edit can't silently re-collapse it.
///
/// `.hif` is deliberately **not** in the loop below — R6 put it there
/// (on the strength of Canon's real files carrying the `heix` HEVC
/// brand), R8 excluded it from both `Heic` and `Heif` outright (that
/// evidence proves frequency, not totality — IANA permits `.hif` for
/// either subtype); see [`r8_hif_routes_to_other_not_heic_or_heif`] for
/// that pin specifically.
#[test]
fn r6_heic_routes_to_its_own_variant_and_the_heif_route_is_gone_for_heic() {
  for ext in ["heic", "HEIC"] {
    let v: Format = ext.parse().unwrap();
    assert_eq!(v, Format::Heic, "`{ext}` must route to Heic");
    assert_ne!(v, Format::Heif, "`{ext}` must NOT route to Heif");
  }
  // And the survivor stays exactly where it was: `.heif` alone on `Heif`.
  assert_eq!("heif".parse::<Format>().unwrap(), Format::Heif);
  assert_eq!("HEIF".parse::<Format>().unwrap(), Format::Heif);
}

/// R8 regression: `.hif` was briefly (R6) routed unconditionally to
/// `Heic`, reasoning from Canon's real files (the dominant real-world
/// `.hif` producer) carrying the HEVC `heix` brand. That evidence proves
/// frequency, not totality: IANA's `image/heic` registration lists the
/// file extension as `"hif (for subtypes heif and heic)"`, so `.hif`
/// legitimately names either subtype and a `FromStr` total mapping to
/// one variant can't be justified by extension alone — the same
/// reasoning that already keeps [`Format::Avif`]'s IANA-listed
/// `heif`/`hif` spellings off `Avif`. `.hif` now parses to
/// [`Format::Other`], carrying its own name — pinned explicitly, any
/// case, so a future edit can't silently re-attach it to either variant.
#[test]
fn r8_hif_routes_to_other_not_heic_or_heif() {
  for ext in ["hif", "HIF", "Hif"] {
    let v: Format = ext.parse().unwrap();
    assert_eq!(
      v,
      Format::other(ext),
      "`{ext}` must route to the open escape, carrying its own name"
    );
    assert_eq!(
      v.as_str(),
      ext,
      "`{ext}` must survive verbatim in the escape, not folded"
    );
    assert!(v.is_other(), "`{ext}` must NOT resolve to a named variant");
    assert_ne!(v, Format::Heic, "`{ext}` must NOT route to Heic");
    assert_ne!(v, Format::Heif, "`{ext}` must NOT route to Heif");
  }
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with `Self::other` running that same lookup, so
/// one **named** meaning is one value under the derived `Eq` / `Hash`.
/// Covers both text faces: the `as_str()` slugs and the `extensions()`
/// aliases.
#[test]
fn format_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &[
    "jpeg", "png", "heif", "heic", "avif", "tiff", "webp", "gif", "bmp", "dng", "cr2", "cr3",
    "nef", "nrw", "arw", "orf", "rw2", "raf", "pef", "srw", "rwl", "iiq", "3fr", "x3f", "mrw",
    "gpr",
  ];
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
  assert_eq!("jpg", "JPG".parse::<Format>().unwrap().as_extension());
  assert_eq!("cr2", "CR2".parse::<Format>().unwrap().as_str());

  // Every alias extension is also lowercase-canonical and case-folds.
  // `.hif` is deliberately absent — R8 excluded it from every roster, so
  // it is not a named-variant alias (see `r8_hif_routes_to_other_not_heic_or_heif`).
  const ALIASES: &[&str] = &["jpg", "jpe", "heic", "tif", "dib", "ori"];
  for alias in ALIASES {
    assert!(!alias.bytes().any(|b| b.is_ascii_uppercase()));
    let v: Format = alias.parse().unwrap();
    assert!(!v.is_other(), "alias `{alias}` should be a named variant");
    let shouted: Format = alias.to_ascii_uppercase().parse().unwrap();
    assert_eq!(v, shouted, "alias `{alias}` does not fold by case");
  }

  // `other()` heals a canonical name to the named variant...
  assert_eq!(Format::other("JPG"), Format::Jpeg);
  assert_eq!(Format::other("jpg"), Format::Jpeg);

  // ...and a documented alias extension, same as `FromStr`.
  assert_eq!(Format::other("dib"), Format::Bmp);
  assert_eq!(Format::other("DIB"), Format::Bmp);

  // ...but a genuine stranger keeps its own spelling verbatim — the
  // escape is a passthrough, not a fold target.
  let escaped: Format = "WEIRD_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "WEIRD_X");
  assert_eq!(Format::other("WEIRD_X"), escaped);
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
