//! Geographic location (GPS / EXIF) with ISO 6709 parse + format.
//!
//! Holds decimal-degree latitude / longitude plus an optional WGS84
//! altitude in metres. Parsing accepts (and `to_iso6709` produces)
//! the **degrees-only** form of ISO 6709 — `±DD.dddd±DDD.dddd[±AAA[.aaa]]/`
//! — which is the form findit-proto stores location in on
//! `MediaMeta.location_iso6709`.

use smol_bytes::Utf8Bytes;

/// Geographic location — decimal-degree latitude / longitude with an
/// optional altitude in metres above the WGS84 reference ellipsoid.
///
/// Construct via [`Self::try_new`] (validating the lat/lon ranges) or
/// parse from an ISO 6709 string via [`Self::from_iso6709`] /
/// `parse::<GeoLocation>()` (the [`core::str::FromStr`] impl).
/// Serialise back via [`Self::to_iso6709`] / [`core::fmt::Display`].
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::geo_location")
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoLocation {
  lat: f64,
  lon: f64,
  altitude: Option<f32>,
}

// Optional `serde` impls grouped in one gated `const` block: a single
// `#[cfg]` covers both directions, and the validate-on-deserialize shadow
// stays private to the block (no module-namespace pollution).
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
const _: () = {
  use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};

  impl Serialize for GeoLocation {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
      // golden-rule §9: `altitude` is optional — skip the field entirely
      // when absent rather than emitting `"altitude":null`. The deserialize
      // `Shadow` already has `#[serde(default)]` on it, so an omitted key
      // round-trips back to `None`.
      let len = 2 + usize::from(self.altitude.is_some());
      let mut st = ser.serialize_struct("GeoLocation", len)?;
      st.serialize_field("lat", &self.lat)?;
      st.serialize_field("lon", &self.lon)?;
      match &self.altitude {
        Some(alt) => st.serialize_field("altitude", alt)?,
        None => st.skip_field("altitude")?,
      }
      st.end()
    }
  }

  // Routes deserialize through `try_new` so out-of-range coordinates are
  // rejected and a non-finite altitude is normalised, instead of being
  // materialised directly by a field derive.
  #[derive(Deserialize)]
  struct Shadow {
    lat: f64,
    lon: f64,
    #[serde(default)]
    altitude: Option<f32>,
  }

  impl<'de> Deserialize<'de> for GeoLocation {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
      let s = Shadow::deserialize(de)?;
      GeoLocation::try_new(s.lat, s.lon, s.altitude).map_err(serde::de::Error::custom)
    }
  }
};

impl Default for GeoLocation {
  /// `(0.0, 0.0, None)` — "Null Island" with unknown altitude. This
  /// is a legal in-range coordinate, the conventional sentinel for
  /// "no recorded location", and matches the `DefaultInstance` seed
  /// the `buffa` wire impl uses.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      lat: 0.0,
      lon: 0.0,
      altitude: None,
    }
  }
}

