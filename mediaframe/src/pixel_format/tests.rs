use super::*;
// `format!` lives in `alloc` under no_std + alloc; bring it in via the
// crate-root `extern crate alloc as std;` alias. Under `feature = "std"`
// this resolves to the real `std::format`.
#[cfg(any(feature = "std", feature = "alloc"))]
use std::format;

#[test]
fn default_is_none() {
  assert!(matches!(PixelFormat::default(), PixelFormat::None));
  assert_eq!(PixelFormat::None.as_str(), "none");
  assert_eq!("none".parse(), Ok(PixelFormat::None));
}

#[test]
fn round_trip_u32_for_known_variants() {
  let all = [
    PixelFormat::None,
    PixelFormat::Yuv420p,
    PixelFormat::Yuv444p,
    PixelFormat::Yuv420p10Le,
    PixelFormat::Yuv422p16Le,
    PixelFormat::Yuva444p,
    PixelFormat::Nv12,
    PixelFormat::P010Le,
    PixelFormat::P416Le,
    PixelFormat::Yuyv422,
    PixelFormat::V210,
    PixelFormat::V410Be,
    PixelFormat::Ayuv64Le,
    PixelFormat::Rgb24,
    PixelFormat::Bgra,
    PixelFormat::Rgb565Le,
    PixelFormat::Rgba64Le,
    PixelFormat::Rgbf32Le,
    PixelFormat::Gbrp,
    PixelFormat::Gbrap16Le,
    PixelFormat::Gbrapf32Le,
    PixelFormat::Gray8,
    PixelFormat::Gray16Le,
    PixelFormat::Ya16Le,
    PixelFormat::Monowhite,
    PixelFormat::Pal8,
    PixelFormat::BayerBggr8,
    PixelFormat::BayerRggb16Le,
    PixelFormat::BayerGrbg10Be,
    PixelFormat::BayerBggr14Be,
  ];
  for fmt in all {
    assert_eq!(
      PixelFormat::from_u32(fmt.to_u32().expect("named format has an id")),
      Some(fmt.clone()),
      "round-trip failed for {fmt:?}"
    );
  }
}

#[test]
fn garbage_u32_is_rejected_not_invented() {
  assert_eq!(PixelFormat::from_u32(99_999), None);
  assert_eq!(PixelFormat::from_u32(1), None);
  assert_eq!(PixelFormat::from_u32(u32::MAX), None);
  // Code 0 is FFmpeg's own "no format".
  assert_eq!(PixelFormat::from_u32(0), Some(PixelFormat::None));
}

// `format!` requires an allocator; gate to alloc-or-std builds.
// The `Display` impl itself works in bare-core mode via
// `write!`-style sinks — only this test's assertion strategy needs
// alloc.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn display_uses_ffmpeg_lowercase_names() {
  assert_eq!(format!("{}", PixelFormat::Yuv420p), "yuv420p");
  assert_eq!(format!("{}", PixelFormat::Nv12), "nv12");
  assert_eq!(format!("{}", PixelFormat::P010Le), "p010le");
  assert_eq!(format!("{}", PixelFormat::Rgba64Le), "rgba64le");
  assert_eq!(format!("{}", PixelFormat::None), "none");
}

// Sub-16-bit Bayer are mediaframe extensions (no FFmpeg pixel format);
// their Display slugs follow the same lowercase convention, but they are
// not FFmpeg names, so they live here rather than in the FFmpeg-name test.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn display_uses_lowercase_names_for_bayer_extensions() {
  assert_eq!(format!("{}", PixelFormat::BayerBggr12Le), "bayer_bggr12le");
  assert_eq!(format!("{}", PixelFormat::BayerRggb10Be), "bayer_rggb10be");
}

#[test]
fn is_bayer_partition() {
  assert!(PixelFormat::BayerBggr8.is_bayer());
  assert!(PixelFormat::BayerRggb16Le.is_bayer());
  assert!(PixelFormat::BayerGrbg12Le.is_bayer());
  assert!(PixelFormat::BayerGbrg10Be.is_bayer());
  assert!(PixelFormat::BayerGrbg14Be.is_bayer());
  assert!(!PixelFormat::Yuv420p.is_bayer());
  assert!(!PixelFormat::Rgb24.is_bayer());
  assert!(!PixelFormat::None.is_bayer());
}

#[test]
fn is_variant_helpers_compile() {
  assert!(PixelFormat::Yuv420p.is_yuv_420_p());
  assert!(PixelFormat::Nv12.is_nv_12());
  assert!(PixelFormat::P010Le.is_p_010_le());
  assert!(!PixelFormat::Yuv420p.is_none());
}

#[test]
fn clone_and_eq() {
  // `Copy` went with `Other(Utf8Bytes)` — the escape carries a name.
  let p = PixelFormat::Nv12;
  let q = p.clone();
  assert_eq!(p, q);
  assert_ne!(p, PixelFormat::Yuv420p);
}

