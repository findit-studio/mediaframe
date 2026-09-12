use super::*;

#[test]
fn defaults_match_spec() {
  // The five FFmpeg colour enums default to their `Unspecified`
  // variant (FFmpeg `UNSPECIFIED` code: 2 for primaries/transfer/
  // matrix, 0 for range/chroma).
  assert!(matches!(Matrix::default(), Matrix::Unspecified));
  assert!(matches!(Primaries::default(), Primaries::Unspecified));
  assert!(matches!(Transfer::default(), Transfer::Unspecified));
  assert!(matches!(DynamicRange::default(), DynamicRange::Unspecified));
  assert!(matches!(
    ChromaLocation::default(),
    ChromaLocation::Unspecified
  ));
  // `DcpTargetGamut` has no FFmpeg analog; its default is `DciP3`.
  assert!(matches!(DcpTargetGamut::default(), DcpTargetGamut::DciP3));
}

#[test]
fn is_variant_helpers_compile_for_each_enum() {
  assert!(Matrix::Bt709.is_bt_709());
  assert!(Matrix::Rgb.is_rgb());
  assert!(Primaries::Bt2020.is_bt_2020());
  assert!(Transfer::SmpteSt2084Pq.is_smpte_st_2084_pq());
  assert!(DynamicRange::Full.is_full());
  assert!(ChromaLocation::Center.is_center());
}

#[test]
fn clone_and_eq() {
  // `Copy` went with `Other(Utf8Bytes)` — the escape carries a name, and
  // a name is not a register-sized value.
  let m1 = Matrix::Bt709;
  let m2 = m1.clone();
  assert_eq!(m1, m2);
}

#[test]
fn color_info_default_is_all_unspecified() {
  let ci = Info::default();
  assert_eq!(ci, Info::UNSPECIFIED);
  assert!(ci.primaries().is_unspecified());
  // Matrix is now stored as `Unspecified` too (the FFmpeg
  // height-fallback is a consumer concern, not stored).
  assert!(ci.matrix().is_unspecified());
  assert!(ci.transfer().is_unspecified());
  assert!(ci.range().is_unspecified());
  assert!(ci.chroma_location().is_unspecified());
}

#[test]
fn color_info_builders_chain() {
  let ci = Info::UNSPECIFIED
    .with_primaries(Primaries::Bt2020)
    .with_transfer(Transfer::SmpteSt2084Pq)
    .with_matrix(Matrix::Bt2020Ncl)
    .with_range(DynamicRange::Limited)
    .with_chroma_location(ChromaLocation::Left);
  assert!(ci.primaries().is_bt_2020());
  assert!(ci.transfer().is_smpte_st_2084_pq());
  assert!(ci.matrix().is_bt_2020_ncl());
  assert!(ci.range().is_limited());
  assert!(ci.chroma_location().is_left());
}

#[test]
fn color_info_setters_chain() {
  let mut ci = Info::UNSPECIFIED;
  ci.set_primaries(Primaries::Bt709)
    .set_transfer(Transfer::Bt709)
    .set_matrix(Matrix::Bt709)
    .set_range(DynamicRange::Limited)
    .set_chroma_location(ChromaLocation::Left);
  assert!(ci.primaries().is_bt_709());
  assert!(ci.range().is_limited());
}

#[test]
fn color_info_const_construction() {
  const CI: Info = Info::new(
    Primaries::Bt709,
    Transfer::Bt709,
    Matrix::Bt709,
    DynamicRange::Limited,
    ChromaLocation::Left,
  );
  assert!(CI.matrix().is_bt_709());
}

#[cfg(feature = "std")]
#[test]
fn as_str_matches_display() {
  use std::format;
  // Spot-check: every variant's Display goes through `as_str()`.
  for (s, d) in [
    (Matrix::Bt709.as_str(), format!("{}", Matrix::Bt709)),
    (Matrix::Bt2020Ncl.as_str(), format!("{}", Matrix::Bt2020Ncl)),
    (Matrix::YCgCo.as_str(), format!("{}", Matrix::YCgCo)),
  ] {
    assert_eq!(s, d, "Matrix as_str/Display mismatch");
  }
  // Pre-existing slugs are byte-stable (no churn).
  assert_eq!(Matrix::Bt2020Ncl.as_str(), "bt2020nc");
  assert_eq!(Matrix::Smpte240m.as_str(), "smpte240m");
  assert_eq!(Matrix::YCgCo.as_str(), "ycgco");
  assert_eq!(Primaries::SmpteSt428.as_str(), "smpte428");
  assert_eq!(Transfer::SmpteSt2084Pq.as_str(), "smpte2084");
  assert_eq!(Transfer::Bt2020_10Bit.as_str(), "bt2020-10");
  // `Gamma22`/`Gamma28` keep the pre-existing gamma slugs.
  assert_eq!(Transfer::Gamma22.as_str(), "gamma22");
  assert_eq!(Transfer::Gamma28.as_str(), "gamma28");
  assert_eq!(DynamicRange::Limited.as_str(), "tv");
  assert_eq!(DynamicRange::Full.as_str(), "pc");
  assert_eq!(ChromaLocation::TopLeft.as_str(), "topleft");
}

