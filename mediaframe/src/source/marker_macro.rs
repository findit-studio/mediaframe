// The `marker!` macro is `#[macro_export]`-exported at the crate root.
// When the crate is compiled with no per-format features enabled, the
// definition is "unused" from the local crate's perspective even though
// downstream consumers (and our own walker_macro) reference it. Silence
// the local-only warning.
#![allow(unused_macros)]

//! `marker!` — generates a zero-sized source-format marker type with
//! the canonical shape used throughout `mediaframe::source`.
//!
//! The macro emits four items:
//! 1. A `pub struct $name(());` (or `$name<const BE: bool = false>(());`
//!    for endian-aware markers). The single `()` field is private so
//!    callers cannot construct the marker via direct literal — they
//!    must use the constructor below. This forward-compats the marker
//!    shape: if a future variant needs internal config, fields can be
//!    added without breaking the public construction API.
//! 2. `impl $name { pub const fn new() -> Self }` — the only way to
//!    obtain an instance from outside the defining module. ZST, so
//!    the constructor is zero-cost and `const`.
//! 3. `impl crate::source::sealed::Sealed for $name {}` — seals the
//!    marker against external [`crate::SourceFormat`] implementors.
//! 4. `impl crate::SourceFormat for $name {}` — opts the marker into
//!    the sealed format-tag trait.
//!
//! # Forms
//!
//! ```ignore
//! marker! {
//!   /// Zero-sized marker for the FOO source format.
//!   struct Foo;
//! }
//!
//! marker! {
//!   /// Zero-sized marker for the FOO source format,
//!   /// parameterized over the endianness of its pixel data.
//!   struct Foo<const BE: bool = false>;
//! }
//!
//! marker! {
//!   /// Zero-sized marker for a high-bit-depth source format
//!   /// parameterized over the active bit depth.
//!   struct Foo<const BITS: u32 = 16>;
//! }
//!
//! marker! {
//!   /// Zero-sized marker for a high-bit-depth source format
//!   /// parameterized over both the active bit depth and the
//!   /// byte order of its pixel data.
//!   struct Foo<const BITS: u32, const BE: bool = false>;
//! }
//! ```

/// Generates the canonical marker quartet (`struct` + `new()` +
/// `Sealed` + `SourceFormat`) for a source-format marker type.
///
/// Four forms are supported:
/// - Bare: `struct Foo;`
/// - Endian-aware: `struct Foo<const BE: bool = false>;`
/// - Arbitrary const-generic: `struct Foo<const BITS: u32 = 16>;`
/// - Bit-depth + endian: `struct Foo<const BITS: u32, const BE: bool = false>;`
///
/// See [module-level docs](crate::source) for the conventions and
/// rationale behind the `(())`-field + `pub const fn new()` shape.
#[macro_export]
macro_rules! marker {
  // Bare unit-style marker (no endian generic).
  (
    $(#[$attr:meta])*
    struct $name:ident;
  ) => {
    $(#[$attr])*
    pub struct $name(());

    impl $name {
      #[allow(clippy::new_without_default)]
      /// Constructs the marker. Zero-cost — this is a ZST.
      #[inline]
      pub const fn new() -> Self {
        Self(())
      }
    }

    impl $crate::source::sealed::Sealed for $name {}
    impl $crate::SourceFormat for $name {}
  };

  // Endian-aware marker — const-generic over `BE`. The `= false`
  // default makes `$name` an alias for the little-endian variant
  // (back-compat with pre-Phase-4 callers). Default value is a
  // literal (`false`) — `$literal` fragment matches there because
  // `expr` cannot legally be followed by `>` in macro rules.
  (
    $(#[$attr:meta])*
    struct $name:ident<const BE: bool $(= $default:literal)?>;
  ) => {
    $(#[$attr])*
    pub struct $name<const BE: bool $(= $default)?>(());

    impl<const BE: bool> $name<BE> {
      #[allow(clippy::new_without_default)]
      /// Constructs the marker. Zero-cost — this is a ZST.
      #[inline]
      pub const fn new() -> Self {
        Self(())
      }
    }

    impl<const BE: bool> $crate::source::sealed::Sealed for $name<BE> {}
    impl<const BE: bool> $crate::SourceFormat for $name<BE> {}
  };

  // Arbitrary const-generic marker — `<const NAME: TYPE>` or
  // `<const NAME: TYPE = DEFAULT>`. Used for markers parameterised
  // over non-bool types, e.g. `Bayer16<const BITS: u32 = 16>`.
  //
  // Default value is a literal (`$default:literal`) — `expr` cannot
  // legally follow `>` in macro rules, so we accept a `literal` here,
  // which covers all primitive-const defaults (`8u32`, `16`, `false`,
  // etc.).
  (
    $(#[$attr:meta])*
    struct $name:ident<const $param:ident: $ty:ty $(= $default:literal)?>;
  ) => {
    $(#[$attr])*
    pub struct $name<const $param: $ty $(= $default)?>(());

    impl<const $param: $ty> $name<$param> {
      #[allow(clippy::new_without_default)]
      /// Constructs the marker. Zero-cost — this is a ZST.
      #[inline]
      pub const fn new() -> Self {
        Self(())
      }
    }

    impl<const $param: $ty> $crate::source::sealed::Sealed for $name<$param> {}
    impl<const $param: $ty> $crate::SourceFormat for $name<$param> {}
  };

  // Bit-depth + endian marker — `<const BITS: u32, const BE: bool = false>`.
  // Used by markers that are both bit-depth- and byte-order-aware, e.g.
  // the high-bit-depth Bayer family (`Bayer16<const BITS: u32, const BE>`).
  // The `BE = false` default keeps `$name<BITS>` an alias for the
  // little-endian variant (back-compat with single-generic callers).
  //
  // Only `BE` carries a default; the leading `BITS` parameter is always
  // explicit at the call sites that use this arm (every Bayer alias pins
  // `BITS`). Default value is a literal for the same `expr`-can't-precede-
  // `>` reason as the other arms.
  (
    $(#[$attr:meta])*
    struct $name:ident<const $bparam:ident: $bty:ty, const $eparam:ident: bool $(= $edefault:literal)?>;
  ) => {
    $(#[$attr])*
    pub struct $name<const $bparam: $bty, const $eparam: bool $(= $edefault)?>(());

    impl<const $bparam: $bty, const $eparam: bool> $name<$bparam, $eparam> {
      #[allow(clippy::new_without_default)]
      /// Constructs the marker. Zero-cost — this is a ZST.
      #[inline]
      pub const fn new() -> Self {
        Self(())
      }
    }

    impl<const $bparam: $bty, const $eparam: bool> $crate::source::sealed::Sealed
      for $name<$bparam, $eparam> {}
    impl<const $bparam: $bty, const $eparam: bool> $crate::SourceFormat
      for $name<$bparam, $eparam> {}
  };
}