impl GeoLocation {
  /// Constructs a `GeoLocation` after validating that
  /// `-90.0 <= lat <= 90.0` and `-180.0 <= lon <= 180.0`. `altitude`
  /// is metres above the WGS84 reference ellipsoid; `None` = unknown.
  ///
  /// A non-finite `altitude` (`NaN` / `±inf`) is **not** a meaningful
  /// altitude and is normalised to `None` (unknown) — unlike lat/lon,
  /// a *present* coordinate can't be "unknown", so those are rejected,
  /// whereas a bad altitude simply degrades to "no recorded altitude".
  /// This keeps the field invariant "`altitude` is `None` or finite"
  /// so [`Self::to_iso6709`] never emits a `NaN`→`0` cast artifact.
  ///
  /// # Errors
  ///
  /// - [`GeoLocationError::LatOutOfRange`] when `lat` is outside
  ///   `[-90.0, 90.0]` (or is NaN / non-finite).
  /// - [`GeoLocationError::LonOutOfRange`] when `lon` is outside
  ///   `[-180.0, 180.0]` (or is NaN / non-finite).
  pub fn try_new(lat: f64, lon: f64, altitude: Option<f32>) -> Result<Self, GeoLocationError> {
    if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
      return Err(GeoLocationError::LatOutOfRange(lat));
    }
    if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
      return Err(GeoLocationError::LonOutOfRange(lon));
    }
    Ok(Self {
      lat,
      lon,
      altitude: normalize_altitude(altitude),
    })
  }

  /// Returns the latitude in decimal degrees (`-90.0..=90.0`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn lat(&self) -> f64 {
    self.lat
  }

  /// Returns the longitude in decimal degrees (`-180.0..=180.0`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn lon(&self) -> f64 {
    self.lon
  }

  /// Returns the altitude in metres above WGS84, or `None` when
  /// unknown.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn altitude(&self) -> Option<f32> {
    self.altitude
  }

  /// Sets the altitude to `Some(altitude)` in place (a non-finite
  /// value normalises to `None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_altitude(&mut self, altitude: f32) -> &mut Self {
    self.altitude = normalize_altitude(Some(altitude));
    self
  }

  /// Assigns the raw altitude wrapper in place (`None` = unknown; a
  /// non-finite `Some` normalises to `None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_altitude(&mut self, altitude: Option<f32>) -> &mut Self {
    self.altitude = normalize_altitude(altitude);
    self
  }

  /// Clears the altitude (`None` = unknown).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_altitude(&mut self) -> &mut Self {
    self.altitude = None;
    self
  }

  /// Returns a new `GeoLocation` with the altitude set to
  /// `Some(altitude)` (consuming builder; a non-finite value
  /// normalises to `None`; useful for chaining off [`Self::try_new`]).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_altitude(mut self, altitude: f32) -> Self {
    self.altitude = normalize_altitude(Some(altitude));
    self
  }

  /// Returns a new `GeoLocation` with the raw altitude wrapper assigned
  /// (consuming builder; `None` = unknown; a non-finite `Some`
  /// normalises to `None`).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_altitude(mut self, altitude: Option<f32>) -> Self {
    self.altitude = normalize_altitude(altitude);
    self
  }

  /// Parses an ISO 6709 location string in the **degrees-only** form
  /// `±DD.dddd±DDD.dddd[±AAA[.aaa]]/` (trailing `/` mandatory).
  ///
  /// Both the latitude (2 integer digits before the optional `.`) and
  /// longitude (3 integer digits) are required to carry an explicit
  /// `+`/`-` sign prefix. Fractional digits are optional but, when
  /// present, are introduced by a literal `.`. The altitude block is
  /// optional; when present it likewise carries a `±` prefix and is
  /// interpreted as metres above the WGS84 ellipsoid.
  ///
  /// The all-zero coordinate `"+00.0000+000.0000/"` ("Null Island")
  /// is a legitimate, in-range coordinate and is accepted; only
  /// genuine range violations are rejected.
  ///
  /// # Errors
  ///
  /// - [`GeoLocationError::Iso6709Malformed`] if `s` does not match
  ///   the expected shape (missing `/`, missing sign, wrong integer-
  ///   digit count, non-numeric body, etc.).
  /// - [`GeoLocationError::LatOutOfRange`] / [`GeoLocationError::LonOutOfRange`]
  ///   when the parsed numeric values fall outside their legal ranges.
  pub fn from_iso6709(s: &str) -> Result<Self, GeoLocationError> {
    let Some(body) = s.strip_suffix('/') else {
      return Err(GeoLocationError::Iso6709Malformed(Utf8Bytes::from(s)));
    };

    // Lat token: must start with `+`/`-`. The next sign character
    // (after byte index 0) terminates it.
    let bytes = body.as_bytes();
    if bytes.is_empty() || (bytes[0] != b'+' && bytes[0] != b'-') {
      return Err(GeoLocationError::Iso6709Malformed(Utf8Bytes::from(s)));
    }
    let lon_start = match next_sign(bytes, 1) {
      Some(i) => i,
      None => return Err(GeoLocationError::Iso6709Malformed(Utf8Bytes::from(s))),
    };

    let lat_tok = &body[..lon_start];

    // Lon token starts at `lon_start`. The optional altitude token,
    // if any, starts at the next `+`/`-` after that.
    let alt_start = next_sign(bytes, lon_start + 1);
    let (lon_tok, alt_tok) = match alt_start {
      Some(i) => (&body[lon_start..i], Some(&body[i..])),
      None => (&body[lon_start..], None),
    };

    let lat = parse_signed_fixed(lat_tok, 2)
      .ok_or_else(|| GeoLocationError::Iso6709Malformed(Utf8Bytes::from(s)))?;
    let lon = parse_signed_fixed(lon_tok, 3)
      .ok_or_else(|| GeoLocationError::Iso6709Malformed(Utf8Bytes::from(s)))?;
    let altitude = match alt_tok {
      Some(tok) => Some(
        parse_signed_altitude(tok)
          .ok_or_else(|| GeoLocationError::Iso6709Malformed(Utf8Bytes::from(s)))? as f32,
      ),
      None => None,
    };

    Self::try_new(lat, lon, altitude)
  }

  /// Formats this location as an ISO 6709 string in the
  /// degrees-only form (e.g. `"+48.8566+002.3522/"` for Paris,
  /// `"-23.5505-046.6333+760/"` for São Paulo with altitude).
  ///
  /// Latitude is emitted with a 2-digit zero-padded integer part,
  /// longitude with a 3-digit zero-padded integer part, both with 4
  /// fractional digits. The altitude, when present, is emitted as a
  /// signed integer (rounded to the nearest metre) to match the
  /// representation findit-proto round-trips through.
  pub fn to_iso6709(&self) -> std::string::String {
    use core::fmt::Write as _;
    let mut out = std::string::String::with_capacity(24);
    // Latitude: ±DD.dddd
    let (lat_sign, lat_int, lat_frac) = split_signed_fixed(self.lat, 4);
    let _ = write!(&mut out, "{}{:02}.{:04}", lat_sign, lat_int, lat_frac);
    // Longitude: ±DDD.dddd
    let (lon_sign, lon_int, lon_frac) = split_signed_fixed(self.lon, 4);
    let _ = write!(&mut out, "{}{:03}.{:04}", lon_sign, lon_int, lon_frac);
    // Altitude: ±A (signed integer metres, omitted when absent).
    if let Some(alt) = self.altitude {
      // Round-half-away-from-zero via (|x| + 0.5).trunc().
      let neg = is_negative_f32(alt);
      let mag = if neg { -(alt as f64) } else { alt as f64 };
      let rounded = (mag + 0.5) as u64;
      let _ = write!(&mut out, "{}{}", if neg { '-' } else { '+' }, rounded);
    }
    out.push('/');
    out
  }
}