#[test]
fn enum_u32_uses_ffmpeg_codes_and_round_trips() {
  // `to_u32()` returns the real FFmpeg n9.0 code point for the
  // named variants (spot-checks against libavutil/pixfmt.h).
  assert_eq!(Primaries::Unspecified.to_u32(), Some(2));
  assert_eq!(Primaries::Bt709.to_u32(), Some(1));
  assert_eq!(Primaries::Ebu3213E.to_u32(), Some(22));
  assert_eq!(Transfer::Unspecified.to_u32(), Some(2));
  assert_eq!(Transfer::SmpteSt2084Pq.to_u32(), Some(16));
  assert_eq!(Transfer::AribStdB67Hlg.to_u32(), Some(18));
  assert_eq!(Matrix::Rgb.to_u32(), Some(0));
  assert_eq!(Matrix::Unspecified.to_u32(), Some(2));
  assert_eq!(Matrix::Ictcp.to_u32(), Some(14));
  assert_eq!(DynamicRange::Unspecified.to_u32(), Some(0));
  assert_eq!(DynamicRange::Limited.to_u32(), Some(1));
  assert_eq!(DynamicRange::Full.to_u32(), Some(2));
  assert_eq!(ChromaLocation::Unspecified.to_u32(), Some(0));

  // `default()` is the `Unspecified` variant for the five FFmpeg
  // enums (NOT necessarily wire id 0).
  assert_eq!(Matrix::default(), Matrix::Unspecified);
  assert_eq!(Primaries::default(), Primaries::Unspecified);
  assert_eq!(Transfer::default(), Transfer::Unspecified);
  assert_eq!(DynamicRange::default(), DynamicRange::Unspecified);
  assert_eq!(ChromaLocation::default(), ChromaLocation::Unspecified);
  assert_eq!(DcpTargetGamut::default(), DcpTargetGamut::DciP3);

  // Round-trip `from_u32(to_u32()) == v` for EVERY named variant.
  for m in [
    Matrix::Rgb,
    Matrix::Bt601,
    Matrix::Bt709,
    Matrix::Unspecified,
    Matrix::Fcc,
    Matrix::Bt470Bg,
    Matrix::Smpte170M,
    Matrix::Smpte240m,
    Matrix::YCgCo,
    Matrix::Bt2020Ncl,
    Matrix::Bt2020Cl,
    Matrix::Smpte2085,
    Matrix::ChromaDerivedNcl,
    Matrix::ChromaDerivedCl,
    Matrix::Ictcp,
    Matrix::IptC2,
    Matrix::YCgCoRe,
    Matrix::YCgCoRo,
  ] {
    assert_eq!(Matrix::from_u32(m.to_u32().unwrap()), Some(m));
  }
  for p in [
    Primaries::Bt709,
    Primaries::Unspecified,
    Primaries::Bt470M,
    Primaries::Bt470Bg,
    Primaries::Smpte170M,
    Primaries::Smpte240M,
    Primaries::Film,
    Primaries::Bt2020,
    Primaries::SmpteSt428,
    Primaries::SmpteRp431,
    Primaries::SmpteEg432,
    Primaries::Ebu3213E,
  ] {
    assert_eq!(Primaries::from_u32(p.to_u32().unwrap()), Some(p));
  }
  for t in [
    Transfer::Bt709,
    Transfer::Unspecified,
    Transfer::Gamma22,
    Transfer::Gamma28,
    Transfer::Smpte170M,
    Transfer::Smpte240M,
    Transfer::Linear,
    Transfer::Log100,
    Transfer::Log316,
    Transfer::Iec6196624,
    Transfer::Bt1361Ecg,
    Transfer::Iec6196621,
    Transfer::Bt2020_10Bit,
    Transfer::Bt2020_12Bit,
    Transfer::SmpteSt2084Pq,
    Transfer::SmpteSt428,
    Transfer::AribStdB67Hlg,
  ] {
    assert_eq!(Transfer::from_u32(t.to_u32().unwrap()), Some(t));
  }
  for r in [
    DynamicRange::Unspecified,
    DynamicRange::Limited,
    DynamicRange::Full,
  ] {
    assert_eq!(DynamicRange::from_u32(r.to_u32().unwrap()), Some(r));
  }
  for c in [
    ChromaLocation::Unspecified,
    ChromaLocation::Left,
    ChromaLocation::Center,
    ChromaLocation::TopLeft,
    ChromaLocation::Top,
    ChromaLocation::BottomLeft,
    ChromaLocation::Bottom,
  ] {
    assert_eq!(ChromaLocation::from_u32(c.to_u32().unwrap()), Some(c));
  }
  for g in [
    DcpTargetGamut::DciP3,
    DcpTargetGamut::Rec709,
    DcpTargetGamut::Rec2020,
  ] {
    assert_eq!(DcpTargetGamut::from_u32(g.to_u32().unwrap()), Some(g));
  }

  // A code this build names nothing for is REJECTED, not invented
  // into a payload-bearing value: `from_u32` is a boundary helper
  // over FFmpeg's number space, and a number carries no name.
  assert_eq!(Matrix::from_u32(9_999), None);
  // Reserved FFmpeg code 3 is named by no FFmpeg enum.
  assert_eq!(Primaries::from_u32(3), None);
  assert_eq!(Primaries::from_u32(0), None);
  assert_eq!(Transfer::from_u32(3), None);
  assert_eq!(DynamicRange::from_u32(7), None);
  assert_eq!(ChromaLocation::from_u32(42), None);
  assert_eq!(DcpTargetGamut::from_u32(9_999), None);
}

