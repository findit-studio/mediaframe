//! mediaframe xtask — dev-only automation.
//!
//! Subcommands:
//! - `cargo xtask sync` — fetches FFmpeg's `libavutil/pixfmt.h` at the
//!   pinned release tag and regenerates **both** vendored files
//!   deterministically:
//!   - `xtask/vendor/ffmpeg-pixfmts.txt` — the lowercase
//!     `AV_PIX_FMT_<NAME>` slug list.
//!   - `xtask/vendor/ffmpeg-color.txt` — the five colour enums
//!     (`AVColorPrimaries`, `AVColorTransferCharacteristic`,
//!     `AVColorSpace`, `AVColorRange`, `AVChromaLocation`) as
//!     `<ENUM> <NAME> <VALUE>` lines (C-counter rules; `*_NB`
//!     sentinels and the post-NB custom extensions skipped; aliases
//!     collapsed to one entry per distinct code).
//!
//!   Requires `curl` on `PATH`. Re-running on an unchanged FFMPEG_TAG
//!   reproduces byte-identical files (the `Fetched:` date is the only
//!   volatile line) so the working tree stays clean.
//! - `cargo xtask check` — verifies mediaframe against both vendored
//!   files:
//!   - `PixelFormat`: reads the `as_str()` match in
//!     `src/pixel_format/mod.rs` and diffs slugs —
//!     **missing-from-mediaframe** (FFmpeg has it, we don't) fails CI;
//!     **mediaframe extras** (cinema-RAW etc.) are informational.
//!   - Colour enums: reads the `to_u32()` matches in `src/color/mod.rs`
//!     and asserts every distinct FFmpeg colour code has a named
//!     mediaframe variant mapping to that value (and the covering
//!     variant's id is `< DOMAIN_EXT_BASE` — the FFmpeg ingest path
//!     never yields a mediaframe-domain variant). A missing code
//!     fails CI. mediaframe-domain variants (id `>= DOMAIN_EXT_BASE`,
//!     e.g. `Matrix::Bt601`, which H.273 / FFmpeg does not
//!     enumerate) are exempt from FFmpeg coverage and additionally
//!     asserted disjoint from the vendored FFmpeg colour codes.
//!
//! - `cargo xtask gen-lang` — regenerates
//!   `mediaframe/src/lang/registry/table.rs` from the two vendored BCP 47
//!   registries (`xtask/vendor/language-subtag-registry.txt` and
//!   `xtask/vendor/iso639-2.txt`). `cargo xtask check` renders the same
//!   text in memory and diffs it byte for byte, and `cargo xtask sync`
//!   re-fetches both files from their authorities. See [`lang`], where the
//!   two-file argument is made in full.
//!
//! The FFmpeg vendored files are plain text (not the FFmpeg header
//! verbatim), which sidesteps the LGPL question that would apply to
//! checking in `pixfmt.h` itself. The two BCP 47 registries are vendored
//! verbatim — both are published as public-domain data by their
//! registrars.

use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::{Path, PathBuf},
  process::{Command, ExitCode},
};

/// FFmpeg release tag pinned for the check. Bump deliberately when you
/// want to sync against a newer FFmpeg.
const FFMPEG_TAG: &str = "n9.0";

/// Path (relative to the mediaframe workspace root) of the vendored
/// slug list.
const VENDOR_PATH: &str = "xtask/vendor/ffmpeg-pixfmts.txt";

/// Path (relative to the workspace root) of the vendored colour-enum
/// table (`<ENUM> <FFMPEG_NAME> <VALUE>` per line).
const COLOR_VENDOR_PATH: &str = "xtask/vendor/ffmpeg-color.txt";

/// Path (relative to the workspace root) of the PixelFormat source
/// file whose `as_str()` table is the source of truth for our slugs.
const PIXEL_FORMAT_RS: &str = "mediaframe/src/pixel_format/mod.rs";

/// Path (relative to the workspace root) of the colour-enum source
/// file whose `to_u32()` matches are the source of truth.
const COLOR_RS: &str = "mediaframe/src/color/mod.rs";

/// Path (relative to the workspace root) of the vendored codec-name
/// table. Format: one `<media_type> <name> [<props>]` per line, sorted;
/// `<props>` is an optional comma-separated list of `AV_CODEC_PROP_*`
/// tokens (prefix stripped) and is omitted entirely when FFmpeg's
/// `codec_desc.c` has no `.props` initializer for the codec. See the
/// header inside the file itself for the canonical format.
const CODEC_VENDOR_PATH: &str = "xtask/vendor/ffmpeg-codecs.txt";

/// Path (relative to the workspace root) of the codec-enum source file
/// whose `as_str()` matches are the source of truth.
const CODEC_RS: &str = "mediaframe/src/codec/mod.rs";

/// Path (relative to the workspace root) of the codec module's external
/// test file. The crate keeps unit tests in a sibling `tests.rs`, so the
/// generator emits two files and `check` verifies both.
const CODEC_TESTS_RS: &str = "mediaframe/src/codec/tests.rs";

/// The BCP 47 language registries — the fourth vendored authority, and the
/// only one that generates a whole Rust module rather than checking a
/// hand-written one. Its vendored files, generator and freshness diff all
/// live in [`lang`], which `check` / `sync` / `gen-lang` call into.
mod lang;

/// The mediaframe codec enums that are vendored from
/// `xtask/vendor/ffmpeg-codecs.txt`, paired with their FFmpeg media type
/// (`AVMEDIA_TYPE_*`, lowercased). Drives `check_codec`'s two-way
/// vendor-table sync (every FFmpeg name has a variant, every variant
/// names a real FFmpeg codec) and the corresponding slice of
/// `build_codec_module` / `build_codec_tests`'s output.
///
/// [`AttachmentCodec`] is deliberately **not** here — see
/// [`ATTACHMENT_CODECS`] for why it has no vendored table to sync
/// against. It still participates in generation (`build_codec_module`,
/// `build_codec_tests`) and in the freshness diff (`check_codec`'s
/// Stage 2 rebuilds and byte-compares the *whole* file), just not in
/// this list's per-media-type vendor coverage loop.
const CODEC_ENUMS: &[(&str, &str)] = &[
  ("video", "VideoCodec"),
  ("audio", "AudioCodec"),
  ("subtitle", "SubtitleCodec"),
  ("data", "DataCodec"),
];

/// Every codec-enum type name the generator emits, `CODEC_ENUMS`'s four
/// vendor-backed names plus `AttachmentCodec`. Used where code needs the
/// full set of `impl <Enum> { pub fn as_str ... }` blocks to parse
/// (`parse_codec_named_strings`) rather than just the vendor-checked
/// subset.
const ALL_CODEC_ENUM_NAMES: &[&str] = &[
  "VideoCodec",
  "AudioCodec",
  "SubtitleCodec",
  "DataCodec",
  "AttachmentCodec",
];

/// `AttachmentCodec`'s roster: FFmpeg codec short names actually
/// assigned to an `AVMEDIA_TYPE_ATTACHMENT` stream, sorted.
///
/// **Not** vendored from `codec_desc.c` like [`CODEC_ENUMS`]'s four
/// media types. `libavcodec/codec_desc.c` at FFmpeg n9.0 carries
/// **zero** descriptors with `.type = AVMEDIA_TYPE_ATTACHMENT` — checked
/// directly (`grep AVMEDIA_TYPE_ATTACHMENT libavcodec/codec_desc.c`
/// against both the pinned n9.0 source and a fresh FFmpeg `master`
/// checkout returns nothing, either version). The `codec_id.h` comment
/// `/* other specific kind of codecs (generally used for attachments) */`
/// that precedes `AV_CODEC_ID_TTF` and its neighbours describes their
/// historical grouping, not an `AVMEDIA_TYPE_ATTACHMENT` classification:
/// `codec_desc.c` actually types `ttf` / `scte_35` / `epg` / `otf` /
/// `klv` / `dvd_nav_packet` / `timed_id3` / `bin_data` / `smpte_2038` /
/// `smpte_436m_anc` as `AVMEDIA_TYPE_DATA` (see `DataCodec`, which
/// carries all of them) and `bintext` / `xbin` / `idf` as
/// `AVMEDIA_TYPE_VIDEO`.
///
/// The one place FFmpeg itself assigns a codec id to an
/// `AVMEDIA_TYPE_ATTACHMENT` stream is `libavformat/matroskadec.c`'s
/// `mkv_mime_tags` table (the Matroska/WebM demuxer is also the only
/// FFmpeg demuxer that ever sets `codecpar->codec_type =
/// AVMEDIA_TYPE_ATTACHMENT` with a specific, non-`NONE` codec id — APE
/// tag binary attachments (`libavformat/apetag.c`) take the same stream
/// role but leave `codec_id` at `AV_CODEC_ID_NONE`):
///
/// ```c
/// static const CodecMime mkv_mime_tags[] = {
///     {"application/x-truetype-font", AV_CODEC_ID_TTF},
///     {"application/x-font"         , AV_CODEC_ID_TTF},
///     {"application/vnd.ms-opentype", AV_CODEC_ID_OTF},
///     {"binary"                     , AV_CODEC_ID_BIN_DATA},
///     {""                           , AV_CODEC_ID_NONE}
/// };
/// ```
///
/// Three distinct codec ids result: `ttf`, `otf`, `bin_data` — this
/// list transcribes their `codec_desc.c` short names verbatim (the two
/// MIME types that both target `AV_CODEC_ID_TTF` collapse to one entry).
/// Every one of the three also has a same-named, same-string `DataCodec`
/// variant: it is genuinely the same FFmpeg codec id wearing two
/// different track-role hats, not a coincidence or a bug to dedupe.
///
/// Bumping `FFMPEG_TAG` does not regenerate this list — there is no
/// `cargo xtask sync` step that reaches `matroskadec.c`. Re-derive it
/// by hand (repeat the `codec_desc.c` + `matroskadec.c` census in this
/// doc comment) on a deliberate FFmpeg version bump.
const ATTACHMENT_CODECS: &[&str] = &["bin_data", "otf", "ttf"];

/// The five FFmpeg colour C enums to parse, paired with the
/// `AVCOL_*` / `AVCHROMA_*` prefix to strip and the mediaframe
/// enum name whose `to_u32()` match maps it.
const COLOR_ENUMS: &[(&str, &str, &str)] = &[
  ("AVColorPrimaries", "AVCOL_PRI_", "Primaries"),
  ("AVColorTransferCharacteristic", "AVCOL_TRC_", "Transfer"),
  ("AVColorSpace", "AVCOL_SPC_", "Matrix"),
  ("AVColorRange", "AVCOL_RANGE_", "DynamicRange"),
  ("AVChromaLocation", "AVCHROMA_LOC_", "ChromaLocation"),
];

fn main() -> ExitCode {
  let cmd = std::env::args()
    .nth(1)
    .unwrap_or_else(|| "help".to_string());
  match cmd.as_str() {
    "check" | "check-pixel-format" | "check-codec" | "check-lang" => check(),
    "sync" | "sync-pixel-format" | "sync-codec" | "sync-lang" => sync(),
    "gen-codec" => gen_codec(),
    "gen-lang" => gen_lang(),
    "help" | "--help" | "-h" => {
      print_help();
      ExitCode::SUCCESS
    }
    other => {
      eprintln!("unknown subcommand: {other}");
      print_help();
      ExitCode::FAILURE
    }
  }
}

fn print_help() {
  eprintln!(
    "mediaframe xtask\n\n\
         Subcommands:\n  \
         check    Verify mediaframe against the vendored tables:\n           \
                    - PixelFormat slugs ({VENDOR_PATH})\n           \
                    - Colour-enum codes ({COLOR_VENDOR_PATH})\n           \
                    - Codec short names ({CODEC_VENDOR_PATH}) plus the\n           \
                      hand-curated AttachmentCodec roster (ATTACHMENT_CODECS)\n           \
                    - The BCP 47 language table ({lang_table})\n  \
         sync       Fetch FFmpeg {FFMPEG_TAG} (pixfmt.h + codec_desc.c) and the\n             \
                    two BCP 47 registries, and regenerate the vendored files\n             \
                    deterministically\n  \
         gen-codec  Regenerate mediaframe/src/codec/mod.rs from the vendored\n             \
                    table ({CODEC_VENDOR_PATH}) plus ATTACHMENT_CODECS, via\n             \
                    quote + prettyplease\n  \
         gen-lang   Regenerate {lang_table} from the two\n             \
                    vendored BCP 47 registries\n  \
         help       Show this help\n",
    lang_table = lang::TABLE_RS
  );
}

/// Repo root = workspace manifest dir's parent (xtask is a child member).
fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .map(Path::to_path_buf)
    .unwrap_or_else(|| PathBuf::from("."))
}

// ---------- check ----------------------------------------------------------

/// Runs the pixel-format check and the colour-enum check; the overall
/// status fails if either fails (both always run so a single
/// invocation reports every gap).
fn check() -> ExitCode {
  let root = workspace_root();
  let pf_ok = check_pixfmt(&root);
  println!();
  let color_ok = check_color(&root);
  println!();
  let codec_ok = check_codec(&root);
  println!();
  let lang_ok = lang::check(&root);
  if pf_ok && color_ok && codec_ok && lang_ok {
    ExitCode::SUCCESS
  } else {
    ExitCode::FAILURE
  }
}

