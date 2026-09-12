//! Still-image vocabulary — standard photo formats plus the camera-RAW
//! family.
//!
//! mediaframe's other vocabularies cover video / audio / subtitle
//! streams; before this module no still-image formats existed anywhere
//! in the crate, so a consumer filtering by [`crate::container::Format`]
//! and [`crate::audio::ContainerFormat`] (the two container rosters) had
//! no way to recognise a photograph. This household is the fix — a flat
//! top-level module, the same shape as [`crate::container`] and
//! [`crate::audio`], not nested under either.
//!
//! # The RAW roster is a curated cut, not ExifTool's full list
//!
//! ExifTool recognises several dozen camera-RAW extensions (one or more
//! per manufacturer generation). [`Format`]'s RAW variants are a
//! deliberately curated subset — current or historically substantial
//! single-vendor formats — with the long tail left to the
//! [`Format::Other`] escape. Named:
//!
//! - **Adobe DNG** ([`Format::Dng`]) — the open, cross-vendor standard.
//! - **Canon** — [`Format::Cr2`], [`Format::Cr3`].
//! - **Nikon** — [`Format::Nef`], [`Format::Nrw`] (the compact-camera
//!   second extension).
//! - **Sony** — [`Format::Arw`].
//! - **Olympus / OM System** — [`Format::Orf`] (the `.ori` spelling is
//!   an extension alias, not a second format — see [`Format::extensions`]).
//! - **Panasonic** — [`Format::Rw2`].
//! - **Fujifilm** — [`Format::Raf`].
//! - **Pentax** — [`Format::Pef`].
//! - **Samsung** — [`Format::Srw`].
//! - **Leica** — [`Format::Rwl`].
//! - **Phase One** — [`Format::Iiq`] (professional medium format).
//! - **Hasselblad** — [`Format::Threefr`] (`.3fr`; see the variant's own
//!   doc for the digit-leading-name note).
//! - **Sigma** — [`Format::X3f`] (Foveon sensor).
//! - **Minolta** — [`Format::Mrw`] (legacy, but real archival volume and
//!   the direct lineage ancestor of Sony's Alpha mount / [`Format::Arw`]).
//! - **GoPro** — [`Format::Gpr`] (DNG-based; current product).
//!
//! **Deliberately excluded** (ride [`Format::Other`] if ever met), with
//! reasons: `CRW` (Canon's pre-CR2 format, superseded); `ARQ` (Sony's
//! *derived* pixel-shift composite, not a sensor read-out); `SR2`/`SRF`
//! (Sony's older predecessors to `ARW`); `DCR`/`KDC` (Kodak — exited the
//! camera business); `MEF` (Mamiya, very low volume); `FFF`/`MOS`
//! (Hasselblad/Leaf legacy backs, very low volume); `CS1` (Sinar,
//! ultra-niche studio product); `K25` (1990s point-and-shoot antique);
//! `LRI` (Light Co., a discontinued product from a defunct company);
//! the bare `RAW` extension (ExifTool itself resolves it ambiguously —
//! Kyocera Contax N Digital *or* Panasonic — and the string would
//! collide with this doc's own "RAW family" heading); and the
//! space-containing `"Canon 1D RAW"` legacy special-case (not a real
//! filename extension). Two more are excluded for a different reason —
//! wrong domain, not low volume: `CRM` ("Canon RAW **Movie**") and
//! `R3D` (RedCode — cinema-camera video) are motion formats despite the
//! "RAW" name, not stills; `CZI` (Zeiss) is microscopy, not photography.
//!
//! # ffmpeg census
//!
//! ffmpeg (n9.0, this repo's pinned tag family) demuxes standard stills
//! through the generic `image2` / `*_pipe` family (`jpeg_pipe`,
//! `png_pipe`, `tiff_pipe`, `webp_pipe`, `gif_pipe`, `bmp_pipe`, …) —
//! there is no dedicated per-format demuxer the way `mov`/`mkv`/`webm`
//! each get one. HEIF/HEIC/AVIF are **not** in that pipe family at all:
//! they are ISOBMFF-boxed, so ffmpeg reads them through the shared
//! `mov,mp4,m4a,3gp,3g2,mj2` demuxer — the exact demuxer
//! [`crate::container::Format::Mov`] / [`crate::container::Format::Mp4`]
//! already name. This mirrors ExifTool's own table, where `CR3`
//! (`crate::image::Format::Cr3`) *also* resolves to base type `MOV` —
//! camera-RAW and modern still-image formats routinely ride a
//! video-container byte format while remaining, functionally, a single
//! still image. `Format` names them by their *content* (a photograph),
//! not by which demuxer happens to open the bytes; findit's walk keeps
//! offering files by extension regardless of which internal container
//! carries them.
//!
//! # ExifTool census (the `exifast` tree, read-only local oracle)
//!
//! `exifast`'s `filetype_data.rs` is a 1:1 port of ExifTool's own
//! `%fileTypeLookup`, including its canonical write-extension choices
//! (`ExifTool.pm:595` picks `"jpg"` for `JPEG`; `ExifTool.pm:598` picks
//! `"tif"` for `TIFF`) and its extension-alias table (`JPG`/`JPE` alias
//! `JPEG`; `TIF` aliases `TIFF`; `ORI` aliases `ORF`). ExifTool's own
//! `HIF` → `HEIF` alias is a static extension-to-FileType *fallback*
//! (`exifast` has not yet ported the brand-level `ftyp` parsing that
//! would let it classify a real file's actual content — `FORMATS.md`
//! marks still-image/RAW modules Stage 2, deferred — so this one entry
//! predates any byte-level check and is not treated as decisive; see
//! [`Format::Heic`]'s own doc for the byte-level evidence that overrides
//! it). [`Format::as_extension`] and [`Format::extensions`] otherwise
//! follow ExifTool's choices directly.
//!
//! **`Heic` and `Heif` are separate variants (R6), not one collapsed
//! variant.** R3 originally merged `HEIC`/`HEIF`/`HIF` onto one
//! [`Format::Heif`], reasoning ExifTool keeps them as "nearly identical"
//! top-level file types both based on `MOV`. That reasoning didn't
//! apply the module's own identical-bytes test: IANA's `image/heic` and
//! `image/heif` registrations gate the two subtypes on **mutually
//! exclusive required ISOBMFF brands** — `heif` requires the generic
//! `mif1` brand (any coding format); `heic` requires one of
//! `heic`/`heix`/`heim`/`heis` (HEVC-coded specifically) — a
//! structurally-signaled subtype distinction, the same shape as the
//! `Avif` vs `Heif` split this module already drew for exactly this
//! reason. See [`Format::Heic`]'s own doc for the full citation, and for
//! why `.hif` is excluded from both variants rather than routed to
//! either (R8).

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;

