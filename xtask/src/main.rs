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
//!     `src/pixel_format.rs` and diffs slugs —
//!     **missing-from-mediaframe** (FFmpeg has it, we don't) fails CI;
//!     **mediaframe extras** (cinema-RAW etc.) are informational.
//!   - Colour enums: reads the `to_u32()` matches in `src/color.rs`
//!     and asserts every distinct FFmpeg colour code has a named
//!     mediaframe variant mapping to that value (and the covering
//!     variant's id is `< DOMAIN_EXT_BASE` — the FFmpeg ingest path
//!     never yields a mediaframe-domain variant). A missing code
//!     fails CI. mediaframe-domain variants (id `>= DOMAIN_EXT_BASE`,
//!     e.g. `ColorMatrix::Bt601`, which H.273 / FFmpeg does not
//!     enumerate) are exempt from FFmpeg coverage and additionally
//!     asserted disjoint from the vendored FFmpeg colour codes.
//!
//! The vendored files are plain text (not the FFmpeg header verbatim),
//! which sidesteps the LGPL question that would apply to checking in
//! `pixfmt.h` itself.

use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::{Path, PathBuf},
  process::{Command, ExitCode},
};

/// FFmpeg release tag pinned for the check. Bump deliberately when you
/// want to sync against a newer FFmpeg.
const FFMPEG_TAG: &str = "n8.1";

/// Path (relative to the mediaframe workspace root) of the vendored
/// slug list.
const VENDOR_PATH: &str = "xtask/vendor/ffmpeg-pixfmts.txt";

/// Path (relative to the workspace root) of the vendored colour-enum
/// table (`<ENUM> <FFMPEG_NAME> <VALUE>` per line).
const COLOR_VENDOR_PATH: &str = "xtask/vendor/ffmpeg-color.txt";

/// Path (relative to the workspace root) of the PixelFormat source
/// file whose `as_str()` table is the source of truth for our slugs.
const PIXEL_FORMAT_RS: &str = "mediaframe/src/pixel_format.rs";

/// Path (relative to the workspace root) of the colour-enum source
/// file whose `to_u32()` matches are the source of truth.
const COLOR_RS: &str = "mediaframe/src/color.rs";

/// Path (relative to the workspace root) of the vendored codec-name
/// table. Format: one `<media_type> <name> [<props>]` per line, sorted;
/// `<props>` is an optional comma-separated list of `AV_CODEC_PROP_*`
/// tokens (prefix stripped) and is omitted entirely when FFmpeg's
/// `codec_desc.c` has no `.props` initializer for the codec. See the
/// header inside the file itself for the canonical format.
const CODEC_VENDOR_PATH: &str = "xtask/vendor/ffmpeg-codecs.txt";

/// Path (relative to the workspace root) of the codec-enum source file
/// whose `as_str()` matches are the source of truth.
const CODEC_RS: &str = "mediaframe/src/codec.rs";

/// The mediaframe codec enums and their corresponding FFmpeg media
/// type (`AVMEDIA_TYPE_*`, lowercased).
const CODEC_ENUMS: &[(&str, &str)] = &[
  ("video", "VideoCodec"),
  ("audio", "AudioCodec"),
  ("subtitle", "SubtitleCodec"),
];

/// The five FFmpeg colour C enums to parse, paired with the
/// `AVCOL_*` / `AVCHROMA_*` prefix to strip and the mediaframe
/// enum name whose `to_u32()` match maps it.
const COLOR_ENUMS: &[(&str, &str, &str)] = &[
  ("AVColorPrimaries", "AVCOL_PRI_", "ColorPrimaries"),
  (
    "AVColorTransferCharacteristic",
    "AVCOL_TRC_",
    "ColorTransfer",
  ),
  ("AVColorSpace", "AVCOL_SPC_", "ColorMatrix"),
  ("AVColorRange", "AVCOL_RANGE_", "ColorRange"),
  ("AVChromaLocation", "AVCHROMA_LOC_", "ChromaLocation"),
];