/// `PixelFormat` slug coverage vs. `xtask/vendor/ffmpeg-pixfmts.txt`.
fn check_pixfmt(root: &Path) -> bool {
  let vendor = match fs::read_to_string(root.join(VENDOR_PATH)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {VENDOR_PATH}: {e}");
      eprintln!("hint:  run `cargo xtask sync` first to populate the vendored list");
      return false;
    }
  };
  let pf_rs = match fs::read_to_string(root.join(PIXEL_FORMAT_RS)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {PIXEL_FORMAT_RS}: {e}");
      return false;
    }
  };

  let ffmpeg = parse_vendored(&vendor);
  let mediaframe = parse_as_str_slugs(&pf_rs);

  let missing_from_mediaframe: BTreeSet<_> = ffmpeg.difference(&mediaframe).cloned().collect();
  let extras_in_mediaframe: BTreeSet<_> = mediaframe.difference(&ffmpeg).cloned().collect();

  println!("FFmpeg pinned: {FFMPEG_TAG}");
  println!("FFmpeg slugs  : {}", ffmpeg.len());
  println!("mediaframe    : {} known slugs", mediaframe.len());
  println!();

  if !extras_in_mediaframe.is_empty() {
    println!(
      "  mediaframe extras (slugs not in FFmpeg {FFMPEG_TAG} — cinema-RAW etc.): {}",
      extras_in_mediaframe.len()
    );
    for s in &extras_in_mediaframe {
      println!("    {s}");
    }
    println!();
  }

  if missing_from_mediaframe.is_empty() {
    println!("OK: every FFmpeg {FFMPEG_TAG} pixel format is covered by mediaframe.");
    true
  } else {
    eprintln!(
      "FAIL: {} FFmpeg pixel format(s) missing from mediaframe::PixelFormat:",
      missing_from_mediaframe.len()
    );
    for s in &missing_from_mediaframe {
      eprintln!("    {s}");
    }
    eprintln!(
      "\nAction: add the missing variants to `PixelFormat`,\n  \
             extend `as_str()` and the `to_u32`/`from_u32` tables."
    );
    false
  }
}

/// Colour-enum coverage: every distinct FFmpeg colour code in
/// `xtask/vendor/ffmpeg-color.txt` must have a named mediaframe
/// variant whose `to_u32()` returns that value (and a non-empty
/// `as_str()`), parsed from `src/color/mod.rs`. The reverse direction
/// (mediaframe `Unknown(n)`) is intentionally NOT asserted.
fn check_color(root: &Path) -> bool {
  let vendor = match fs::read_to_string(root.join(COLOR_VENDOR_PATH)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {COLOR_VENDOR_PATH}: {e}");
      eprintln!("hint:  run `cargo xtask sync` first to populate the vendored list");
      return false;
    }
  };
  let color_rs = match fs::read_to_string(root.join(COLOR_RS)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {COLOR_RS}: {e}");
      return false;
    }
  };

  // mediaframe-domain colour-id base (ids `>=` this have no H.273
  // code and are never produced by the FFmpeg ingest path).
  let domain_base = match parse_domain_ext_base(&color_rs) {
    Some(b) => b,
    None => {
      eprintln!(
        "error: cannot parse `pub const DOMAIN_EXT_BASE: u32 = ...;` \
                 from {COLOR_RS} — the colour domain-extension check needs it."
      );
      return false;
    }
  };

  // FFmpeg side: ENUM -> { distinct code -> first FFmpeg name }.
  let ffmpeg = parse_color_vendored(&vendor);
  // mediaframe side: ENUM -> { variant-ident -> (value, has_slug) }.
  let mediaframe = parse_color_named_codes(&color_rs, domain_base);

  let mut ok = true;
  let mut total_codes = 0usize;
  for (_c_enum, _prefix, vf_enum) in COLOR_ENUMS {
    let ff_codes = match ffmpeg.get(*vf_enum) {
      Some(m) => m,
      None => {
        eprintln!(
          "FAIL: no vendored FFmpeg entries for {vf_enum} — \
                   regenerate {COLOR_VENDOR_PATH} via `cargo xtask sync`."
        );
        ok = false;
        continue;
      }
    };
    let empty = BTreeMap::new();
    let vf_named = mediaframe.get(*vf_enum).unwrap_or(&empty);
    for (code, ff_name) in ff_codes {
      // FFmpeg `RESERVED*` codes (e.g. AVCOL_*_RESERVED0 = 0,
      // AVCOL_*_RESERVED = 3) are intentionally NOT named in
      // mediaframe — they fall to `Unknown(n)` losslessly. Skip
      // them; they are kept in the vendored file only for header
      // fidelity. (`RGB`/`UNSPECIFIED`/etc. are NOT reserved.)
      if ff_name.starts_with("RESERVED") {
        continue;
      }
      total_codes += 1;
      // No FFmpeg/H.273 code may itself land in the mediaframe
      // domain-extension band — that band is reserved for concepts
      // FFmpeg does NOT enumerate.
      if *code >= domain_base {
        eprintln!(
          "FAIL: FFmpeg color code {vf_enum} = {code} (FFmpeg \
                   {ff_name}) collides with the mediaframe domain band \
                   (>= DOMAIN_EXT_BASE = {domain_base})."
        );
        ok = false;
      }
      // A code is covered iff some NAMED variant's `to_u32()` maps
      // to it (this mirrors `from_u32(code)` landing on a non-Unknown
      // variant whose `to_u32()` round-trips to `code`). That covering
      // variant's id must be `< DOMAIN_EXT_BASE` — the FFmpeg ingest
      // path never yields a domain variant.
      match vf_named.values().find(|nc| nc.value == *code) {
        None => {
          eprintln!(
            "FAIL: missing FFmpeg color code {vf_enum} = {code} \
                     (FFmpeg {ff_name}) — extend the enum + \
                     to_u32/from_u32 so a named variant maps to {code}."
          );
          ok = false;
        }
        Some(nc) if !nc.has_slug => {
          eprintln!(
            "FAIL: {vf_enum} variant for FFmpeg code {code} \
                     ({ff_name}) has an empty as_str() slug."
          );
          ok = false;
        }
        Some(nc) if nc.value >= domain_base => {
          eprintln!(
            "FAIL: {vf_enum} variant covering FFmpeg code {code} \
                     ({ff_name}) maps to a domain id {} (>= \
                     DOMAIN_EXT_BASE = {domain_base}) — the FFmpeg \
                     path must never yield a domain variant.",
            nc.value
          );
          ok = false;
        }
        Some(_) => {}
      }
    }
  }

  // Domain invariant (b): `Matrix::Bt601` is a mediaframe-domain
  // concept — its id must be `>= DOMAIN_EXT_BASE` AND absent from the
  // vendored FFmpeg colour table (no domain/FFmpeg collision).
  let empty = BTreeMap::new();
  let cm_named = mediaframe.get("Matrix").unwrap_or(&empty);
  match cm_named.get("Bt601") {
    None => {
      eprintln!(
        "FAIL: Matrix::Bt601 not found in {COLOR_RS} to_u32() — \
                 it is a required mediaframe-domain variant."
      );
      ok = false;
    }
    Some(nc) => {
      if nc.value < domain_base {
        eprintln!(
          "FAIL: Matrix::Bt601.to_u32() = {} must be >= \
                   DOMAIN_EXT_BASE ({domain_base}) — it is a \
                   mediaframe-domain concept, not an FFmpeg code.",
          nc.value
        );
        ok = false;
      }
      let cm_ff = ffmpeg.get("Matrix").cloned().unwrap_or_default();
      if cm_ff.contains_key(&nc.value) {
        eprintln!(
          "FAIL: Matrix::Bt601 id {} collides with a vendored \
                   FFmpeg color-matrix code — domain ids must be disjoint.",
          nc.value
        );
        ok = false;
      }
    }
  }

  if ok {
    println!(
      "OK: every FFmpeg {FFMPEG_TAG} color code ({total_codes} across \
             {} enums) is covered by mediaframe; mediaframe-domain \
             variants (id >= DOMAIN_EXT_BASE = {domain_base}, e.g. \
             Matrix::Bt601) are exempt from FFmpeg coverage and \
             verified disjoint.",
      COLOR_ENUMS.len()
    );
  }
  ok
}

