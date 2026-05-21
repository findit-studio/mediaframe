//! Embedded audio metadata tags — FFmpeg / Vorbis-Comment / iTunes
//! atom-style key-value side data (artist, album, year, genre, …).

use smol_str::SmolStr;

/// Embedded media-metadata tags carried alongside an audio stream.
///
/// Read from FFmpeg `AVFormatContext.metadata` /
/// `AVStream.metadata` / Vorbis Comments / ID3v2 frames / MP4 `udta`
/// atoms (`©nam`, `©ART`, `©alb`, `aART`, `trkn`, `disk`, …) /
/// FLAC tags. Field names mirror the FFmpeg metadata-key convention
/// (lowercase ASCII).
///
/// **Absent-vs-empty convention:**
/// - String fields use `SmolStr`; an empty string `""` means
///   "absent" (no separate `Option` wrapper).
/// - Numeric fields use `Option<u16>` because `0` is a *valid*
///   value (year `0` exists historically; "track 0" sometimes
///   appears in test files), so the absent state must be distinct.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tags {
  title: SmolStr,
  artist: SmolStr,
  album_artist: SmolStr,
  album: SmolStr,
  composer: SmolStr,
  genre: SmolStr,
  comment: SmolStr,
  year: Option<u16>,
  track_number: Option<u16>,
  track_total: Option<u16>,
  disc_number: Option<u16>,
  disc_total: Option<u16>,
  // TODO(lang): swap to `Option<crate::Language>` once the
  // capture-lang subagent's mediaframe::Language lands. Currently a
  // BCP-47 SmolStr placeholder.
  language: Option<SmolStr>,
}