/// …and the name a downstream backend brings survives instead, on the
/// text side, where a name is what there is to keep. Needs the
/// allocator: at the no-alloc tier there is nowhere to put the name and
/// the vocabulary is closed.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn a_name_this_build_does_not_enumerate_survives_as_a_name() {
  let vendor = Matrix::other("ACEScct");
  assert_eq!(vendor.as_str(), "ACEScct");
  assert_eq!(vendor.to_u32(), None);
  assert_eq!("ACEScct".parse(), Ok(vendor));
}

#[test]
fn color_matrix_bt601_is_domain_variant() {
  // Released-API slug restored (the public removal is reverted).
  assert_eq!(Matrix::Bt601.as_str(), "bt601");
  #[cfg(feature = "std")]
  {
    use std::format;
    assert_eq!(format!("{}", Matrix::Bt601), "bt601");
  }

  // `Bt601` lives in the mediaframe-domain extension band at
  // offset 0, NOT an FFmpeg code; it round-trips losslessly.
  assert_eq!(Matrix::Bt601.to_u32(), Some(DOMAIN_EXT_BASE));
  assert_eq!(Matrix::Bt601.to_u32(), Some(0x8000_0000));
  assert_eq!(Matrix::from_u32(0x8000_0000), Some(Matrix::Bt601));
  assert_eq!(
    Matrix::from_u32(Matrix::Bt601.to_u32().unwrap()),
    Some(Matrix::Bt601)
  );

  // Regression: FFmpeg codes 5/6 stay BT.470BG / SMPTE170M and are
  // NEVER decoded as the domain `Bt601` (FFmpeg ingest path never
  // yields a domain variant).
  assert_eq!(Matrix::from_u32(5), Some(Matrix::Bt470Bg));
  assert_eq!(Matrix::from_u32(6), Some(Matrix::Smpte170M));
  assert_ne!(Matrix::from_u32(5), Some(Matrix::Bt601));
  assert_ne!(Matrix::from_u32(6), Some(Matrix::Bt601));

  // `Bt601` is NOT the default (stays `Unspecified`).
  assert_eq!(Matrix::default(), Matrix::Unspecified);
  assert_ne!(Matrix::default(), Matrix::Bt601);

  // An unassigned bit-31 id names nothing (the domain band is
  // append-only, not exhaustive), so it is rejected.
  assert_eq!(Matrix::from_u32(0x8000_00FF), None);

  // `is_variant` helper is generated for the new variant.
  assert!(Matrix::Bt601.is_bt_601());
}

#[test]
fn content_light_level_construct_and_builders() {
  let c = ContentLightLevel::new(1000, 400);
  assert_eq!(c.max_cll(), 1000);
  assert_eq!(c.max_fall(), 400);
  assert_eq!(ContentLightLevel::default(), ContentLightLevel::new(0, 0));
  let c2 = ContentLightLevel::default()
    .with_max_cll(4000)
    .with_max_fall(1000);
  assert_eq!((c2.max_cll(), c2.max_fall()), (4000, 1000));
  let mut c3 = ContentLightLevel::default();
  c3.set_max_cll(600).set_max_fall(200);
  assert_eq!((c3.max_cll(), c3.max_fall()), (600, 200));
}