/// Codec coverage — **two-way** sync plus a generation-freshness diff.
///
/// 1. **mediaframe → FFmpeg** (every named variant's canonical string
///    exists in the vendored table) — fails on a typo'd `as_str()`
///    slug.
/// 2. **FFmpeg → mediaframe** (every vendored short name has a
///    matching named variant on the corresponding enum) — fails when
///    a `cargo xtask sync` added codecs without re-running
///    `cargo xtask gen-codec` (the all-codecs-named invariant).
/// 3. **Prop-token whitelist** (every third-column `AV_CODEC_PROP_*`
///    token sits inside [`KNOWN_CODEC_PROPS`]) — fails on bogus
///    tokens that would otherwise sneak past the BITMAP_SUB-only
///    consumer in the generator.
/// 4. **Generation freshness** (rebuild `codec.rs` content via the
///    same pipeline `gen-codec` uses and diff against the on-disk
///    file) — fails on edits that didn't propagate through the
///    generator (variant order, doc comments, BITMAP_SUB set, …).
///
/// The `Other(Utf8Bytes)` arm is intentionally exempt from coverage —
/// it's the escape hatch for unknown codecs, by design.
fn check_codec(root: &Path) -> bool {
  let vendor = match fs::read_to_string(root.join(CODEC_VENDOR_PATH)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {CODEC_VENDOR_PATH}: {e}");
      eprintln!("hint:  run `cargo xtask sync` first to populate the vendored list");
      return false;
    }
  };
  let codec_rs = match fs::read_to_string(root.join(CODEC_RS)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {CODEC_RS}: {e}");
      return false;
    }
  };

  // Stage 0: vendored-line shape. The third column carries
  // `AV_CODEC_PROP_*` tokens (prefix stripped) defined in FFmpeg's
  // `libavcodec/codec_desc.h`; reject anything outside [`KNOWN_CODEC_PROPS`]
  // and reject any trailing fourth+ column outright. Without this gate a
  // bogus token (e.g. `TEXT_SUB,BOGUS_PROP`) or a corrupted line with
  // extra columns slips silently past — `gen-codec` consumes BITMAP_SUB
  // only today, so non-BITMAP corruption would not show up in the
  // freshness diff.
  if let Err(bad) = validate_vendored_props(&vendor) {
    eprintln!("FAIL: {CODEC_VENDOR_PATH} carries unknown AV_CODEC_PROP_* tokens:");
    for (line_no, line, tok) in &bad {
      eprintln!("    line {line_no}: `{tok}` in `{line}`");
    }
    // Derive the allowed-set message from `KNOWN_CODEC_PROPS` so the
    // diagnostic can't drift from the source of truth (e.g. forgetting
    // to mention `ENHANCEMENT` after adding it to the whitelist).
    let allowed = KNOWN_CODEC_PROPS.join(" / ");
    eprintln!(
      "Action: tokens must come from FFmpeg {FFMPEG_TAG} `codec_desc.h` ({allowed}). \
              If FFmpeg adds a new prop, extend `KNOWN_CODEC_PROPS` and the generator."
    );
    return false;
  }

  // FFmpeg side: media_type -> { codec name }.
  let ffmpeg = parse_codec_vendored(&vendor);
  // mediaframe side: enum-name -> { named-variant -> canonical short string }.
  // All five codec enums, not just `CODEC_ENUMS`'s four vendor-backed ones —
  // `AttachmentCodec`'s Stage 1b below needs its `as_str()` arms too.
  let mediaframe = parse_codec_named_strings(&codec_rs, ALL_CODEC_ENUM_NAMES);

  let mut ok = true;
  let mut total_named = 0usize;
  for (media_type, enum_name) in CODEC_ENUMS {
    let ff_names = match ffmpeg.get(*media_type) {
      Some(m) => m,
      None => {
        eprintln!(
          "FAIL: no vendored FFmpeg entries for media type `{media_type}` — \
                   regenerate {CODEC_VENDOR_PATH} via `cargo xtask sync`."
        );
        ok = false;
        continue;
      }
    };
    let empty = BTreeMap::new();
    let mf_named = mediaframe.get(*enum_name).unwrap_or(&empty);

    // Direction 1: every mediaframe named variant's canonical string must
    // exist on the FFmpeg side (catches typos in `as_str()`).
    let mut missing_from_ffmpeg: BTreeMap<&String, &String> = BTreeMap::new();
    for (variant, canonical) in mf_named {
      if !ff_names.contains(canonical) {
        missing_from_ffmpeg.insert(variant, canonical);
      }
    }

    // Direction 2: every FFmpeg short name must have a matching mediaframe
    // named variant (catches a `cargo xtask sync` bump that added codecs
    // without re-running `cargo xtask gen-codec`). Without this, new
    // codecs would silently parse to `Other(Utf8Bytes)` and `is_*` predicates
    // would miss them — the generated-all-codecs invariant.
    let mf_canonicals: BTreeSet<&String> = mf_named.values().collect();
    let missing_from_mediaframe: BTreeSet<&String> = ff_names
      .iter()
      .filter(|n| !mf_canonicals.contains(n))
      .collect();

    println!(
      "  {enum_name}: {} named variant(s); FFmpeg {} `{media_type}` codec(s)",
      mf_named.len(),
      ff_names.len()
    );
    total_named += mf_named.len();

    if !missing_from_ffmpeg.is_empty() {
      eprintln!(
        "FAIL: {} mediaframe `{enum_name}` named variant(s) NOT found in FFmpeg \
             {FFMPEG_TAG} `{media_type}` codecs:",
        missing_from_ffmpeg.len()
      );
      for (variant, canonical) in &missing_from_ffmpeg {
        eprintln!("    {enum_name}::{variant} → \"{canonical}\"");
      }
      eprintln!(
        "Action: either (a) the variant's canonical string disagrees with FFmpeg's \
                  short name (fix `as_str()`); or (b) the codec doesn't exist as a \
                  separate FFmpeg codec ID (drop the named variant — `Other(Utf8Bytes)` \
                  still round-trips its string)."
      );
      ok = false;
    }

    if !missing_from_mediaframe.is_empty() {
      eprintln!(
        "FAIL: {} FFmpeg {FFMPEG_TAG} `{media_type}` codec(s) NOT covered by mediaframe \
             `{enum_name}` (would silently fall through to `Other(Utf8Bytes)`):",
        missing_from_mediaframe.len()
      );
      for canonical in &missing_from_mediaframe {
        eprintln!("    \"{canonical}\"");
      }
      eprintln!(
        "Action: run `cargo xtask gen-codec` to regenerate {CODEC_RS} from the \
                  current vendored table (the all-codecs-named invariant relies on \
                  this regen step staying in sync with `xtask/vendor/ffmpeg-codecs.txt`)."
      );
      ok = false;
    }
  }

  // Stage 1b: `AttachmentCodec` vs. `ATTACHMENT_CODECS` — the same
  // two-way diff as the loop above, but against the hand-curated
  // `matroskadec.c`-derived roster rather than a vendored `codec_desc.c`
  // media type (see `ATTACHMENT_CODECS`'s doc comment for why no
  // `xtask/vendor/ffmpeg-codecs.txt` entry backs it).
  {
    let expected: BTreeSet<&str> = ATTACHMENT_CODECS.iter().copied().collect();
    let empty = BTreeMap::new();
    let mf_named = mediaframe.get("AttachmentCodec").unwrap_or(&empty);

    let mut missing_from_list: BTreeMap<&String, &String> = BTreeMap::new();
    for (variant, canonical) in mf_named {
      if !expected.contains(canonical.as_str()) {
        missing_from_list.insert(variant, canonical);
      }
    }
    let mf_canonicals: BTreeSet<&String> = mf_named.values().collect();
    let missing_from_mediaframe: BTreeSet<&str> = expected
      .iter()
      .filter(|n| !mf_canonicals.iter().any(|c| c.as_str() == **n))
      .copied()
      .collect();

    println!(
      "  AttachmentCodec: {} named variant(s); {} hand-curated `mkv_mime_tags` codec(s)",
      mf_named.len(),
      expected.len()
    );
    total_named += mf_named.len();

    if !missing_from_list.is_empty() {
      eprintln!(
        "FAIL: {} mediaframe `AttachmentCodec` named variant(s) NOT found in \
              `ATTACHMENT_CODECS`:",
        missing_from_list.len()
      );
      for (variant, canonical) in &missing_from_list {
        eprintln!("    AttachmentCodec::{variant} → \"{canonical}\"");
      }
      eprintln!(
        "Action: either fix `as_str()` or drop the variant — `ATTACHMENT_CODECS` is the \
                  hand-curated source of truth (see its doc comment)."
      );
      ok = false;
    }
    if !missing_from_mediaframe.is_empty() {
      eprintln!(
        "FAIL: {} `ATTACHMENT_CODECS` entry(ies) NOT covered by mediaframe \
              `AttachmentCodec`:",
        missing_from_mediaframe.len()
      );
      for canonical in &missing_from_mediaframe {
        eprintln!("    \"{canonical}\"");
      }
      eprintln!("Action: run `cargo xtask gen-codec` to regenerate {CODEC_RS}.");
      ok = false;
    }
  }

  // Stage 2: generation freshness. Build the codec module the same way
  // `gen-codec` would and diff against the on-disk file — catches edits to
  // the vendored table that haven't been propagated through `gen-codec`,
  // even when the variant-coverage check happens to pass (e.g. variant
  // ordering changes, `BITMAP_SUBTITLES` updates).
  match build_codec_rs(root) {
    Ok((expected_module, expected_tests)) => {
      // Reuse the already-loaded source (`codec_rs`) instead of
      // re-reading the file. The first read at the top of `check_codec`
      // returned `false` on I/O error, so by this point the content is
      // known-good — a second `read_to_string + unwrap_or_default()`
      // would both add redundant I/O and silently mask a real read
      // failure as "stale".
      if expected_module != codec_rs {
        eprintln!(
          "FAIL: {CODEC_RS} is stale vs the vendored FFmpeg table — \
                 run `cargo xtask gen-codec` to refresh it."
        );
        ok = false;
      }
      // The suite is the half that embeds the vendored name table, so a
      // stale `tests.rs` is the more dangerous of the two: it would keep
      // asserting round-trips for a codec list nobody ships any more.
      match fs::read_to_string(root.join(CODEC_TESTS_RS)) {
        Ok(on_disk) if on_disk == expected_tests => {}
        Ok(_) => {
          eprintln!(
            "FAIL: {CODEC_TESTS_RS} is stale vs the vendored FFmpeg table — \
                   run `cargo xtask gen-codec` to refresh it."
          );
          ok = false;
        }
        Err(e) => {
          eprintln!("FAIL: cannot read {CODEC_TESTS_RS}: {e}");
          ok = false;
        }
      }
    }
    Err(e) => {
      eprintln!("FAIL: could not build expected {CODEC_RS}: {e}");
      ok = false;
    }
  }

  println!("FFmpeg pinned: {FFMPEG_TAG}");
  println!(
    "mediaframe   : {total_named} named codec variant(s) across {} enum(s)",
    ALL_CODEC_ENUM_NAMES.len()
  );
  if ok {
    println!(
      "OK: mediaframe codec enums and FFmpeg {FFMPEG_TAG} are in two-way sync \
       (and {CODEC_RS} is up-to-date)."
    );
  }
  ok
}

/// Every `AV_CODEC_PROP_*` token FFmpeg n9.0
/// `libavcodec/codec_desc.h` defines (prefix stripped). Listed in
/// definition order. Bump this in lockstep with [`FFMPEG_TAG`].
const KNOWN_CODEC_PROPS: &[&str] = &[
  "INTRA_ONLY",  // (1 << 0)
  "LOSSY",       // (1 << 1)
  "LOSSLESS",    // (1 << 2)
  "REORDER",     // (1 << 3)
  "FIELDS",      // (1 << 4) — interlaced fields
  "ENHANCEMENT", // (1 << 5) — LCEVC and friends
  "BITMAP_SUB",  // (1 << 16) — OCR trigger for SubtitleCodec
  "TEXT_SUB",    // (1 << 17) — searchable text subtitles
];

/// Walk `xtask/vendor/ffmpeg-codecs.txt` and report any shape
/// violation: a third-column token outside [`KNOWN_CODEC_PROPS`], **or**
/// any unexpected fourth+ column (the vendor file is source-of-truth —
/// silent acceptance of trailing junk would let real corruption slip
/// past). Returns `(line_no, line, what)` triples so the caller can
/// report every mismatch in one shot. Empty `Ok(())` = the table is
/// clean.
fn validate_vendored_props(text: &str) -> Result<(), Vec<(usize, String, String)>> {
  let known: BTreeSet<&str> = KNOWN_CODEC_PROPS.iter().copied().collect();
  let mut bad: Vec<(usize, String, String)> = Vec::new();
  for (i, raw) in text.lines().enumerate() {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let mut it = line.split_whitespace();
    // The minimum shape is `<media_type> <name>` (props optional). A
    // line with only one token is a truncated/corrupted entry and must
    // fail the check — silently skipping it would reduce coverage
    // without surfacing the corruption.
    let Some(media) = it.next() else { continue };
    if it.next().is_none() {
      bad.push((
        i + 1,
        line.to_string(),
        format!("missing `<name>` column after `<media_type>` = `{media}`"),
      ));
      continue;
    }
    let Some(props) = it.next() else { continue };
    for tok in props.split(',').filter(|t| !t.is_empty()) {
      if !known.contains(tok) {
        bad.push((i + 1, line.to_string(), tok.to_string()));
      }
    }
    // Strict shape: no fourth column allowed. A vendored line is
    // exactly `<media_type> <name>` (no props) or
    // `<media_type> <name> <comma-joined-props>` — anything beyond is
    // corruption (extra whitespace-separated tokens, accidental
    // unjoined props, copy-paste glitches, …).
    if let Some(extra) = it.next() {
      bad.push((
        i + 1,
        line.to_string(),
        format!("unexpected trailing column `{extra}`"),
      ));
    }
  }
  if bad.is_empty() { Ok(()) } else { Err(bad) }
}

/// Parse `xtask/vendor/ffmpeg-codecs.txt` into `media_type → {name}`.
///
/// Format: one `<media_type> <name> [<props>]` per line — `<props>`
/// (a comma-separated list of `AV_CODEC_PROP_*` tokens with the
/// prefix stripped) is optional. This particular parser only needs
/// the first two columns for the coverage check; any third column is
/// silently discarded. Blank lines and `#` comments are skipped. See
/// [`build_codec_rs_with_counts`] for the parser that also consumes
/// the props column.
fn parse_codec_vendored(text: &str) -> BTreeMap<String, BTreeSet<String>> {
  let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
  for line in text.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let mut it = line.split_whitespace();
    let (Some(ty), Some(name)) = (it.next(), it.next()) else {
      continue;
    };
    out
      .entry(ty.to_string())
      .or_default()
      .insert(name.to_string());
  }
  out
}

/// Parse `enum_names`' `mediaframe::codec::<Enum>::as_str()` match blocks
/// and emit `enum-name → { variant-ident → canonical-short-string }`. The
/// `Self::Other(s) => s.as_str()` arm is skipped.
///
/// Takes the name list explicitly rather than reading [`CODEC_ENUMS`]
/// directly: `AttachmentCodec` needs its `as_str()` parsed too, and it
/// is not in `CODEC_ENUMS` (no vendored media type backs it — see
/// [`ATTACHMENT_CODECS`]). Callers pass [`ALL_CODEC_ENUM_NAMES`].
fn parse_codec_named_strings(
  rs: &str,
  enum_names: &[&str],
) -> BTreeMap<String, BTreeMap<String, String>> {
  let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
  for enum_name in enum_names {
    // Locate `impl <EnumName> {` then the `pub fn as_str(&self) -> &str`
    // body that follows. We accept any whitespace between the `impl` and
    // the as_str body; the match arms are scanned line-by-line.
    let impl_marker = format!("impl {enum_name} {{");
    let Some(impl_at) = rs.find(&impl_marker) else {
      continue;
    };
    let after = &rs[impl_at..];
    let Some(asstr_at) = after.find("pub fn as_str") else {
      continue;
    };
    let body = &after[asstr_at..];
    let Some(open) = body.find('{') else { continue };
    let arms_region = &body[open + 1..];

    let mut variants: BTreeMap<String, String> = BTreeMap::new();
    for line in arms_region.lines() {
      let line = line.trim();
      if line.starts_with('}') {
        // End of the `as_str` body (the outermost closing brace).
        break;
      }
      // Match arm:  `Self::H264 => "h264",`
      let Some(rest) = line.strip_prefix("Self::") else {
        continue;
      };
      let Some(arrow) = rest.find("=>") else {
        continue;
      };
      let variant = rest[..arrow].trim().trim_end_matches('(');
      // Skip the catch-all `Other(s)` arm.
      if rest[..arrow].contains('(') {
        continue;
      }
      let after_arrow = &rest[arrow + 2..];
      let Some(start) = after_arrow.find('"') else {
        continue;
      };
      let inner = &after_arrow[start + 1..];
      let Some(end) = inner.find('"') else { continue };
      let canonical = &inner[..end];
      variants.insert(variant.to_string(), canonical.to_string());
    }
    if !variants.is_empty() {
      out.insert(enum_name.to_string(), variants);
    }
  }
  out
}

/// Parse `xtask/vendor/ffmpeg-pixfmts.txt`. Format: one slug per line,
/// `#` comments and blank lines ignored.
fn parse_vendored(text: &str) -> BTreeSet<String> {
  text
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty() && !line.starts_with('#'))
    .map(str::to_string)
    .collect()
}

