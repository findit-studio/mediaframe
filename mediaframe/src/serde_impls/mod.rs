//! Centralised `serde` implementations for the descriptor enums
//! (`feature = "serde"`).
//!
//! # Two laws, and the second one has legs
//!
//! **An open vocabulary is always its slug.** **A closed one splits on
//! the format**: its name where a human will read it, its code where
//! only a machine will. `Serializer::is_human_readable()` is what says
//! which, and the split is per-format rather than per-type — the same
//! value is `"native"` in JSON and a varint in postcard, and neither is
//! a fallback for the other.
//!
//! The reason the two laws differ is the escape arm, not taste. An open
//! vocabulary's `Other(Utf8Bytes)` holds a name and *only* a name: there
//! is no code to fall back to for a value this build has never heard
//! of, so a numeric leg could not carry one and the slug is the only
//! honest wire at either end. A closed vocabulary has no such value —
//! every member has both spellings, so the format gets to choose, and a
//! binary format has no reason to pay for a string it cannot read
//! anyway.
//!
//! ## The open law — always the slug
//!
//! - **Every open vocabulary enum** — codecs, formats, the colour enums,
//!   the pixel format, the frame coded enums — serializes as its
//!   canonical `as_str()` slug: `VideoCodec::H264` ⇄ `"h264"`,
//!   `color::Matrix::Bt709` ⇄ `"bt709"`, `Other("x265")` ⇄ `"x265"` (no
//!   `{"Other": …}` wrapper). One extension idiom, one wire shape, every
//!   format. Round-trip total wherever the `Other(Utf8Bytes)` arm exists
//!   (the `alloc` tier); at the no-alloc tier the same enums are closed,
//!   so an unrecognised slug is a serde error rather than a
//!   silently-invented value. Deserialization goes through the type's
//!   `FromStr`, so it also reads the documented FFmpeg synonyms
//!   (`"gray"` → `PixelFormat::Gray8`, `"unknown"` →
//!   `color::Matrix::Unspecified`); serialization stays canonical, so a
//!   synonym read off the wire is written back in the canonical
//!   spelling.
//! - **`TrackDisposition`** is outside both laws: it is a bit set, not a
//!   name vocabulary, so it serializes as its `u32` bits. The number
//!   *is* the value and there is no name to spell — in any format.
//!
//! ## The closed law — the slug leg and the code leg
//!
//! **Strictly-closed coded enums (no `Other` arm)** —
//! [`crate::audio::BitRateMode`], [`crate::audio::ChannelOrder`] — take
//! both legs, and **both legs are strict**:
//!
//! | leg | shape | read side |
//! |---|---|---|
//! | `is_human_readable()` | the `as_str()` slug (`"cbr"`, `"native"`) | the type's `FromStr`; an unrecognised **name** is a serde error, and a *number* is refused outright — it is not a name |
//! | binary | the `to_u32()` code | `try_from_u32`; an out-of-range **code** is a serde error |
//!
//! Strict on both legs means the same thing on both: an input this
//! vocabulary cannot name is *refused*, never collapsed onto the default
//! variant the way `from_u32` would collapse it (`BitRateMode::from_u32(999)
//! == Cbr`, `ChannelOrder::from_u32(999) == Unspecified`). A corrupt or
//! out-of-range value must fail loudly rather than arrive looking like
//! valid data. The slug leg still folds ASCII case, because that is the
//! whole of the crate's folding and `"CBR"` is the same *name* as
//! `"cbr"` — folding a spelling is not inventing a value.
//!
//! [`crate::subtitle::TrackOrigin`] left this group in 0.5.0 when it
//! gained an `Other` arm: an open vocabulary has no closed code space to
//! police, and no code for its escape to carry, so it moved to the open
//! law above and stays there under both formats.
//!
//! `ChannelOrder`'s code space really is closed, which is what puts it
//! here: it mirrors FFmpeg's `AVChannelOrder`, four members with no
//! vendor range, so every integer outside `0..=3` is a corrupt read
//! rather than a name this build has not heard of.
//!
//! The plain data structs (`color::Info`, `frame::Dimensions`,
//! `audio::Tags`, `audio::ChannelSpec`,
//! `audio::ChannelLayoutDescription`, …) derive serde at their
//! definition site; the
//! validated structs (`capture::GeoLocation`, `audio::Fingerprint`,
//! `audio::CoverArt`, `frame::WhiteBalance`,
//! `frame::ColorCorrectionMatrix`) route deserialize through their
//! checking constructors there too. The `lang` household carries bespoke
//! canonical-text impls for all four of its types in its own module.

/// Implements `Serialize` / `Deserialize` for an *open* enum via its
/// canonical string slug (`as_str()` to serialize, [`FromStr`] to parse).
/// The `FromStr` impl is total (`Err = Infallible`) — unknown slugs ride
/// the enum's `Other` arm — but the deserializer surfaces any error as a
/// serde error for forward-compatibility.
///
/// [`FromStr`]: core::str::FromStr
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

