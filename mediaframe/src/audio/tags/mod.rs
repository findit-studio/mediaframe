//! Embedded audio metadata tags — FFmpeg / Vorbis-Comment / iTunes
//! atom-style key-value side data (artist, album, year, genre, …).

use smol_bytes::Utf8Bytes;

use crate::lang::LanguageId;

/// Embedded media-metadata tags carried alongside an audio stream.
///
/// Read from FFmpeg `AVFormatContext.metadata` /
/// `AVStream.metadata` / Vorbis Comments / ID3v2 frames / MP4 `udta`
/// atoms (`©nam`, `©ART`, `©alb`, `aART`, `trkn`, `disk`, …) /
/// FLAC tags. Field names mirror the FFmpeg metadata-key convention
/// (lowercase ASCII).
///
/// **Absent-vs-present convention** — every field carries its own
/// "absent" sentinel; there is no per-field `Option` wrapper except for
/// `language` (which has no natural zero value):
/// - **String** fields use `Utf8Bytes`; the empty string `""` means absent.
/// - **Numeric** fields use `u16`; `0` means absent. Track / disc numbers
///   are 1-based and a release year is never `0`, so `0` is unambiguous —
///   and it matches the proto3 zero-elision the buffa wire codec applies
///   to these fields (a `0` is not written and decodes back to `0`).
/// - **`language`** is `Option<LanguageId>`: `None` = no language tag,
///   `Some(LanguageId)` = a parsed BCP 47 tag (which may itself be the
///   `und` "undetermined" value — distinct from "tag absent").
///
/// **`language` canonicalisation** — [`LanguageId`] is LOSSLESS: a variant
/// (`de-CH-1901`), an extension (`-u-co-phonebk`) or a private-use
/// sequence (`-x-lorem`) is carried verbatim in the tag's fourth seat and
/// rendered back exactly. What the door DOES change is spelling, and every
/// fold is a column of a vendored registry: an mkv's `ger` and an mp4's
/// `deu` both become `de`, `iw` becomes `he`, and `en-Latn` composes as
/// `en` because the registry says `en` implies Latin. See the
/// [`lang`](crate::lang) module for the whole table.
// `serde(default)` keeps sparse / older-schema JSON deserializable: missing
// fields fall back to the type-level `Default` impl (`Tags::new()` — all
// fields absent), matching the absent-vs-present convention above.
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(default)
)]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::tags")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tags {
  title: Utf8Bytes,
  artist: Utf8Bytes,
  album_artist: Utf8Bytes,
  album: Utf8Bytes,
  composer: Utf8Bytes,
  genre: Utf8Bytes,
  comment: Utf8Bytes,
  /// Release year; `0` means absent.
  year: u16,
  /// 1-based track number; `0` means absent.
  track_number: u16,
  /// Total tracks on the release; `0` means absent.
  track_total: u16,
  /// 1-based disc number; `0` means absent.
  disc_number: u16,
  /// Total discs in the release; `0` means absent.
  disc_total: u16,
  /// Parsed BCP 47 language tag; `None` means no language tag present.
  // golden-rule §9: an `Option` serde field skip-serializes when `None` —
  // absent language is an omitted key, never `"language":null`. The
  // container `serde(default)` restores it on the way back.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  language: Option<LanguageId>,
}