/// Parse the `as_str` match block in `src/pixel_format/mod.rs`, extracting
/// every literal slug. Excludes the `none` sentinel (FFmpeg's
/// `AV_PIX_FMT_NONE`, which the vendored slug list does not carry).
fn parse_as_str_slugs(rs: &str) -> BTreeSet<String> {
  let mut out = BTreeSet::new();
  // Lines look like:   Self::Yuv420p => "yuv420p",
  //               or:  Self::None => "none",
  for line in rs.lines() {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("Self::") {
      // Find the => and then the "..." literal.
      if let Some(arrow) = rest.find("=>") {
        let after = &rest[arrow + 2..].trim_start();
        if let Some(slug) = extract_first_string_literal(after)
          && slug != "none"
        {
          out.insert(slug);
        }
      }
    }
  }
  out
}

fn extract_first_string_literal(s: &str) -> Option<String> {
  let bytes = s.as_bytes();
  let first = bytes.iter().position(|&b| b == b'"')?;
  let rest = &s[first + 1..];
  let end = rest.find('"')?;
  Some(rest[..end].to_string())
}

/// Parse `xtask/vendor/ffmpeg-color.txt`. Format: one
/// `<ENUM> <FFMPEG_NAME> <VALUE>` per line, `#` comments and blank
/// lines ignored. Returns `ENUM -> { distinct code -> first
/// FFmpeg name seen for that code }` (aliases collapse: a code
/// already present keeps its first name).
fn parse_color_vendored(text: &str) -> BTreeMap<String, BTreeMap<u32, String>> {
  let mut out: BTreeMap<String, BTreeMap<u32, String>> = BTreeMap::new();
  for line in text.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let mut it = line.split_whitespace();
    let (Some(e), Some(name), Some(val)) = (it.next(), it.next(), it.next()) else {
      continue;
    };
    let Ok(code) = val.parse::<u32>() else {
      continue;
    };
    out
      .entry(e.to_string())
      .or_default()
      .entry(code)
      .or_insert_with(|| name.to_string());
  }
  out
}

/// One named arm of a colour enum's `to_u32()` match, joined with
/// its `as_str()` slug: `Self::<ident> => <value>` paired with the
/// `Self::<ident> => "<slug>"` literal from the same enum's
/// `as_str()`. The `Other(_)` escape arms carry no code and no literal
/// slug, so they fall out of both scans on their own.
struct NamedCode {
  value: u32,
  /// `true` iff the matching `as_str()` arm yields a non-empty slug.
  has_slug: bool,
}

/// Parse the per-enum `as_str()` + `to_u32()` match blocks in
/// `src/color/mod.rs`. Returns `mediaframe-enum -> { variant-ident ->
/// NamedCode }`. Implementation is line-oriented (matching the
/// existing `parse_as_str_slugs` style): an `impl <Enum> {` opens a
/// scope that the next `impl `/`pub enum `/`pub struct ` closes;
/// inside, `Self::<ident> => <int>,` arms seen after the
/// `fn to_u32(` line are values and `Self::<ident> => "..."` arms
/// after the `fn as_str(` line are slugs.
/// Parse the `pub const DOMAIN_EXT_BASE: u32 = <lit>;` line from
/// `src/color/mod.rs` (the mediaframe-domain colour-id base; ids `>=`
/// this are domain concepts H.273 does not enumerate, never produced
/// by the FFmpeg ingest path). Accepts a decimal or `0x`-hex literal
/// with optional `_` digit separators. Returns `None` if absent /
/// unparseable so the caller can fail loudly.
fn parse_domain_ext_base(rs: &str) -> Option<u32> {
  for raw in rs.lines() {
    let line = raw.trim();
    let Some(rest) = line.strip_prefix("pub const DOMAIN_EXT_BASE") else {
      continue;
    };
    let eq = rest.find('=')?;
    let lit = rest[eq + 1..]
      .trim()
      .trim_end_matches(';')
      .trim()
      .replace('_', "");
    return if let Some(hex) = lit.strip_prefix("0x").or_else(|| lit.strip_prefix("0X")) {
      u32::from_str_radix(hex, 16).ok()
    } else {
      lit.parse::<u32>().ok()
    };
  }
  None
}

/// Resolve a `to_u32()` right-hand side that is either a bare
/// `u32` literal or a `DOMAIN_EXT_BASE`-relative expression
/// (`DOMAIN_EXT_BASE` or `DOMAIN_EXT_BASE + <n>`). Returns the
/// numeric value, or `None` if it is neither (e.g. `*v`).
fn eval_to_u32_rhs(rhs: &str, domain_base: u32) -> Option<u32> {
  let rhs = rhs.trim();
  if let Ok(v) = rhs.parse::<u32>() {
    return Some(v);
  }
  let after = rhs.strip_prefix("DOMAIN_EXT_BASE")?.trim();
  if after.is_empty() {
    return Some(domain_base);
  }
  let off = after.strip_prefix('+')?.trim().replace('_', "");
  let n = off.parse::<u32>().ok()?;
  domain_base.checked_add(n)
}

fn parse_color_named_codes(
  rs: &str,
  domain_base: u32,
) -> BTreeMap<String, BTreeMap<String, NamedCode>> {
  let wanted: BTreeSet<&str> = COLOR_ENUMS.iter().map(|(_, _, vf)| *vf).collect();
  let mut values: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
  let mut slugs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

  let mut cur: Option<String> = None;
  let mut in_to_u32 = false;
  let mut in_as_str = false;
  for raw in rs.lines() {
    let line = raw.trim();
    // A new top-level item ends any open impl scope.
    if line.starts_with("impl ") || line.starts_with("pub enum ") || line.starts_with("pub struct ")
    {
      cur = None;
      in_to_u32 = false;
      in_as_str = false;
      if let Some(rest) = line.strip_prefix("impl ") {
        let name: String = rest
          .chars()
          .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
          .collect();
        if wanted.contains(name.as_str()) {
          cur = Some(name);
        }
      }
      continue;
    }
    let Some(enum_name) = cur.clone() else {
      continue;
    };
    if line.contains("fn to_u32(") {
      in_to_u32 = true;
      in_as_str = false;
      continue;
    }
    if line.contains("fn as_str(") {
      in_as_str = true;
      in_to_u32 = false;
      continue;
    }
    if line.contains("fn from_u32(") {
      in_to_u32 = false;
      in_as_str = false;
      continue;
    }
    let Some(rest) = line.strip_prefix("Self::") else {
      continue;
    };
    if rest.starts_with("Other") {
      continue;
    }
    let Some(arrow) = rest.find("=>") else {
      continue;
    };
    let ident: String = rest
      .chars()
      .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
      .collect();
    if in_to_u32 {
      let val_part = rest[arrow + 2..].trim().trim_end_matches(',').trim();
      if let Some(v) = eval_to_u32_rhs(val_part, domain_base) {
        values.entry(enum_name).or_default().insert(ident, v);
      }
    } else if in_as_str {
      let after = rest[arrow + 2..].trim_start();
      if let Some(slug) = extract_first_string_literal(after)
        && !slug.is_empty()
      {
        slugs.entry(enum_name).or_default().insert(ident);
      }
    }
  }

  let mut out: BTreeMap<String, BTreeMap<String, NamedCode>> = BTreeMap::new();
  for (enum_name, idents) in values {
    let slug_set = slugs.get(&enum_name).cloned().unwrap_or_default();
    let dst = out.entry(enum_name).or_default();
    for (ident, value) in idents {
      let has_slug = slug_set.contains(&ident);
      dst.insert(ident, NamedCode { value, has_slug });
    }
  }
  out
}

// ---------- sync -----------------------------------------------------------

fn sync() -> ExitCode {
  let url =
    format!("https://raw.githubusercontent.com/FFmpeg/FFmpeg/{FFMPEG_TAG}/libavutil/pixfmt.h");
  println!("Fetching {url}");

  let output = match Command::new("curl").args(["-sSL", "--fail", &url]).output() {
    Ok(o) => o,
    Err(e) => {
      eprintln!("error: failed to run `curl`: {e}");
      eprintln!("hint:  install curl, or fetch the file manually and run extraction yourself");
      return ExitCode::FAILURE;
    }
  };
  if !output.status.success() {
    eprintln!("error: curl exited with status {}", output.status);
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    return ExitCode::FAILURE;
  }
  let header = match String::from_utf8(output.stdout) {
    Ok(s) => s,
    Err(_) => {
      eprintln!("error: pixfmt.h returned non-UTF8 content");
      return ExitCode::FAILURE;
    }
  };

  let PixFmtCensus { slugs, hw_seen } = extract_avpixfmt_names(&header);
  if slugs.is_empty() {
    eprintln!("error: parsed 0 AV_PIX_FMT_* identifiers from the fetched header — parse bug?");
    return ExitCode::FAILURE;
  }
  // Prove the exclusion roster before writing anything: a roster entry
  // the pinned header does not declare excludes nothing, so it is a
  // claim about FFmpeg that FFmpeg no longer backs.
  let dead_roster: Vec<&str> = HW_FORMAT_SLUGS
    .iter()
    .copied()
    .filter(|s| !hw_seen.contains(s))
    .collect();
  if !dead_roster.is_empty() {
    eprintln!(
      "error: HW_FORMAT_SLUGS names {} format(s) FFmpeg {FFMPEG_TAG} does not declare:",
      dead_roster.len()
    );
    for s in &dead_roster {
      eprintln!("    {s}");
    }
    eprintln!(
      "Action: drop the stale entry (upstream removed the format) or fix the \
       spelling — the roster is the documented exclusion list, so a dead entry \
       makes it lie."
    );
    return ExitCode::FAILURE;
  }
  println!(
    "Exclusion roster: {}/{} hardware-surface format(s) present in {FFMPEG_TAG}",
    hw_seen.len(),
    HW_FORMAT_SLUGS.len()
  );

  let out_path = workspace_root().join(VENDOR_PATH);
  if let Some(p) = out_path.parent()
    && let Err(e) = fs::create_dir_all(p)
  {
    eprintln!("error: cannot mkdir {}: {e}", p.display());
    return ExitCode::FAILURE;
  }

  let mut body = String::new();
  body.push_str("# FFmpeg AVPixelFormat slugs — vendored for `cargo xtask check`.\n");
  body.push_str(&format!(
    "# Source: FFmpeg {FFMPEG_TAG} libavutil/pixfmt.h\n"
  ));
  body.push_str("# Fetched: ");
  body.push_str(&iso_date_today());
  body.push_str("\n#\n");
  body.push_str("# Regenerate via `cargo xtask sync` after bumping the FFMPEG_TAG constant.\n");
  body.push_str("# One slug per line, lowercase of the AV_PIX_FMT_<NAME> suffix.\n");
  body.push_str("# AV_PIX_FMT_NONE and AV_PIX_FMT_NB sentinels are skipped.\n\n");
  for s in &slugs {
    body.push_str(s);
    body.push('\n');
  }

  if let Err(e) = fs::write(&out_path, &body) {
    eprintln!("error: cannot write {}: {e}", out_path.display());
    return ExitCode::FAILURE;
  }
  println!(
    "Wrote {} slugs to {} ({} bytes)",
    slugs.len(),
    out_path.display(),
    body.len()
  );

  // ---- colour enums (same header) ----
  let colors = extract_color_enums(&header);
  if colors.is_empty() {
    eprintln!("error: parsed 0 colour-enum entries from the fetched header — parse bug?");
    return ExitCode::FAILURE;
  }
  let color_out = workspace_root().join(COLOR_VENDOR_PATH);
  let mut cbody = String::new();
  cbody.push_str("# FFmpeg colour-enum code points — vendored for `cargo xtask check`.\n");
  cbody.push_str(&format!(
    "# Source: FFmpeg {FFMPEG_TAG} libavutil/pixfmt.h\n"
  ));
  cbody.push_str("# Fetched: ");
  cbody.push_str(&iso_date_today());
  cbody.push_str("\n#\n");
  cbody.push_str("# Regenerate via `cargo xtask sync` after bumping the FFMPEG_TAG constant.\n");
  cbody.push_str("# One `<ENUM> <FFMPEG_NAME> <VALUE>` per line; AVColor*/AVChroma* enums,\n");
  cbody.push_str("# C-counter rules. *_NB sentinels, the post-NB custom EXT extensions,\n");
  cbody.push_str("# and the RESERVED*-prefix stripped names are kept verbatim; aliases\n");
  cbody.push_str("# collapse to the first name seen for each distinct value.\n\n");
  for (e, name, val) in &colors {
    cbody.push_str(e);
    cbody.push(' ');
    cbody.push_str(name);
    cbody.push(' ');
    cbody.push_str(&val.to_string());
    cbody.push('\n');
  }
  if let Err(e) = fs::write(&color_out, &cbody) {
    eprintln!("error: cannot write {}: {e}", color_out.display());
    return ExitCode::FAILURE;
  }
  println!(
    "Wrote {} colour entries to {} ({} bytes)",
    colors.len(),
    color_out.display(),
    cbody.len()
  );

  // ---- codec descriptors (libavcodec/codec_desc.c) ----
  let codec_url =
    format!("https://raw.githubusercontent.com/FFmpeg/FFmpeg/{FFMPEG_TAG}/libavcodec/codec_desc.c");
  println!("Fetching {codec_url}");
  let codec_output = match Command::new("curl")
    .args(["-sSL", "--fail", &codec_url])
    .output()
  {
    Ok(o) => o,
    Err(e) => {
      eprintln!("error: failed to run `curl` for codec_desc.c: {e}");
      return ExitCode::FAILURE;
    }
  };
  if !codec_output.status.success() {
    eprintln!(
      "error: curl exited with status {} for codec_desc.c",
      codec_output.status
    );
    eprintln!("stderr: {}", String::from_utf8_lossy(&codec_output.stderr));
    return ExitCode::FAILURE;
  }
  let codec_src = match String::from_utf8(codec_output.stdout) {
    Ok(s) => s,
    Err(_) => {
      eprintln!("error: codec_desc.c returned non-UTF8 content");
      return ExitCode::FAILURE;
    }
  };
  let mut descriptors = extract_codec_descriptors(&codec_src);
  if descriptors.is_empty() {
    eprintln!(
      "error: parsed 0 codec descriptors from codec_desc.c — parse bug or upstream restructure?"
    );
    return ExitCode::FAILURE;
  }
  // Sort by (media_type, name) for deterministic output.
  descriptors.sort();

  let codec_out = workspace_root().join(CODEC_VENDOR_PATH);
  let mut kbody = String::new();
  kbody.push_str("# FFmpeg codec short names — vendored for `cargo xtask check`.\n");
  kbody.push_str(&format!(
    "# Source: FFmpeg {FFMPEG_TAG} libavcodec/codec_desc.c\n"
  ));
  kbody.push_str("# Fetched: ");
  kbody.push_str(&iso_date_today());
  kbody.push_str("\n#\n");
  kbody.push_str("# Regenerate via `cargo xtask sync` after bumping the FFMPEG_TAG constant.\n");
  kbody.push_str("# Format: `<media_type> <name> [<props>]` — one descriptor per line, sorted.\n");
  kbody.push_str(
    "# `<media_type>` is the lowercased AVMEDIA_TYPE_* suffix\n\
     # (video / audio / subtitle / data / attachment).\n",
  );
  kbody.push_str(
    "# `<props>` is a comma-separated list of `AV_CODEC_PROP_*` tokens\n\
     # (prefix stripped, e.g. `BITMAP_SUB`, `TEXT_SUB`, `LOSSY`). Optional —\n\
     # codecs with no `.props` initializer omit the column entirely. The\n\
     # generator (`cargo xtask gen-codec`) uses this set to derive predicate\n\
     # methods like `SubtitleCodec::is_image_based()` (= `BITMAP_SUB`).\n\n",
  );
  for (ty, name, props) in &descriptors {
    kbody.push_str(ty);
    kbody.push(' ');
    kbody.push_str(name);
    if !props.is_empty() {
      kbody.push(' ');
      let joined: Vec<&str> = props.iter().map(String::as_str).collect();
      kbody.push_str(&joined.join(","));
    }
    kbody.push('\n');
  }
  if let Err(e) = fs::write(&codec_out, &kbody) {
    eprintln!("error: cannot write {}: {e}", codec_out.display());
    return ExitCode::FAILURE;
  }
  println!(
    "Wrote {} codec descriptors to {} ({} bytes)",
    descriptors.len(),
    codec_out.display(),
    kbody.len()
  );

  if let Err(e) = lang::sync(&workspace_root()) {
    eprintln!("{e}");
    return ExitCode::FAILURE;
  }

  ExitCode::SUCCESS
}