/// Implements `Serialize` / `Deserialize` via a `u32` whose every value is
/// meaningful wire data — the bit-set case, where the number *is* the
/// value and there is no name to spell. `TrackDisposition` is the only
/// such type; name vocabularies use [`serde_via_str!`] instead.
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
/// FFmpeg-coded enum — one with no escape arm at all — with a leg per
/// format: the `as_str()` slug where `is_human_readable()`, the
/// `to_u32()` code where it is not.
///
/// Both legs are strict, and strict means the same thing on each: an
/// input this vocabulary cannot name is **refused**, never collapsed
/// onto the default variant the way `from_u32` would collapse it. The
/// slug leg refuses an unrecognised name through the type's own
/// `FromStr`; the code leg refuses an out-of-range code through
/// `try_from_u32`.
///
/// Two properties of the slug leg are load-bearing and easy to lose:
///
/// * **A number is not a name.** The visitor implements `visit_str` and
///   nothing else, so a JSON `1` reaches serde's default `visit_u64` and
///   comes back as an `invalid type` error rather than being read as a
///   code. The legs are alternatives, not a chain of fallbacks — a
///   human-readable document that carries a bare integer here is
///   malformed, not merely terse.
/// * **Case still folds.** `FromStr` goes through the crate's one ASCII
///   folding gate, so `"CBR"` and `"cbr"` are one value. Folding a
///   *spelling* is not inventing a *value*, which is the line strictness
///   is drawn on.
///
/// An open vocabulary does **not** get this treatment — see
/// [`serde_via_str!`] and the two laws in the module docs. Its
/// `Other(Utf8Bytes)` holds a name with no code behind it, so a numeric
/// leg would have nothing to write.
// Both invocations are heap-tier — gated on
// `any(feature = "std", feature = "alloc")`. Under bare `--features serde`
// (no-alloc tier) they are cfg'd out and the macro is unused; the `allow`
// silences the resulting `unused_macros` lint, exactly as for `serde_via_str!`.
#[allow(unused_macros)]
macro_rules! serde_via_slug_or_code {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        if ser.is_human_readable() {
          ser.serialize_str(self.as_str())
        } else {
          ser.serialize_u32(self.to_u32())
        }
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        if de.is_human_readable() {
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
        } else {
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
    }
  };
}

// ── The bit set: a number is its only faithful spelling ──
serde_via_code!(crate::disposition::TrackDisposition);

// ── Name vocabularies available at every capability tier ──
// Open at the `alloc` tier (an unrecognised slug rides `Other`), closed at
// the no-alloc tier (it is a serde error) — one wire shape either way.
serde_via_str!(crate::color::Matrix);
serde_via_str!(crate::color::Primaries);
serde_via_str!(crate::color::Transfer);
serde_via_str!(crate::color::DynamicRange);
serde_via_str!(crate::color::ChromaLocation);
serde_via_str!(crate::color::DcpTargetGamut);
serde_via_str!(crate::pixel_format::PixelFormat);
serde_via_str!(crate::frame::Rotation);
serde_via_str!(crate::frame::FieldOrder);
serde_via_str!(crate::frame::StereoMode);

// ── The RAW / bayer vocabularies (behind the `bayer` feature) ──
// Closed: they name sensor layouts and demosaic algorithms, not an open
// space a backend extends, so an unrecognised slug is a serde error.
// `WhiteBalance` / `ColorCorrectionMatrix` are float structs and carry
// their own validating impls at their definition site.
#[cfg(feature = "bayer")]
serde_via_str!(crate::frame::BayerPattern);
#[cfg(feature = "bayer")]
serde_via_str!(crate::frame::BayerDemosaic);
#[cfg(feature = "bayer")]
serde_via_str!(crate::frame::WbChannel);

// ── Name vocabularies that need the allocator for their own payloads ──
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::VideoCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::AudioCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::SubtitleCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::DataCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::AttachmentCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::container::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::image::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::subtitle::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::subtitle::TrackOrigin);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ChannelLayout);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::SampleFormat);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ContainerFormat);

// ── Strictly-closed coded enums (no `Unknown` escape) ──
// Use `serde_via_slug_or_code!` — the slug where a human reads it, the
// code where only a machine does, and both legs strict: an unrecognised
// name or an out-of-range code is a serde error, never canonicalised to
// the default (which `from_u32` would do for
// `BitRateMode::from_u32(999) == Cbr` and
// `ChannelOrder::from_u32(999) == Unspecified`).
//
// No exceptions here: every member of this group takes both legs. A
// closed vocabulary pinned to one shape would be exactly the asymmetry
// the two-law split exists to remove.
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_slug_or_code!(crate::audio::BitRateMode);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_slug_or_code!(crate::audio::ChannelOrder);

#[cfg(all(test, feature = "std"))]
mod tests;