/// Splits a finite `f64` into `(sign char, integer part as u64, frac
/// part scaled to 10^frac_digits as u64)`. The fractional component
/// is rounded half-away-from-zero and clamped to `10^frac_digits - 1`
/// (the truncation guards against carry into the integer part, which
/// for our 2/3-digit lat/lon would never happen in practice but is
/// defensively bounded).
fn split_signed_fixed(v: f64, frac_digits: u32) -> (char, u64, u64) {
  let neg = is_negative_f64(v);
  let mag = if neg { -v } else { v };
  let int_part = mag as u64;
  let frac_part_f = mag - (int_part as f64);
  let scale = pow10_u64(frac_digits) as f64;
  let scaled = (frac_part_f * scale + 0.5) as u64;
  let frac = scaled.min(pow10_u64(frac_digits) - 1);
  (if neg { '-' } else { '+' }, int_part, frac)
}

/// `core`-friendly sign predicate for `f64`. Uses `to_bits` to read
/// the IEEE-754 sign bit, so it also catches `-0.0`.
fn is_negative_f64(v: f64) -> bool {
  v.to_bits() & (1u64 << 63) != 0
}

/// `core`-friendly sign predicate for `f32`.
fn is_negative_f32(v: f32) -> bool {
  v.to_bits() & (1u32 << 31) != 0
}

/// `10u64.pow(n)` is `const`-fn; expose it as a stand-alone helper
/// so the call site is grep-able.
const fn pow10_u64(n: u32) -> u64 {
  10u64.pow(n)
}

impl core::str::FromStr for GeoLocation {
  type Err = GeoLocationError;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_str(s: &str) -> Result<Self, GeoLocationError> {
    Self::from_iso6709(s)
  }
}

impl core::fmt::Display for GeoLocation {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_iso6709())
  }
}

