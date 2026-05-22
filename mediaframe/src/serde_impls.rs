//! Centralised `serde` implementations for the descriptor enums
//! (`feature = "serde"`).
//!
//! The wire shape mirrors what the storage backends (sqlx / mongodb /
//! async-graphql) independently chose, so a serde-`json` column matches
//! their representation byte-for-byte:
//!
//! - **Open** codec / format enums (those with an `Other(SmolStr)` escape
//!   arm and a total [`FromStr`](core::str::FromStr)) serialize as their
//!   canonical `as_str()` slug — e.g. `VideoCodec::H264` ⇄ `"h264"`,
//!   `Other("x265")` ⇄ `"x265"` (no `{"Other": …}` wrapper).
//! - **Closed** FFmpeg-coded enums (those with `to_u32()` / `from_u32()`
//!   and no `FromStr`) serialize as their `u32` code — e.g.
//!   `color::Matrix::Bt709` ⇄ `1`.
//!
//! Both round-trips are total: an unrecognised slug rides the `Other`
//! arm, an unrecognised code the `Unknown` arm. The plain data structs
//! (`color::Info`, `frame::Dimensions`, `audio::Tags`, …) derive serde
//! at their definition site; the validated structs
//! (`capture::GeoLocation`, `audio::Fingerprint`, `audio::CoverArt`)
//! route deserialize through their checking constructors there too.
//! `lang::Language` carries a bespoke BCP-47 string impl in its module.

/// Implements `Serialize` / `Deserialize` for an *open* enum via its
/// canonical string slug (`as_str()` to serialize, [`FromStr`] to parse).
/// The `FromStr` impl is total (`Err = Infallible`) — unknown slugs ride
/// the enum's `Other` arm — but the deserializer surfaces any error as a
/// serde error for forward-compatibility.
///
/// [`FromStr`]: core::str::FromStr
// All invocations are heap-tier (codecs / formats); unused under the
// no-alloc tier where only the coded enums exist.
#[allow(unused_macros)]
macro_rules! serde_via_str {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
          type Value = $t;
          fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(concat!("a ", stringify!($t), " slug string"))
          }
          #[inline]
          fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<$t>().map_err(serde::de::Error::custom)
          }
        }
        de.deserialize_str(V)
      }
    }
  };
}

/// Implements `Serialize` / `Deserialize` for a *closed* FFmpeg-coded enum
/// **whose `from_u32` is lossless** — i.e. it has an `Unknown(u32)` escape
/// arm so any code round-trips losslessly. Use this for enums where every
/// `u32` is meaningful wire data.
macro_rules! serde_via_code {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_u32(self.to_u32())
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      #[inline]
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(<$t>::from_u32(<u32 as serde::Deserialize>::deserialize(
          de,
        )?))
      }
    }
  };
}

/// Implements `Serialize` / `Deserialize` for a **strictly-closed**
/// FFmpeg-coded enum — one with NO `Unknown(u32)` escape arm — via its
/// `to_u32()` / `try_from_u32()` pair. Adversarial / corrupt codes outside
/// the enumerated set are rejected as serde errors instead of silently
/// canonicalising to the default variant (which `from_u32` would do).
macro_rules! serde_via_code_strict {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_u32(self.to_u32())
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      #[inline]
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = <u32 as serde::Deserialize>::deserialize(de)?;
        <$t>::try_from_u32(v).ok_or_else(|| {
          serde::de::Error::custom(::std::format!(
            "{}: unknown wire code {}",
            stringify!($t),
            v
          ))
        })
      }
    }
  };
}

