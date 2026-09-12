use super::{Case, FOLD_CAP, fold, lookup};

#[test]
fn fold_lowercases_ascii_and_leaves_the_rest_alone() {
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("BT709", &mut buf), Some(&b"bt709"[..]));
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(
    fold("Chroma-Derived-NC", &mut buf),
    Some(&b"chroma-derived-nc"[..])
  );
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("yuv420p", &mut buf), Some(&b"yuv420p"[..]));
  // Non-ASCII passes through untouched — no locale-dependent mapping.
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("İ", &mut buf), Some("İ".as_bytes()));
}

/// The fold hands back the folded bytes themselves — the key the
/// `b"slug"` tables compare against — so the slice is the buffer prefix
/// of exactly the input's length, and an input that *fills* the buffer
/// is still a hit (only one byte more is the miss).
#[test]
fn fold_returns_the_folded_bytes_up_to_capacity() {
  let brim = core::str::from_utf8(&[b'X'; FOLD_CAP]).unwrap();
  let mut buf = [0u8; FOLD_CAP];
  let folded = fold(brim, &mut buf).expect("an input that exactly fills the buffer folds");
  assert_eq!(folded.len(), FOLD_CAP);
  assert!(folded.iter().all(|b| *b == b'x'));

  // The empty slug is an empty key, not a miss — a lookup, not a length
  // check, is what rejects it.
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("", &mut buf), Some(&b""[..]));
}

#[test]
fn an_input_longer_than_any_slug_is_a_miss_not_a_panic() {
  let mut buf = [0u8; FOLD_CAP];
  let long = core::str::from_utf8(&[b'x'; FOLD_CAP + 1]).unwrap();
  assert_eq!(fold(long, &mut buf), None);
}

// --- `Case` / `lookup` — the axis every household's `FromStr` declares.
//
// The two `lookup` tests below are tier-agnostic (`Case` and `lookup`
// compile at every capability tier, same as `fold`); the fourcc-shaped
// proof further down needs `Other(Utf8Bytes)`, so it is gated the same way
// a real open household is.

/// [`Case::Insensitive`] is [`lookup`] doing exactly what every existing
/// household's call site did before this axis existed: fold, falling
/// back to the unfolded original past [`FOLD_CAP`]. This is the mode all
/// 22 households declare, so this parity is what "zero behaviour change"
/// rests on.
#[test]
fn lookup_insensitive_matches_fold_unwrap_or() {
  for input in ["BT709", "Bt709", "bt709", "Chroma-Derived-NC", ""] {
    let mut via_fold = [0u8; FOLD_CAP];
    let mut via_lookup = [0u8; FOLD_CAP];
    assert_eq!(
      lookup(Case::Insensitive, input, &mut via_lookup),
      fold(input, &mut via_fold).unwrap_or(input.as_bytes()),
      "lookup(Insensitive, {input:?}, _) diverged from fold(...).unwrap_or(...)"
    );
  }

  // Past `FOLD_CAP`, `fold` misses and the existing fallback — now
  // inside `lookup` itself — hands back the unfolded original. Still a
  // miss for any table (no canonical slug is this long), never a panic.
  let long = core::str::from_utf8(&[b'X'; FOLD_CAP + 1]).unwrap();
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(lookup(Case::Insensitive, long, &mut buf), long.as_bytes());
}

/// [`Case::Sensitive`] takes `s`'s own bytes, unfolded — no case healing,
/// and (unlike [`Case::Insensitive`]) no [`FOLD_CAP`] limit, since there
/// is nothing to fold into a bounded buffer.
#[test]
fn lookup_sensitive_takes_the_bytes_verbatim() {
  let mut a = [0u8; FOLD_CAP];
  let mut b = [0u8; FOLD_CAP];
  assert_eq!(lookup(Case::Sensitive, "AVC1", &mut a), b"AVC1");
  assert_eq!(lookup(Case::Sensitive, "avc1", &mut b), b"avc1");

  // The whole point of the axis: two spellings `Case::Insensitive` would
  // fold to the same key stay distinct under `Case::Sensitive`.
  let mut a = [0u8; FOLD_CAP];
  let mut b = [0u8; FOLD_CAP];
  assert_ne!(
    lookup(Case::Sensitive, "AVC1", &mut a),
    lookup(Case::Sensitive, "avc1", &mut b)
  );
}

/// `Case::Sensitive` never consults [`FOLD_CAP`] — an input far past it
/// still comes back whole, where [`Case::Insensitive`] (via [`fold`])
/// would already have fallen back to "too long to fold" territory.
#[test]
fn lookup_sensitive_has_no_fold_cap_limit() {
  let long_bytes = [b'x'; FOLD_CAP * 4];
  let long = core::str::from_utf8(&long_bytes).unwrap();
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(lookup(Case::Sensitive, long, &mut buf), long.as_bytes());
}

