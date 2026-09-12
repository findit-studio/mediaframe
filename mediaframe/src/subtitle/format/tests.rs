use super::*;
use ::std::string::ToString;

/// Every named variant's slug round-trips through `as_str` →
/// `FromStr`.
const NAMED_SLUGS: &[(&str, Format)] = &[
  ("srt", Format::Srt),
  ("webvtt", Format::WebVtt),
  ("ass", Format::Ass),
  ("ssa", Format::Ssa),
  ("microdvd", Format::Sub),
  ("mpl2", Format::Mpl2),
  ("lrc", Format::Lrc),
  ("sami", Format::Smi),
  ("stl", Format::Stl),
  ("subviewer", Format::Sbv),
  ("ttml", Format::Ttml),
  ("mov_text", Format::MovText),
  ("dvd_subtitle", Format::DvdSub),
  ("hdmv_pgs_subtitle", Format::HdmvPgs),
  ("dvb_subtitle", Format::DvbSub),
  ("xsub", Format::XSub),
];

#[test]
fn as_str_round_trips_for_every_named_variant() {
  for (slug, variant) in NAMED_SLUGS {
    assert_eq!(variant.as_str(), *slug, "as_str mismatch for {variant:?}");
    let parsed: Format = slug.parse().unwrap();
    assert_eq!(&parsed, variant, "FromStr mismatch for {slug:?}");
  }
}

/// One format, one variant, one slug. The pair `PgsSub` / `HdmvPgs`
/// was merged in 0.5.0; what remains is a plain round trip with no
/// canonicalisation step to explain.
#[test]
fn hdmv_pgs_round_trips() {
  assert_eq!(Format::HdmvPgs.as_str(), "hdmv_pgs_subtitle");
  let parsed: Format = "hdmv_pgs_subtitle".parse().unwrap();
  assert_eq!(parsed, Format::HdmvPgs);
}

#[test]
fn from_str_is_total_for_unknown_slug() {
  let parsed: Format = "definitely_not_a_real_subtitle_format_xyz".parse().unwrap();
  assert!(matches!(parsed, Format::Other(_)));
  assert_eq!(parsed.as_str(), "definitely_not_a_real_subtitle_format_xyz");
}

#[test]
fn is_image_based_classifies_known_variants() {
  // Image-based.
  assert_eq!(Format::DvdSub.is_image_based(), Some(true));
  assert_eq!(Format::HdmvPgs.is_image_based(), Some(true));
  assert_eq!(Format::DvbSub.is_image_based(), Some(true));
  assert_eq!(Format::XSub.is_image_based(), Some(true));
  // Text-based.
  assert_eq!(Format::Srt.is_image_based(), Some(false));
  assert_eq!(Format::WebVtt.is_image_based(), Some(false));
  assert_eq!(Format::Ass.is_image_based(), Some(false));
  assert_eq!(Format::MovText.is_image_based(), Some(false));
  // Unknown.
  assert_eq!(
    Format::Other(Utf8Bytes::from("weird")).is_image_based(),
    None,
  );
}

#[test]
fn display_matches_as_str() {
  for (_slug, variant) in NAMED_SLUGS {
    assert_eq!(variant.to_string(), variant.as_str());
  }
  assert_eq!(
    Format::Other(Utf8Bytes::from("custom_fmt")).to_string(),
    "custom_fmt",
  );
}

#[test]
fn is_variant_predicates() {
  assert!(Format::Srt.is_srt());
  assert!(!Format::Srt.is_web_vtt());
  assert!(Format::Other(Utf8Bytes::from("x")).is_other());
}

#[test]
fn as_extension_matches_disk_form() {
  // Text-based formats: extension is the canonical .ext (often differs
  // from the FFmpeg slug, e.g. WebVtt slug "webvtt" vs ext "vtt").
  for (variant, ext) in [
    (Format::Srt, "srt"),
    (Format::WebVtt, "vtt"),
    (Format::Ass, "ass"),
    (Format::Ssa, "ssa"),
    (Format::Sub, "sub"),
    (Format::Mpl2, "mpl"),
    (Format::Lrc, "lrc"),
    (Format::Smi, "smi"),
    (Format::Stl, "stl"),
    (Format::Sbv, "sbv"),
    (Format::Ttml, "ttml"),
  ] {
    assert_eq!(variant.as_extension(), ext, "{variant:?}");
  }
  // Image-based + container-embedded: no standalone extension.
  for variant in [
    Format::MovText,
    Format::DvdSub,
    Format::HdmvPgs,
    Format::DvbSub,
    Format::XSub,
  ] {
    assert_eq!(variant.as_extension(), "", "{variant:?}");
  }
  // Other: unknown.
  assert_eq!(Format::Other(Utf8Bytes::from("custom")).as_extension(), "");
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with `Self::other` running that same lookup, so
/// one **named** meaning is one value under the derived `Eq` / `Hash`.
#[test]
fn format_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &["srt", "webvtt", "ass", "ssa", "ttml", "mov_text"];
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
  assert_eq!("srt", "SRT".parse::<Format>().unwrap().as_str());

  // `other()` heals a canonical name to the named variant...
  assert_eq!(Format::other("SRT"), Format::Srt);
  assert_eq!(Format::other("srt"), Format::Srt);

  // ...but a genuine stranger keeps its own spelling verbatim — the
  // escape is a passthrough, not a fold target.
  let escaped: Format = "SRT_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "SRT_X");
  assert_eq!(Format::other("SRT_X"), escaped);
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
