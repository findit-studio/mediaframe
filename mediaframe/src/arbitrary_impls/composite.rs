// Cluster C — audio composite metadata, capture, language.
//
// Owned types (verify each — these all have public constructors / `try_new`,
// some validated):
//   AUDIO COMPOSITE (via public constructor / builders):
//     - audio::Loudness          (new(f32, f32, f32, f32))
//     - audio::Fingerprint       (try_new(algo, value) — algo non-empty)
//     - audio::CoverArt          (try_new(mime, data) — both non-empty)
//     - audio::Tags              (new() + builder setters; representative subset OK)
//   CAPTURE:
//     - capture::Device          (new() + .with_make(..).with_model(..) etc.)
//     - capture::GeoLocation     (try_new(lat, lon, altitude) — ranges)
//   LANGUAGE:
//     - lang::Language           (from_bcp47(<curated tag>))
//
// VALIDATION STRATEGY for `try_new` types: construct VALID inputs (in-range
// floats via `u.int_in_range`, non-empty strings with a fallback like "x"),
// then `.unwrap()` — never feed attacker-controlled values into a fallible
// constructor + `.unwrap()`.