/// A test-only fourcc-shaped vocabulary, written in the exact shape a
/// real household takes (roster, `Other(Utf8Bytes)` escape, `other()`
/// delegating to `FromStr`) — proving `Case::Sensitive` end-to-end so the
/// first real sensitive household inherits machinery already proven,
/// rather than being the first thing to exercise it. No household
/// declares `Sensitive` yet (see the module doc's census), so this is
/// the axis's only current witness.
///
/// `Other(Utf8Bytes)` needs a heap, same as every real open household —
/// gated the same way.
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestFourCc {
  /// Canonical spelling `"AVC1"` — deliberately mixed-case, the way a
  /// real fourcc is. The property under test is that no *other* casing
  /// heals to this.
  Avc1,
  /// Canonical spelling `"H264"`.
  H264,
  /// The open escape: an unrecognised — or wrongly-cased — spelling,
  /// carried verbatim.
  Other(smol_bytes::Utf8Bytes),
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl TestFourCc {
  fn as_str(&self) -> &str {
    match self {
      Self::Avc1 => "AVC1",
      Self::H264 => "H264",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Mirrors every real household's `other()`: delegates to `FromStr`
  /// rather than a second table, so it inherits whichever `Case` this
  /// type declares with no extra wiring.
  fn other(slug: impl AsRef<str>) -> Self {
    <Self as core::str::FromStr>::from_str(slug.as_ref()).unwrap()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::str::FromStr for TestFourCc {
  type Err = core::convert::Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; FOLD_CAP];
    // The one word a real `Sensitive` household's `FromStr` would also
    // write here, in place of `Case::Insensitive` — everything else
    // about this call is identical to the 22 insensitive households.
    let key = lookup(Case::Sensitive, s, &mut buf);
    Ok(match key {
      b"AVC1" => Self::Avc1,
      b"H264" => Self::H264,
      _ => Self::Other(smol_bytes::Utf8Bytes::from(s)),
    })
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn sensitive_exact_case_hits_the_named_variant() {
  assert_eq!("AVC1".parse::<TestFourCc>().unwrap(), TestFourCc::Avc1);
  assert_eq!("H264".parse::<TestFourCc>().unwrap(), TestFourCc::H264);
}

/// The sealed law's central claim: under `Case::Sensitive`, a case
/// variant of a roster name is not healed, it is a table miss — the same
/// verbatim-stranger escape as any other unrecognised spelling. Every one
/// of these spellings would parse to `Avc1` or `H264` under
/// `Case::Insensitive` (see the mirrored households' own tests); here
/// each is its own `Other`.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn sensitive_case_variant_of_a_roster_name_is_a_table_miss_not_a_healed_hit() {
  for spelling in ["avc1", "Avc1", "aVC1", "AVc1", "h264"] {
    let v = spelling.parse::<TestFourCc>().unwrap();
    assert_eq!(
      v,
      TestFourCc::Other(smol_bytes::Utf8Bytes::from(spelling)),
      "{spelling:?} healed to a named variant under Case::Sensitive"
    );
    assert_eq!(v.as_str(), spelling);
  }
}

/// A genuine stranger's spelling survives the round trip unmodified —
/// the same verbatim-escape guarantee `Case::Insensitive` households
/// carry, unaffected by which `Case` the household declared.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn sensitive_stranger_round_trips_through_other_verbatim() {
  let v = "VeNdOr_Tag".parse::<TestFourCc>().unwrap();
  assert_eq!(
    v,
    TestFourCc::Other(smol_bytes::Utf8Bytes::from("VeNdOr_Tag"))
  );
  assert_eq!(v.as_str(), "VeNdOr_Tag");
  assert_eq!(v.as_str().parse::<TestFourCc>().unwrap(), v);
}

/// `other()` delegates to `FromStr` (mirroring every real household
/// since mediaframe#64), so it inherits `Case::Sensitive` with no second
/// opinion of its own: an exact spelling still resolves to the named
/// variant, and a case variant is still a stranger, through the escape
/// constructor exactly as through `FromStr` directly.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn sensitive_other_inherits_the_declared_case_with_no_extra_wiring() {
  assert_eq!(TestFourCc::other("AVC1"), TestFourCc::Avc1);
  assert_eq!(
    TestFourCc::other("avc1"),
    TestFourCc::Other(smol_bytes::Utf8Bytes::from("avc1"))
  );
}