/// Hardware-frame markers — FFmpeg pixel formats whose buffers live
/// in GPU memory. mediaframe intentionally excludes these per the
/// `pixel_format` module docs: a frame carrying GPU-resident buffers
/// must be transferred to a CPU format before reaching a mediaframe
/// consumer.
///
/// Every entry must name a format the pinned header actually declares
/// — [`sync`] proves that and refuses to write a table built from a
/// roster with a dead entry. A slug FFmpeg has dropped filters nothing,
/// so leaving it here would quietly turn the exclusion list into a
/// claim about a header that no longer says it (`xvmc` sat here for
/// exactly that reason until the n9.0 bump).
const HW_FORMAT_SLUGS: &[&str] = &[
  "amf_surface",
  "cuda",
  "d3d11",
  "d3d11va_vld",
  "d3d12",
  "drm_prime",
  "dxva2_vld",
  "mediacodec",
  "mmal",
  "ohcodec",
  "opencl",
  "qsv",
  "vaapi",
  "vdpau",
  "videotoolbox",
  "vulkan",
];

/// What one pass over the pinned `pixfmt.h` `AVPixelFormat` enum found:
/// the CPU-side slugs to vendor, and which [`HW_FORMAT_SLUGS`] entries
/// actually fired.
struct PixFmtCensus {
  /// Lowercased `AV_PIX_FMT_<NAME>` suffixes, hardware surfaces and the
  /// `NONE` / `NB` sentinels removed.
  slugs: BTreeSet<String>,
  /// The roster entries the header declared. Compared against the full
  /// roster this is what makes a stale exclusion visible.
  hw_seen: BTreeSet<&'static str>,
}

fn extract_avpixfmt_names(header: &str) -> PixFmtCensus {
  let mut out = BTreeSet::new();
  let mut hw_seen = BTreeSet::new();
  let mut in_enum = false;
  for raw in header.lines() {
    let line = raw.trim();
    if line.starts_with("enum AVPixelFormat") {
      in_enum = true;
      continue;
    }
    if !in_enum {
      continue;
    }
    if line == "};" {
      break;
    }
    if let Some(rest) = line.strip_prefix("AV_PIX_FMT_") {
      let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
      if name.is_empty() {
        continue;
      }
      if name == "NONE" || name == "NB" {
        continue;
      }
      let slug = name.to_ascii_lowercase();
      if let Some(hw) = HW_FORMAT_SLUGS.iter().find(|h| **h == slug) {
        hw_seen.insert(*hw);
        continue;
      }
      out.insert(slug);
    }
  }
  PixFmtCensus {
    slugs: out,
    hw_seen,
  }
}

/// Parse the five colour C enums from `pixfmt.h`, applying C
/// enumerator rules: a running counter starts at 0 and increments
/// per entry, overridden when an explicit `= N` is present (the
/// counter then continues from `N + 1`). An `= AVCOL_xxx` /
/// `= AVCHROMA_xxx` alias resolves to that already-seen entry's
/// value (no counter step) and is recorded only if its distinct
/// value is new (collapsing aliases like `AVCOL_PRI_JEDEC_P22 =
/// AVCOL_PRI_EBU3213`). `*_NB` sentinels terminate the enum (this
/// also drops the post-`NB` custom `*_EXT_BASE` extensions, which
/// are not part of the H.273 code points mediaframe models).
/// Returns `(mediaframe-enum-name, ffmpeg-name, value)` rows in
/// declaration order, one per distinct value.
fn extract_color_enums(header: &str) -> Vec<(String, String, u32)> {
  let mut out: Vec<(String, String, u32)> = Vec::new();
  for (c_enum, prefix, vf_enum) in COLOR_ENUMS {
    let mut in_enum = false;
    let mut counter: u32 = 0;
    // raw FFmpeg name (sans prefix) -> value, for alias resolution.
    let mut seen_names: BTreeMap<String, u32> = BTreeMap::new();
    // distinct values already emitted for this enum.
    let mut seen_values: BTreeSet<u32> = BTreeSet::new();
    for raw in header.lines() {
      let line = raw.trim();
      if !in_enum {
        if line.starts_with(&format!("enum {c_enum}")) {
          in_enum = true;
        }
        continue;
      }
      if line == "};" {
        break;
      }
      let Some(rest) = line.strip_prefix(prefix) else {
        continue;
      };
      let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
      if name.is_empty() {
        continue;
      }
      // `*_NB` (and `*_EXT_NB`) sentinel: end of the ABI enum.
      if name == "NB" || name.ends_with("_NB") {
        break;
      }
      // Determine the value: explicit `= N`, alias `= AVCOL_*`, or
      // the running counter.
      let after_name = rest[name.len()..].trim_start();
      let value = if let Some(eq) = after_name.strip_prefix('=') {
        let rhs = eq.trim();
        if let Some(n) = rhs
          .chars()
          .take_while(|c| c.is_ascii_digit())
          .collect::<String>()
          .parse::<u32>()
          .ok()
          .filter(|_| rhs.starts_with(|c: char| c.is_ascii_digit()))
        {
          counter = n.wrapping_add(1);
          n
        } else {
          // Alias: `= AVCOL_PRI_EBU3213` etc. Resolve via the
          // already-seen raw name (prefix-stripped). No counter step.
          let alias_target: String = rhs
            .strip_prefix(prefix)
            .unwrap_or(rhs)
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
          match seen_names.get(&alias_target) {
            Some(v) => *v,
            None => continue, // unresolved alias — skip defensively
          }
        }
      } else {
        let v = counter;
        counter = counter.wrapping_add(1);
        v
      };
      seen_names.insert(name.clone(), value);
      // One entry per distinct value (collapse aliases).
      if seen_values.insert(value) {
        out.push((vf_enum.to_string(), name, value));
      }
    }
  }
  out
}

fn iso_date_today() -> String {
  // Avoid pulling chrono / time for one date string. Shell out to
  // `date` — available on every dev box and CI runner xtask supports.
  Command::new("date")
    .args(["-u", "+%Y-%m-%d"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|| "unknown".to_string())
}

/// Parse FFmpeg's `libavcodec/codec_desc.c` for the
/// `codec_descriptors[]` table and return `(media_type, short_name)`
/// pairs for every entry.
///
/// Strategy: locate the `codec_descriptors[]` array, then iterate
/// line-by-line tracking the current `.type = AVMEDIA_TYPE_<X>,` and
/// `.name = "<short>",`. On the descriptor's closing brace (`},` or
/// `}` on its own line at the array depth) emit the pair if both
/// fields were seen. `NULL_IF_CONFIG_SMALL(...)` and other macro-wrapped
/// fields are ignored — `.name` is always a bare string literal in
/// codec_desc.c.
/// One parsed descriptor: `(media_type, short_name, props)`.
///
/// `props` carries every `AV_CODEC_PROP_*` token referenced inside the
/// descriptor (with the `AV_CODEC_PROP_` prefix stripped). The only place
/// these tokens appear in `codec_desc.c` is in the `.props` initializer
/// expression, so collecting them per-block recovers the canonical
/// FFmpeg-side property set without parsing the multi-line `|` expression
/// shape.
type CodecDescriptor = (String, String, BTreeSet<String>);

fn extract_codec_descriptors(source: &str) -> Vec<CodecDescriptor> {
  let mut out: Vec<CodecDescriptor> = Vec::new();
  let Some(arr_at) = source.find("codec_descriptors[]") else {
    return out;
  };
  // Skip past the array's opening `{`.
  let after_arr = &source[arr_at..];
  let Some(open_at) = after_arr.find('{') else {
    return out;
  };
  let body = &after_arr[open_at + 1..];

  let mut current_type: Option<String> = None;
  let mut current_name: Option<String> = None;
  let mut current_props: BTreeSet<String> = BTreeSet::new();
  let mut depth_in_descriptor: i32 = 0;

  for raw in body.lines() {
    let line = raw.trim();

    // End of the array — the array's closing brace.
    if depth_in_descriptor == 0 && (line == "};" || line.starts_with("};")) {
      break;
    }

    // Track sub-block depth inside a descriptor (rare nested braces).
    let opens = line.matches('{').count() as i32;
    let closes = line.matches('}').count() as i32;

    // Entering a top-level descriptor block (a `{ ` on its own line
    // or the start of an entry, with nothing previously open).
    if depth_in_descriptor == 0 && opens > 0 {
      current_type = None;
      current_name = None;
      current_props.clear();
    }
    depth_in_descriptor += opens - closes;

    // Field extraction.
    if let Some(rest) = line.strip_prefix(".type") {
      if let Some(eq) = rest.find('=') {
        let val = rest[eq + 1..].trim().trim_end_matches(',').trim();
        if let Some(t) = val.strip_prefix("AVMEDIA_TYPE_") {
          current_type = Some(t.to_lowercase());
        }
      }
    } else if let Some(rest) = line.strip_prefix(".name")
      && let Some(eq) = rest.find('=')
    {
      let after_eq = &rest[eq + 1..];
      if let Some(start) = after_eq.find('"') {
        let inner = &after_eq[start + 1..];
        if let Some(end) = inner.find('"') {
          current_name = Some(inner[..end].to_string());
        }
      }
    }

    // Collect any AV_CODEC_PROP_<NAME> tokens on this line — they only
    // appear inside `.props` initializers, so per-block accumulation
    // captures the property set even when `.props = A | B,` is split
    // across multiple lines with the `|` continuations.
    let mut cursor = line;
    while let Some(idx) = cursor.find("AV_CODEC_PROP_") {
      let after = &cursor[idx + "AV_CODEC_PROP_".len()..];
      let end = after
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(after.len());
      let tok = &after[..end];
      if !tok.is_empty() && tok != "NB" {
        current_props.insert(tok.to_string());
      }
      cursor = &after[end..];
    }

    // Closed back to array depth — descriptor finished.
    if depth_in_descriptor == 0 && closes > 0 {
      if let (Some(t), Some(n)) = (current_type.take(), current_name.take()) {
        out.push((t, n, std::mem::take(&mut current_props)));
      }
      current_props.clear();
    }
  }
  out
}

// ---------- gen-codec ------------------------------------------------------
//
// Regenerate `mediaframe/src/codec/mod.rs` from `xtask/vendor/ffmpeg-codecs.txt`
// using the same `quote!` / `proc-macro2` / `prettyplease` toolchain proc-
// macros use. Single source of truth (the vendored table) → single
// generated module; `cargo xtask check` is the CI gate against drift.

use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

fn gen_codec() -> ExitCode {
  let root = workspace_root();
  let (counts, module, tests) = match build_codec_rs_with_counts(&root) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("{e}");
      return ExitCode::FAILURE;
    }
  };
  let out_path = root.join(CODEC_RS);
  let tests_path = root.join(CODEC_TESTS_RS);
  for (path, content) in [(&out_path, &module), (&tests_path, &tests)] {
    if let Some(dir) = path.parent()
      && let Err(e) = fs::create_dir_all(dir)
    {
      eprintln!("error: cannot create {}: {e}", dir.display());
      return ExitCode::FAILURE;
    }
    if let Err(e) = fs::write(path, content) {
      eprintln!("error: cannot write {}: {e}", path.display());
      return ExitCode::FAILURE;
    }
  }
  let (v, a, s, d, t) = counts;
  println!(
    "Wrote {} codec variants ({} video + {} audio + {} subtitle + {} data + {} attachment) \
     to {} ({} bytes) and its suite to {} ({} bytes)",
    v + a + s + d + t,
    v,
    a,
    s,
    d,
    t,
    out_path.display(),
    module.len(),
    tests_path.display(),
    tests.len()
  );
  ExitCode::SUCCESS
}

