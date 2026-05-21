//! BCP-47 language tag — language + optional script + optional
//! region, validated by `icu_locale_core`.
//!
//! Supersedes the dropped `medialang` crate plan: mediaframe owns the
//! single canonical language type, engine crates (`whispercpp::Lang`
//! and friends) stay engine-internal and boundary-convert in.
//!
//! Note on representation: the spec calls for wrapping
//! `icu_locale_core::LanguageIdentifier`, but that full struct includes a
//! heap-backed `Variants` collection and is therefore NOT `Copy`.
//! Since the public API only needs `language` + optional `script` +
//! optional `region` (and the brief explicitly requires `Copy`,
//! no-alloc), this wrapper is a triple of the underlying
//! `tinystr::TinyAsciiStr`-backed subtag types — which **are** `Copy`
//! — rather than the full `LanguageIdentifier`. Variants/extensions
//! that appear in the source BCP-47 string are validated by
//! `LanguageIdentifier::try_from_bytes` and then discarded.

use icu_locale_core::{
  LanguageIdentifier,
  subtags::{Language as IcuLanguage, Region as IcuRegion, Script as IcuScript},
};
use smol_str::SmolStr;

/// Validated BCP-47 language tag — language subtag plus optional
/// script + region. Wraps the `icu_locale_core` subtag types directly so
/// the whole value is `Copy` and heap-free.
///
/// Construct from a full tag string with [`Self::from_bcp47`] or
/// from already-validated components with [`Self::try_new`]. The
/// canonical short codes are exposed via [`Self::language`],
/// [`Self::script`], [`Self::region`]; the round-trip BCP-47 text
/// form is via [`Self::to_bcp47`] / [`core::fmt::Display`].
///
/// The default is the BCP-47 `"und"` tag (undetermined language —
/// ISO 639-3); detect it with [`Self::is_undetermined`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Language {
  language: IcuLanguage,
  script: Option<IcuScript>,
  region: Option<IcuRegion>,
}

impl Language {
  /// Constructs the undetermined language (`"und"`) — the same value
  /// [`Default::default`] yields. `const`, allocation-free.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      language: IcuLanguage::UNKNOWN,
      script: None,
      region: None,
    }
  }

  /// Constructs a `Language` from the three (optionally-present)
  /// subtag strings, validating each via the corresponding
  /// `icu_locale_core` parser. `lang` is required (use `"und"` to mean
  /// "undetermined"); `script` / `region` are optional.
  ///
  /// # Errors
  ///
  /// - [`LanguageError::InvalidLanguage`] when `lang` is not a
  ///   well-formed ISO 639-1/2/3 language subtag.
  /// - [`LanguageError::InvalidScript`] when `script` is `Some`
  ///   but not a well-formed ISO 15924 4-letter script code.
  /// - [`LanguageError::InvalidRegion`] when `region` is `Some`
  ///   but not a well-formed ISO 3166-1 alpha-2 / UN M.49 code.
  pub fn try_new(
    lang: &str,
    script: Option<&str>,
    region: Option<&str>,
  ) -> Result<Self, LanguageError> {
    let language = IcuLanguage::try_from_str(lang)
      .map_err(|_| LanguageError::InvalidLanguage(SmolStr::from(lang)))?;
    let script = match script {
      Some(s) => Some(
        IcuScript::try_from_str(s).map_err(|_| LanguageError::InvalidScript(SmolStr::from(s)))?,
      ),
      None => None,
    };
    let region = match region {
      Some(r) => Some(
        IcuRegion::try_from_str(r).map_err(|_| LanguageError::InvalidRegion(SmolStr::from(r)))?,
      ),
      None => None,
    };
    Ok(Self {
      language,
      script,
      region,
    })
  }

  /// Parses a full BCP-47 language tag like `"en"`, `"en-US"`, or
  /// `"zh-Hant-TW"`. Variants and extensions that appear in the
  /// input are validated by `icu_locale_core` and then discarded — only
  /// the language / script / region subtags are retained.
  ///
  /// # Errors
  ///
  /// - [`LanguageError::MalformedBcp47`] when the input is not a
  ///   well-formed BCP-47 language identifier.
  pub fn from_bcp47(s: &str) -> Result<Self, LanguageError> {
    let id = LanguageIdentifier::try_from_str(s)
      .map_err(|_| LanguageError::MalformedBcp47(SmolStr::from(s)))?;
    Ok(Self {
      language: id.language,
      script: id.script,
      region: id.region,
    })
  }

  /// Returns the language subtag as its canonical short code (e.g.
  /// `"en"`, `"zh"`, or `"und"` for undetermined).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn language(&self) -> &str {
    self.language.as_str()
  }

  /// Returns the script subtag's canonical 4-letter ISO 15924 code
  /// (e.g. `"Hant"`, `"Latn"`), or `None` when no script subtag is
  /// present.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn script(&self) -> Option<&str> {
    self.script.as_ref().map(|s| s.as_str())
  }

  /// Returns the region subtag's canonical code (e.g. `"US"`,
  /// `"TW"`, `"419"`), or `None` when no region subtag is present.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn region(&self) -> Option<&str> {
    self.region.as_ref().map(|r| r.as_str())
  }

  /// Formats this `Language` as its canonical BCP-47 representation
  /// (e.g. `"en"`, `"en-US"`, `"zh-Hant-TW"`, `"und"`).
  pub fn to_bcp47(&self) -> std::string::String {
    use core::fmt::Write as _;
    let mut out = std::string::String::with_capacity(11);
    let _ = out.write_str(self.language.as_str());
    if let Some(s) = self.script.as_ref() {
      let _ = out.write_str("-");
      let _ = out.write_str(s.as_str());
    }
    if let Some(r) = self.region.as_ref() {
      let _ = out.write_str("-");
      let _ = out.write_str(r.as_str());
    }
    out
  }

  /// Returns `true` when the language subtag is the ISO 639-3 `"und"`
  /// (undetermined) — the value [`Default::default`] yields.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_undetermined(&self) -> bool {
    self.language.is_unknown()
  }
}