#[test]
fn chroma_coord_and_mastering_display() {
  let red = ChromaCoord::new(34000, 16000);
  let green = ChromaCoord::new(13250, 34500);
  let blue = ChromaCoord::new(7500, 3000);
  let wp = ChromaCoord::default().with_x(15635).with_y(16450);
  assert_eq!((red.x(), red.y()), (34000, 16000));
  assert_eq!((wp.x(), wp.y()), (15635, 16450));

  const MD: MasteringDisplay = MasteringDisplay::new(
    [
      ChromaCoord::new(34000, 16000),
      ChromaCoord::new(13250, 34500),
      ChromaCoord::new(7500, 3000),
    ],
    ChromaCoord::new(15635, 16450),
    10_000_000,
    50,
  );
  assert_eq!(MD.display_primaries()[1], green);
  assert_eq!(MD.white_point(), ChromaCoord::new(15635, 16450));
  assert_eq!(MD.max_luminance(), 10_000_000);
  assert_eq!(MD.min_luminance(), 50);

  let mut md = MasteringDisplay::default();
  md.set_display_primaries([red, green, blue])
    .set_white_point(wp)
    .set_max_luminance(40_000_000)
    .set_min_luminance(5);
  assert_eq!(md.display_primaries()[2], blue);
  assert_eq!(md.min_luminance(), 5);
}

#[test]
fn hdr_static_metadata_optionals() {
  let empty = HdrStaticMetadata::default();
  assert!(empty.mastering().is_none());
  assert!(empty.content_light().is_none());

  let cll = ContentLightLevel::new(1000, 400);
  let md = MasteringDisplay::new(
    [
      ChromaCoord::new(1, 2),
      ChromaCoord::new(3, 4),
      ChromaCoord::new(5, 6),
    ],
    ChromaCoord::new(7, 8),
    9,
    10,
  );
  let h = HdrStaticMetadata::new(Some(md), Some(cll));
  assert_eq!(h.mastering(), Some(md));
  assert_eq!(h.content_light(), Some(cll));

  let h2 = HdrStaticMetadata::default()
    .with_content_light(Some(cll))
    .with_mastering(None);
  assert_eq!(h2.content_light(), Some(cll));
  assert!(h2.mastering().is_none());
}

#[test]
fn dolby_vision_config_default_and_accessors() {
  let d = DolbyVisionConfig::default();
  assert_eq!(d.profile(), 0);
  assert_eq!(d.level(), 0);
  assert!(!d.rpu_present());
  assert!(!d.el_present());
  assert_eq!(d.bl_signal_compat_id(), 0);

  let c = DolbyVisionConfig::new(8, 9, true, false, 1);
  assert_eq!(c.profile(), 8);
  assert_eq!(c.level(), 9);
  assert!(c.rpu_present());
  assert!(!c.el_present());
  assert_eq!(c.bl_signal_compat_id(), 1);

  let c2 = DolbyVisionConfig::default()
    .with_profile(5)
    .with_level(6)
    .with_rpu_present()
    .with_el_present()
    .with_bl_signal_compat_id(2);
  assert_eq!(
    (
      c2.profile(),
      c2.level(),
      c2.rpu_present(),
      c2.el_present(),
      c2.bl_signal_compat_id()
    ),
    (5, 6, true, true, 2)
  );

  // Raw consuming setters (`maybe_*`).
  let c2b = DolbyVisionConfig::default()
    .maybe_rpu_present(true)
    .maybe_el_present(false);
  assert!(c2b.rpu_present());
  assert!(!c2b.el_present());

  let mut c3 = DolbyVisionConfig::default();
  c3.set_profile(7)
    .set_level(4)
    .set_rpu_present()
    .set_el_present()
    .set_bl_signal_compat_id(4);
  assert_eq!(c3, DolbyVisionConfig::new(7, 4, true, true, 4));

  // In-place raw setter (`update_*`) and `clear_*`.
  c3.update_el_present(false);
  assert!(!c3.el_present());
  c3.clear_rpu_present();
  assert!(!c3.rpu_present());
  c3.update_rpu_present(true);
  assert!(c3.rpu_present());
}