/// Still-image format — standard photo formats plus curated camera-RAW.
///
/// Closed-ish vocabulary, the same shape as [`crate::container::Format`]:
/// not FFmpeg-coded (no `to_u32`/`from_u32`), the [`Self::Other`] arm
/// preserves an unrecognised slug losslessly.
///
/// `as_str` returns the canonical **format-name** slug (`"jpeg"`,
/// `"heif"`, `"tiff"`, …) — this is the `Display` / `serde` wire and is
/// not always the same spelling as the on-disk file extension; see
/// [`Self::as_extension`] and [`Self::extensions`] for the extension
/// face, which is where the real multi-spelling aliasing lives
/// (`jpg`/`jpeg`, `tif`/`tiff`, `bmp`/`dib`, …). The same
/// name-vs-extension split already exists on the container siblings —
/// [`crate::container::Format::MpegTs`] renders `"mpegts"` but extends
/// `"ts"`; [`crate::container::Format::Ogg`] renders `"ogg"` but extends
/// `"ogv"` — this module just has more instances of it, because still-
/// image formats have more genuine multi-spelling extensions than video
/// containers do.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::image_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum Format {
  /// Joint Photographic Experts Group (canonical extension `.jpg`;
  /// `.jpeg` / `.jpe` are recognised aliases — see
  /// [`Self::extensions`]).
  Jpeg,
  /// Portable Network Graphics (`.png`).
  Png,
  /// High Efficiency Image Format — the **generic** ISOBMFF-boxed
  /// still-image container (`.heif`; no alias). Requires the `mif1`
  /// major/compatible brand per IANA's own `image/heif` registration:
  /// *"The MIME subtype name may be 'heif' only if the file conforms to
  /// the requirements of the 'mif1' brand"* — `mif1` places no
  /// constraint on the coding format inside, which is exactly what
  /// separates this variant from [`Self::Heic`]'s HEVC-specific brands.
  ///
  /// **No longer includes `.heic`/`.hif`** (R6 correction — see
  /// [`Self::Heic`]'s own doc for the byte-level evidence). R3 had
  /// collapsed `HEIC`/`HEIF`/`HIF` onto this one variant, reasoning
  /// ExifTool treats them as "nearly identical" file types — but never
  /// applied this module's own identical-bytes test to that reasoning;
  /// IANA's registrations gate `heif` vs `heic` on mutually exclusive
  /// required brands, the same structural-signal shape already used to
  /// keep [`Self::Avif`] a separate variant.
  Heif,
  /// High Efficiency Image Format, **HEVC-coded** — requires one of the
  /// `heic`/`heix`/`heim`/`heis` major/compatible brands per IANA's own
  /// `image/heic` registration (`.heic` is the common on-disk spelling;
  /// no alias — see below for `.hif`, which is deliberately **not**
  /// one).
  ///
  /// **Not [`Self::Heif`]** — an R6 correction (Codex R6 HIGH finding,
  /// same class as the R5 promotions, per the user's R5 甲 ruling
  /// applied directly: structurally-distinct formats get their own
  /// variant, no new consultation needed). IANA's `image/heic`
  /// registration: *"The MIME subtype name may be 'heic' only if the
  /// file conforms to the requirements of the 'heic', 'heix', 'heim',
  /// or 'heis' brand"* — a real, ISOBMFF-`ftyp`-box-encoded distinction
  /// from `Heif`'s generic `mif1` requirement, not a filename
  /// convention.
  ///
  /// **`.hif` is deliberately excluded from both `Heic` and `Heif`** —
  /// an R8 correction to R6's own work. R6 routed `.hif` here on the
  /// strength of Canon's real files (the dominant real-world `.hif`
  /// producer: EOS-1D X Mark III onward, HDR-PQ stills) carrying
  /// `major_brand = 'heix'` with compatible brands `['mif1', 'heix']`
  /// and HEVC-coded (`hvc1`-item) tile data, per independent
  /// reverse-engineering documentation of the real byte layout
  /// (`lclevy/canon_cr3/heif.md`: `` `ftyp (major_brand=b'heix', ...,
  /// [b'mif1', b'heix'])` ``). **That evidence is real, but it proves
  /// frequency, not totality**: IANA's own file-extension field says
  /// `.hif` names *either* subtype (`"hif (for subtypes heif and
  /// heic)"`) — Canon being the dominant producer means most `.hif`
  /// files in the wild are probably `heix`-branded, not that `.hif`
  /// *is* `heix`-branded by extension alone. A `FromStr` total mapping
  /// from `.hif` to one variant would claim the latter, which the spec
  /// itself contradicts — the identical-bytes law says an extension
  /// with two legitimately different possible byte layouts is
  /// ambiguous, full stop, the same reasoning that already keeps
  /// [`Self::Avif`]'s IANA-listed `heif`/`hif` spellings off `Avif`
  /// too. `.hif` therefore parses to [`Self::Other`], carrying its own
  /// name, rather than guessing. The Canon evidence stays recorded here
  /// because it's exactly what a **future content-aware door** would
  /// need — a byte-inspection tier that reads a real file's `ftyp` box
  /// and routes `.hif` by actual `major_brand` rather than by
  /// extension alone — but mediaframe has no such tier today (this
  /// crate parses text, not file content), so that door is named, not
  /// built.
  Heic,
  /// AV1 Image File Format (`.avif`) — kept distinct from
  /// [`Self::Heif`] / [`Self::Heic`] (ExifTool and every other oracle
  /// this module consulted keep it a separate file type; its own
  /// required brand, AV1-coded, inside the same family of ISOBMFF-boxed
  /// container — the same structural-signal shape that separates `Heic`
  /// from `Heif`, R6).
  Avif,
  /// Tagged Image File Format (canonical extension `.tif`; `.tiff` is
  /// a recognised alias — ExifTool's own canonical write extension is
  /// also `.tif`, `ExifTool.pm:598`). Camera RAW formats are routinely
  /// TIFF-structured at the container level but are **not** aliases of
  /// this variant — each RAW format keeps its own named variant below.
  Tiff,
  /// Google WebP (`.webp`).
  Webp,
  /// CompuServe Graphics Interchange Format (`.gif`).
  Gif,
  /// Windows Bitmap (canonical extension `.bmp`; `.dib` — Device
  /// Independent Bitmap, the byte-identical GDI in-memory form — is a
  /// recognised alias).
  Bmp,
  /// Adobe Digital Negative (`.dng`) — the one open, cross-vendor RAW
  /// standard; also the format [`Self::Gpr`] (GoPro) is built on.
  Dng,
  /// Canon RAW 2 (`.cr2`) — the long-running Canon DSLR RAW format,
  /// huge historical install base. The auto-derived predicate name
  /// would be digit-snake-case (`is_cr_2`); the hand-written
  /// [`Self::is_cr2`] uses the cleaner name.
  #[is_variant(ignore)]
  Cr2,
  /// Canon RAW 3 (`.cr3`) — current Canon mirrorless RAW, MOV-boxed.
  /// The auto-derived predicate name would be digit-snake-case
  /// (`is_cr_3`); the hand-written [`Self::is_cr3`] uses the cleaner
  /// name.
  #[is_variant(ignore)]
  Cr3,
  /// Nikon (RAW) Electronic Format (`.nef`).
  Nef,
  /// Nikon RAW, second extension — used on compacts and some DSLR
  /// bodies alongside [`Self::Nef`] (`.nrw`).
  Nrw,
  /// Sony Alpha RAW format (`.arw`).
  Arw,
  /// Olympus / OM System RAW format (`.orf`; `.ori` is a recognised
  /// alias — ExifTool's own table aliases `ORI` to `ORF` directly).
  Orf,
  /// Panasonic RAW 2 (`.rw2`). The auto-derived predicate name would
  /// be digit-snake-case (`is_rw_2`); the hand-written [`Self::is_rw2`]
  /// uses the cleaner name.
  #[is_variant(ignore)]
  Rw2,
  /// FujiFilm RAW Format (`.raf`).
  Raf,
  /// Pentax (RAW) Electronic Format (`.pef`).
  Pef,
  /// Samsung RAW format (`.srw`).
  Srw,
  /// Leica RAW (`.rwl`).
  Rwl,
  /// Phase One Intelligent Image Quality RAW (`.iiq`) — professional
  /// medium format.
  Iiq,
  /// Hasselblad RAW format (`.3fr`). **Variant naming note:** mirrors
  /// [`crate::container::Format::Threegp`] — Rust identifiers cannot
  /// start with a digit, so the variant spells out `Three`; the
  /// `as_str()` / `FromStr` surface still returns / matches the
  /// canonical `"3fr"` slug.
  Threefr,
  /// Sigma RAW format (`.x3f`) — Foveon-sensor cameras. The
  /// auto-derived predicate name would be digit-snake-case
  /// (`is_x_3f`); the hand-written [`Self::is_x3f`] uses the cleaner
  /// name.
  #[is_variant(ignore)]
  X3f,
  /// Minolta RAW format (`.mrw`) — legacy, but real archival volume
  /// and the direct lineage ancestor of Sony's Alpha-mount
  /// [`Self::Arw`].
  Mrw,
  /// GoPro's "General Purpose RAW" (`.gpr`) — DNG-based; current
  /// product.
  Gpr,
  /// A format not enumerated above — carries the extension-style slug
  /// verbatim. Lossless escape.
  Other(Utf8Bytes),
}

