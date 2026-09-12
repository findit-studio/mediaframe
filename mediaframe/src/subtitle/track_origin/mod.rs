//! [`TrackOrigin`] — provenance axis for a subtitle track:
//! where the bytes come from relative to the media file.

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;

/// Where this subtitle track came from, relative to the media file
/// it accompanies.
///
/// Open vocabulary: the named variants cover the buckets a media
/// pipeline sorts tracks into, and [`Self::Other`] carries a
/// classification this build does not name. mediaframe is a shared
/// library, not one pipeline's private enum — the set of provenances
/// worth distinguishing belongs to whoever is doing the classifying,
/// so a downstream that tracks a distinction this crate has not heard
/// of keeps its **name** rather than losing it to a nearby variant.
///
/// `Embedded` is the default — the typical case is a subtitle stream
/// multiplexed inside the container. The named variants keep stable,
/// append-only `u32` ids; [`Self::Other`] has none (see
/// [`Self::to_u32`]), so the slug from [`Self::as_str`] is the
/// spelling that always survives.
///
/// `#[non_exhaustive]` is retained: promoting a slug that rides
/// `Other` today into a named variant tomorrow stays a minor change.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::track_origin")
)]
pub enum TrackOrigin {
  /// Stream multiplexed into the container alongside the video /
  /// audio tracks (e.g. an `.mkv` with embedded `.srt`-equivalent
  /// subtitle streams). The default origin.
  #[default]
  Embedded,
  /// Separate subtitle file living next to the media file on disk
  /// — typically an external `.srt` / `.vtt` / `.ass` paired with
  /// the video by filename stem.
  Sidecar,
  /// Externally sourced — downloaded from an online subtitle
  /// database, or otherwise obtained ready-made from outside the
  /// original media container. The track existed as subtitle text
  /// before this pipeline saw the media; contrast [`Self::Derived`],
  /// which this pipeline produced from the media itself.
  External,
  /// Produced by a processing pass over the media rather than
  /// obtained as subtitle text — an automatic-speech-recognition
  /// transcript, a machine translation of another track, OCR of
  /// image-based subtitles, or any comparable derivation.
  ///
  /// The axis this variant adds is *provenance-by-derivation*: the
  /// bytes have no existence independent of a pipeline stage, so
  /// their quality, licensing and re-generability differ from a
  /// track that arrived as text. [`Self::External`] covers the
  /// arrived-as-text case even when the arrival was a download.
  Derived,
  /// A provenance this vocabulary does not name — carried verbatim. The
  /// crate-wide extension idiom: a downstream classifier naming an
  /// origin mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  Other(Utf8Bytes),
}

impl TrackOrigin {
  /// Canonical lowercase slug for this origin (`"embedded"` /
  /// `"sidecar"` / `"external"` / `"derived"`). Stable; matches what
  /// [`core::fmt::Display`] produces. [`Self::Other`] returns the
  /// wrapped slug verbatim.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Embedded => "embedded",
      Self::Sidecar => "sidecar",
      Self::External => "external",
      Self::Derived => "derived",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id for the named origins: `Embedded=0`,
  /// `Sidecar=1`, `External=2`, `Derived=3`. Append-only — never
  /// renumber.
  ///
  /// [`None`] for [`Self::Other`]: it names an origin this build has
  /// no id for, and inventing one would lose the name. The slug from
  /// [`Self::as_str`] is the spelling that always survives — which is
  /// why the wire codecs (`serde`, `buffa`) carry the slug and not
  /// this number.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Embedded => 0,
      Self::Sidecar => 1,
      Self::External => 2,
      Self::Derived => 3,
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unknown ids fall back to the default
  /// ([`Self::Embedded`]): the numeric space is this crate's own and
  /// an id it never assigned carries no name to preserve, so
  /// [`Self::Other`] is not the right home for one. The round-trip is
  /// exact for the enumerated ids `0`/`1`/`2`/`3`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Embedded,
      1 => Self::Sidecar,
      2 => Self::External,
      3 => Self::Derived,
      _ => Self::Embedded,
    }
  }

  /// Strict counterpart to [`Self::from_u32`]: returns `None` for any code
  /// outside the enumerated set, instead of silently mapping it to the
  /// default. Used by the strict deserialize path so adversarial / corrupt
  /// wire values fail loudly rather than masquerading as `Embedded`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_from_u32(v: u32) -> Option<Self> {
    match v {
      0 => Some(Self::Embedded),
      1 => Some(Self::Sidecar),
      2 => Some(Self::External),
      3 => Some(Self::Derived),
      _ => None,
    }
  }

  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`](core::str::FromStr)'s
  /// own match table, walked through [`Self::from_str`] rather than
  /// duplicated here — so a canonical spelling returns that **named**
  /// variant, never a second value for a meaning this vocabulary already
  /// has one for. Only a genuine stranger reaches [`Self::Other`],
  /// carrying the caller's spelling verbatim: the escape is a lossless
  /// passthrough for a name this build does not know, not a fold target.
  pub fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  TrackOrigin,
  "track origin",
  [Embedded, Sidecar, External, Derived],
  escape: Other
);

/// The error [`TrackOrigin`] would return if its vocabulary were closed.
///
/// **Not reachable, and no longer `FromStr::Err`.** [`TrackOrigin`] gained
/// its [`Other`](TrackOrigin::Other) escape in 0.5.0, and this module is
/// compiled only at the `alloc` tier, so there is no tier at which the
/// parse can refuse — `FromStr::Err` is
/// [`Infallible`](core::convert::Infallible) unconditionally.
///
/// It is kept, exported and unchanged, on the same policy as the nine
/// all-tier vocabularies that split their `Err` by tier in 0.5.0: removing
/// an exported type is itself a breaking change, and the type is the
/// vocabulary's refusal *name*, which becomes real again the day this
/// module grows a no-alloc tier. Unlike those nine — whose lean build
/// still returns theirs — nothing constructs this one today. That is the
/// honest statement, and it is why the type is documented rather than
/// quietly left looking live.
///
/// Opaque and sealed: the input is deliberately not retained (the input is
/// attacker-controlled on the deserialization path). `#[non_exhaustive]`
/// keeps it constructible only here, so it can grow structure later
/// without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a track-origin name")]
#[non_exhaustive]
pub struct ParseTrackOriginError;

impl core::str::FromStr for TrackOrigin {
  /// Unconditionally [`Infallible`](core::convert::Infallible) — unlike
  /// the all-tier vocabularies, this one needs no `cfg` split, because
  /// the module itself exists only where the escape does. There is no
  /// tier at which the parse can refuse, so there is nothing to fork.
  type Err = core::convert::Infallible;

  /// Parses the canonical slug [`Self::as_str`] renders — the exact
  /// inverse of [`Display`](core::fmt::Display).
  ///
  /// # Errors
  ///
  /// Never, and the type says so: a slug this vocabulary does not name
  /// rides [`Self::Other`], carrying the caller's spelling verbatim, so
  /// `Self::Err` is uninhabited and a caller can discharge it with an
  /// irrefutable `let Ok(o) = s.parse::<TrackOrigin>();`.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"embedded" => Self::Embedded,
      b"sidecar" => Self::Sidecar,
      b"external" => Self::External,
      b"derived" => Self::Derived,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

#[cfg(test)]
mod tests;