#[test]
fn primaries_chromaticities_and_white_point() {
  // Unspecified / a name we do not know carry no defined primaries.
  assert!(Primaries::Unspecified.chromaticities().is_none());
  assert!(Primaries::Unspecified.white_point().is_none());
  #[cfg(any(feature = "std", feature = "alloc"))]
  {
    assert!(Primaries::other("vendor-gamut").chromaticities().is_none());
    assert!(Primaries::other("vendor-gamut").white_point().is_none());
  }

  // Every defined variant has both primaries and a white point.
  for p in [
    Primaries::Bt709,
    Primaries::Bt470M,
    Primaries::Bt470Bg,
    Primaries::Smpte170M,
    Primaries::Smpte240M,
    Primaries::Film,
    Primaries::Bt2020,
    Primaries::SmpteSt428,
    Primaries::SmpteRp431,
    Primaries::SmpteEg432,
    Primaries::Ebu3213E,
  ] {
    assert!(p.chromaticities().is_some(), "{p:?} missing primaries");
    assert!(p.white_point().is_some(), "{p:?} missing white point");
  }

  // Coordinates are ST 2086 units (decimal × 50000), cross-checked
  // against FFmpeg `av_csp_primaries_desc` (libavutil/csp.c).
  // BT.709 / sRGB: R(0.640,0.330) G(0.300,0.600) B(0.150,0.060), D65.
  assert_eq!(
    Primaries::Bt709.chromaticities(),
    Some([
      ChromaCoord::new(32000, 16500),
      ChromaCoord::new(15000, 30000),
      ChromaCoord::new(7500, 3000),
    ])
  );
  assert_eq!(
    Primaries::Bt709.white_point(),
    Some(ChromaCoord::new(15635, 16450))
  );

  // BT.2020: R(0.708,0.292) G(0.170,0.797) B(0.131,0.046), D65.
  assert_eq!(
    Primaries::Bt2020.chromaticities(),
    Some([
      ChromaCoord::new(35400, 14600),
      ChromaCoord::new(8500, 39850),
      ChromaCoord::new(6550, 2300),
    ])
  );
  assert_eq!(
    Primaries::Bt2020.white_point(),
    Some(ChromaCoord::new(15635, 16450))
  );

  // DCI-P3 (RP 431-2): P3 primaries with DCI white (0.314, 0.351).
  assert_eq!(
    Primaries::SmpteRp431.chromaticities(),
    Some([
      ChromaCoord::new(34000, 16000),
      ChromaCoord::new(13250, 34500),
      ChromaCoord::new(7500, 3000),
    ])
  );
  assert_eq!(
    Primaries::SmpteRp431.white_point(),
    Some(ChromaCoord::new(15700, 17550))
  );

  // Display-P3 (EG 432-1): identical P3 primaries, but D65 white.
  assert_eq!(
    Primaries::SmpteEg432.chromaticities(),
    Primaries::SmpteRp431.chromaticities()
  );
  assert_eq!(
    Primaries::SmpteEg432.white_point(),
    Some(ChromaCoord::new(15635, 16450))
  );
  assert_ne!(
    Primaries::SmpteEg432.white_point(),
    Primaries::SmpteRp431.white_point()
  );

  // SMPTE 170M and 240M share primaries (and D65).
  assert_eq!(
    Primaries::Smpte170M.chromaticities(),
    Primaries::Smpte240M.chromaticities()
  );

  // SMPTE ST 428 follows FFmpeg csp.c — D-Cinema primaries with the
  // equal-energy white point E (1/3 → 16667), NOT the XYZ identity.
  assert_eq!(
    Primaries::SmpteSt428.chromaticities(),
    Some([
      ChromaCoord::new(36750, 13250),
      ChromaCoord::new(13700, 35900),
      ChromaCoord::new(8350, 450),
    ])
  );
  assert_eq!(
    Primaries::SmpteSt428.white_point(),
    Some(ChromaCoord::new(16667, 16667))
  );

  // Usable in const context (mirrors the enum's other const fns). The
  // value is reached through a `const` **reference**, not built as a
  // temporary: at the `alloc` tier `Other(Utf8Bytes)` gives this enum a
  // destructor const-eval cannot run — the carrier's heap arm holds a
  // `bytes::Bytes`, whose inline `AtomicPtr` makes the type non-`freeze`,
  // so the temporary cannot be promoted and its drop lands inside the
  // const body (`E0493`). A `&'static` const drops nothing, so the call
  // is what it always was: a `const fn` evaluated at compile time, here
  // and at the no-alloc tier alike.
  const P3: &Primaries = &Primaries::SmpteEg432;
  const P3_WHITE: Option<ChromaCoord> = P3.white_point();
  assert_eq!(P3_WHITE, Some(ChromaCoord::new(15635, 16450)));
}