/// Bespoke serde for [`SampleFormat`](crate::audio::SampleFormat) — it has
/// *both* `Unknown(u32)` (lossless numeric escape) **and** `Other(SmolStr)`
/// (lossless string escape). The generic `serde_via_str!` would route
/// `Unknown(12345)` through `as_str()` → `"unknown"` → `Other("unknown")`,
/// destroying the original code; the generic `serde_via_code!` would lose
/// the `Other` string variants.
///
/// **Wire shape — branches on `Serializer::is_human_readable()`:**
///
/// - **Self-describing (JSON / YAML / RON / TOML / etc.)** — bare value:
///   `Unknown(v)` → number; named slug + `Other` → string. The visitor
///   uses `deserialize_any` to choose the arm at decode time.
/// - **Non-self-describing (bincode / postcard / etc.)** — explicit
///   2-variant tagged enum (`{Code(u32), Slug(String)}`), since
///   `deserialize_any` is not supported on these formats. Wire bytes are
///   compact and the variant tag drives reconstruction unambiguously.
#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use crate::audio::SampleFormat;
  use core::{fmt, str::FromStr};

  // Tagged representation used only on non-self-describing formats. The
  // derive picks a compact discriminant + payload; downstream binary serde
  // drivers know exactly how to round-trip it without `deserialize_any`.
  #[derive(serde::Serialize, serde::Deserialize)]
  enum BinaryWire<'a> {
    Code(u32),
    Slug(::std::borrow::Cow<'a, str>),
  }

  impl serde::Serialize for SampleFormat {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
      if ser.is_human_readable() {
        // Bare value — current human-readable shape.
        match self {
          SampleFormat::Unknown(v) => ser.serialize_u32(*v),
          other => ser.serialize_str(other.as_str()),
        }
      } else {
        // Tagged wire for binary formats.
        match self {
          SampleFormat::Unknown(v) => BinaryWire::Code(*v).serialize(ser),
          other => BinaryWire::Slug(::std::borrow::Cow::Borrowed(other.as_str())).serialize(ser),
        }
      }
    }
  }

  impl<'de> serde::Deserialize<'de> for SampleFormat {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
      if de.is_human_readable() {
        // Self-describing: accept either a u32 OR a string via `deserialize_any`.
        struct V;
        impl serde::de::Visitor<'_> for V {
          type Value = SampleFormat;
          fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a SampleFormat slug string or a u32 FFmpeg code")
          }
          fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            // `FromStr` is `Infallible`; non-named slugs ride `Other(SmolStr)`.
            SampleFormat::from_str(v).map_err(serde::de::Error::custom)
          }
          // Numeric inputs route through `from_u32` so `Unknown(v)` survives.
          // Cover the integer width spread serde drivers actually produce.
          fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
            Ok(SampleFormat::from_u32(v))
          }
          fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            u32::try_from(v)
              .map(SampleFormat::from_u32)
              .map_err(|_| serde::de::Error::custom("SampleFormat u32 code overflow"))
          }
          fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            u32::try_from(v)
              .map(SampleFormat::from_u32)
              .map_err(|_| serde::de::Error::custom("SampleFormat u32 code out of range"))
          }
        }
        de.deserialize_any(V)
      } else {
        // Non-self-describing: drive the tagged wire enum, then convert.
        let w = BinaryWire::deserialize(de)?;
        Ok(match w {
          BinaryWire::Code(v) => SampleFormat::from_u32(v),
          // `FromStr` is `Infallible` for `SampleFormat`.
          BinaryWire::Slug(s) => SampleFormat::from_str(&s).unwrap(),
        })
      }
    }
  }
};

// ── Closed FFmpeg-coded enums (available at every capability tier) ──
serde_via_code!(crate::color::Matrix);
serde_via_code!(crate::color::Primaries);
serde_via_code!(crate::color::Transfer);
serde_via_code!(crate::color::DynamicRange);
serde_via_code!(crate::color::ChromaLocation);
serde_via_code!(crate::color::DcpTargetGamut);
serde_via_code!(crate::pixel_format::PixelFormat);
serde_via_code!(crate::frame::Rotation);
serde_via_code!(crate::frame::FieldOrder);
serde_via_code!(crate::frame::StereoMode);
serde_via_code!(crate::disposition::TrackDisposition);

// ── Open slug enums (heap-tier: codecs / formats carry `Other(SmolStr)`) ──
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::VideoCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::AudioCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::SubtitleCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::container::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::subtitle::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ChannelLayout);
// `SampleFormat` has BOTH `Unknown(u32)` and `Other(SmolStr)` — the bespoke
// impl below (immediately after the macros) covers it; do NOT route it
// through `serde_via_str!` (would silently drop `Unknown(v)` codes through
// `as_str()` → `"unknown"` → `Other("unknown")`).
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ContainerFormat);

// ── Strictly-closed coded enums (no `Unknown` escape) ──
// Use `serde_via_code_strict!` — adversarial / unknown wire codes are
// rejected as serde errors, not silently canonicalised to the default
// (which `from_u32` would do for `TrackOrigin::from_u32(999) == Embedded`).
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_code_strict!(crate::subtitle::TrackOrigin);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_code_strict!(crate::audio::BitRateMode);