/// Errors returned by [`GeoLocation`] constructors / parsers.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum GeoLocationError {
  /// Latitude is outside the legal `[-90.0, 90.0]` range (or is
  /// non-finite).
  #[error("latitude out of range [-90, 90]: {0}")]
  LatOutOfRange(f64),
  /// Longitude is outside the legal `[-180.0, 180.0]` range (or is
  /// non-finite).
  #[error("longitude out of range [-180, 180]: {0}")]
  LonOutOfRange(f64),
  /// The input string is not a recognisable ISO 6709 degrees-only
  /// location (the offending input is wrapped verbatim).
  #[error("malformed ISO 6709 location string: {0:?}")]
  Iso6709Malformed(Utf8Bytes),
}

// ----------------------------------------------------------------------------
// ISO 6709 helpers (hand-rolled — no regex / no chrono).
// ----------------------------------------------------------------------------

/// Returns the index of the next `+` or `-` byte at or after `from`,
/// or `None` if none exists.
fn next_sign(bytes: &[u8], from: usize) -> Option<usize> {
  let mut i = from;
  while i < bytes.len() {
    if bytes[i] == b'+' || bytes[i] == b'-' {
      return Some(i);
    }
    i += 1;
  }
  None
}

/// Collapses a non-finite altitude (`NaN` / `±inf`) to `None`. A finite
/// `Some(v)` passes through unchanged. The single funnel every altitude
/// entry point (`try_new`, `set_/with_/update_/maybe_altitude`) routes
/// through so the field invariant "`altitude` is `None` or finite" holds.
#[cfg_attr(not(tarpaulin), inline(always))]
const fn normalize_altitude(altitude: Option<f32>) -> Option<f32> {
  match altitude {
    Some(v) if v.is_finite() => Some(v),
    _ => None,
  }
}

/// Parses a signed fixed-width decimal: `±` + exactly `int_digits`
/// integer ASCII digits, optionally followed by `.` and 1+ fractional
/// digits. Returns the decimal value or `None` on malformed input.
fn parse_signed_fixed(tok: &str, int_digits: usize) -> Option<f64> {
  let bytes = tok.as_bytes();
  if bytes.len() < 1 + int_digits {
    return None;
  }
  let neg = match bytes[0] {
    b'+' => false,
    b'-' => true,
    _ => return None,
  };
  // Integer part: must be exactly `int_digits` ASCII digits.
  let int_end = 1 + int_digits;
  let int_slice = &tok[1..int_end];
  if int_slice.bytes().any(|b| !b.is_ascii_digit()) {
    return None;
  }
  // Whatever follows must be empty or `.<digits>`.
  let frac = if int_end == tok.len() {
    0.0
  } else {
    let rest = &tok[int_end..];
    let rest_bytes = rest.as_bytes();
    if rest_bytes[0] != b'.' || rest_bytes.len() < 2 {
      return None;
    }
    let frac_str = &rest[1..];
    if frac_str.bytes().any(|b| !b.is_ascii_digit()) {
      return None;
    }
    frac_str.parse::<f64>().ok()? / pow10_u64(frac_str.len() as u32) as f64
  };
  let int_val: f64 = int_slice.parse::<u64>().ok()? as f64;
  let mag = int_val + frac;
  Some(if neg { -mag } else { mag })
}

/// Parses the optional altitude token (`±` + 1+ integer digits +
/// optional `.<frac>`). Width is not fixed.
fn parse_signed_altitude(tok: &str) -> Option<f64> {
  let bytes = tok.as_bytes();
  if bytes.len() < 2 {
    return None;
  }
  let neg = match bytes[0] {
    b'+' => false,
    b'-' => true,
    _ => return None,
  };
  let body = &tok[1..];
  // Split on optional `.`
  let (int_part, frac_part) = match body.find('.') {
    Some(i) => {
      let (lhs, rhs) = body.split_at(i);
      (lhs, &rhs[1..])
    }
    None => (body, ""),
  };
  if int_part.is_empty() || int_part.bytes().any(|b| !b.is_ascii_digit()) {
    return None;
  }
  if !frac_part.is_empty() && frac_part.bytes().any(|b| !b.is_ascii_digit()) {
    return None;
  }
  let int_val: f64 = int_part.parse::<u64>().ok()? as f64;
  let frac = if frac_part.is_empty() {
    0.0
  } else {
    frac_part.parse::<f64>().ok()? / pow10_u64(frac_part.len() as u32) as f64
  };
  let mag = int_val + frac;
  Some(if neg { -mag } else { mag })
}

#[cfg(test)]
mod tests;
