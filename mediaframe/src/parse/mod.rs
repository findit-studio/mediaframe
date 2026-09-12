//! The case-sensitivity axis every vocabulary parse table declares, and
//! the two matching strategies that axis chooses between.
//!
//! # The law
//!
//! Case-sensitivity is a **per-household constitutional attribute**, not
//! a crate-wide default. Every vocabulary's [`FromStr`](core::str::FromStr)
//! names its [`Case`] explicitly, as the first argument to the one call
//! that resolves its match key ([`lookup`]) — right beside the `match`
//! table that key feeds. There is no [`Default`] impl for [`Case`], so a
//! household cannot omit the choice; leaving it out is a compile error
//! (a missing function argument), not a silently-assumed mode. Both
//! strategies are **one implementation each** ([`fold`] for
//! [`Case::Insensitive`]; the verbatim pass-through in [`lookup`] for
//! [`Case::Sensitive`]) that every household shares through [`lookup`] —
//! a `FromStr` body never reimplements either, it only names which one
//! applies.
//!
//! - [`Case::Insensitive`]: the roster is matched through [`fold`] —
//!   ASCII case-folded, so `"BT709"`, `"Bt709"` and `"bt709"` are one
//!   value, the same **named** variant. Only a genuine stranger — a
//!   spelling the folded lookup still misses — reaches the
//!   `Other(Utf8Bytes)` escape, and it carries the caller's spelling
//!   **verbatim**: folding a name nobody claims would destroy
//!   information (vendor fourccs and codec tags are routinely
//!   case-sensitive) for no compensating benefit, now that the lookup
//!   already catches every case-variant of a name this crate *does*
//!   recognise. **Every household in this crate is `Insensitive` today**
//!   — all 22 are lowercase-slug domains with no distinctly-cased roster
//!   member, so this axis lands with zero behaviour change; it exists so
//!   the choice is provable rather than assumed.
//! - [`Case::Sensitive`]: the roster is matched on **exact bytes**, no
//!   folding, no [`FOLD_CAP`] length cap (there is nothing to fold, so
//!   nothing to fit in a fold buffer — an exact comparison either equals
//!   a slug or it does not, at any length). A case variant of a roster
//!   name is not a healed hit here, it is an ordinary table miss — the
//!   same verbatim-stranger escape as any other unrecognised spelling.
//!   Right for a **fourcc-shaped** domain (vendor codec tags, container
//!   FourCCs, and similar wire-vocabulary alphabets that are
//!   case-sensitive **by convention**, where two different casings can
//!   legitimately name two different real-world values): folding one
//!   spelling onto another there would silently substitute a different,
//!   *wrong* roster member for the caller's actual value —
//!   error-beats-wrong-value. No household needs this yet; [`lookup`]'s
//!   own tests pin the strategy end-to-end against a test-only table (see
//!   `parse::tests`), so the first real `Sensitive` household inherits
//!   machinery already proven, rather than being the first thing to
//!   exercise it.
//!
//! `Self::other` (every open household's escape constructor) already
//! runs its own [`FromStr`] lookup rather than a second table — so it
//! inherits whichever [`Case`] the household declared with no extra
//! wiring, exactly as it inherits the rest of the match table. One axis,
//! named once per household, however the value is reached.
//!
//! Folding — where [`Case::Insensitive`] chooses it — is deliberately
//! **ASCII-only**. These are FFmpeg / H.273 / file extension identifiers;
//! Unicode case folding is locale-sensitive (Turkish dotless i maps `I`
//! to `ı`, not `i`) in ways a wire vocabulary must not be, and would make
//! the canonical form depend on who is reading.
//!
//! The lookup itself is on the **byte side**: [`lookup`] yields the
//! resolved bytes and every table arm is a `b"slug"` literal (spelled in
//! whatever case that household's own [`Case`] compares against — the
//! canonical lowercase slug for every `Insensitive` household today). A
//! `FromStr` added here writes its table that way.
//!
//! The errors these parses return live with the vocabularies themselves,
//! one per type — a `Rational` that is malformed and a `Matrix` that names
//! nothing are different failures, and the type is what says which.