impl Default for Tags {
  /// Delegates to [`Tags::new`] — every field absent / empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl Tags {
  /// Constructs a fresh `Tags` with every field absent /
  /// empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      title: SmolStr::new_inline(""),
      artist: SmolStr::new_inline(""),
      album_artist: SmolStr::new_inline(""),
      album: SmolStr::new_inline(""),
      composer: SmolStr::new_inline(""),
      genre: SmolStr::new_inline(""),
      comment: SmolStr::new_inline(""),
      year: None,
      track_number: None,
      track_total: None,
      disc_number: None,
      disc_total: None,
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
  /// Year (`None` if absent; `Some(0)` and `Some(9999)` are both
  /// legal — `0` is *not* a sentinel here).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn year(&self) -> Option<u16> {
    self.year
  }
  /// 1-based track number (`None` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn track_number(&self) -> Option<u16> {
    self.track_number
  }
  /// Total number of tracks on the release (`None` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn track_total(&self) -> Option<u16> {
    self.track_total
  }
  /// 1-based disc number (`None` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn disc_number(&self) -> Option<u16> {
    self.disc_number
  }
  /// Total number of discs in the release (`None` if absent).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn disc_total(&self) -> Option<u16> {
    self.disc_total
  }
  /// BCP-47 language tag (`None` if absent). TODO(lang): swap to
  /// `Option<crate::Language>` once the capture-lang subagent's
  /// `mediaframe::Language` lands.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn language(&self) -> Option<&str> {
    self.language.as_deref()
  }

  /// Sets the title (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_title(mut self, v: impl Into<SmolStr>) -> Self {
    self.title = v.into();
    self
  }
  /// Sets the artist (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_artist(mut self, v: impl Into<SmolStr>) -> Self {
    self.artist = v.into();
    self
  }
  /// Sets the album artist (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_album_artist(mut self, v: impl Into<SmolStr>) -> Self {
    self.album_artist = v.into();
    self
  }
  /// Sets the album (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_album(mut self, v: impl Into<SmolStr>) -> Self {
    self.album = v.into();
    self
  }
  /// Sets the composer (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_composer(mut self, v: impl Into<SmolStr>) -> Self {
    self.composer = v.into();
    self
  }
  /// Sets the genre (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_genre(mut self, v: impl Into<SmolStr>) -> Self {
    self.genre = v.into();
    self
  }
  /// Sets the comment (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_comment(mut self, v: impl Into<SmolStr>) -> Self {
    self.comment = v.into();
    self
  }
  /// Sets the year to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_year(mut self, v: u16) -> Self {
    self.year = Some(v);
    self
  }
  /// Assigns the raw year wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_year(mut self, v: Option<u16>) -> Self {
    self.year = v;
    self
  }
  /// Sets the track number to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_track_number(mut self, v: u16) -> Self {
    self.track_number = Some(v);
    self
  }
  /// Assigns the raw track-number wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_track_number(mut self, v: Option<u16>) -> Self {
    self.track_number = v;
    self
  }
  /// Sets the track total to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_track_total(mut self, v: u16) -> Self {
    self.track_total = Some(v);
    self
  }
  /// Assigns the raw track-total wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_track_total(mut self, v: Option<u16>) -> Self {
    self.track_total = v;
    self
  }
  /// Sets the disc number to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_disc_number(mut self, v: u16) -> Self {
    self.disc_number = Some(v);
    self
  }
  /// Assigns the raw disc-number wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_disc_number(mut self, v: Option<u16>) -> Self {
    self.disc_number = v;
    self
  }
  /// Sets the disc total to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_disc_total(mut self, v: u16) -> Self {
    self.disc_total = Some(v);
    self
  }
  /// Assigns the raw disc-total wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_disc_total(mut self, v: Option<u16>) -> Self {
    self.disc_total = v;
    self
  }
  /// Sets the language tag to `Some(v)` (consuming builder).
  /// TODO(lang): swap to `crate::Language` once it lands.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_language(mut self, v: impl Into<SmolStr>) -> Self {
    self.language = Some(v.into());
    self
  }
  /// Assigns the raw language wrapper (consuming builder).
  /// TODO(lang): swap to `Option<crate::Language>` once it lands.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn maybe_language(mut self, v: Option<SmolStr>) -> Self {
    self.language = v;
    self
  }

  /// Sets the title in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_title(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.title = v.into();
    self
  }
  /// Sets the artist in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_artist(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.artist = v.into();
    self
  }
  /// Sets the album artist in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_album_artist(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.album_artist = v.into();
    self
  }
  /// Sets the album in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_album(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.album = v.into();
    self
  }
  /// Sets the composer in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_composer(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.composer = v.into();
    self
  }
  /// Sets the genre in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_genre(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.genre = v.into();
    self
  }
  /// Sets the comment in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_comment(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.comment = v.into();
    self
  }
  /// Sets the year to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_year(&mut self, v: u16) -> &mut Self {
    self.year = Some(v);
    self
  }
  /// Assigns the raw year wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_year(&mut self, v: Option<u16>) -> &mut Self {
    self.year = v;
    self
  }
  /// Clears the year (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_year(&mut self) -> &mut Self {
    self.year = None;
    self
  }
  /// Sets the track number to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_track_number(&mut self, v: u16) -> &mut Self {
    self.track_number = Some(v);
    self
  }
  /// Assigns the raw track-number wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_track_number(&mut self, v: Option<u16>) -> &mut Self {
    self.track_number = v;
    self
  }
  /// Clears the track number (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_track_number(&mut self) -> &mut Self {
    self.track_number = None;
    self
  }
  /// Sets the track total to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_track_total(&mut self, v: u16) -> &mut Self {
    self.track_total = Some(v);
    self
  }
  /// Assigns the raw track-total wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_track_total(&mut self, v: Option<u16>) -> &mut Self {
    self.track_total = v;
    self
  }
  /// Clears the track total (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_track_total(&mut self) -> &mut Self {
    self.track_total = None;
    self
  }
  /// Sets the disc number to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_disc_number(&mut self, v: u16) -> &mut Self {
    self.disc_number = Some(v);
    self
  }
  /// Assigns the raw disc-number wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_disc_number(&mut self, v: Option<u16>) -> &mut Self {
    self.disc_number = v;
    self
  }
  /// Clears the disc number (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_disc_number(&mut self) -> &mut Self {
    self.disc_number = None;
    self
  }
  /// Sets the disc total to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_disc_total(&mut self, v: u16) -> &mut Self {
    self.disc_total = Some(v);
    self
  }
  /// Assigns the raw disc-total wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_disc_total(&mut self, v: Option<u16>) -> &mut Self {
    self.disc_total = v;
    self
  }
  /// Clears the disc total (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_disc_total(&mut self) -> &mut Self {
    self.disc_total = None;
    self
  }
  /// Sets the language tag to `Some(v)` in place. TODO(lang): swap to
  /// `crate::Language` once it lands.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_language(&mut self, v: impl Into<SmolStr>) -> &mut Self {
    self.language = Some(v.into());
    self
  }
  /// Assigns the raw language wrapper in place. TODO(lang): swap to
  /// `Option<crate::Language>` once it lands.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn update_language(&mut self, v: Option<SmolStr>) -> &mut Self {
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
mod tests {
  use super::*;

