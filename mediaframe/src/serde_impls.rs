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

/// Implements `Serialize` / `Deserialize` for a *closed* FFmpeg-coded
/// enum via its `to_u32()` / `from_u32()` round-trip. Total in both
/// directions — an unrecognised code rides the enum's `Unknown` arm.
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
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::SampleFormat);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ContainerFormat);

// ── Closed coded enums that live in heap-tier modules ──
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_code!(crate::subtitle::TrackOrigin);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_code!(crate::audio::BitRateMode);

#[cfg(all(test, feature = "std"))]
mod tests {
  use crate::{
    audio::{ChannelLayout, Fingerprint, Tags},
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
  }
}