/// Capacity of the stack buffer [`fold`] folds into.
///
/// The longest canonical slug in the crate is well under this; the buffer
/// exists because the coded vocabularies are available at the crate's
/// no-alloc tier, where there is no heap to fold into. An input that does
/// not fit cannot name a variant either, so the caller treats the
/// overflow as an ordinary miss.
///
/// Only [`Case::Insensitive`] folds, so only that strategy is subject to
/// this cap — [`Case::Sensitive`] compares the input's own bytes and has
/// no length limit. Every [`lookup`] call site still allocates a buffer
/// of this size regardless of which [`Case`] it declares, so the two
/// strategies stay one declaration word apart at the call site; the
/// buffer a `Sensitive` lookup allocates but never touches is a 64-byte
/// stack value, not a cost worth a divergent call shape.
pub(crate) const FOLD_CAP: usize = 64;

/// ASCII-fold `s` into `buf`, returning the lowercase bytes, or [`None`]
/// when `s` is longer than any slug can be.
///
/// The lookup tables compare on the byte side — every `FromStr` in the
/// crate matches `b"slug"` literals — so the fold hands back the buffer
/// itself and never re-derives a `str` from it. That conversion would be
/// a second O(n) pass over bytes this function just wrote, to prove a
/// property no caller consumes.
///
/// The [`Case::Insensitive`] half of [`lookup`] — a `FromStr` body never
/// calls this directly; it declares its [`Case`] and lets [`lookup`]
/// choose. Allocation-free, so the lookup gate is the same at every
/// capability tier.
pub(crate) fn fold<'b>(s: &str, buf: &'b mut [u8; FOLD_CAP]) -> Option<&'b [u8]> {
  let bytes = s.as_bytes();
  let n = bytes.len();
  if n > FOLD_CAP {
    return None;
  }
  buf[..n].copy_from_slice(bytes);
  buf[..n].make_ascii_lowercase();
  Some(&buf[..n])
}

/// The two case-matching strategies a household's parse table can
/// declare — the sealed per-household axis; see the module doc for the
/// full law.
///
/// Deliberately has **no [`Default`] impl**. That absence is the
/// enforcement mechanism: [`lookup`] takes a `Case` as a required
/// argument, so a household that does not name one does not compile,
/// rather than silently inheriting a crate-wide assumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Case {
  /// Match through [`fold`] — any-case spelling of a roster name resolves
  /// to that named variant.
  Insensitive,
  /// Match on exact bytes, unfolded — a case variant of a roster name is
  /// itself a table miss, indistinguishable from a spelling the roster
  /// never had.
  ///
  /// No household declares this yet (`Case`'s own module doc has the
  /// census), so a non-test build never constructs it — `parse::tests`
  /// is what does, against a test-only table, and `#[cfg(test)]` code
  /// does not count for a plain build's dead-code analysis.
  #[allow(dead_code)]
  Sensitive,
}

/// Resolve `s` to the byte key a household's `b"slug"` match table
/// compares against, under `case`'s strategy — the one call every
/// `FromStr` in this crate makes to reach its table, and the household's
/// declaration site for [`Case`].
///
/// [`Case::Insensitive`] folds through [`fold`], falling back to the
/// unfolded original past [`FOLD_CAP`] (an input that long cannot name a
/// variant either, so the fallback is still an ordinary miss, not a
/// panic). [`Case::Sensitive`] takes `s`'s own bytes verbatim: there is
/// nothing to fold, so `buf` goes unused on that path — every call site
/// still supplies one, so naming the [`Case`] is the *only* thing that
/// differs between a `FromStr` that folds and one that does not.
pub(crate) fn lookup<'b>(case: Case, s: &'b str, buf: &'b mut [u8; FOLD_CAP]) -> &'b [u8] {
  match case {
    Case::Insensitive => fold(s, buf).unwrap_or(s.as_bytes()),
    Case::Sensitive => s.as_bytes(),
  }
}

#[cfg(test)]
mod tests;