impl Default for Format {
  /// `Other("")` — the wire-zero / "absent" sentinel, matching every
  /// other open format vocabulary in the crate.
  #[inline]
  fn default() -> Self {
    Self::Other(Utf8Bytes::new())
  }
}

impl Format {
  /// True iff this is [`Self::Cr2`]. Hand-written to override the
  /// auto-derived `is_cr_2` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_cr2(&self) -> bool {
    matches!(self, Self::Cr2)
  }

  /// True iff this is [`Self::Cr3`]. Hand-written to override the
  /// auto-derived `is_cr_3` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_cr3(&self) -> bool {
    matches!(self, Self::Cr3)
  }

  /// True iff this is [`Self::Rw2`]. Hand-written to override the
  /// auto-derived `is_rw_2` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_rw2(&self) -> bool {
    matches!(self, Self::Rw2)
  }

  /// True iff this is [`Self::X3f`]. Hand-written to override the
  /// auto-derived `is_x_3f` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_x3f(&self) -> bool {
    matches!(self, Self::X3f)
  }

  /// Canonical format-name slug (`"jpeg"`, `"heif"`, `"tiff"`, `"cr2"`,
  /// …). This is the `Display` / `serde` wire — see the type doc for
  /// why it is not always the on-disk extension.
  pub fn as_str(&self) -> &str {
    match self {
      Self::Jpeg => "jpeg",
      Self::Png => "png",
      Self::Heif => "heif",
      Self::Heic => "heic",
      Self::Avif => "avif",
      Self::Tiff => "tiff",
      Self::Webp => "webp",
      Self::Gif => "gif",
      Self::Bmp => "bmp",
      Self::Dng => "dng",
      Self::Cr2 => "cr2",
      Self::Cr3 => "cr3",
      Self::Nef => "nef",
      Self::Nrw => "nrw",
      Self::Arw => "arw",
      Self::Orf => "orf",
      Self::Rw2 => "rw2",
      Self::Raf => "raf",
      Self::Pef => "pef",
      Self::Srw => "srw",
      Self::Rwl => "rwl",
      Self::Iiq => "iiq",
      Self::Threefr => "3fr",
      Self::X3f => "x3f",
      Self::Mrw => "mrw",
      Self::Gpr => "gpr",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Primary file-on-disk extension (without the leading dot —
  /// `"jpg"`, `"heic"`, `"tif"`, …) — the **first** entry of
  /// [`Self::extensions`], always.
  ///
  /// Returns `""` for [`Self::Other`] — the open variant carries a
  /// format-name slug, not an extension, so the mapping is unknown.
  /// Returns `&'static str` (not `&str`) so the value is compile-time
  /// stable and the method is `const`.
  #[inline(always)]
  pub const fn as_extension(&self) -> &'static str {
    match self {
      Self::Jpeg => "jpg",
      Self::Png => "png",
      Self::Heif => "heif",
      Self::Heic => "heic",
      Self::Avif => "avif",
      Self::Tiff => "tif",
      Self::Webp => "webp",
      Self::Gif => "gif",
      Self::Bmp => "bmp",
      Self::Dng => "dng",
      Self::Cr2 => "cr2",
      Self::Cr3 => "cr3",
      Self::Nef => "nef",
      Self::Nrw => "nrw",
      Self::Arw => "arw",
      Self::Orf => "orf",
      Self::Rw2 => "rw2",
      Self::Raf => "raf",
      Self::Pef => "pef",
      Self::Srw => "srw",
      Self::Rwl => "rwl",
      Self::Iiq => "iiq",
      Self::Threefr => "3fr",
      Self::X3f => "x3f",
      Self::Mrw => "mrw",
      Self::Gpr => "gpr",
      Self::Other(_) => "",
    }
  }

  /// Every recognised on-disk spelling for this format, canonical
  /// first (== [`Self::as_extension`]) and aliases after. [`FromStr`]
  /// accepts every entry, ignore-case; a walk collecting "every
  /// spelling this format might be saved under" should iterate this
  /// rather than call [`Self::as_extension`] alone.
  ///
  /// Most variants carry exactly one spelling. The real multi-spelling
  /// entries are `Jpeg` (`jpg`/`jpeg`/`jpe`), `Tiff` (`tif`/`tiff`),
  /// `Bmp` (`bmp`/`dib`) and `Orf` (`orf`/`ori`) — see the module doc's
  /// ExifTool-census note for where each alias comes from. `Heif` and
  /// `Heic` both carry no alias — see `Heic`'s own doc for `.hif`,
  /// which is deliberately excluded from both rather than guessed.
  ///
  /// Returns `&[]` for [`Self::Other`] — the open variant carries a
  /// format-name slug, not a known extension set.
  #[inline]
  pub const fn extensions(&self) -> &'static [&'static str] {
    match self {
      Self::Jpeg => &["jpg", "jpeg", "jpe"],
      Self::Png => &["png"],
      Self::Heif => &["heif"],
      Self::Heic => &["heic"],
      Self::Avif => &["avif"],
      Self::Tiff => &["tif", "tiff"],
      Self::Webp => &["webp"],
      Self::Gif => &["gif"],
      Self::Bmp => &["bmp", "dib"],
      Self::Dng => &["dng"],
      Self::Cr2 => &["cr2"],
      Self::Cr3 => &["cr3"],
      Self::Nef => &["nef"],
      Self::Nrw => &["nrw"],
      Self::Arw => &["arw"],
      Self::Orf => &["orf", "ori"],
      Self::Rw2 => &["rw2"],
      Self::Raf => &["raf"],
      Self::Pef => &["pef"],
      Self::Srw => &["srw"],
      Self::Rwl => &["rwl"],
      Self::Iiq => &["iiq"],
      Self::Threefr => &["3fr"],
      Self::X3f => &["x3f"],
      Self::Mrw => &["mrw"],
      Self::Gpr => &["gpr"],
      Self::Other(_) => &[],
    }
  }

  /// The open escape for a slug this vocabulary does not name.
  ///
  /// Runs the ignore-case parse first — [`FromStr`]'s own match table
  /// (canonical spelling and every documented alias extension), walked
  /// through [`Self::from_str`] rather than duplicated here — so a
  /// recognised spelling returns that **named** variant, never a second
  /// value for a meaning this vocabulary already has one for. Only a
  /// genuine stranger reaches [`Self::Other`], carrying the caller's
  /// spelling verbatim: the escape is a lossless passthrough for a name
  /// this build does not know, not a fold target.
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::from_str(slug.as_ref()).unwrap()
  }
}