#[test]
fn primaries_is_cie_xyz() {
  // Only ST 428-1 encodes color directly in CIE XYZ.
  assert!(Primaries::SmpteSt428.is_cie_xyz());

  // Every other defined primary set (and the unknowns) is an RGB gamut.
  for p in [
    Primaries::Bt709,
    Primaries::Bt470M,
    Primaries::Bt470Bg,
    Primaries::Smpte170M,
    Primaries::Smpte240M,
    Primaries::Film,
    Primaries::Bt2020,
    Primaries::SmpteRp431,
    Primaries::SmpteEg432,
    Primaries::Ebu3213E,
    Primaries::Unspecified,
  ] {
    assert!(!p.is_cie_xyz(), "{p:?} is an RGB gamut, not CIE XYZ");
  }

  // The XYZ interpretation is independent of `chromaticities()`, which
  // still reports FFmpeg's tabulated RGB primaries for ST 428-1.
  assert!(Primaries::SmpteSt428.chromaticities().is_some());

  // Usable in a const context (mirrors the enum's other const fns) —
  // proven at compile time. Through `const` references, for the reason
  // spelled out in `primaries_chromaticities_and_white_points`.
  const ST428: &Primaries = &Primaries::SmpteSt428;
  const BT2020: &Primaries = &Primaries::Bt2020;
  const _: () = assert!(ST428.is_cie_xyz());
  const _: () = assert!(!BT2020.is_cie_xyz());
}

/// Every **named** variant of every coded colour enum must survive
/// `as_str()` → `FromStr` unchanged, and no two named variants may share
/// a slug (a collision would silently make one of them unparseable).
///
/// The sweep enumerates variants through `from_u32` over both id ranges
/// (H.273 codes and the `DOMAIN_EXT_BASE` mediaframe extensions) rather
/// than a hand-written list, so a variant added later is covered without
/// touching this test.
#[test]
fn every_named_colour_variant_round_trips_through_its_slug() {
  macro_rules! sweep {
    ($ty:ty) => {{
      let mut named = 0usize;
      let mut codes = [0u32; 64];
      for code in (0..=1024u32).chain(DOMAIN_EXT_BASE..=DOMAIN_EXT_BASE + 1024) {
        let Some(value) = <$ty>::from_u32(code) else {
          continue;
        };
        let slug = value.as_str();
        assert_eq!(
          slug.parse::<$ty>(),
          Ok(value.clone()),
          "{} slug {slug:?} does not parse back to {value:?}",
          stringify!($ty)
        );
        assert!(
          !slug.bytes().any(|b| b.is_ascii_uppercase()),
          "{} slug {slug:?} is not lowercase-canonical",
          stringify!($ty)
        );
        {
          let mut upper = [0u8; 64];
          let n = slug.len();
          upper[..n].copy_from_slice(slug.as_bytes());
          upper[..n].make_ascii_uppercase();
          let upper = core::str::from_utf8(&upper[..n]).unwrap();
          assert_eq!(
            upper.parse::<$ty>(),
            Ok(value.clone()),
            "{} does not fold {upper:?} onto {slug:?}",
            stringify!($ty)
          );
        }
        for prior in codes.iter().take(named) {
          let prior = <$ty>::from_u32(*prior).expect("recorded code names a variant");
          assert_ne!(
            prior.as_str(),
            slug,
            "{} has two variants spelled {slug:?}",
            stringify!($ty)
          );
        }
        codes[named] = code;
        named += 1;
      }
      assert!(
        named > 0,
        "{} sweep found no named variants — the id range is wrong",
        stringify!($ty)
      );
    }};
  }

  sweep!(Matrix);
  sweep!(Primaries);
  sweep!(Transfer);
  sweep!(DynamicRange);
  sweep!(ChromaLocation);
  sweep!(DcpTargetGamut);
}