impl Default for Tags {
  /// Delegates to [`Tags::new`] — every field absent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl Tags {
  /// Constructs a fresh `Tags` with every field absent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      title: Utf8Bytes::new(),
      artist: Utf8Bytes::new(),
      album_artist: Utf8Bytes::new(),
      album: Utf8Bytes::new(),
      composer: Utf8Bytes::new(),
      genre: Utf8Bytes::new(),
      comment: Utf8Bytes::new(),
      year: 0,
      track_number: 0,
      track_total: 0,
      disc_number: 0,
      disc_total: 0,
      language: None,
    }
  }

  /// Track title (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn title(&self) -> &str {
    self.title.as_str()
  }
  /// Track artist (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn artist(&self) -> &str {
    self.artist.as_str()
  }
  /// Album artist — distinct from per-track `artist` for
  /// compilations / split-credit releases (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn album_artist(&self) -> &str {
    self.album_artist.as_str()
  }
  /// Album title (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn album(&self) -> &str {
    self.album.as_str()
  }
  /// Composer (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn composer(&self) -> &str {
    self.composer.as_str()
  }
  /// Genre (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn genre(&self) -> &str {
    self.genre.as_str()
  }
  /// Free-form comment (`""` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn comment(&self) -> &str {
    self.comment.as_str()
  }
  /// Release year (`0` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn year(&self) -> u16 {
    self.year
  }
  /// 1-based track number (`0` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn track_number(&self) -> u16 {
    self.track_number
  }
  /// Total number of tracks on the release (`0` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn track_total(&self) -> u16 {
    self.track_total
  }
  /// 1-based disc number (`0` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn disc_number(&self) -> u16 {
    self.disc_number
  }
  /// Total number of discs in the release (`0` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn disc_total(&self) -> u16 {
    self.disc_total
  }
  /// Parsed BCP 47 language tag (`None` if no language tag is present).
  ///
  /// A BORROW, where every other accessor on this type hands back a copy.
  /// [`LanguageId`]'s three bounded seats are each `Copy` inline ASCII, but
  /// its fourth — the lossless tail — is heap-backed, which is what makes
  /// the whole identity `Clone` and not `Copy`. `.cloned()` is the owned
  /// value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn language(&self) -> Option<&LanguageId> {
    self.language.as_ref()
  }

  /// Sets the title (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_title(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.title = v.into();
    self
  }
  /// Sets the artist (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_artist(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.artist = v.into();
    self
  }
  /// Sets the album artist (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_album_artist(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.album_artist = v.into();
    self
  }
  /// Sets the album (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_album(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.album = v.into();
    self
  }
  /// Sets the composer (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_composer(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.composer = v.into();
    self
  }
  /// Sets the genre (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_genre(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.genre = v.into();
    self
  }
  /// Sets the comment (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_comment(mut self, v: impl Into<Utf8Bytes>) -> Self {
    self.comment = v.into();
    self
  }
  /// Sets the release year (consuming builder); `0` clears it.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_year(mut self, v: u16) -> Self {
    self.year = v;
    self
  }
  /// Sets the track number (consuming builder); `0` clears it.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_track_number(mut self, v: u16) -> Self {
    self.track_number = v;
    self
  }
  /// Sets the track total (consuming builder); `0` clears it.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_track_total(mut self, v: u16) -> Self {
    self.track_total = v;
    self
  }
  /// Sets the disc number (consuming builder); `0` clears it.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_disc_number(mut self, v: u16) -> Self {
    self.disc_number = v;
    self
  }
  /// Sets the disc total (consuming builder); `0` clears it.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_disc_total(mut self, v: u16) -> Self {
    self.disc_total = v;
    self
  }
  /// Sets the language tag to `Some(v)` (consuming builder).
  ///
  /// Not `const`, and the boundary is exactly this: **does the overwrite
  /// drop a `Utf8Bytes`?** Assigning over this field drops the
  /// `Option<LanguageId>` that was there, and a `LanguageId` may hold a
  /// heap-backed tail — so the drop glue is real and cannot run at compile
  /// time (`E0493`). Read against this type's other setters the rule is the
  /// same one: the `u16` fields overwrite a value with no destructor and
  /// stay `const`, and the `Utf8Bytes` fields drop a possibly-heap-backed
  /// string and never were. It is the tail, and only the tail, that puts
  /// `language` on the second list.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_language(mut self, v: LanguageId) -> Self {
    self.language = Some(v);
    self
  }
  /// Assigns the raw language wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn maybe_language(mut self, v: Option<LanguageId>) -> Self {
    self.language = v;
    self
  }

  /// Sets the title in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_title(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.title = v.into();
    self
  }
  /// Sets the artist in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_artist(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.artist = v.into();
    self
  }
  /// Sets the album artist in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_album_artist(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.album_artist = v.into();
    self
  }
  /// Sets the album in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_album(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.album = v.into();
    self
  }
  /// Sets the composer in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_composer(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.composer = v.into();
    self
  }
  /// Sets the genre in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_genre(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.genre = v.into();
    self
  }
  /// Sets the comment in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_comment(&mut self, v: impl Into<Utf8Bytes>) -> &mut Self {
    self.comment = v.into();
    self
  }
  /// Sets the release year in place; `0` clears it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_year(&mut self, v: u16) -> &mut Self {
    self.year = v;
    self
  }
  /// Sets the track number in place; `0` clears it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_track_number(&mut self, v: u16) -> &mut Self {
    self.track_number = v;
    self
  }
  /// Sets the track total in place; `0` clears it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_track_total(&mut self, v: u16) -> &mut Self {
    self.track_total = v;
    self
  }
  /// Sets the disc number in place; `0` clears it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_disc_number(&mut self, v: u16) -> &mut Self {
    self.disc_number = v;
    self
  }
  /// Sets the disc total in place; `0` clears it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_disc_total(&mut self, v: u16) -> &mut Self {
    self.disc_total = v;
    self
  }
  /// Sets the language tag to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_language(&mut self, v: LanguageId) -> &mut Self {
    self.language = Some(v);
    self
  }
  /// Assigns the raw language wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn update_language(&mut self, v: Option<LanguageId>) -> &mut Self {
    self.language = v;
    self
  }
  /// Clears the language tag (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn clear_language(&mut self) -> &mut Self {
    self.language = None;
    self
  }
}

#[cfg(test)]
mod tests;