#[cfg(all(test, feature = "std"))]
mod tests {
  use crate::{
    audio::{ChannelLayout, CoverArt, Fingerprint, Tags},
    capture::GeoLocation,
    codec::VideoCodec,
    color::{self, Matrix},
    disposition::TrackDisposition,
    frame::{Dimensions, SampleAspectRatio},
    lang::Language,
  };

  fn round_trip<T>(v: &T) -> T
  where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
  {
    let json = serde_json::to_string(v).unwrap();
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(*v, back, "round-trip mismatch via {json}");
    back
  }

  #[test]
  fn open_enum_serializes_as_slug() {
    assert_eq!(
      serde_json::to_string(&VideoCodec::H264).unwrap(),
      "\"h264\""
    );
    round_trip(&VideoCodec::H264);
    // Unknown slug rides the `Other` arm losslessly.
    let custom = VideoCodec::Other(smol_str::SmolStr::new("zzcodec"));
    assert_eq!(serde_json::to_string(&custom).unwrap(), "\"zzcodec\"");
    round_trip(&custom);
    round_trip(&ChannelLayout::default());
  }

  #[test]
  fn closed_enum_serializes_as_code() {
    let json = serde_json::to_string(&Matrix::Bt709).unwrap();
    assert_eq!(json, Matrix::Bt709.to_u32().to_string());
    round_trip(&Matrix::Bt709);
    // Unknown code rides the `Unknown` arm losslessly.
    let unknown: Matrix = serde_json::from_str("250").unwrap();
    assert_eq!(unknown.to_u32(), 250);
  }

  #[test]
  fn structs_round_trip() {
    round_trip(&color::Info::default());
    round_trip(&Dimensions::new(1920, 1080));
    round_trip(&SampleAspectRatio::new(
      40,
      core::num::NonZeroU32::new(33).unwrap(),
    ));
    round_trip(&Tags::new().with_title("Song").with_year(2026));
    round_trip(&(TrackDisposition::DEFAULT | TrackDisposition::FORCED));
  }

  #[test]
  fn language_round_trips_as_bcp47() {
    let l = Language::from_bcp47("zh-Hant-TW").unwrap();
    assert_eq!(serde_json::to_string(&l).unwrap(), "\"zh-Hant-TW\"");
    round_trip(&l);
    round_trip(&Language::default());
  }