/// `"unknown"` is still not a value any colour enum can **hold**: the
/// arm that collapsed every payload onto that one string is gone and
/// nothing renders the word.
///
/// The FFmpeg-synonym law changed only the reading. On the four enums
/// whose FFmpeg name table spells `AVCOL_*_UNSPECIFIED` as `"unknown"`,
/// the word is now recognised as that spelling and parses to
/// `Unspecified` — which renders `"unspecified"`, so the word still
/// never comes back out, and the H.273 code survives instead of being
/// lost to a `to_u32`-less escape. `ChromaLocation` (FFmpeg spells that
/// one `"unspecified"` as well) and the mediaframe-domain
/// `DcpTargetGamut` have no such synonym, so there the word stays an
/// ordinary unenumerated name on the escape.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn unknown_is_no_longer_a_colour_value() {
  for parsed in [
    "unknown".parse::<Matrix>().unwrap().as_str(),
    "unknown".parse::<Primaries>().unwrap().as_str(),
    "unknown".parse::<Transfer>().unwrap().as_str(),
    "unknown".parse::<DynamicRange>().unwrap().as_str(),
  ] {
    assert_eq!(parsed, "unspecified");
  }
  assert_eq!("unknown".parse(), Ok(Matrix::Unspecified));
  assert_eq!(Matrix::Unspecified.to_u32(), Some(2));

  for parsed in [
    "unknown".parse::<ChromaLocation>().unwrap().as_str(),
    "unknown".parse::<DcpTargetGamut>().unwrap().as_str(),
  ] {
    assert_eq!(parsed, "unknown");
  }
  assert!("unknown".parse::<ChromaLocation>().unwrap().is_other());
}

/// The FFmpeg synonyms are **accepted, never emitted**, and none of them
/// shadows a canonical slug.
///
/// Paired with `every_named_colour_variant_round_trips_through_its_slug`
/// (every canonical slug parses back to its own variant), the two
/// directions make a collision impossible: a synonym that shadowed a
/// canonical slug would break that sweep, and one that lost to a
/// canonical arm would fail the `as_str() != synonym` assertion here.
#[test]
fn ffmpeg_synonyms_are_accepted_but_never_emitted() {
  macro_rules! check {
    ($ty:ty, $table:expr) => {{
      for (synonym, canonical) in $table {
        let from_synonym = synonym
          .parse::<$ty>()
          .expect("a documented FFmpeg synonym parses");
        assert_eq!(
          from_synonym.as_str(),
          *canonical,
          "{}: FFmpeg name {synonym:?} must parse to the variant spelled {canonical:?}",
          stringify!($ty)
        );
        assert_ne!(
          from_synonym.as_str(),
          *synonym,
          "{}: {synonym:?} is emitted by a variant — a canonical slug, not a synonym",
          stringify!($ty)
        );
        assert_eq!(
          canonical.parse::<$ty>(),
          Ok(from_synonym.clone()),
          "{}: the canonical slug and its FFmpeg synonym must name one value",
          stringify!($ty)
        );
        // The fold gate applies to synonyms too — one name, one value.
        let mut upper = [0u8; 64];
        let n = synonym.len();
        upper[..n].copy_from_slice(synonym.as_bytes());
        upper[..n].make_ascii_uppercase();
        let upper = core::str::from_utf8(&upper[..n]).unwrap();
        assert_eq!(
          upper.parse::<$ty>(),
          Ok(from_synonym),
          "{} does not fold the synonym {upper:?}",
          stringify!($ty)
        );
      }
    }};
  }

  check!(Matrix, MATRIX_FFMPEG_SYNONYMS);
  check!(Primaries, PRIMARIES_FFMPEG_SYNONYMS);
  check!(Transfer, TRANSFER_FFMPEG_SYNONYMS);
  check!(DynamicRange, DYNAMIC_RANGE_FFMPEG_SYNONYMS);
}

/// A synonym is scoped to the type whose FFmpeg name table carries it.
/// `bt470m` / `bt470bg` are canonical [`Primaries`] slugs *and* the
/// FFmpeg spelling of two [`Transfer`] curves this crate calls
/// `gamma22` / `gamma28`. FFmpeg overloads the words the same way; the
/// types are what keep the two readings apart.
#[test]
fn a_synonym_is_scoped_to_its_type() {
  assert_eq!("bt470m".parse(), Ok(Primaries::Bt470M));
  assert_eq!("bt470bg".parse(), Ok(Primaries::Bt470Bg));
  assert_eq!("bt470m".parse(), Ok(Transfer::Gamma22));
  assert_eq!("bt470bg".parse(), Ok(Transfer::Gamma28));
  assert_eq!(Primaries::Bt470M.as_str(), "bt470m");
  assert_eq!(Transfer::Gamma22.as_str(), "gamma22");
}

/// The escape is total on the `alloc` tier: every name round-trips, and
/// a genuine stranger's spelling survives verbatim — the lookup above
/// already catches every case-variant of a name this crate *does* know,
/// so folding the leftover strangers would only destroy information.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn unnamed_slugs_ride_the_escape_verbatim() {
  let m: Matrix = "definitely-not-a-matrix".parse().unwrap();
  assert!(m.is_other());
  assert_eq!(m.as_str(), "definitely-not-a-matrix");
  assert_eq!(
    "VENDOR-Gamut".parse::<Matrix>().unwrap().as_str(),
    "VENDOR-Gamut",
    "the escape must preserve the caller's spelling verbatim"
  );
  assert_eq!("".parse::<Matrix>().unwrap().as_str(), "");
}