/// Regenerate `mediaframe/src/lang/registry/table.rs` from the two vendored
/// BCP 47 registries. The freshness gate is `cargo xtask check`, which
/// renders the same text in memory and diffs it against what is on disk.
fn gen_lang() -> ExitCode {
  match lang::generate(&workspace_root()) {
    Ok(bytes) => {
      println!("Wrote {} ({bytes} bytes)", lang::TABLE_RS);
      ExitCode::SUCCESS
    }
    Err(e) => {
      eprintln!("{e}");
      ExitCode::FAILURE
    }
  }
}

/// Build the **expected** content of `mediaframe/src/codec/mod.rs` from
/// `xtask/vendor/ffmpeg-codecs.txt`. Used both by `gen-codec` (writes the
/// result to disk) and by `check_codec`'s freshness check (compares the
/// result to the on-disk file).
///
/// Pipeline: vendor file → `BTreeSet` per media type → `quote!` `TokenStream`
/// → `syn::parse2` → `prettyplease::unparse` → `rustfmt --emit=stdout`. The
/// final `rustfmt` step is required for CI parity — `prettyplease` is
/// rustfmt-adjacent but block-wraps long match arms and renders multi-line
/// docs as `/** */`, neither of which survives `cargo fmt --check`.
fn build_codec_rs(root: &Path) -> Result<(String, String), String> {
  build_codec_rs_with_counts(root).map(|(_, module, tests)| (module, tests))
}

/// Named codec counts by media type: `(video, audio, subtitle, data,
/// attachment)`.
type CodecCounts = (usize, usize, usize, usize, usize);

/// Everything `gen-codec` produces: the counts, `codec/mod.rs`, and its
/// sibling `codec/tests.rs`.
type GeneratedCodec = (CodecCounts, String, String);

/// The crate keeps unit tests in a sibling file, so the generator emits
/// the pair and the freshness check compares both — a stale `tests.rs`
/// is exactly as wrong as a stale `mod.rs`, and it is the half that
/// carries the vendored name table.
fn build_codec_rs_with_counts(root: &Path) -> Result<GeneratedCodec, String> {
  let vendor = fs::read_to_string(root.join(CODEC_VENDOR_PATH)).map_err(|e| {
    format!(
      "error: cannot read {CODEC_VENDOR_PATH}: {e}\n\
         hint:  run `cargo xtask sync` first to populate the vendored list"
    )
  })?;

  // Parse the vendored table into `media_type -> name -> {prop tokens}`.
  // The third column is comma-separated, prefix-stripped `AV_CODEC_PROP_*`
  // tokens; codecs with no `.props` initializer have no third column.
  let mut by_type: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
  for line in vendor.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let mut it = line.split_whitespace();
    let (Some(ty), Some(name)) = (it.next(), it.next()) else {
      continue;
    };
    let props: BTreeSet<String> = it
      .next()
      .map(|s| {
        s.split(',')
          .filter(|t| !t.is_empty())
          .map(str::to_string)
          .collect()
      })
      .unwrap_or_default();
    by_type
      .entry(ty.to_string())
      .or_default()
      .insert(name.to_string(), props);
  }
  let video = by_type.remove("video").unwrap_or_default();
  let audio = by_type.remove("audio").unwrap_or_default();
  let subtitle = by_type.remove("subtitle").unwrap_or_default();
  let data = by_type.remove("data").unwrap_or_default();

  // `AttachmentCodec` has no vendored media type to `.remove()` — its
  // roster is the hand-curated `ATTACHMENT_CODECS` const (see its doc
  // comment). Reshaped into the same `CodecsWithProps` shape so it can
  // go through the identical `build_codec_enum` path as the other four;
  // none of its entries carry FFmpeg `.props` (`codec_desc.c` has no
  // `AVMEDIA_TYPE_ATTACHMENT` descriptors to read `.props` from), so
  // every value is an empty set.
  let attachment: CodecsWithProps = ATTACHMENT_CODECS
    .iter()
    .map(|&name| (name.to_string(), BTreeSet::new()))
    .collect();

  let edition = read_mediaframe_edition(root)?;
  let render = |ts: TokenStream| -> Result<String, String> {
    let parsed: syn::File = syn::parse2(ts)
      .map_err(|e| format!("internal error: generated token stream is not parseable: {e}"))?;
    run_rustfmt(&prettyplease::unparse(&parsed), &edition)
  };
  let module = render(build_codec_module(
    &video,
    &audio,
    &subtitle,
    &data,
    &attachment,
  ))?;
  let tests = render(build_codec_tests(
    &video,
    &audio,
    &subtitle,
    &data,
    &attachment,
  ))?;
  Ok((
    (
      video.len(),
      audio.len(),
      subtitle.len(),
      data.len(),
      attachment.len(),
    ),
    module,
    tests,
  ))
}

/// Read the `edition = "<year>"` field from `mediaframe/Cargo.toml`.
///
/// `rustfmt` needs an explicit `--edition` when fed source over stdin
/// (no manifest to consult); hard-coding it would silently desync from
/// a future edition bump and could format differently or even fail on
/// edge-case syntax. Reading the value from the manifest keeps the
/// generator a single source of truth with the crate it formats.
fn read_mediaframe_edition(root: &Path) -> Result<String, String> {
  let manifest_path = root.join("mediaframe/Cargo.toml");
  let manifest = fs::read_to_string(&manifest_path)
    .map_err(|e| format!("error: cannot read {}: {e}", manifest_path.display()))?;
  // Parse the first top-level `edition = "<year>"` line. Comments and
  // values inside a `[features]` array don't match this prefix shape,
  // so a manual scan is reliable enough — adding a `toml` dep just for
  // one field would be overkill.
  for raw in manifest.lines() {
    let line = raw.trim_start();
    if let Some(rest) = line.strip_prefix("edition")
      && let Some(after_eq) = rest.split_once('=').map(|(_, v)| v.trim())
    {
      let trimmed = after_eq.trim_matches('"');
      if trimmed.chars().all(|c| c.is_ascii_digit()) && !trimmed.is_empty() {
        return Ok(trimmed.to_string());
      }
    }
  }
  Err(format!(
    "error: could not find `edition = \"<year>\"` in {}",
    manifest_path.display()
  ))
}

/// Pipe `source` through `rustfmt --edition=<edition> --emit=stdout`
/// and return the formatted result. The `edition` comes from
/// [`read_mediaframe_edition`] so it stays aligned with the crate's
/// manifest. Going via stdin/stdout (not a file) lets the generator
/// stay side-effect-free for `check_codec`'s freshness diff.
fn run_rustfmt(source: &str, edition: &str) -> Result<String, String> {
  use std::{io::Write, process::Stdio};

  let mut child = Command::new("rustfmt")
    .arg(format!("--edition={edition}"))
    .arg("--emit=stdout")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|e| {
      format!(
        "error: failed to invoke rustfmt: {e}\n\
                 hint:  install via `rustup component add rustfmt`"
      )
    })?;

  child
    .stdin
    .as_mut()
    .expect("piped stdin")
    .write_all(source.as_bytes())
    .map_err(|e| format!("error: write to rustfmt stdin failed: {e}"))?;

  let out = child
    .wait_with_output()
    .map_err(|e| format!("error: wait on rustfmt failed: {e}"))?;
  if !out.status.success() {
    return Err(format!(
      "rustfmt exited with status {}: {}",
      out.status,
      String::from_utf8_lossy(&out.stderr)
    ));
  }
  String::from_utf8(out.stdout).map_err(|e| format!("rustfmt stdout was not UTF-8: {e}"))
}

/// Convert an FFmpeg codec short name into a valid Rust identifier.
///
/// Rules:
/// - Split on `_` or `.` (the only separators FFmpeg uses).
/// - Each non-empty segment: uppercase the first char, lowercase the rest.
/// - If the result starts with a digit (`4xm`, `8svx_exp`, `012v`),
///   prepend `N` (for **n**umeric-start — a leading underscore would
///   carry Rust's "intentionally unused / private" semantics, which is
///   wrong for a public enum variant callers are meant to use). `N` also
///   keeps the derived `IsVariant` methods (`is_n012v`, `is_n4xm`, …)
///   in canonical snake_case, removing the need for a module-level
///   `#[allow(non_snake_case)]`.
///
/// Examples:
/// `h264`→`H264`, `pcm_s16le`→`PcmS16le`, `dvb_subtitle`→`DvbSubtitle`,
/// `acelp.kelvin`→`AcelpKelvin`, `4xm`→`N4xm`, `012v`→`N012v`,
/// `8svx_exp`→`N8svxExp`.
fn codec_ident(name: &str) -> String {
  let mut out = String::with_capacity(name.len());
  for seg in name.split(['_', '.']) {
    if seg.is_empty() {
      continue;
    }
    let mut chars = seg.chars();
    if let Some(first) = chars.next() {
      out.extend(first.to_uppercase());
    }
    for c in chars {
      out.extend(c.to_lowercase());
    }
  }
  if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
    out.insert(0, 'N');
  }
  out
}

/// One media type's codecs, keyed by FFmpeg short name → set of
/// `AV_CODEC_PROP_*` tokens (prefix stripped) carried in `.props`.
type CodecsWithProps = BTreeMap<String, BTreeSet<String>>;