roster!(
  Format,
  "image format",
  [
    Jpeg, Png, Heif, Heic, Avif, Tiff, Webp, Gif, Bmp, Dng, Cr2, Cr3, Nef,
    Nrw, Arw, Orf, Rw2, Raf, Pef, Srw, Rwl, Iiq, Threefr, X3f, Mrw, Gpr
  ],
  escape: Other
);

impl FromStr for Format {
  type Err = core::convert::Infallible;
  /// Recognise a canonical format-name slug **or any alias extension**
  /// from [`Self::extensions`]; unknown values land in [`Self::Other`]
  /// (infallible, lossless). Ignore-case, via the crate's one ASCII
  /// case-folding gate.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"jpg" | b"jpeg" | b"jpe" => Self::Jpeg,
      b"png" => Self::Png,
      b"heif" => Self::Heif,
      b"heic" => Self::Heic,
      b"avif" => Self::Avif,
      b"tif" | b"tiff" => Self::Tiff,
      b"webp" => Self::Webp,
      b"gif" => Self::Gif,
      b"bmp" | b"dib" => Self::Bmp,
      b"dng" => Self::Dng,
      b"cr2" => Self::Cr2,
      b"cr3" => Self::Cr3,
      b"nef" => Self::Nef,
      b"nrw" => Self::Nrw,
      b"arw" => Self::Arw,
      b"orf" | b"ori" => Self::Orf,
      b"rw2" => Self::Rw2,
      b"raf" => Self::Raf,
      b"pef" => Self::Pef,
      b"srw" => Self::Srw,
      b"rwl" => Self::Rwl,
      b"iiq" => Self::Iiq,
      b"3fr" => Self::Threefr,
      b"x3f" => Self::X3f,
      b"mrw" => Self::Mrw,
      b"gpr" => Self::Gpr,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}

#[cfg(test)]
mod tests;