/// `Self::other` runs the ignore-case parse first: a canonical spelling
/// or a documented FFmpeg-synonym alias resolves to the **named**
/// variant, not a second value for a meaning these types already name —
/// the equality-heals fixture this escape exists to guarantee.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn other_resolves_canonical_and_alias_spellings_to_the_named_variant() {
  // Canonical spelling, both cases.
  assert_eq!(Matrix::other("bt709"), Matrix::Bt709);
  assert_eq!(Matrix::other("BT709"), Matrix::Bt709);
  assert_eq!(Primaries::other("bt2020"), Primaries::Bt2020);
  assert_eq!(Primaries::other("BT2020"), Primaries::Bt2020);
  assert_eq!(Transfer::other("smpte2084"), Transfer::SmpteSt2084Pq);
  assert_eq!(Transfer::other("SMPTE2084"), Transfer::SmpteSt2084Pq);
  assert_eq!(DynamicRange::other("tv"), DynamicRange::Limited);
  assert_eq!(DynamicRange::other("TV"), DynamicRange::Limited);
  assert_eq!(ChromaLocation::other("topleft"), ChromaLocation::TopLeft);
  assert_eq!(ChromaLocation::other("TOPLEFT"), ChromaLocation::TopLeft);

  // A documented FFmpeg-synonym alias, both cases.
  assert_eq!(Matrix::other("gbr"), Matrix::Rgb);
  assert_eq!(Matrix::other("GBR"), Matrix::Rgb);
  assert_eq!(Primaries::other("unknown"), Primaries::Unspecified);
  assert_eq!(Primaries::other("UNKNOWN"), Primaries::Unspecified);
  assert_eq!(Transfer::other("bt470m"), Transfer::Gamma22);
  assert_eq!(Transfer::other("BT470M"), Transfer::Gamma22);
  assert_eq!(Transfer::other("bt470bg"), Transfer::Gamma28);
  assert_eq!(DynamicRange::other("unknown"), DynamicRange::Unspecified);
}

/// `DcpTargetGamut` gained `as_str`/`Display` to match the five H.273
/// enums; pin the spellings so they cannot drift silently.
#[test]
fn dcp_target_gamut_slugs_are_stable() {
  assert_eq!(DcpTargetGamut::DciP3.as_str(), "dci-p3");
  assert_eq!(DcpTargetGamut::Rec709.as_str(), "rec709");
  assert_eq!(DcpTargetGamut::Rec2020.as_str(), "rec2020");
  assert_eq!("dci-p3".parse(), Ok(DcpTargetGamut::DciP3));
}

/// The runtime half of the `ROSTER` contract for `Matrix` / `Primaries` / `Transfer` / `DynamicRange` / `ChromaLocation` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(Matrix::ROSTER, "Matrix", Matrix::as_str);
  crate::roster_tests::check(Primaries::ROSTER, "Primaries", Primaries::as_str);
  crate::roster_tests::check(Transfer::ROSTER, "Transfer", Transfer::as_str);
  crate::roster_tests::check(DynamicRange::ROSTER, "DynamicRange", DynamicRange::as_str);
  crate::roster_tests::check(
    ChromaLocation::ROSTER,
    "ChromaLocation",
    ChromaLocation::as_str,
  );
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
  let Ok(v) = "not-a-colour-matrix".parse::<Matrix>();
  assert_eq!(v, Matrix::other("not-a-colour-matrix"));
  let Ok(v) = "not-a-primaries-set".parse::<Primaries>();
  assert_eq!(v, Primaries::other("not-a-primaries-set"));
  let Ok(v) = "not-a-transfer-function".parse::<Transfer>();
  assert_eq!(v, Transfer::other("not-a-transfer-function"));
  let Ok(v) = "not-a-dynamic-range".parse::<DynamicRange>();
  assert_eq!(v, DynamicRange::other("not-a-dynamic-range"));
  let Ok(v) = "not-a-chroma-siting".parse::<ChromaLocation>();
  assert_eq!(v, ChromaLocation::other("not-a-chroma-siting"));
  // `DcpTargetGamut` is the tenth vocabulary of this shape. It is outside
  // the enum-mirror roster, but its parse is total on exactly the same
  // terms, so it carries the same proof.
  let Ok(v) = "not-a-target-gamut".parse::<DcpTargetGamut>();
  assert_eq!(v, DcpTargetGamut::other("not-a-target-gamut"));
}