  #[test]
  fn validated_structs_check_on_deserialize() {
    let g = GeoLocation::try_new(48.8584, 2.2945, Some(330.0)).unwrap();
    round_trip(&g);
    // Out-of-range latitude is rejected, not silently materialised.
    assert!(
      serde_json::from_str::<GeoLocation>(r#"{"lat":999.0,"lon":0.0,"altitude":null}"#).is_err()
    );

    let fp = Fingerprint::try_new("chromaprint", &b"\x01\x02\x03"[..]).unwrap();
    round_trip(&fp);
    // Empty algorithm violates the invariant and must be rejected.
    assert!(serde_json::from_str::<Fingerprint>(r#"{"algorithm":"","value":[1,2,3]}"#).is_err());

    let art = CoverArt::try_new("image/png", &b"\x89PNG"[..]).unwrap();
    round_trip(&art);
    // Empty mime violates the invariant and must be rejected.
    assert!(serde_json::from_str::<CoverArt>(r#"{"mime":"","data":[1]}"#).is_err());
  }

  // ── Codex round 1 findings ──

  /// `SampleFormat` has both `Unknown(u32)` (lossless numeric escape) and
  /// `Other(SmolStr)` (lossless string escape). Every round-trip must
  /// preserve which arm a value came from — earlier the type rode the pure
  /// string path, so `Unknown(12345)` → `"unknown"` → `Other("unknown")`
  /// silently destroyed the FFmpeg code.
  #[test]
  fn sample_format_preserves_unknown_u32() {
    use crate::audio::SampleFormat;
    // Named variant — slug.
    assert_eq!(
      serde_json::to_string(&SampleFormat::S16).unwrap(),
      "\"s16\""
    );
    round_trip(&SampleFormat::S16);
    // `Other` slug variant — string.
    let other = SampleFormat::Other(smol_str::SmolStr::new("custom"));
    assert_eq!(serde_json::to_string(&other).unwrap(), "\"custom\"");
    round_trip(&other);
    // `Unknown(v)` — numeric, MUST stay `Unknown(v)` after round-trip.
    for v in [12_345u32, 0xDEAD_BEEFu32, u32::MAX] {
      let fmt = SampleFormat::Unknown(v);
      assert_eq!(serde_json::to_string(&fmt).unwrap(), v.to_string());
      let back: SampleFormat = serde_json::from_str(&v.to_string()).unwrap();
      assert_eq!(back, fmt, "lost Unknown({v}) on round-trip");
    }
    // A pure numeric input that happens to match a named variant's code
    // *does* canonicalise to the named arm — that's `from_u32`'s contract.
    let from_named_code: SampleFormat = serde_json::from_str("1").unwrap();
    assert_eq!(from_named_code, SampleFormat::S16);
  }

  /// Strictly-closed coded enums (no `Unknown` arm) must REJECT unknown
  /// wire codes instead of silently mapping them to the default. Previously
  /// `from_u32(999)` quietly returned `Embedded` / `Cbr`, so corrupt input
  /// looked like valid data on the consumer side.
  #[test]
  fn closed_coded_enums_reject_unknown_codes() {
    use crate::{audio::BitRateMode, subtitle::TrackOrigin};

    for o in [
      TrackOrigin::Embedded,
      TrackOrigin::Sidecar,
      TrackOrigin::External,
    ] {
      round_trip(&o);
    }
    for m in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
      round_trip(&m);
    }

    // Out-of-range codes are rejected — not canonicalised to the default.
    assert!(serde_json::from_str::<TrackOrigin>("999").is_err());
    assert!(serde_json::from_str::<TrackOrigin>("3").is_err());
    assert!(serde_json::from_str::<BitRateMode>("999").is_err());
    assert!(serde_json::from_str::<BitRateMode>("3").is_err());
  }

  // ── Codex round 2 findings ──

  /// `SampleFormat`'s `deserialize_any` path only works on self-describing
  /// formats. Non-self-describing binary formats (bincode/postcard) need
  /// an explicit tagged wire; the impl branches on `is_human_readable()`
  /// and serializes through a 2-variant `BinaryWire` enum. This test
  /// exercises the binary branch via postcard.
  #[test]
  fn sample_format_postcard_binary_roundtrip() {
    use crate::audio::SampleFormat;

    fn binary_round_trip(v: &SampleFormat) -> SampleFormat {
      let bytes = postcard::to_allocvec(v).expect("postcard serialize");
      postcard::from_bytes::<SampleFormat>(&bytes).expect("postcard deserialize")
    }

    // Named — `Slug` arm of the wire.
    assert_eq!(binary_round_trip(&SampleFormat::S16), SampleFormat::S16);
    // `Other` slug — also `Slug` arm.
    let other = SampleFormat::Other(smol_str::SmolStr::new("custom"));
    assert_eq!(binary_round_trip(&other), other);
    // `Unknown(v)` — `Code` arm; the u32 must survive losslessly.
    for v in [12_345u32, 0xDEAD_BEEFu32, u32::MAX] {
      let fmt = SampleFormat::Unknown(v);
      let back = binary_round_trip(&fmt);
      assert_eq!(back, fmt, "lost Unknown({v}) on postcard round-trip");
    }
  }

  /// Default-backed metadata structs must accept sparse JSON — missing
  /// fields default rather than failing — so older / partial records
  /// remain readable as the schema evolves. `serde(default)` at the
  /// container level routes missing fields through `Default`.
  #[test]
  fn sparse_json_uses_serde_default_on_default_backed_structs() {
    use crate::{
      audio::{Loudness, Tags},
      capture::Device,
    };

    // Tags: only `title` present; the rest fall back to absent sentinels.
    let t: Tags = serde_json::from_str(r#"{"title":"hello"}"#).unwrap();
    let expected = Tags::new().with_title(smol_str::SmolStr::new("hello"));
    assert_eq!(t, expected);

    // Tags: completely empty object → fully-default value (no missing-field error).
    let empty: Tags = serde_json::from_str("{}").unwrap();
    assert_eq!(empty, Tags::default());

    // Device: only `make` present.
    let d: Device = serde_json::from_str(r#"{"make":"Apple"}"#).unwrap();
    let expected = Device::new().with_make(smol_str::SmolStr::new("Apple"));
    assert_eq!(d, expected);

    // Loudness: partial measurement.
    let l: Loudness = serde_json::from_str(r#"{"integrated_lufs":-23.0}"#).unwrap();
    assert_eq!(l, Loudness::new(-23.0, 0.0, 0.0, 0.0));
  }
}