fn build_codec_module(
  video: &CodecsWithProps,
  audio: &CodecsWithProps,
  subtitle: &CodecsWithProps,
  data: &CodecsWithProps,
  attachment: &CodecsWithProps,
) -> TokenStream {
  let video_enum = build_codec_enum("VideoCodec", "video", video, false, false);
  let audio_enum = build_codec_enum("AudioCodec", "audio", audio, false, false);
  let subtitle_enum = build_codec_enum("SubtitleCodec", "subtitle", subtitle, true, true);
  let data_enum = build_codec_enum("DataCodec", "data", data, true, false);
  let attachment_enum = build_codec_enum("AttachmentCodec", "attachment", attachment, true, false);

  // Module-level docs (inner-doc attributes — `prettyplease` renders them
  // as `//!` lines on output). The pinned tag and the variant counts are
  // interpolated rather than spelled out: a hand-written "n8.1" or a
  // hand-written "(281)" is a fact about the vendored table that no longer
  // regenerates with it, and both had gone stale by the n9.0 bump.
  let generated_from =
    format!(" **Generated** from `xtask/vendor/ffmpeg-codecs.txt` (FFmpeg {FFMPEG_TAG}");
  let small_enums = format!(
    " [`SubtitleCodec`] ({}), [`DataCodec`] ({}), and [`AttachmentCodec`]",
    subtitle.len(),
    data.len()
  );
  let large_enums = format!(
    " ({}) are small enough to carry the pair; [`VideoCodec`] ({})",
    attachment.len(),
    video.len()
  );
  let audio_count = format!(" and [`AudioCodec`] ({}) do not. The line is", audio.len());
  let module_doc: Vec<TokenStream> = [
    " Stream-descriptor **codec** vocabulary for video, audio, subtitle,",
    " data, and attachment tracks.",
    "",
    generated_from.as_str(),
    " `libavcodec/codec_desc.c`) by `cargo xtask gen-codec` — except",
    " [`AttachmentCodec`], whose roster comes from a different FFmpeg",
    " source; see its own doc comment for why. Every codec FFmpeg knows",
    " under media types `video` / `audio` / `subtitle` / `data` has a",
    " named variant here; the `Other(Utf8Bytes)` arm remains a lossless",
    " escape for codecs added in a future FFmpeg release before this",
    " file is regenerated (or, for `AttachmentCodec`, before",
    " `ATTACHMENT_CODECS` is re-derived by hand).",
    "",
    " Regenerate in two steps:",
    " 1. `cargo xtask sync`       — refreshes the vendored table.",
    " 2. `cargo xtask gen-codec`  — regenerates this file from it.",
    "",
    " `cargo xtask check` verifies every named variant's canonical string",
    " exists in the vendored table (or, for `AttachmentCodec`, in",
    " `ATTACHMENT_CODECS`) — CI gate against drift.",
    "",
    " **Derive threshold.** `Unwrap` / `TryUnwrap` generate three methods",
    " per variant, so an enum in the hundreds pays that in compile time for",
    " one reachable payload arm.",
    small_enums.as_str(),
    large_enums.as_str(),
    audio_count.as_str(),
    " variant count, not principle — reach for `Other(_)` on the large two",
    " with a `match` or [`IsVariant`](derive_more::IsVariant)'s `is_other`.",
  ]
  .iter()
  .map(|line| quote! { #![doc = #line] })
  .collect();

  quote! {
    #(#module_doc)*

    use core::str::FromStr;

    use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
    use smol_bytes::Utf8Bytes;

    #video_enum

    #audio_enum

    #subtitle_enum

    #data_enum

    #attachment_enum

    #[cfg(test)]
    mod tests;
  }
}

fn build_codec_enum(
  type_name: &str,
  media_type: &str,
  codecs: &CodecsWithProps,
  wants_unwrap: bool,
  is_subtitle: bool,
) -> TokenStream {
  let enum_ident = format_ident!("{}", type_name);
  // `AttachmentCodec` alone has no `codec_desc.c` media type behind it
  // (see `ATTACHMENT_CODECS`'s doc comment) — its enum-level prose says
  // so instead of claiming FFmpeg media-type coverage it doesn't have.
  let is_attachment = type_name == "AttachmentCodec";

  // The `quickcheck` feature derives `quickcheck_richderive::Arbitrary` on
  // each codec enum, routing generation through a per-type helper fn
  // (`quickcheck_helpers::strings::{video,audio,subtitle}_codec`). Since
  // `codec.rs` is generated, that derive `cfg_attr` must be emitted *here*
  // — a hand-edit would not survive `gen-codec` and would fail the
  // freshness diff in `cargo xtask check`.
  let qc_helper = format!("crate::quickcheck_helpers::strings::{media_type}_codec");

  let variants: Vec<(Ident, String)> = codecs
    .keys()
    .map(|n| (Ident::new(&codec_ident(n), Span::call_site()), n.clone()))
    .collect();

  let variant_decls = variants.iter().map(|(ident, name)| {
    let doc = format!(" FFmpeg `\"{name}\"`.");
    quote! {
      #[doc = #doc]
      #ident,
    }
  });

  let as_str_arms = variants.iter().map(|(ident, name)| {
    quote! { Self::#ident => #name, }
  });

  // The parse table compares on the byte side — `crate::parse::lookup`
  // (declared `Case::Insensitive`, same as every other household today)
  // hands back the resolved bytes — so the arms are `b"name"` literals.
  let from_str_arms = variants.iter().map(|(ident, name)| {
    let name = Literal::byte_string(name.as_bytes());
    quote! { #name => Self::#ident, }
  });

  // `ROSTER` and its completeness witness are emitted from the *same*
  // `variants` vector the declaration is built from, so declaration order
  // and roster order cannot disagree — there is one list, used three
  // times. The hand-written vocabularies get the same pair from the
  // crate's `roster!` macro; here the generator is already the single
  // source, and emitting plain items (rather than a macro call) keeps the
  // output deterministic for the byte-for-byte freshness diff in
  // `cargo xtask check`.
  //
  // The witness is one or-pattern rather than one arm per variant: with
  // 200-plus variants that is the difference between a readable generated
  // file and several hundred lines of noise, and `E0004` names the missing
  // variant either way.
  let roster_entries = variants.iter().map(|(ident, _)| quote! { Self::#ident });
  let witness_pats = variants
    .iter()
    .map(|(ident, _)| quote! { #enum_ident::#ident });
  let roster_doc =
    format!(" Every {media_type} codec this vocabulary names, in declaration order.");

  let other_doc = if is_attachment {
    " The open escape for a codec id not in `ATTACHMENT_CODECS`.".to_string()
  } else {
    format!(" The open escape for a codec name FFmpeg {FFMPEG_TAG} does not carry.")
  };

  let enum_doc = if is_attachment {
    " Attachment codec family — the FFmpeg codec ids `libavformat/matroskadec.c`'s \
     `mkv_mime_tags` table assigns to an `AVMEDIA_TYPE_ATTACHMENT` stream \
     (`ATTACHMENT_CODECS`; see its doc comment for the full census — \
     `libavcodec/codec_desc.c` has no `AVMEDIA_TYPE_ATTACHMENT` media type to \
     enumerate here the way `DataCodec` and the other vendored enums are).\n\n \
     `#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` \
     arm is the lossless escape for an attachment codec id this list does not \
     (yet) name."
      .to_string()
  } else {
    format!(
      " {} codec family — every codec FFmpeg {FFMPEG_TAG} knows under media type `{}`.\n\n \
       `#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` \
       arm is the lossless escape for codecs added upstream before this file is \
       regenerated.",
      type_name.strip_suffix("Codec").unwrap_or(type_name),
      media_type
    )
  };

  // `Unwrap` / `TryUnwrap` generate three methods per variant, so the two
  // 200-plus-variant codec enums (`VideoCodec`, `AudioCodec`) would pay
  // ~1500 generated methods in compile time for one reachable payload
  // arm (`Other`). `SubtitleCodec`, `DataCodec`, and `AttachmentCodec`
  // are all small enough (a few dozen variants or fewer) that the same
  // pair is cheap, so `wants_unwrap` is true for all three. The rule is
  // written on the module (with the live counts), not left implicit
  // here.
  let unwrap_derives = if wants_unwrap {
    quote! {
      #[derive(Unwrap, TryUnwrap)]
      #[unwrap(ref, ref_mut)]
      #[try_unwrap(ref, ref_mut)]
    }
  } else {
    quote! {}
  };

  // Subtitle `is_image_based()` is sourced from the vendored `.props` set
  // (FFmpeg's `AV_CODEC_PROP_BITMAP_SUB` flag), not a hand-maintained
  // constant. Returns `Option<bool>`: `Some(true)` for bitmap subtitles,
  // `Some(false)` for known-text subtitles, `None` for `Other(_)` because
  // an unknown codec name has no FFmpeg `.props` record we can consult.
  // Subtitle-specific: `DataCodec` and `AttachmentCodec` carry no
  // `.props`-derived predicate — neither vendors `.props` (`DataCodec`'s
  // FFmpeg entries carry none; `AttachmentCodec` isn't vendored from
  // `codec_desc.c` at all), and there is no analogous question to ask of
  // a data or attachment codec the way "is this bitmap or text" applies
  // to subtitles.

  let extra_impl = if is_subtitle {
    let mut bitmap_idents: Vec<Ident> = Vec::new();
    let mut non_bitmap_idents: Vec<Ident> = Vec::new();
    for (ident, name) in &variants {
      let props = codecs.get(name).cloned().unwrap_or_default();
      if props.contains("BITMAP_SUB") {
        bitmap_idents.push(ident.clone());
      } else {
        // Includes `TEXT_SUB` codecs *and* the no-`.props` codecs
        // (arib_caption, dvb_teletext, ivtv_vbi …) — FFmpeg classifies
        // none of them as bitmap, so OCR is not the right pipeline.
        non_bitmap_idents.push(ident.clone());
      }
    }
    let bitmap_arms = bitmap_idents.iter().map(|i| quote! { Self::#i });
    let non_bitmap_arms = non_bitmap_idents.iter().map(|i| quote! { Self::#i });
    let bitmap_count = bitmap_idents.len();
    let non_bitmap_count = non_bitmap_idents.len();
    // Two separate `#[doc]` attributes (the first an empty line) give
    // `prettyplease` a paragraph break after the bullet list — without
    // it, the trailing parenthetical on stable rust 1.95+ trips
    // `clippy::doc_lazy_continuation` (a list-continuation line with no
    // indent reads as a malformed sub-item).
    let counts_blank = String::new();
    let counts_doc = format!(
      " ({bitmap_count} bitmap / {non_bitmap_count} non-bitmap variant(s) per FFmpeg {FFMPEG_TAG})."
    );
    let no_props_doc = format!("   codecs that carry no `.props` at all in FFmpeg {FFMPEG_TAG}).");
    quote! {
      /// Is this a **bitmap** (image-based) subtitle codec, requiring an
      /// OCR pipeline stage to extract searchable text?
      ///
      /// - `Some(true)`: matches FFmpeg's `AV_CODEC_PROP_BITMAP_SUB` flag.
      /// - `Some(false)`: a known FFmpeg subtitle codec without
      ///   `AV_CODEC_PROP_BITMAP_SUB` (text codecs and teletext/VBI-style
      #[doc = #no_props_doc]
      /// - `None`: [`Self::Other`] — the codec name is not in the vendored
      ///   FFmpeg table, so we cannot consult `.props`.
      #[doc = #counts_blank]
      #[doc = #counts_doc]
      pub fn is_image_based(&self) -> Option<bool> {
        match self {
          #(#bitmap_arms)|* => Some(true),
          #(#non_bitmap_arms)|* => Some(false),
          Self::Other(_) => None,
        }
      }
    }
  } else {
    quote! {}
  };

  quote! {
    #[doc = #enum_doc]
    #[cfg_attr(
      feature = "quickcheck",
      derive(::quickcheck_richderive::Arbitrary),
      quickcheck(arbitrary = #qc_helper)
    )]
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
    #unwrap_derives
    #[display("{}", self.as_str())]
    #[non_exhaustive]
    pub enum #enum_ident {
      #(#variant_decls)*
      /// A codec not enumerated above — carries the FFmpeg short name
      /// verbatim.
      Other(Utf8Bytes),
    }

    impl #enum_ident {
      /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
      pub fn as_str(&self) -> &str {
        match self {
          #(#as_str_arms)*
          Self::Other(s) => s.as_str(),
        }
      }

      #[doc = #other_doc]
      ///
      /// Runs the ignore-case parse first — [`Self::from_str`] rather than
      /// a duplicated table — so a canonical short name returns that
      /// **named** variant, never a second value for a meaning this
      /// vocabulary already has one for. Only a genuine stranger reaches
      /// [`Self::Other`], carrying the caller's spelling verbatim: the
      /// escape is a lossless passthrough for a name this build does not
      /// know, not a fold target.
      pub fn other(slug: impl AsRef<str>) -> Self {
        Self::from_str(slug.as_ref()).unwrap()
      }

      #extra_impl
    }

    impl #enum_ident {
      #[doc = #roster_doc]
      ///
      /// A slice rather than an array: how many codecs this build carries
      /// is a fact about the vendored FFmpeg table, not part of the type,
      /// so a regeneration that adds one stays a minor change.
      ///
      /// [`Self::Other`] is not a member. The roster answers "which names
      /// does this build know", and the escape is precisely the arm that
      /// carries a name it does not.
      pub const ROSTER: &'static [Self] = &[#(#roster_entries),*];
    }

    // The witness that `ROSTER` above is complete: a regeneration that
    // adds a codec makes this `match` non-exhaustive, and the compiler
    // names the variant missing from the roster.
    const _: () = {
      #[allow(dead_code)]
      fn every_variant_is_rostered(v: &#enum_ident) {
        match v {
          #(#witness_pats)|* => (),
          #enum_ident::Other(_) => (),
        }
      }
    };

    impl FromStr for #enum_ident {
      type Err = core::convert::Infallible;

      /// Recognise an FFmpeg codec short name, case-insensitively; unknown
      /// values land in [`Self::Other`] (infallible, lossless), carrying
      /// the caller's spelling verbatim.
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0u8; crate::parse::FOLD_CAP];
        let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
        Ok(match folded {
          #(#from_str_arms)*
          _ => Self::Other(Utf8Bytes::from(s)),
        })
      }
    }
  }
}

fn build_codec_tests(
  video: &CodecsWithProps,
  audio: &CodecsWithProps,
  subtitle: &CodecsWithProps,
  data: &CodecsWithProps,
  attachment: &CodecsWithProps,
) -> TokenStream {
  // Embed the (media_type, short_name) pairs as a const inside the
  // generated module rather than `include_str!`ing the workspace-only
  // `xtask/vendor/*.txt`. The vendored file lives in the `xtask` crate
  // which is excluded from `cargo publish`, so any `include_str!` that
  // traverses `../..` would break on the packaged source. Embedding
  // keeps the in-crate test suite hermetic. `attachment`'s pairs are not
  // FFmpeg-vendored (see `ATTACHMENT_CODECS`'s doc comment) but are
  // embedded the same way, so `vendored_of("attachment")` and the
  // roster-completeness check below work identically for all five.
  let pair_arms = video
    .keys()
    .map(|n| quote! { ("video", #n) })
    .chain(audio.keys().map(|n| quote! { ("audio", #n) }))
    .chain(subtitle.keys().map(|n| quote! { ("subtitle", #n) }))
    .chain(data.keys().map(|n| quote! { ("data", #n) }))
    .chain(attachment.keys().map(|n| quote! { ("attachment", #n) }));
  let (video_len, audio_len, subtitle_len, data_len, attachment_len) = (
    video.len(),
    audio.len(),
    subtitle.len(),
    data.len(),
    attachment.len(),
  );
  quote! {
    use super::*;
      // Bring `ToString` into scope explicitly. Under `feature = "std"`
      // the trait is in the prelude, but `--no-default-features
      // --features alloc` only has the core prelude — the codec module
      // is alloc-gated (see `lib.rs`), so the trait *exists*, it just
      // needs to be named. `lib.rs` aliases `extern crate alloc as
      // std` for the alloc-only build, so `::std::string::ToString`
      // resolves to the right path in either mode.
      use ::std::string::ToString;

      /// Every `(media_type, FFmpeg short name)` pair this module was
      /// generated from — embedded at codegen so the test suite stays
      /// self-contained when `mediaframe` is packaged for crates.io.
      const VENDORED_PAIRS: &[(&str, &str)] = &[#(#pair_arms,)*];

      fn vendored_of(media: &'static str) -> impl Iterator<Item = &'static str> {
        VENDORED_PAIRS
          .iter()
          .filter_map(move |(m, n)| (*m == media).then_some(*n))
      }

      #[test]
      fn every_video_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("video") {
          let c: VideoCodec = name.parse().unwrap();
          assert!(!c.is_other(), "video `{name}` should parse to a named variant");
          assert_eq!(c.as_str(), name, "round-trip mismatch for `{name}`");
          n += 1;
        }
        assert!(n > 0, "vendored video list is empty?");
      }

      #[test]
      fn every_audio_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("audio") {
          let c: AudioCodec = name.parse().unwrap();
          assert!(!c.is_other(), "audio `{name}` should parse to a named variant");
          assert_eq!(c.as_str(), name);
          n += 1;
        }
        assert!(n > 0);
      }

      #[test]
      fn every_subtitle_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("subtitle") {
          let c: SubtitleCodec = name.parse().unwrap();
          assert!(!c.is_other(), "subtitle `{name}` should parse to a named variant");
          assert_eq!(c.as_str(), name);
          n += 1;
        }
        assert!(n > 0);
      }

      #[test]
      fn every_data_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("data") {
          let c: DataCodec = name.parse().unwrap();
          assert!(!c.is_other(), "data `{name}` should parse to a named variant");
          assert_eq!(c.as_str(), name, "round-trip mismatch for `{name}`");
          n += 1;
        }
        assert!(n > 0, "vendored data list is empty?");
      }

      #[test]
      fn every_attachment_codec_round_trips_to_named_variant() {
        let mut n = 0usize;
        for name in vendored_of("attachment") {
          let c: AttachmentCodec = name.parse().unwrap();
          assert!(!c.is_other(), "attachment `{name}` should parse to a named variant");
          assert_eq!(c.as_str(), name, "round-trip mismatch for `{name}`");
          n += 1;
        }
        assert!(n > 0, "ATTACHMENT_CODECS is empty?");
      }

      /// `ttf`, `otf`, and `bin_data` are the three codec ids `DataCodec`
      /// and `AttachmentCodec` share — the same FFmpeg codec id wearing
      /// two different track-role hats (see `ATTACHMENT_CODECS`'s doc
      /// comment). Confirms the overlap is real rather than one enum
      /// silently missing a name the other carries.
      #[test]
      fn attachment_codecs_are_also_named_data_codecs() {
        for name in vendored_of("attachment") {
          let a: AttachmentCodec = name.parse().unwrap();
          let d: DataCodec = name.parse().unwrap();
          assert!(!a.is_other());
          assert!(!d.is_other(), "`{name}` should also be a named DataCodec variant");
          assert_eq!(a.as_str(), d.as_str());
        }
      }

      #[test]
      fn unknown_codec_preserves_string_through_other() {
        let v: VideoCodec = "definitely_not_a_real_codec_xyz".parse().unwrap();
        assert!(v.is_other());
        assert_eq!(v.as_str(), "definitely_not_a_real_codec_xyz");
      }

      /// Every codec name is lowercase-canonical and no two collide once
      /// folded — the precondition that makes the case-insensitive lookup
      /// a function rather than a coin flip.
      #[test]
      fn codec_names_are_lowercase_canonical_and_fold_without_collision() {
        for (media, name) in VENDORED_PAIRS {
          assert!(
            !name.bytes().any(|b| b.is_ascii_uppercase()),
            "{media} codec `{name}` is not lowercase-canonical"
          );
          // The vendored list is sorted and deduplicated per media type,
          // and every entry is already lowercase, so distinctness there is
          // distinctness after folding.
          let same: usize = VENDORED_PAIRS
            .iter()
            .filter(|(m, n)| m == media && n.eq_ignore_ascii_case(name))
            .count();
          assert_eq!(same, 1, "{media} has two codecs spelled `{name}`");
        }
      }

      /// The lookup folds, but the escape does not: an uppercase spelling
      /// of a known codec is that codec (`Self::other` runs the same
      /// ignore-case match `FromStr` does, so the two can never diverge),
      /// while an uppercase spelling of an unknown one keeps its own
      /// spelling verbatim — the escape is a lossless passthrough, not a
      /// fold target.
      /// `SubtitleCodec`, `DataCodec`, and `AttachmentCodec` are the open
      /// enums on the `Unwrap` / `TryUnwrap` pair (see the module doc's
      /// derive threshold); the two 200-plus-variant codec enums stay
      /// exempt on compile-time grounds, which is why this names those
      /// three and not `VideoCodec` / `AudioCodec`.
      #[test]
      fn subtitle_codec_unwrap_other_borrowed_view() {
        let v = SubtitleCodec::other("vendor_sub");
        assert_eq!(v.unwrap_other_ref().as_str(), "vendor_sub");
        assert!(v.try_unwrap_other_ref().is_ok());
        assert!(SubtitleCodec::Srt.try_unwrap_other_ref().is_err());
      }

      #[test]
      fn data_codec_unwrap_other_borrowed_view() {
        let v = DataCodec::other("vendor_data");
        assert_eq!(v.unwrap_other_ref().as_str(), "vendor_data");
        assert!(v.try_unwrap_other_ref().is_ok());
        assert!(DataCodec::BinData.try_unwrap_other_ref().is_err());
      }

      #[test]
      fn attachment_codec_unwrap_other_borrowed_view() {
        let v = AttachmentCodec::other("vendor_attachment");
        assert_eq!(v.unwrap_other_ref().as_str(), "vendor_attachment");
        assert!(v.try_unwrap_other_ref().is_ok());
        assert!(AttachmentCodec::Ttf.try_unwrap_other_ref().is_err());
      }

      /// `Self::other` runs the ignore-case parse first: a canonical
      /// short name (any case) returns the **named** variant, never a
      /// second `Other` value for a meaning this vocabulary already
      /// names — the equality-heals fixture that motivated this whole
      /// escape hatch.
      #[test]
      fn other_resolves_a_canonical_name_to_the_named_variant() {
        assert_eq!(VideoCodec::other("h264"), VideoCodec::H264);
        assert_eq!(VideoCodec::other("H264"), VideoCodec::H264);
        assert_eq!(VideoCodec::other("HeVc"), VideoCodec::Hevc);
        assert_eq!(AudioCodec::other("AAC"), AudioCodec::Aac);
        assert_eq!(SubtitleCodec::other("SRT"), SubtitleCodec::Srt);
        assert_eq!(DataCodec::other("KLV"), DataCodec::Klv);
        assert_eq!(AttachmentCodec::other("OTF"), AttachmentCodec::Otf);
      }

      #[test]
      fn codec_lookup_folds_but_the_escape_preserves_spelling() {
        assert_eq!("H264".parse(), Ok(VideoCodec::H264));
        assert_eq!("HeVc".parse(), Ok(VideoCodec::Hevc));
        assert_eq!("AAC".parse(), Ok(AudioCodec::Aac));
        assert_eq!("SRT".parse(), Ok(SubtitleCodec::Srt));
        assert_eq!("KLV".parse(), Ok(DataCodec::Klv));
        assert_eq!("OTF".parse(), Ok(AttachmentCodec::Otf));

        // A genuine stranger's spelling survives verbatim — round-trips
        // through `as_str`, and `Self::other` agrees with `FromStr`.
        let v: VideoCodec = "VENDOR_Codec".parse().unwrap();
        assert!(v.is_other());
        assert_eq!(v.as_str(), "VENDOR_Codec");
        assert_eq!(VideoCodec::other("VENDOR_Codec"), v);

        // Two spellings of the same stranger are two distinct `Other`
        // values now — nothing folds them together, unlike a name this
        // vocabulary actually knows.
        assert_ne!("vendor_codec".parse::<VideoCodec>().unwrap(), v);
      }

      #[test]
      fn subtitle_image_based_set_matches_ffmpeg() {
        for n in ["dvb_subtitle", "hdmv_pgs_subtitle", "dvd_subtitle", "xsub"] {
          let c: SubtitleCodec = n.parse().unwrap();
          assert_eq!(c.is_image_based(), Some(true), "`{n}` should be image-based");
        }
        for n in ["subrip", "ass", "ssa", "webvtt", "mov_text", "ttml", "microdvd"] {
          let c: SubtitleCodec = n.parse().unwrap();
          assert_eq!(c.is_image_based(), Some(false), "`{n}` should NOT be image-based");
        }
      }

      #[test]
      fn subtitle_image_based_is_unknown_for_other() {
        // `Other(_)` round-trips the string but isn't in the vendored
        // FFmpeg `.props` table — caller can't classify it as text or
        // bitmap without a fresh `cargo xtask sync` + `gen-codec`.
        let c: SubtitleCodec = "not_a_real_subtitle_codec_zzz".parse().unwrap();
        assert!(c.is_other());
        assert_eq!(c.is_image_based(), None);
      }

      /// Each `ROSTER` is exactly the vendored name list for its media
      /// type: same length, same order, no repeats, and every entry
      /// round-trips through its own slug. The completeness half is the
      /// `match` witness beside each declaration — a codec added by a
      /// regeneration cannot reach the roster without passing `E0004`
      /// first, and cannot reach the *right place* in it without matching
      /// the vendored order asserted here. `AttachmentCodec` goes through
      /// the identical helper — `vendored_of("attachment")` walks
      /// `ATTACHMENT_CODECS`' embedded pairs rather than a `codec_desc.c`
      /// table, but the completeness contract this asserts is the same.
      #[test]
      fn rosters_match_the_vendored_tables() {
        fn check<T>(roster: &'static [T], media: &'static str, expected_len: usize)
        where
          T: ::core::str::FromStr + ::core::fmt::Debug + ::core::fmt::Display + PartialEq,
          T::Err: ::core::fmt::Debug,
        {
          assert_eq!(roster.len(), expected_len, "{media} roster length");
          for (entry, name) in roster.iter().zip(vendored_of(media)) {
            assert_eq!(
              entry.to_string(),
              name,
              "{media} roster is out of declaration order at `{name}`"
            );
            assert_eq!(
              &name.parse::<T>().unwrap(),
              entry,
              "{media} roster entry `{name}` does not round-trip"
            );
          }
        }

        // `VENDORED_PAIRS` is sorted and deduplicated per media type (the
        // generator builds it from a `BTreeMap`), and the enum is emitted
        // from that same map, so equal length plus pairwise-equal slugs is
        // both an order check and a no-duplicates check.
        check::<VideoCodec>(VideoCodec::ROSTER, "video", #video_len);
        check::<AudioCodec>(AudioCodec::ROSTER, "audio", #audio_len);
        check::<SubtitleCodec>(SubtitleCodec::ROSTER, "subtitle", #subtitle_len);
        check::<DataCodec>(DataCodec::ROSTER, "data", #data_len);
        check::<AttachmentCodec>(AttachmentCodec::ROSTER, "attachment", #attachment_len);
      }

      /// No roster carries the open escape — it holds names this build
      /// knows, and `Other` is the arm for one it does not.
      #[test]
      fn rosters_exclude_the_escape() {
        assert!(VideoCodec::ROSTER.iter().all(|c| !c.is_other()));
        assert!(AudioCodec::ROSTER.iter().all(|c| !c.is_other()));
        assert!(SubtitleCodec::ROSTER.iter().all(|c| !c.is_other()));
        assert!(DataCodec::ROSTER.iter().all(|c| !c.is_other()));
        assert!(AttachmentCodec::ROSTER.iter().all(|c| !c.is_other()));
      }

      #[test]
      fn display_matches_as_str() {
        assert_eq!(VideoCodec::H264.to_string(), "h264");
        assert_eq!(AudioCodec::Opus.to_string(), "opus");
        assert_eq!(SubtitleCodec::Webvtt.to_string(), "webvtt");
        assert_eq!(DataCodec::Klv.to_string(), "klv");
        assert_eq!(AttachmentCodec::BinData.to_string(), "bin_data");
        assert_eq!(
          VideoCodec::Other(Utf8Bytes::from("custom_codec")).to_string(),
          "custom_codec"
        );
      }
  }
}
