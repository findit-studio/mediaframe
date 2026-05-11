//! videoframe xtask — dev-only automation.
//!
//! Subcommands:
//! - `cargo xtask sync` — fetches FFmpeg's `libavutil/pixfmt.h` at the
//!   pinned release tag, extracts every `AV_PIX_FMT_<NAME>` identifier,
//!   and writes the lowercase slug list to
//!   `xtask/vendor/ffmpeg-pixfmts.txt`. Requires `curl` on `PATH`.
//! - `cargo xtask check` — reads the vendored slug list and the
//!   `pub const fn as_str(&self) -> &'static str` match in
//!   `src/pixel_format.rs`, then reports the diff:
//!   - **missing-from-videoframe**: FFmpeg has it, we don't. Fails CI.
//!   - **videoframe extras**: we have a slug FFmpeg doesn't (cinema-RAW
//!     etc.). Informational only.
//!
//! The vendored file is a plain text list of identifier slugs (not the
//! FFmpeg header verbatim), which sidesteps the LGPL question that
//! would apply to checking in `pixfmt.h` itself.

use std::{
  collections::BTreeSet,
  fs,
  path::{Path, PathBuf},
  process::{Command, ExitCode},
};

/// FFmpeg release tag pinned for the check. Bump deliberately when you
/// want to sync against a newer FFmpeg.
const FFMPEG_TAG: &str = "n8.1";

/// Path (relative to the videoframe workspace root) of the vendored
/// slug list.
const VENDOR_PATH: &str = "xtask/vendor/ffmpeg-pixfmts.txt";

/// Path (relative to the workspace root) of the PixelFormat source
/// file whose `as_str()` table is the source of truth for our slugs.
const PIXEL_FORMAT_RS: &str = "src/pixel_format.rs";

fn main() -> ExitCode {
  let cmd = std::env::args()
    .nth(1)
    .unwrap_or_else(|| "help".to_string());
  match cmd.as_str() {
    "check" | "check-pixel-format" => check(),
    "sync" | "sync-pixel-format" => sync(),
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
    "videoframe xtask\n\n\
         Subcommands:\n  \
         check    Diff PixelFormat slugs against vendored FFmpeg list ({VENDOR_PATH})\n  \
         sync     Fetch latest FFmpeg pixfmt.h from {FFMPEG_TAG} and refresh the vendored list\n  \
         help     Show this help\n"
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

fn check() -> ExitCode {
  let root = workspace_root();
  let vendor = match fs::read_to_string(root.join(VENDOR_PATH)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {VENDOR_PATH}: {e}");
      eprintln!("hint:  run `cargo xtask sync` first to populate the vendored list");
      return ExitCode::FAILURE;
    }
  };
  let pf_rs = match fs::read_to_string(root.join(PIXEL_FORMAT_RS)) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {PIXEL_FORMAT_RS}: {e}");
      return ExitCode::FAILURE;
    }
  };

  let ffmpeg = parse_vendored(&vendor);
  let videoframe = parse_as_str_slugs(&pf_rs);

  let missing_from_videoframe: BTreeSet<_> = ffmpeg.difference(&videoframe).cloned().collect();
  let extras_in_videoframe: BTreeSet<_> = videoframe.difference(&ffmpeg).cloned().collect();

  println!("FFmpeg pinned: {FFMPEG_TAG}");
  println!("FFmpeg slugs  : {}", ffmpeg.len());
  println!("videoframe    : {} known slugs", videoframe.len());
  println!();

  if !extras_in_videoframe.is_empty() {
    println!(
      "  videoframe extras (slugs not in FFmpeg {FFMPEG_TAG} — cinema-RAW etc.): {}",
      extras_in_videoframe.len()
    );
    for s in &extras_in_videoframe {
      println!("    {s}");
    }
    println!();
  }

  if missing_from_videoframe.is_empty() {
    println!("OK: every FFmpeg {FFMPEG_TAG} pixel format is covered by videoframe.");
    ExitCode::SUCCESS
  } else {
    eprintln!(
      "FAIL: {} FFmpeg pixel format(s) missing from videoframe::PixelFormat:",
      missing_from_videoframe.len()
    );
    for s in &missing_from_videoframe {
      eprintln!("    {s}");
    }
    eprintln!(
      "\nAction: add the missing variants to `PixelFormat`,\n  \
             extend `as_str()` and the `to_u32`/`from_u32` tables."
    );
    ExitCode::FAILURE
  }
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
  ExitCode::SUCCESS
}

/// Hardware-frame markers — FFmpeg pixel formats whose buffers live
/// in GPU memory. videoframe intentionally excludes these per the
/// `pixel_format` module docs: a frame carrying GPU-resident buffers
/// must be transferred to a CPU format before reaching a videoframe
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