impl Default for Language {
  /// Delegates to [`Language::new`] — the BCP-47 `"und"`
  /// (undetermined language).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl core::fmt::Display for Language {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_bcp47())
  }
}

impl core::str::FromStr for Language {
  type Err = LanguageError;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_str(s: &str) -> Result<Self, LanguageError> {
    Self::from_bcp47(s)
  }
}

/// Errors returned by [`Language`] constructors / parsers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageError {
  /// The language subtag is not a well-formed ISO 639-1/2/3 code.
  #[error("invalid BCP-47 language subtag: {0:?}")]
  InvalidLanguage(SmolStr),
  /// The script subtag is not a well-formed ISO 15924 4-letter code.
  #[error("invalid BCP-47 script subtag: {0:?}")]
  InvalidScript(SmolStr),
  /// The region subtag is not a well-formed ISO 3166-1 alpha-2 / UN
  /// M.49 code.
  #[error("invalid BCP-47 region subtag: {0:?}")]
  InvalidRegion(SmolStr),
  /// The input is not a well-formed BCP-47 language identifier.
  #[error("malformed BCP-47 language tag: {0:?}")]
  MalformedBcp47(SmolStr),
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::str::FromStr;

  // Compile-time check that `Language` is `Copy`. (If the icu_locale_core
  // upgrade ever breaks this, the build fails here rather than at a
  // distant call site that relied on it.)
  const fn _is_copy<T: Copy>() {}
  const _: () = _is_copy::<Language>();

  #[test]
  fn default_is_und() {
    let l = Language::default();
    assert_eq!(l.language(), "und");
    assert!(l.script().is_none());
    assert!(l.region().is_none());
    assert!(l.is_undetermined());
  }

  #[test]
  fn from_bcp47_lang_only() {
    let l = Language::from_bcp47("en").unwrap();
    assert_eq!(l.language(), "en");
    assert!(l.script().is_none());
    assert!(l.region().is_none());
    assert!(!l.is_undetermined());
    assert_eq!(l.to_bcp47(), "en");
  }

  #[test]
  fn from_bcp47_lang_region() {
    let l = Language::from_bcp47("en-US").unwrap();
    assert_eq!(l.language(), "en");
    assert_eq!(l.region(), Some("US"));
    assert!(l.script().is_none());
    assert_eq!(l.to_bcp47(), "en-US");
  }

  #[test]
  fn from_bcp47_lang_script_region() {
    let l = Language::from_bcp47("zh-Hant-TW").unwrap();
    assert_eq!(l.language(), "zh");
    assert_eq!(l.script(), Some("Hant"));
    assert_eq!(l.region(), Some("TW"));
    assert_eq!(l.to_bcp47(), "zh-Hant-TW");
  }

  #[test]
  fn from_bcp47_und() {
    let l = Language::from_bcp47("und").unwrap();
    assert!(l.is_undetermined());
    assert_eq!(l.to_bcp47(), "und");
  }

  #[test]
  fn from_bcp47_rejects_bogus() {
    let err = Language::from_bcp47("xx-yy-zz-bogus").unwrap_err();
    assert!(matches!(err, LanguageError::MalformedBcp47(_)));
  }

  #[test]
  fn try_new_components() {
    let l = Language::try_new("en", None, Some("US")).unwrap();
    assert_eq!(l.language(), "en");
    assert_eq!(l.region(), Some("US"));

    let l = Language::try_new("zh", Some("Hant"), Some("TW")).unwrap();
    assert_eq!(l.script(), Some("Hant"));
    assert_eq!(l.region(), Some("TW"));
  }

  #[test]
  fn try_new_rejects_each_subtag() {
    assert!(matches!(
      Language::try_new("!!", None, None),
      Err(LanguageError::InvalidLanguage(_))
    ));
    assert!(matches!(
      Language::try_new("en", Some("###"), None),
      Err(LanguageError::InvalidScript(_))
    ));
    assert!(matches!(
      Language::try_new("en", None, Some("###")),
      Err(LanguageError::InvalidRegion(_))
    ));
  }

  #[test]
  fn from_str_smoke() {
    let l: Language = "en-US".parse().unwrap();
    assert_eq!(l.language(), "en");
    assert_eq!(l.region(), Some("US"));
  }

  #[test]
  fn display_round_trip() {
    let l = Language::from_bcp47("zh-Hant-TW").unwrap();
    let rendered = std::format!("{}", l);
    assert_eq!(rendered, "zh-Hant-TW");
    let parsed = Language::from_str(&rendered).unwrap();
    assert_eq!(parsed, l);
  }
}