#[test]
fn canonical_resolves_yuvj_aliases_to_full_range() {
  use crate::color::DynamicRange;
  assert_eq!(
    PixelFormat::Yuvj411p.canonical(),
    (PixelFormat::Yuv411p, Some(DynamicRange::Full))
  );
  assert_eq!(
    PixelFormat::Yuvj420p.canonical(),
    (PixelFormat::Yuv420p, Some(DynamicRange::Full))
  );
  assert_eq!(
    PixelFormat::Yuvj422p.canonical(),
    (PixelFormat::Yuv422p, Some(DynamicRange::Full))
  );
  assert_eq!(
    PixelFormat::Yuvj440p.canonical(),
    (PixelFormat::Yuv440p, Some(DynamicRange::Full))
  );
  assert_eq!(
    PixelFormat::Yuvj444p.canonical(),
    (PixelFormat::Yuv444p, Some(DynamicRange::Full))
  );
}

#[test]
fn canonical_resolves_gray_alpha_aliases_to_ya8() {
  assert_eq!(PixelFormat::Gray8a.canonical(), (PixelFormat::Ya8, None));
  assert_eq!(PixelFormat::Y400a.canonical(), (PixelFormat::Ya8, None));
}

#[test]
fn canonical_resolves_xv30le_to_v410le() {
  assert_eq!(PixelFormat::Xv30Le.canonical(), (PixelFormat::V410Le, None));
}

// `Xv30Be` (big-endian V410) resolves to the matching `V410Be` variant,
// mirroring the LE `Xv30Le` → `V410Le` mapping and preserving byte order.
#[test]
fn canonical_resolves_xv30be_to_v410be() {
  assert_eq!(PixelFormat::Xv30Be.canonical(), (PixelFormat::V410Be, None));
  // `V410Be` is itself canonical — a fixed point with no pinned range.
  assert_eq!(PixelFormat::V410Be.canonical(), (PixelFormat::V410Be, None));
}

#[test]
fn canonical_is_identity_for_non_aliases() {
  for fmt in [
    PixelFormat::None,
    PixelFormat::Yuv420p,
    PixelFormat::Yuv444p,
    PixelFormat::V410Le,
    PixelFormat::Ya8,
    PixelFormat::Nv12,
    PixelFormat::P010Le,
    PixelFormat::Rgb24,
    PixelFormat::Gray8,
    PixelFormat::Gbrp,
    PixelFormat::Pal8,
    PixelFormat::BayerBggr8,
  ] {
    assert_eq!(
      fmt.clone().canonical(),
      (fmt.clone(), None),
      "non-alias {fmt:?} must be its own canonical form"
    );
  }
}

// Idempotence: resolving an alias yields a *canonical* format, and that
// canonical format is not itself an alias — `canonical()` on it returns
// itself with no pinned range (a fixed point).
#[test]
fn canonical_is_idempotent() {
  let aliases = [
    PixelFormat::Yuvj411p,
    PixelFormat::Yuvj420p,
    PixelFormat::Yuvj422p,
    PixelFormat::Yuvj440p,
    PixelFormat::Yuvj444p,
    PixelFormat::Gray8a,
    PixelFormat::Y400a,
    PixelFormat::Xv30Le,
  ];
  for alias in aliases {
    let (canon, _range) = alias.clone().canonical();
    assert_eq!(
      canon.clone().canonical(),
      (canon.clone(), None),
      "canonical form {canon:?} of alias {alias:?} must be a fixed point with no pinned range"
    );
  }
}

/// Every named pixel format must survive `as_str()` → `FromStr`
/// unchanged, with no two variants sharing a slug. The sweep enumerates
/// through `from_u32` (ids run `0..=947`) rather than a hand-written
/// list, so a format added later is covered without touching this test.
#[test]
fn every_named_pixel_format_round_trips_through_its_slug() {
  // A fixed array, not a `Vec`: this type is available at the crate's
  // no-alloc tier and the test has to build there too.
  let mut named = 0usize;
  let mut codes = [0u32; 512];
  for code in 0..=4096u32 {
    let Some(value) = PixelFormat::from_u32(code) else {
      continue;
    };
    let slug = value.as_str();
    assert_eq!(
      slug.parse::<PixelFormat>(),
      Ok(value.clone()),
      "slug {slug:?} does not parse back to {value:?}"
    );
    assert!(
      !slug.bytes().any(|b| b.is_ascii_uppercase()),
      "pixel-format slug {slug:?} is not lowercase-canonical"
    );
    {
      let mut upper = [0u8; 64];
      let n = slug.len();
      upper[..n].copy_from_slice(slug.as_bytes());
      upper[..n].make_ascii_uppercase();
      let upper = core::str::from_utf8(&upper[..n]).unwrap();
      assert_eq!(
        upper.parse::<PixelFormat>(),
        Ok(value.clone()),
        "PixelFormat does not fold {upper:?} onto {slug:?}"
      );
    }
    for prior in codes.iter().take(named) {
      let prior = PixelFormat::from_u32(*prior).expect("recorded code names a format");
      assert_ne!(
        prior.as_str(),
        slug,
        "two pixel formats are spelled {slug:?}"
      );
    }
    codes[named] = code;
    named += 1;
  }
  assert!(
    named > 200,
    "sweep found only {named} named formats — the id range is wrong"
  );
}