  #[test]
  fn default_is_all_empty_or_none() {
    let t = Tags::default();
    assert_eq!(t.title(), "");
    assert_eq!(t.artist(), "");
    assert_eq!(t.album_artist(), "");
    assert_eq!(t.album(), "");
    assert_eq!(t.composer(), "");
    assert_eq!(t.genre(), "");
    assert_eq!(t.comment(), "");
    assert_eq!(t.year(), None);
    assert_eq!(t.track_number(), None);
    assert_eq!(t.track_total(), None);
    assert_eq!(t.disc_number(), None);
    assert_eq!(t.disc_total(), None);
    assert_eq!(t.language(), None);
  }

  #[test]
  fn new_matches_default() {
    assert_eq!(Tags::new(), Tags::default());
  }

  #[test]
  fn with_builders_roundtrip_every_field() {
    let t = Tags::new()
      .with_title("My Track")
      .with_artist("Artist X")
      .with_album_artist("Various Artists")
      .with_album("Best Album")
      .with_composer("Composer Y")
      .with_genre("Electronic")
      .with_comment("ripped 2026")
      .with_year(2026)
      .with_track_number(3)
      .with_track_total(12)
      .with_disc_number(1)
      .with_disc_total(2)
      .with_language("en-US");
    assert_eq!(t.title(), "My Track");
    assert_eq!(t.artist(), "Artist X");
    assert_eq!(t.album_artist(), "Various Artists");
    assert_eq!(t.album(), "Best Album");
    assert_eq!(t.composer(), "Composer Y");
    assert_eq!(t.genre(), "Electronic");
    assert_eq!(t.comment(), "ripped 2026");
    assert_eq!(t.year(), Some(2026));
    assert_eq!(t.track_number(), Some(3));
    assert_eq!(t.track_total(), Some(12));
    assert_eq!(t.disc_number(), Some(1));
    assert_eq!(t.disc_total(), Some(2));
    assert_eq!(t.language(), Some("en-US"));
  }

  #[test]
  fn setters_mutate_in_place() {
    let mut t = Tags::new();
    t.set_title("Foo").set_artist("Bar").set_year(1999);
    assert_eq!(t.title(), "Foo");
    assert_eq!(t.artist(), "Bar");
    assert_eq!(t.year(), Some(1999));
  }

  #[test]
  fn option_mutator_vocabulary_covers_set_update_clear() {
    // present-value forms
    let t = Tags::new().with_year(2026).with_track_number(3);
    assert_eq!(t.year(), Some(2026));
    assert_eq!(t.track_number(), Some(3));

    // raw-wrapper forms (consuming + in-place)
    let t = Tags::new().maybe_year(Some(1999)).maybe_disc_total(None);
    assert_eq!(t.year(), Some(1999));
    assert_eq!(t.disc_total(), None);

    let mut t = Tags::new();
    t.update_year(Some(2000)).update_track_total(Some(10));
    assert_eq!(t.year(), Some(2000));
    assert_eq!(t.track_total(), Some(10));

    // clear forms
    t.clear_year().clear_track_total();
    assert_eq!(t.year(), None);
    assert_eq!(t.track_total(), None);

    // language vocabulary
    let mut t = Tags::new();
    t.set_language("en-US");
    assert_eq!(t.language(), Some("en-US"));
    t.update_language(Some(SmolStr::new("fr-FR")));
    assert_eq!(t.language(), Some("fr-FR"));
    t.clear_language();
    assert_eq!(t.language(), None);
    let t = Tags::new().with_language("de-DE").maybe_language(None);
    assert_eq!(t.language(), None);
  }

  #[test]
  fn year_zero_is_meaningful_not_absent() {
    let t = Tags::new().with_year(0);
    assert_eq!(t.year(), Some(0));
    assert_ne!(t.year(), None);
  }
}