fn main() -> ExitCode {
  let cmd = std::env::args()
    .nth(1)
    .unwrap_or_else(|| "help".to_string());
  match cmd.as_str() {
    "check" | "check-pixel-format" | "check-codec" => check(),
    "sync" | "sync-pixel-format" | "sync-codec" => sync(),
    "gen-codec" => gen_codec(),
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
         check    Verify mediaframe against vendored FFmpeg tables:\n           \
                    - PixelFormat slugs ({VENDOR_PATH})\n           \
                    - Colour-enum codes ({COLOR_VENDOR_PATH})\n           \
                    - Codec short names ({CODEC_VENDOR_PATH})\n  \
         sync       Fetch FFmpeg {FFMPEG_TAG} (pixfmt.h + codec_desc.c) and\n             \
                    regenerate the vendored files deterministically\n  \
         gen-codec  Regenerate mediaframe/src/codec.rs from the vendored\n             \
                    table ({CODEC_VENDOR_PATH}) via quote + prettyplease\n  \
         help       Show this help\n"
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
  if pf_ok && color_ok && codec_ok {
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
/// `as_str()`), parsed from `src/color.rs`. The reverse direction
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

  // Domain invariant (b): `ColorMatrix::Bt601` is a mediaframe-domain
  // concept — its id must be `>= DOMAIN_EXT_BASE` AND absent from the
  // vendored FFmpeg colour table (no domain/FFmpeg collision).
  let empty = BTreeMap::new();
  let cm_named = mediaframe.get("ColorMatrix").unwrap_or(&empty);
  match cm_named.get("Bt601") {
    None => {
      eprintln!(
        "FAIL: ColorMatrix::Bt601 not found in {COLOR_RS} to_u32() — \
                 it is a required mediaframe-domain variant."
      );
      ok = false;
    }
    Some(nc) => {
      if nc.value < domain_base {
        eprintln!(
          "FAIL: ColorMatrix::Bt601.to_u32() = {} must be >= \
                   DOMAIN_EXT_BASE ({domain_base}) — it is a \
                   mediaframe-domain concept, not an FFmpeg code.",
          nc.value
        );
        ok = false;
      }
      let cm_ff = ffmpeg.get("ColorMatrix").cloned().unwrap_or_default();
      if cm_ff.contains_key(&nc.value) {
        eprintln!(
          "FAIL: ColorMatrix::Bt601 id {} collides with a vendored \
                   FFmpeg ColorMatrix code — domain ids must be disjoint.",
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
             ColorMatrix::Bt601) are exempt from FFmpeg coverage and \
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
/// The `Other(SmolStr)` arm is intentionally exempt from coverage —
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

  // Stage 0: prop-token whitelist. The third vendored column carries
  // `AV_CODEC_PROP_*` tokens (prefix stripped); reject any token outside
  // FFmpeg n8.1's `codec.h` enumeration so an edit like
  // `subtitle ass TEXT_SUB,BOGUS_PROP` fails CI instead of silently
  // sneaking past — the generator only consumes BITMAP_SUB today, so
  // without this gate non-BITMAP token corruption is invisible to the
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
  let mediaframe = parse_codec_named_strings(&codec_rs);

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
    // codecs would silently parse to `Other(SmolStr)` and `is_*` predicates
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
                  separate FFmpeg codec ID (drop the named variant — `Other(SmolStr)` \
                  still round-trips its string)."
      );
      ok = false;
    }

    if !missing_from_mediaframe.is_empty() {
      eprintln!(
        "FAIL: {} FFmpeg {FFMPEG_TAG} `{media_type}` codec(s) NOT covered by mediaframe \
             `{enum_name}` (would silently fall through to `Other(SmolStr)`):",
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

  // Stage 2: generation freshness. Build the codec module the same way
  // `gen-codec` would and diff against the on-disk file — catches edits to
  // the vendored table that haven't been propagated through `gen-codec`,
  // even when the variant-coverage check happens to pass (e.g. variant
  // ordering changes, `BITMAP_SUBTITLES` updates).
  match build_codec_rs(&root) {
    Ok(expected) => {
      let actual = fs::read_to_string(root.join(CODEC_RS)).unwrap_or_default();
      if expected != actual {
        eprintln!(
          "FAIL: {CODEC_RS} is stale vs the vendored FFmpeg table — \
                 run `cargo xtask gen-codec` to refresh it."
        );
        ok = false;
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
    CODEC_ENUMS.len()
  );
  if ok {
    println!(
      "OK: mediaframe codec enums and FFmpeg {FFMPEG_TAG} are in two-way sync \
       (and {CODEC_RS} is up-to-date)."
    );
  }
  ok
}

/// Every `AV_CODEC_PROP_*` token FFmpeg n8.1
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

/// Walk `xtask/vendor/ffmpeg-codecs.txt` and report any third-column
/// token that isn't in [`KNOWN_CODEC_PROPS`]. Returns the offending
/// `(line_no, line, token)` triples so the caller can report all
/// mismatches in one shot. Empty `Ok(())` = the prop column is clean.
fn validate_vendored_props(text: &str) -> Result<(), Vec<(usize, String, String)>> {
  let known: BTreeSet<&str> = KNOWN_CODEC_PROPS.iter().copied().collect();
  let mut bad: Vec<(usize, String, String)> = Vec::new();
  for (i, raw) in text.lines().enumerate() {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let mut it = line.split_whitespace();
    // Skip media_type + name; props (if any) sit in the third column.
    if it.next().is_none() || it.next().is_none() {
      continue;
    }
    let Some(props) = it.next() else { continue };
    for tok in props.split(',').filter(|t| !t.is_empty()) {
      if !known.contains(tok) {
        bad.push((i + 1, line.to_string(), tok.to_string()));
      }
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

/// Parse the three `mediaframe::codec::<Enum>::as_str()` match blocks and
/// emit `enum-name → { variant-ident → canonical-short-string }`. The
/// `Self::Other(s) => s.as_str()` arm is skipped.
fn parse_codec_named_strings(rs: &str) -> BTreeMap<String, BTreeMap<String, String>> {
  let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
  for (_, enum_name) in CODEC_ENUMS {
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

/// Parse the `as_str(&self) -> &'static str` match block in
/// `src/pixel_format.rs`, extracting every literal slug.
/// Excludes the `unknown` sentinel.
fn parse_as_str_slugs(rs: &str) -> BTreeSet<String> {
  let mut out = BTreeSet::new();
  // Lines look like:   Self::Yuv420p => "yuv420p",
  //               or:  Self::Unknown(_) => "unknown",
  for line in rs.lines() {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("Self::") {
      // Find the => and then the "..." literal.
      if let Some(arrow) = rest.find("=>") {
        let after = &rest[arrow + 2..].trim_start();
        if let Some(slug) = extract_first_string_literal(after)
          && slug != "unknown"
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
/// `as_str()`. The `Unknown(v) => *v` passthrough and the
/// `Unknown(_) => "unknown"` sentinel are excluded.
struct NamedCode {
  value: u32,
  /// `true` iff the matching `as_str()` arm yields a non-empty slug.
  has_slug: bool,
}

/// Parse the per-enum `as_str()` + `to_u32()` match blocks in
/// `src/color.rs`. Returns `mediaframe-enum -> { variant-ident ->
/// NamedCode }`. Implementation is line-oriented (matching the
/// existing `parse_as_str_slugs` style): an `impl <Enum> {` opens a
/// scope that the next `impl `/`pub enum `/`pub struct ` closes;
/// inside, `Self::<ident> => <int>,` arms seen after the
/// `fn to_u32(` line are values and `Self::<ident> => "..."` arms
/// after the `fn as_str(` line are slugs.
/// Parse the `pub const DOMAIN_EXT_BASE: u32 = <lit>;` line from
/// `src/color.rs` (the mediaframe-domain colour-id base; ids `>=`
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
    if rest.starts_with("Unknown") {
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

  let slugs = extract_avpixfmt_names(&header);
  if slugs.is_empty() {
    eprintln!("error: parsed 0 AV_PIX_FMT_* identifiers from the fetched header — parse bug?");
    return ExitCode::FAILURE;
  }

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

  ExitCode::SUCCESS
}

/// Hardware-frame markers — FFmpeg pixel formats whose buffers live
/// in GPU memory. mediaframe intentionally excludes these per the
/// `pixel_format` module docs: a frame carrying GPU-resident buffers
/// must be transferred to a CPU format before reaching a mediaframe
/// consumer.
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
  "xvmc",
];

fn extract_avpixfmt_names(header: &str) -> BTreeSet<String> {
  let mut out = BTreeSet::new();
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
      if HW_FORMAT_SLUGS.contains(&slug.as_str()) {
        continue;
      }
      out.insert(slug);
    }
  }
  out
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
    } else if let Some(rest) = line.strip_prefix(".name") {
      if let Some(eq) = rest.find('=') {
        let after_eq = &rest[eq + 1..];
        if let Some(start) = after_eq.find('"') {
          let inner = &after_eq[start + 1..];
          if let Some(end) = inner.find('"') {
            current_name = Some(inner[..end].to_string());
          }
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
// Regenerate `mediaframe/src/codec.rs` from `xtask/vendor/ffmpeg-codecs.txt`
// using the same `quote!` / `proc-macro2` / `prettyplease` toolchain proc-
// macros use. Single source of truth (the vendored table) → single
// generated module; `cargo xtask check` is the CI gate against drift.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

fn gen_codec() -> ExitCode {
  let root = workspace_root();
  let (counts, content) = match build_codec_rs_with_counts(&root) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("{e}");
      return ExitCode::FAILURE;
    }
  };
  let out_path = root.join(CODEC_RS);
  if let Err(e) = fs::write(&out_path, &content) {
    eprintln!("error: cannot write {}: {e}", out_path.display());
    return ExitCode::FAILURE;
  }
  let (v, a, s) = counts;
  println!(
    "Wrote {} codec variants ({} video + {} audio + {} subtitle) to {} ({} bytes)",
    v + a + s,
    v,
    a,
    s,
    out_path.display(),
    content.len()
  );
  ExitCode::SUCCESS
}

/// Build the **expected** content of `mediaframe/src/codec.rs` from
/// `xtask/vendor/ffmpeg-codecs.txt`. Used both by `gen-codec` (writes the
/// result to disk) and by `check_codec`'s freshness check (compares the
/// result to the on-disk file).
///
/// Pipeline: vendor file → `BTreeSet` per media type → `quote!` `TokenStream`
/// → `syn::parse2` → `prettyplease::unparse` → `rustfmt --emit=stdout`. The
/// final `rustfmt` step is required for CI parity — `prettyplease` is
/// rustfmt-adjacent but block-wraps long match arms and renders multi-line
/// docs as `/** */`, neither of which survives `cargo fmt --check`.
fn build_codec_rs(root: &Path) -> Result<String, String> {
  build_codec_rs_with_counts(root).map(|(_, s)| s)
}

fn build_codec_rs_with_counts(root: &Path) -> Result<((usize, usize, usize), String), String> {
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

  let module = build_codec_module(&video, &audio, &subtitle);
  let parsed: syn::File = syn::parse2(module)
    .map_err(|e| format!("internal error: generated token stream is not parseable: {e}"))?;
  let pretty = prettyplease::unparse(&parsed);
  let formatted = run_rustfmt(&pretty)?;
  Ok(((video.len(), audio.len(), subtitle.len()), formatted))
}

/// Pipe `source` through `rustfmt --edition=2024 --emit=stdout` and return
/// the formatted result. Going via stdin/stdout (not a file) lets the
/// generator stay side-effect-free for `check_codec`'s freshness diff.
fn run_rustfmt(source: &str) -> Result<String, String> {
  use std::{io::Write, process::Stdio};

  let mut child = Command::new("rustfmt")
    .arg("--edition=2024")
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
  for seg in name.split(|c: char| c == '_' || c == '.') {
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
) -> TokenStream {
  let video_enum = build_codec_enum("VideoCodec", "video", video, false);
  let audio_enum = build_codec_enum("AudioCodec", "audio", audio, false);
  let subtitle_enum = build_codec_enum("SubtitleCodec", "subtitle", subtitle, true);

  // Module-level docs (inner-doc attributes — `prettyplease` renders them
  // as `//!` lines on output).
  let module_doc: Vec<TokenStream> = [
    " Stream-descriptor **codec** vocabulary for video, audio, and subtitle",
    " tracks.",
    "",
    " **Generated** from `xtask/vendor/ffmpeg-codecs.txt` (FFmpeg n8.1",
    " `libavcodec/codec_desc.c`) by `cargo xtask gen-codec`. Every codec",
    " FFmpeg knows under media types `video` / `audio` / `subtitle` has",
    " a named variant here; the `Other(SmolStr)` arm remains a lossless",
    " escape for codecs added in a future FFmpeg release before this file",
    " is regenerated.",
    "",
    " Regenerate in two steps:",
    " 1. `cargo xtask sync`       — refreshes the vendored table.",
    " 2. `cargo xtask gen-codec`  — regenerates this file from it.",
    "",
    " `cargo xtask check` verifies every named variant's canonical string",
    " exists in the vendored table — CI gate against drift.",
  ]
  .iter()
  .map(|line| quote! { #![doc = #line] })
  .collect();

  let tests = build_codec_tests(video, audio, subtitle);

  quote! {
    #(#module_doc)*

    use core::str::FromStr;

    use derive_more::{Display, IsVariant};
    use smol_str::SmolStr;

    #video_enum

    #audio_enum

    #subtitle_enum

    #tests
  }
}

fn build_codec_enum(
  type_name: &str,
  media_type: &str,
  codecs: &CodecsWithProps,
  is_subtitle: bool,
) -> TokenStream {
  let enum_ident = format_ident!("{}", type_name);

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

  let from_str_arms = variants.iter().map(|(ident, name)| {
    quote! { #name => Self::#ident, }
  });

  let enum_doc = format!(
    " {} codec family — every codec FFmpeg n8.1 knows under media type `{}`.\n\n \
     `#[non_exhaustive]` keeps future additions non-breaking; the `Other(SmolStr)` \
     arm is the lossless escape for codecs added upstream before this file is \
     regenerated.",
    type_name.strip_suffix("Codec").unwrap_or(type_name),
    media_type
  );

  // Subtitle `is_image_based()` is sourced from the vendored `.props` set
  // (FFmpeg n8.1's `AV_CODEC_PROP_BITMAP_SUB` flag), not a hand-maintained
  // constant. Returns `Option<bool>`: `Some(true)` for bitmap subtitles,
  // `Some(false)` for known-text subtitles, `None` for `Other(_)` because
  // an unknown codec name has no FFmpeg `.props` record we can consult.
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
      " ({bitmap_count} bitmap / {non_bitmap_count} non-bitmap variant(s) per FFmpeg n8.1)."
    );
    quote! {
      /// Is this a **bitmap** (image-based) subtitle codec, requiring an
      /// OCR pipeline stage to extract searchable text?
      ///
      /// - `Some(true)`: matches FFmpeg's `AV_CODEC_PROP_BITMAP_SUB` flag.
      /// - `Some(false)`: a known FFmpeg subtitle codec without
      ///   `AV_CODEC_PROP_BITMAP_SUB` (text codecs and teletext/VBI-style
      ///   codecs that carry no `.props` at all in FFmpeg n8.1).
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
    #[display("{}", self.as_str())]
    #[non_exhaustive]
    pub enum #enum_ident {
      #(#variant_decls)*
      /// A codec not enumerated above — carries the FFmpeg short name
      /// verbatim.
      Other(SmolStr),
    }

    impl #enum_ident {
      /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
      pub fn as_str(&self) -> &str {
        match self {
          #(#as_str_arms)*
          Self::Other(s) => s.as_str(),
        }
      }

      #extra_impl
    }

    impl FromStr for #enum_ident {
      type Err = core::convert::Infallible;

      /// Recognise an FFmpeg codec short name; unknown values land in
      /// [`Self::Other`] (infallible, lossless).
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
          #(#from_str_arms)*
          other => Self::Other(SmolStr::new(other)),
        })
      }
    }
  }
}

fn build_codec_tests(
  video: &CodecsWithProps,
  audio: &CodecsWithProps,
  subtitle: &CodecsWithProps,
) -> TokenStream {
  // Embed the (media_type, short_name) pairs as a const inside the
  // generated module rather than `include_str!`ing the workspace-only
  // `xtask/vendor/*.txt`. The vendored file lives in the `xtask` crate
  // which is excluded from `cargo publish`, so any `include_str!` that
  // traverses `../..` would break on the packaged source. Embedding
  // keeps the in-crate test suite hermetic.
  let pair_arms = video
    .keys()
    .map(|n| quote! { ("video", #n) })
    .chain(audio.keys().map(|n| quote! { ("audio", #n) }))
    .chain(subtitle.keys().map(|n| quote! { ("subtitle", #n) }));
  quote! {
    #[cfg(test)]
    mod tests {
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
      fn unknown_codec_preserves_string_through_other() {
        let v: VideoCodec = "definitely_not_a_real_codec_xyz".parse().unwrap();
        assert!(v.is_other());
        assert_eq!(v.as_str(), "definitely_not_a_real_codec_xyz");
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

      #[test]
      fn display_matches_as_str() {
        assert_eq!(VideoCodec::H264.to_string(), "h264");
        assert_eq!(AudioCodec::Opus.to_string(), "opus");
        assert_eq!(SubtitleCodec::Webvtt.to_string(), "webvtt");
        assert_eq!(
          VideoCodec::Other(SmolStr::new("custom_codec")).to_string(),
          "custom_codec"
        );
      }
    }
  }
}