/// The FFmpeg synonyms are **accepted, never emitted**, and none of them
/// shadows a canonical slug.
///
/// Two directions make that airtight together with
/// [`every_named_pixel_format_round_trips_through_its_slug`], which
/// proves every canonical slug still parses back to its own variant:
/// were a synonym to collide with some variant's slug, either the
/// synonym arm would shadow it (breaking that sweep) or the canonical
/// arm would win (breaking the `as_str() != synonym` assertion here).
#[test]
fn ffmpeg_synonyms_are_accepted_but_never_emitted() {
  for (synonym, canonical) in FFMPEG_SYNONYMS {
    let from_synonym = synonym
      .parse::<PixelFormat>()
      .expect("a documented FFmpeg synonym parses");
    assert_eq!(
      from_synonym.as_str(),
      *canonical,
      "FFmpeg name {synonym:?} must parse to the variant spelled {canonical:?}"
    );
    assert_ne!(
      from_synonym.as_str(),
      *synonym,
      "{synonym:?} is emitted by a variant — it is a canonical slug, not a synonym"
    );
    assert_eq!(
      canonical.parse::<PixelFormat>(),
      Ok(from_synonym.clone()),
      "the canonical slug and its FFmpeg synonym must name one value"
    );
    // The fold gate applies to synonyms too — one name, one value.
    let mut upper = [0u8; 64];
    let n = synonym.len();
    upper[..n].copy_from_slice(synonym.as_bytes());
    upper[..n].make_ascii_uppercase();
    let upper = core::str::from_utf8(&upper[..n]).unwrap();
    assert_eq!(
      upper.parse::<PixelFormat>(),
      Ok(from_synonym),
      "PixelFormat does not fold the synonym {upper:?}"
    );
  }
}

/// A vendor format mediaframe has never heard of keeps its **name**.
/// That is the whole point of the escape: the old `Unknown(u32)` handed
/// a downstream RAW/sensor backend a bare number and `as_str() ==
/// "unknown"`, while a downstream codec got a first-class value.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn an_unnamed_pixel_format_keeps_its_name() {
  let vendor: PixelFormat = "yuv420q".parse().unwrap();
  assert!(vendor.is_other());
  assert_eq!(vendor.as_str(), "yuv420q");
  assert_eq!(vendor.to_u32(), None);
  assert_eq!("yuv420q".parse(), Ok(vendor));
  // A genuine stranger's spelling survives verbatim — the escape is a
  // passthrough, not a fold target.
  assert_eq!(PixelFormat::other("YUV420Q").as_str(), "YUV420Q");
}

/// `Self::other` runs the ignore-case parse first: a canonical spelling
/// or a documented FFmpeg-synonym alias (`gray`/`monob`/`monow`)
/// resolves to the **named** variant, not a second value for a meaning
/// this vocabulary already names — the equality-heals fixture this
/// escape exists to guarantee.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn other_resolves_canonical_and_alias_spellings_to_the_named_variant() {
  assert_eq!(PixelFormat::other("gray8"), PixelFormat::Gray8);
  assert_eq!(PixelFormat::other("GRAY8"), PixelFormat::Gray8);
  assert_eq!(PixelFormat::other("gray"), PixelFormat::Gray8);
  assert_eq!(PixelFormat::other("GRAY"), PixelFormat::Gray8);
  assert_eq!(PixelFormat::other("monob"), PixelFormat::Monoblack);
  assert_eq!(PixelFormat::other("monow"), PixelFormat::Monowhite);
}

/// At the no-alloc tier there is nowhere to put a name, so the same
/// vocabulary is closed and the parse fails instead.
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[test]
fn an_unnamed_pixel_format_is_rejected_without_an_allocator() {
  // The error's *type* is what names the vocabulary now — one per
  // vocabulary, so two failures are told apart by the compiler rather
  // than by a string field.
  let err: ParsePixelFormatError = "yuv420q".parse::<PixelFormat>().unwrap_err();
  let _ = err;
}

/// The runtime half of the `ROSTER` contract for `PixelFormat` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(PixelFormat::ROSTER, "PixelFormat", PixelFormat::as_str);
}

/// **Type-level** totality for the vocabularies whose escape arm exists at
/// this tier: each `let Ok(..) = ..` below is an *irrefutable* pattern, and
/// it can only be irrefutable because `FromStr::Err` is uninhabited here.
/// Narrow `Self::Err` back to a refusal type and these stop compiling
/// (`E0005`) — the totality is checked, not merely asserted.
///
/// The no-alloc half of the same claim is the build itself: the sibling
/// tests gated on `not(any(std, alloc))` still name the vocabulary's own
/// error, and a `--no-default-features` build type-checks them.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn unnamed_slugs_parse_totally_at_this_tier() {
  let Ok(v) = "not-a-pixel-format".parse::<PixelFormat>();
  assert_eq!(v, PixelFormat::other("not-a-pixel-format"));
}
