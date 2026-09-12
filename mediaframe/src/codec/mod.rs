//! Stream-descriptor **codec** vocabulary for video, audio, subtitle,
//! data, and attachment tracks.
//!
//! **Generated** from `xtask/vendor/ffmpeg-codecs.txt` (FFmpeg n9.0
//! `libavcodec/codec_desc.c`) by `cargo xtask gen-codec` — except
//! [`AttachmentCodec`], whose roster comes from a different FFmpeg
//! source; see its own doc comment for why. Every codec FFmpeg knows
//! under media types `video` / `audio` / `subtitle` / `data` has a
//! named variant here; the `Other(Utf8Bytes)` arm remains a lossless
//! escape for codecs added in a future FFmpeg release before this
//! file is regenerated (or, for `AttachmentCodec`, before
//! `ATTACHMENT_CODECS` is re-derived by hand).
//!
//! Regenerate in two steps:
//! 1. `cargo xtask sync`       — refreshes the vendored table.
//! 2. `cargo xtask gen-codec`  — regenerates this file from it.
//!
//! `cargo xtask check` verifies every named variant's canonical string
//! exists in the vendored table (or, for `AttachmentCodec`, in
//! `ATTACHMENT_CODECS`) — CI gate against drift.
//!
//! **Derive threshold.** `Unwrap` / `TryUnwrap` generate three methods
//! per variant, so an enum in the hundreds pays that in compile time for
//! one reachable payload arm.
//! [`SubtitleCodec`] (27), [`DataCodec`] (11), and [`AttachmentCodec`]
//! (3) are small enough to carry the pair; [`VideoCodec`] (279)
//! and [`AudioCodec`] (222) do not. The line is
//! variant count, not principle — reach for `Other(_)` on the large two
//! with a `match` or [`IsVariant`](derive_more::IsVariant)'s `is_other`.
use core::str::FromStr;
use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_bytes::Utf8Bytes;
/** Video codec family — every codec FFmpeg n9.0 knows under media type `video`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::video_codec")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum VideoCodec {
  /// FFmpeg `"012v"`.
  N012v,
  /// FFmpeg `"4xm"`.
  N4xm,
  /// FFmpeg `"8bps"`.
  N8bps,
  /// FFmpeg `"a64_multi"`.
  A64Multi,
  /// FFmpeg `"a64_multi5"`.
  A64Multi5,
  /// FFmpeg `"aasc"`.
  Aasc,
  /// FFmpeg `"agm"`.
  Agm,
  /// FFmpeg `"aic"`.
  Aic,
  /// FFmpeg `"alias_pix"`.
  AliasPix,
  /// FFmpeg `"amv"`.
  Amv,
  /// FFmpeg `"anm"`.
  Anm,
  /// FFmpeg `"ansi"`.
  Ansi,
  /// FFmpeg `"apng"`.
  Apng,
  /// FFmpeg `"apv"`.
  Apv,
  /// FFmpeg `"arbc"`.
  Arbc,
  /// FFmpeg `"argo"`.
  Argo,
  /// FFmpeg `"asv1"`.
  Asv1,
  /// FFmpeg `"asv2"`.
  Asv2,
  /// FFmpeg `"aura"`.
  Aura,
  /// FFmpeg `"aura2"`.
  Aura2,
  /// FFmpeg `"av1"`.
  Av1,
  /// FFmpeg `"avrn"`.
  Avrn,
  /// FFmpeg `"avrp"`.
  Avrp,
  /// FFmpeg `"avs"`.
  Avs,
  /// FFmpeg `"avs2"`.
  Avs2,
  /// FFmpeg `"avs3"`.
  Avs3,
  /// FFmpeg `"avui"`.
  Avui,
  /// FFmpeg `"bethsoftvid"`.
  Bethsoftvid,
  /// FFmpeg `"bfi"`.
  Bfi,
  /// FFmpeg `"binkvideo"`.
  Binkvideo,
  /// FFmpeg `"bintext"`.
  Bintext,
  /// FFmpeg `"bitpacked"`.
  Bitpacked,
  /// FFmpeg `"bmp"`.
  Bmp,
  /// FFmpeg `"bmv_video"`.
  BmvVideo,
  /// FFmpeg `"brender_pix"`.
  BrenderPix,
  /// FFmpeg `"c93"`.
  C93,
  /// FFmpeg `"cavs"`.
  Cavs,
  /// FFmpeg `"cdgraphics"`.
  Cdgraphics,
  /// FFmpeg `"cdtoons"`.
  Cdtoons,
  /// FFmpeg `"cdxl"`.
  Cdxl,
  /// FFmpeg `"cfhd"`.
  Cfhd,
  /// FFmpeg `"cinepak"`.
  Cinepak,
  /// FFmpeg `"clearvideo"`.
  Clearvideo,
  /// FFmpeg `"cljr"`.
  Cljr,
  /// FFmpeg `"cllc"`.
  Cllc,
  /// FFmpeg `"cmv"`.
  Cmv,
  /// FFmpeg `"cpia"`.
  Cpia,
  /// FFmpeg `"cri"`.
  Cri,
  /// FFmpeg `"cscd"`.
  Cscd,
  /// FFmpeg `"cyuv"`.
  Cyuv,
  /// FFmpeg `"daala"`.
  Daala,
  /// FFmpeg `"dds"`.
  Dds,
  /// FFmpeg `"dfa"`.
  Dfa,
  /// FFmpeg `"dirac"`.
  Dirac,
  /// FFmpeg `"dnxhd"`.
  Dnxhd,
  /// FFmpeg `"dnxuc"`.
  Dnxuc,
  /// FFmpeg `"dpx"`.
  Dpx,
  /// FFmpeg `"dsicinvideo"`.
  Dsicinvideo,
  /// FFmpeg `"dvvideo"`.
  Dvvideo,
  /// FFmpeg `"dxa"`.
  Dxa,
  /// FFmpeg `"dxtory"`.
  Dxtory,
  /// FFmpeg `"dxv"`.
  Dxv,
  /// FFmpeg `"escape124"`.
  Escape124,
  /// FFmpeg `"escape130"`.
  Escape130,
  /// FFmpeg `"evc"`.
  Evc,
  /// FFmpeg `"exr"`.
  Exr,
  /// FFmpeg `"ffv1"`.
  Ffv1,
  /// FFmpeg `"ffvhuff"`.
  Ffvhuff,
  /// FFmpeg `"fic"`.
  Fic,
  /// FFmpeg `"fits"`.
  Fits,
  /// FFmpeg `"flashsv"`.
  Flashsv,
  /// FFmpeg `"flashsv2"`.
  Flashsv2,
  /// FFmpeg `"flic"`.
  Flic,
  /// FFmpeg `"flv1"`.
  Flv1,
  /// FFmpeg `"fmvc"`.
  Fmvc,
  /// FFmpeg `"fraps"`.
  Fraps,
  /// FFmpeg `"frwu"`.
  Frwu,
  /// FFmpeg `"g2m"`.
  G2m,
  /// FFmpeg `"gdv"`.
  Gdv,
  /// FFmpeg `"gem"`.
  Gem,
  /// FFmpeg `"gif"`.
  Gif,
  /// FFmpeg `"h261"`.
  H261,
  /// FFmpeg `"h263"`.
  H263,
  /// FFmpeg `"h263i"`.
  H263i,
  /// FFmpeg `"h263p"`.
  H263p,
  /// FFmpeg `"h264"`.
  H264,
  /// FFmpeg `"hap"`.
  Hap,
  /// FFmpeg `"hdr"`.
  Hdr,
  /// FFmpeg `"hevc"`.
  Hevc,
  /// FFmpeg `"hnm4video"`.
  Hnm4video,
  /// FFmpeg `"hq_hqa"`.
  HqHqa,
  /// FFmpeg `"hqx"`.
  Hqx,
  /// FFmpeg `"huffyuv"`.
  Huffyuv,
  /// FFmpeg `"hymt"`.
  Hymt,
  /// FFmpeg `"idcin"`.
  Idcin,
  /// FFmpeg `"idf"`.
  Idf,
  /// FFmpeg `"iff_ilbm"`.
  IffIlbm,
  /// FFmpeg `"imm4"`.
  Imm4,
  /// FFmpeg `"imm5"`.
  Imm5,
  /// FFmpeg `"indeo2"`.
  Indeo2,
  /// FFmpeg `"indeo3"`.
  Indeo3,
  /// FFmpeg `"indeo4"`.
  Indeo4,
  /// FFmpeg `"indeo5"`.
  Indeo5,
  /// FFmpeg `"interplayvideo"`.
  Interplayvideo,
  /// FFmpeg `"ipu"`.
  Ipu,
  /// FFmpeg `"jpeg2000"`.
  Jpeg2000,
  /// FFmpeg `"jpegls"`.
  Jpegls,
  /// FFmpeg `"jpegxl"`.
  Jpegxl,
  /// FFmpeg `"jpegxl_anim"`.
  JpegxlAnim,
  /// FFmpeg `"jpegxs"`.
  Jpegxs,
  /// FFmpeg `"jv"`.
  Jv,
  /// FFmpeg `"kgv1"`.
  Kgv1,
  /// FFmpeg `"kmvc"`.
  Kmvc,
  /// FFmpeg `"lagarith"`.
  Lagarith,
  /// FFmpeg `"lcevc"`.
  Lcevc,
  /// FFmpeg `"lead"`.
  Lead,
  /// FFmpeg `"ljpeg"`.
  Ljpeg,
  /// FFmpeg `"loco"`.
  Loco,
  /// FFmpeg `"lscr"`.
  Lscr,
  /// FFmpeg `"m101"`.
  M101,
  /// FFmpeg `"mad"`.
  Mad,
  /// FFmpeg `"magicyuv"`.
  Magicyuv,
  /// FFmpeg `"mdec"`.
  Mdec,
  /// FFmpeg `"media100"`.
  Media100,
  /// FFmpeg `"mimic"`.
  Mimic,
  /// FFmpeg `"mjpeg"`.
  Mjpeg,
  /// FFmpeg `"mjpegb"`.
  Mjpegb,
  /// FFmpeg `"mmvideo"`.
  Mmvideo,
  /// FFmpeg `"mobiclip"`.
  Mobiclip,
  /// FFmpeg `"motionpixels"`.
  Motionpixels,
  /// FFmpeg `"mpeg1video"`.
  Mpeg1video,
  /// FFmpeg `"mpeg2video"`.
  Mpeg2video,
  /// FFmpeg `"mpeg4"`.
  Mpeg4,
  /// FFmpeg `"msa1"`.
  Msa1,
  /// FFmpeg `"mscc"`.
  Mscc,
  /// FFmpeg `"msmpeg4v1"`.
  Msmpeg4v1,
  /// FFmpeg `"msmpeg4v2"`.
  Msmpeg4v2,
  /// FFmpeg `"msmpeg4v3"`.
  Msmpeg4v3,
  /// FFmpeg `"msp2"`.
  Msp2,
  /// FFmpeg `"msrle"`.
  Msrle,
  /// FFmpeg `"mss1"`.
  Mss1,
  /// FFmpeg `"mss2"`.
  Mss2,
  /// FFmpeg `"msvideo1"`.
  Msvideo1,
  /// FFmpeg `"mszh"`.
  Mszh,
  /// FFmpeg `"mts2"`.
  Mts2,
  /// FFmpeg `"mv30"`.
  Mv30,
  /// FFmpeg `"mvc1"`.
  Mvc1,
  /// FFmpeg `"mvc2"`.
  Mvc2,
  /// FFmpeg `"mvdv"`.
  Mvdv,
  /// FFmpeg `"mvha"`.
  Mvha,
  /// FFmpeg `"mwsc"`.
  Mwsc,
  /// FFmpeg `"mxpeg"`.
  Mxpeg,
  /// FFmpeg `"notchlc"`.
  Notchlc,
  /// FFmpeg `"nuv"`.
  Nuv,
  /// FFmpeg `"paf_video"`.
  PafVideo,
  /// FFmpeg `"pam"`.
  Pam,
  /// FFmpeg `"pbm"`.
  Pbm,
  /// FFmpeg `"pcx"`.
  Pcx,
  /// FFmpeg `"pdv"`.
  Pdv,
  /// FFmpeg `"pfm"`.
  Pfm,
  /// FFmpeg `"pgm"`.
  Pgm,
  /// FFmpeg `"pgmyuv"`.
  Pgmyuv,
  /// FFmpeg `"pgx"`.
  Pgx,
  /// FFmpeg `"phm"`.
  Phm,
  /// FFmpeg `"photocd"`.
  Photocd,
  /// FFmpeg `"pictor"`.
  Pictor,
  /// FFmpeg `"pixlet"`.
  Pixlet,
  /// FFmpeg `"png"`.
  Png,
  /// FFmpeg `"ppm"`.
  Ppm,
  /// FFmpeg `"prores"`.
  Prores,
  /// FFmpeg `"prores_raw"`.
  ProresRaw,
  /// FFmpeg `"prosumer"`.
  Prosumer,
  /// FFmpeg `"psd"`.
  Psd,
  /// FFmpeg `"ptx"`.
  Ptx,
  /// FFmpeg `"qdraw"`.
  Qdraw,
  /// FFmpeg `"qoi"`.
  Qoi,
  /// FFmpeg `"qpeg"`.
  Qpeg,
  /// FFmpeg `"qtrle"`.
  Qtrle,
  /// FFmpeg `"r10k"`.
  R10k,
  /// FFmpeg `"r210"`.
  R210,
  /// FFmpeg `"rasc"`.
  Rasc,
  /// FFmpeg `"rawvideo"`.
  Rawvideo,
  /// FFmpeg `"rl2"`.
  Rl2,
  /// FFmpeg `"roq"`.
  Roq,
  /// FFmpeg `"rpza"`.
  Rpza,
  /// FFmpeg `"rscc"`.
  Rscc,
  /// FFmpeg `"rtv1"`.
  Rtv1,
  /// FFmpeg `"rv10"`.
  Rv10,
  /// FFmpeg `"rv20"`.
  Rv20,
  /// FFmpeg `"rv30"`.
  Rv30,
  /// FFmpeg `"rv40"`.
  Rv40,
  /// FFmpeg `"rv60"`.
  Rv60,
  /// FFmpeg `"sanm"`.
  Sanm,
  /// FFmpeg `"scpr"`.
  Scpr,
  /// FFmpeg `"screenpresso"`.
  Screenpresso,
  /// FFmpeg `"sga"`.
  Sga,
  /// FFmpeg `"sgi"`.
  Sgi,
  /// FFmpeg `"sgirle"`.
  Sgirle,
  /// FFmpeg `"sheervideo"`.
  Sheervideo,
  /// FFmpeg `"simbiosis_imx"`.
  SimbiosisImx,
  /// FFmpeg `"smackvideo"`.
  Smackvideo,
  /// FFmpeg `"smc"`.
  Smc,
  /// FFmpeg `"smvjpeg"`.
  Smvjpeg,
  /// FFmpeg `"snow"`.
  Snow,
  /// FFmpeg `"sp5x"`.
  Sp5x,
  /// FFmpeg `"speedhq"`.
  Speedhq,
  /// FFmpeg `"srgc"`.
  Srgc,
  /// FFmpeg `"sunrast"`.
  Sunrast,
  /// FFmpeg `"svg"`.
  Svg,
  /// FFmpeg `"svq1"`.
  Svq1,
  /// FFmpeg `"svq3"`.
  Svq3,
  /// FFmpeg `"targa"`.
  Targa,
  /// FFmpeg `"targa_y216"`.
  TargaY216,
  /// FFmpeg `"tdsc"`.
  Tdsc,
  /// FFmpeg `"tgq"`.
  Tgq,
  /// FFmpeg `"tgv"`.
  Tgv,
  /// FFmpeg `"theora"`.
  Theora,
  /// FFmpeg `"thp"`.
  Thp,
  /// FFmpeg `"tiertexseqvideo"`.
  Tiertexseqvideo,
  /// FFmpeg `"tiff"`.
  Tiff,
  /// FFmpeg `"tmv"`.
  Tmv,
  /// FFmpeg `"tqi"`.
  Tqi,
  /// FFmpeg `"truemotion1"`.
  Truemotion1,
  /// FFmpeg `"truemotion2"`.
  Truemotion2,
  /// FFmpeg `"truemotion2rt"`.
  Truemotion2rt,
  /// FFmpeg `"tscc"`.
  Tscc,
  /// FFmpeg `"tscc2"`.
  Tscc2,
  /// FFmpeg `"txd"`.
  Txd,
  /// FFmpeg `"ulti"`.
  Ulti,
  /// FFmpeg `"utvideo"`.
  Utvideo,
  /// FFmpeg `"v210"`.
  V210,
  /// FFmpeg `"v210x"`.
  V210x,
  /// FFmpeg `"vb"`.
  Vb,
  /// FFmpeg `"vble"`.
  Vble,
  /// FFmpeg `"vbn"`.
  Vbn,
  /// FFmpeg `"vc1"`.
  Vc1,
  /// FFmpeg `"vc1image"`.
  Vc1image,
  /// FFmpeg `"vcr1"`.
  Vcr1,
  /// FFmpeg `"vixl"`.
  Vixl,
  /// FFmpeg `"vmdvideo"`.
  Vmdvideo,
  /// FFmpeg `"vmix"`.
  Vmix,
  /// FFmpeg `"vmnc"`.
  Vmnc,
  /// FFmpeg `"vnull"`.
  Vnull,
  /// FFmpeg `"vp3"`.
  Vp3,
  /// FFmpeg `"vp4"`.
  Vp4,
  /// FFmpeg `"vp5"`.
  Vp5,
  /// FFmpeg `"vp6"`.
  Vp6,
  /// FFmpeg `"vp6a"`.
  Vp6a,
  /// FFmpeg `"vp6f"`.
  Vp6f,
  /// FFmpeg `"vp7"`.
  Vp7,
  /// FFmpeg `"vp8"`.
  Vp8,
  /// FFmpeg `"vp9"`.
  Vp9,
  /// FFmpeg `"vqc"`.
  Vqc,
  /// FFmpeg `"vvc"`.
  Vvc,
  /// FFmpeg `"wbmp"`.
  Wbmp,
  /// FFmpeg `"wcmv"`.
  Wcmv,
  /// FFmpeg `"webp"`.
  Webp,
  /// FFmpeg `"webp_anim"`.
  WebpAnim,
  /// FFmpeg `"wmv1"`.
  Wmv1,
  /// FFmpeg `"wmv2"`.
  Wmv2,
  /// FFmpeg `"wmv3"`.
  Wmv3,
  /// FFmpeg `"wmv3image"`.
  Wmv3image,
  /// FFmpeg `"wnv1"`.
  Wnv1,
  /// FFmpeg `"wrapped_avframe"`.
  WrappedAvframe,
  /// FFmpeg `"ws_vqa"`.
  WsVqa,
  /// FFmpeg `"xan_wc3"`.
  XanWc3,
  /// FFmpeg `"xan_wc4"`.
  XanWc4,
  /// FFmpeg `"xbin"`.
  Xbin,
  /// FFmpeg `"xbm"`.
  Xbm,
  /// FFmpeg `"xface"`.
  Xface,
  /// FFmpeg `"xpm"`.
  Xpm,
  /// FFmpeg `"xwd"`.
  Xwd,
  /// FFmpeg `"y41p"`.
  Y41p,
  /// FFmpeg `"ylc"`.
  Ylc,
  /// FFmpeg `"yop"`.
  Yop,
  /// FFmpeg `"yuv4"`.
  Yuv4,
  /// FFmpeg `"zerocodec"`.
  Zerocodec,
  /// FFmpeg `"zlib"`.
  Zlib,
  /// FFmpeg `"zmbv"`.
  Zmbv,
  /// A codec not enumerated above — carries the FFmpeg short name
  /// verbatim.
  Other(Utf8Bytes),
}
impl VideoCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::N012v => "012v",
      Self::N4xm => "4xm",
      Self::N8bps => "8bps",
      Self::A64Multi => "a64_multi",
      Self::A64Multi5 => "a64_multi5",
      Self::Aasc => "aasc",
      Self::Agm => "agm",
      Self::Aic => "aic",
      Self::AliasPix => "alias_pix",
      Self::Amv => "amv",
      Self::Anm => "anm",
      Self::Ansi => "ansi",
      Self::Apng => "apng",
      Self::Apv => "apv",
      Self::Arbc => "arbc",
      Self::Argo => "argo",
      Self::Asv1 => "asv1",
      Self::Asv2 => "asv2",
      Self::Aura => "aura",
      Self::Aura2 => "aura2",
      Self::Av1 => "av1",
      Self::Avrn => "avrn",
      Self::Avrp => "avrp",
      Self::Avs => "avs",
      Self::Avs2 => "avs2",
      Self::Avs3 => "avs3",
      Self::Avui => "avui",
      Self::Bethsoftvid => "bethsoftvid",
      Self::Bfi => "bfi",
      Self::Binkvideo => "binkvideo",
      Self::Bintext => "bintext",
      Self::Bitpacked => "bitpacked",
      Self::Bmp => "bmp",
      Self::BmvVideo => "bmv_video",
      Self::BrenderPix => "brender_pix",
      Self::C93 => "c93",
      Self::Cavs => "cavs",
      Self::Cdgraphics => "cdgraphics",
      Self::Cdtoons => "cdtoons",
      Self::Cdxl => "cdxl",
      Self::Cfhd => "cfhd",
      Self::Cinepak => "cinepak",
      Self::Clearvideo => "clearvideo",
      Self::Cljr => "cljr",
      Self::Cllc => "cllc",
      Self::Cmv => "cmv",
      Self::Cpia => "cpia",
      Self::Cri => "cri",
      Self::Cscd => "cscd",
      Self::Cyuv => "cyuv",
      Self::Daala => "daala",
      Self::Dds => "dds",
      Self::Dfa => "dfa",
      Self::Dirac => "dirac",
      Self::Dnxhd => "dnxhd",
      Self::Dnxuc => "dnxuc",
      Self::Dpx => "dpx",
      Self::Dsicinvideo => "dsicinvideo",
      Self::Dvvideo => "dvvideo",
      Self::Dxa => "dxa",
      Self::Dxtory => "dxtory",
      Self::Dxv => "dxv",
      Self::Escape124 => "escape124",
      Self::Escape130 => "escape130",
      Self::Evc => "evc",
      Self::Exr => "exr",
      Self::Ffv1 => "ffv1",
      Self::Ffvhuff => "ffvhuff",
      Self::Fic => "fic",
      Self::Fits => "fits",
      Self::Flashsv => "flashsv",
      Self::Flashsv2 => "flashsv2",
      Self::Flic => "flic",
      Self::Flv1 => "flv1",
      Self::Fmvc => "fmvc",
      Self::Fraps => "fraps",
      Self::Frwu => "frwu",
      Self::G2m => "g2m",
      Self::Gdv => "gdv",
      Self::Gem => "gem",
      Self::Gif => "gif",
      Self::H261 => "h261",
      Self::H263 => "h263",
      Self::H263i => "h263i",
      Self::H263p => "h263p",
      Self::H264 => "h264",
      Self::Hap => "hap",
      Self::Hdr => "hdr",
      Self::Hevc => "hevc",
      Self::Hnm4video => "hnm4video",
      Self::HqHqa => "hq_hqa",
      Self::Hqx => "hqx",
      Self::Huffyuv => "huffyuv",
      Self::Hymt => "hymt",
      Self::Idcin => "idcin",
      Self::Idf => "idf",
      Self::IffIlbm => "iff_ilbm",
      Self::Imm4 => "imm4",
      Self::Imm5 => "imm5",
      Self::Indeo2 => "indeo2",
      Self::Indeo3 => "indeo3",
      Self::Indeo4 => "indeo4",
      Self::Indeo5 => "indeo5",
      Self::Interplayvideo => "interplayvideo",
      Self::Ipu => "ipu",
      Self::Jpeg2000 => "jpeg2000",
      Self::Jpegls => "jpegls",
      Self::Jpegxl => "jpegxl",
      Self::JpegxlAnim => "jpegxl_anim",
      Self::Jpegxs => "jpegxs",
      Self::Jv => "jv",
      Self::Kgv1 => "kgv1",
      Self::Kmvc => "kmvc",
      Self::Lagarith => "lagarith",
      Self::Lcevc => "lcevc",
      Self::Lead => "lead",
      Self::Ljpeg => "ljpeg",
      Self::Loco => "loco",
      Self::Lscr => "lscr",
      Self::M101 => "m101",
      Self::Mad => "mad",
      Self::Magicyuv => "magicyuv",
      Self::Mdec => "mdec",
      Self::Media100 => "media100",
      Self::Mimic => "mimic",
      Self::Mjpeg => "mjpeg",
      Self::Mjpegb => "mjpegb",
      Self::Mmvideo => "mmvideo",
      Self::Mobiclip => "mobiclip",
      Self::Motionpixels => "motionpixels",
      Self::Mpeg1video => "mpeg1video",
      Self::Mpeg2video => "mpeg2video",
      Self::Mpeg4 => "mpeg4",
      Self::Msa1 => "msa1",
      Self::Mscc => "mscc",
      Self::Msmpeg4v1 => "msmpeg4v1",
      Self::Msmpeg4v2 => "msmpeg4v2",
      Self::Msmpeg4v3 => "msmpeg4v3",
      Self::Msp2 => "msp2",
      Self::Msrle => "msrle",
      Self::Mss1 => "mss1",
      Self::Mss2 => "mss2",
      Self::Msvideo1 => "msvideo1",
      Self::Mszh => "mszh",
      Self::Mts2 => "mts2",
      Self::Mv30 => "mv30",
      Self::Mvc1 => "mvc1",
      Self::Mvc2 => "mvc2",
      Self::Mvdv => "mvdv",
      Self::Mvha => "mvha",
      Self::Mwsc => "mwsc",
      Self::Mxpeg => "mxpeg",
      Self::Notchlc => "notchlc",
      Self::Nuv => "nuv",
      Self::PafVideo => "paf_video",
      Self::Pam => "pam",
      Self::Pbm => "pbm",
      Self::Pcx => "pcx",
      Self::Pdv => "pdv",
      Self::Pfm => "pfm",
      Self::Pgm => "pgm",
      Self::Pgmyuv => "pgmyuv",
      Self::Pgx => "pgx",
      Self::Phm => "phm",
      Self::Photocd => "photocd",
      Self::Pictor => "pictor",
      Self::Pixlet => "pixlet",
      Self::Png => "png",
      Self::Ppm => "ppm",
      Self::Prores => "prores",
      Self::ProresRaw => "prores_raw",
      Self::Prosumer => "prosumer",
      Self::Psd => "psd",
      Self::Ptx => "ptx",
      Self::Qdraw => "qdraw",
      Self::Qoi => "qoi",
      Self::Qpeg => "qpeg",
      Self::Qtrle => "qtrle",
      Self::R10k => "r10k",
      Self::R210 => "r210",
      Self::Rasc => "rasc",
      Self::Rawvideo => "rawvideo",
      Self::Rl2 => "rl2",
      Self::Roq => "roq",
      Self::Rpza => "rpza",
      Self::Rscc => "rscc",
      Self::Rtv1 => "rtv1",
      Self::Rv10 => "rv10",
      Self::Rv20 => "rv20",
      Self::Rv30 => "rv30",
      Self::Rv40 => "rv40",
      Self::Rv60 => "rv60",
      Self::Sanm => "sanm",
      Self::Scpr => "scpr",
      Self::Screenpresso => "screenpresso",
      Self::Sga => "sga",
      Self::Sgi => "sgi",
      Self::Sgirle => "sgirle",
      Self::Sheervideo => "sheervideo",
      Self::SimbiosisImx => "simbiosis_imx",
      Self::Smackvideo => "smackvideo",
      Self::Smc => "smc",
      Self::Smvjpeg => "smvjpeg",
      Self::Snow => "snow",
      Self::Sp5x => "sp5x",
      Self::Speedhq => "speedhq",
      Self::Srgc => "srgc",
      Self::Sunrast => "sunrast",
      Self::Svg => "svg",
      Self::Svq1 => "svq1",
      Self::Svq3 => "svq3",
      Self::Targa => "targa",
      Self::TargaY216 => "targa_y216",
      Self::Tdsc => "tdsc",
      Self::Tgq => "tgq",
      Self::Tgv => "tgv",
      Self::Theora => "theora",
      Self::Thp => "thp",
      Self::Tiertexseqvideo => "tiertexseqvideo",
      Self::Tiff => "tiff",
      Self::Tmv => "tmv",
      Self::Tqi => "tqi",
      Self::Truemotion1 => "truemotion1",
      Self::Truemotion2 => "truemotion2",
      Self::Truemotion2rt => "truemotion2rt",
      Self::Tscc => "tscc",
      Self::Tscc2 => "tscc2",
      Self::Txd => "txd",
      Self::Ulti => "ulti",
      Self::Utvideo => "utvideo",
      Self::V210 => "v210",
      Self::V210x => "v210x",
      Self::Vb => "vb",
      Self::Vble => "vble",
      Self::Vbn => "vbn",
      Self::Vc1 => "vc1",
      Self::Vc1image => "vc1image",
      Self::Vcr1 => "vcr1",
      Self::Vixl => "vixl",
      Self::Vmdvideo => "vmdvideo",
      Self::Vmix => "vmix",
      Self::Vmnc => "vmnc",
      Self::Vnull => "vnull",
      Self::Vp3 => "vp3",
      Self::Vp4 => "vp4",
      Self::Vp5 => "vp5",
      Self::Vp6 => "vp6",
      Self::Vp6a => "vp6a",
      Self::Vp6f => "vp6f",
      Self::Vp7 => "vp7",
      Self::Vp8 => "vp8",
      Self::Vp9 => "vp9",
      Self::Vqc => "vqc",
      Self::Vvc => "vvc",
      Self::Wbmp => "wbmp",
      Self::Wcmv => "wcmv",
      Self::Webp => "webp",
      Self::WebpAnim => "webp_anim",
      Self::Wmv1 => "wmv1",
      Self::Wmv2 => "wmv2",
      Self::Wmv3 => "wmv3",
      Self::Wmv3image => "wmv3image",
      Self::Wnv1 => "wnv1",
      Self::WrappedAvframe => "wrapped_avframe",
      Self::WsVqa => "ws_vqa",
      Self::XanWc3 => "xan_wc3",
      Self::XanWc4 => "xan_wc4",
      Self::Xbin => "xbin",
      Self::Xbm => "xbm",
      Self::Xface => "xface",
      Self::Xpm => "xpm",
      Self::Xwd => "xwd",
      Self::Y41p => "y41p",
      Self::Ylc => "ylc",
      Self::Yop => "yop",
      Self::Yuv4 => "yuv4",
      Self::Zerocodec => "zerocodec",
      Self::Zlib => "zlib",
      Self::Zmbv => "zmbv",
      Self::Other(s) => s.as_str(),
    }
  }
  /// The open escape for a codec name FFmpeg n9.0 does not carry.
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
}
impl VideoCodec {
  /// Every video codec this vocabulary names, in declaration order.
  ///
  /// A slice rather than an array: how many codecs this build carries
  /// is a fact about the vendored FFmpeg table, not part of the type,
  /// so a regeneration that adds one stays a minor change.
  ///
  /// [`Self::Other`] is not a member. The roster answers "which names
  /// does this build know", and the escape is precisely the arm that
  /// carries a name it does not.
  pub const ROSTER: &'static [Self] = &[
    Self::N012v,
    Self::N4xm,
    Self::N8bps,
    Self::A64Multi,
    Self::A64Multi5,
    Self::Aasc,
    Self::Agm,
    Self::Aic,
    Self::AliasPix,
    Self::Amv,
    Self::Anm,
    Self::Ansi,
    Self::Apng,
    Self::Apv,
    Self::Arbc,
    Self::Argo,
    Self::Asv1,
    Self::Asv2,
    Self::Aura,
    Self::Aura2,
    Self::Av1,
    Self::Avrn,
    Self::Avrp,
    Self::Avs,
    Self::Avs2,
    Self::Avs3,
    Self::Avui,
    Self::Bethsoftvid,
    Self::Bfi,
    Self::Binkvideo,
    Self::Bintext,
    Self::Bitpacked,
    Self::Bmp,
    Self::BmvVideo,
    Self::BrenderPix,
    Self::C93,
    Self::Cavs,
    Self::Cdgraphics,
    Self::Cdtoons,
    Self::Cdxl,
    Self::Cfhd,
    Self::Cinepak,
    Self::Clearvideo,
    Self::Cljr,
    Self::Cllc,
    Self::Cmv,
    Self::Cpia,
    Self::Cri,
    Self::Cscd,
    Self::Cyuv,
    Self::Daala,
    Self::Dds,
    Self::Dfa,
    Self::Dirac,
    Self::Dnxhd,
    Self::Dnxuc,
    Self::Dpx,
    Self::Dsicinvideo,
    Self::Dvvideo,
    Self::Dxa,
    Self::Dxtory,
    Self::Dxv,
    Self::Escape124,
    Self::Escape130,
    Self::Evc,
    Self::Exr,
    Self::Ffv1,
    Self::Ffvhuff,
    Self::Fic,
    Self::Fits,
    Self::Flashsv,
    Self::Flashsv2,
    Self::Flic,
    Self::Flv1,
    Self::Fmvc,
    Self::Fraps,
    Self::Frwu,
    Self::G2m,
    Self::Gdv,
    Self::Gem,
    Self::Gif,
    Self::H261,
    Self::H263,
    Self::H263i,
    Self::H263p,
    Self::H264,
    Self::Hap,
    Self::Hdr,
    Self::Hevc,
    Self::Hnm4video,
    Self::HqHqa,
    Self::Hqx,
    Self::Huffyuv,
    Self::Hymt,
    Self::Idcin,
    Self::Idf,
    Self::IffIlbm,
    Self::Imm4,
    Self::Imm5,
    Self::Indeo2,
    Self::Indeo3,
    Self::Indeo4,
    Self::Indeo5,
    Self::Interplayvideo,
    Self::Ipu,
    Self::Jpeg2000,
    Self::Jpegls,
    Self::Jpegxl,
    Self::JpegxlAnim,
    Self::Jpegxs,
    Self::Jv,
    Self::Kgv1,
    Self::Kmvc,
    Self::Lagarith,
    Self::Lcevc,
    Self::Lead,
    Self::Ljpeg,
    Self::Loco,
    Self::Lscr,
    Self::M101,
    Self::Mad,
    Self::Magicyuv,
    Self::Mdec,
    Self::Media100,
    Self::Mimic,
    Self::Mjpeg,
    Self::Mjpegb,
    Self::Mmvideo,
    Self::Mobiclip,
    Self::Motionpixels,
    Self::Mpeg1video,
    Self::Mpeg2video,
    Self::Mpeg4,
    Self::Msa1,
    Self::Mscc,
    Self::Msmpeg4v1,
    Self::Msmpeg4v2,
    Self::Msmpeg4v3,
    Self::Msp2,
    Self::Msrle,
    Self::Mss1,
    Self::Mss2,
    Self::Msvideo1,
    Self::Mszh,
    Self::Mts2,
    Self::Mv30,
    Self::Mvc1,
    Self::Mvc2,
    Self::Mvdv,
    Self::Mvha,
    Self::Mwsc,
    Self::Mxpeg,
    Self::Notchlc,
    Self::Nuv,
    Self::PafVideo,
    Self::Pam,
    Self::Pbm,
    Self::Pcx,
    Self::Pdv,
    Self::Pfm,
    Self::Pgm,
    Self::Pgmyuv,
    Self::Pgx,
    Self::Phm,
    Self::Photocd,
    Self::Pictor,
    Self::Pixlet,
    Self::Png,
    Self::Ppm,
    Self::Prores,
    Self::ProresRaw,
    Self::Prosumer,
    Self::Psd,
    Self::Ptx,
    Self::Qdraw,
    Self::Qoi,
    Self::Qpeg,
    Self::Qtrle,
    Self::R10k,
    Self::R210,
    Self::Rasc,
    Self::Rawvideo,
    Self::Rl2,
    Self::Roq,
    Self::Rpza,
    Self::Rscc,
    Self::Rtv1,
    Self::Rv10,
    Self::Rv20,
    Self::Rv30,
    Self::Rv40,
    Self::Rv60,
    Self::Sanm,
    Self::Scpr,
    Self::Screenpresso,
    Self::Sga,
    Self::Sgi,
    Self::Sgirle,
    Self::Sheervideo,
    Self::SimbiosisImx,
    Self::Smackvideo,
    Self::Smc,
    Self::Smvjpeg,
    Self::Snow,
    Self::Sp5x,
    Self::Speedhq,
    Self::Srgc,
    Self::Sunrast,
    Self::Svg,
    Self::Svq1,
    Self::Svq3,
    Self::Targa,
    Self::TargaY216,
    Self::Tdsc,
    Self::Tgq,
    Self::Tgv,
    Self::Theora,
    Self::Thp,
    Self::Tiertexseqvideo,
    Self::Tiff,
    Self::Tmv,
    Self::Tqi,
    Self::Truemotion1,
    Self::Truemotion2,
    Self::Truemotion2rt,
    Self::Tscc,
    Self::Tscc2,
    Self::Txd,
    Self::Ulti,
    Self::Utvideo,
    Self::V210,
    Self::V210x,
    Self::Vb,
    Self::Vble,
    Self::Vbn,
    Self::Vc1,
    Self::Vc1image,
    Self::Vcr1,
    Self::Vixl,
    Self::Vmdvideo,
    Self::Vmix,
    Self::Vmnc,
    Self::Vnull,
    Self::Vp3,
    Self::Vp4,
    Self::Vp5,
    Self::Vp6,
    Self::Vp6a,
    Self::Vp6f,
    Self::Vp7,
    Self::Vp8,
    Self::Vp9,
    Self::Vqc,
    Self::Vvc,
    Self::Wbmp,
    Self::Wcmv,
    Self::Webp,
    Self::WebpAnim,
    Self::Wmv1,
    Self::Wmv2,
    Self::Wmv3,
    Self::Wmv3image,
    Self::Wnv1,
    Self::WrappedAvframe,
    Self::WsVqa,
    Self::XanWc3,
    Self::XanWc4,
    Self::Xbin,
    Self::Xbm,
    Self::Xface,
    Self::Xpm,
    Self::Xwd,
    Self::Y41p,
    Self::Ylc,
    Self::Yop,
    Self::Yuv4,
    Self::Zerocodec,
    Self::Zlib,
    Self::Zmbv,
  ];
}
const _: () = {
  #[allow(dead_code)]
  fn every_variant_is_rostered(v: &VideoCodec) {
    match v {
      VideoCodec::N012v
      | VideoCodec::N4xm
      | VideoCodec::N8bps
      | VideoCodec::A64Multi
      | VideoCodec::A64Multi5
      | VideoCodec::Aasc
      | VideoCodec::Agm
      | VideoCodec::Aic
      | VideoCodec::AliasPix
      | VideoCodec::Amv
      | VideoCodec::Anm
      | VideoCodec::Ansi
      | VideoCodec::Apng
      | VideoCodec::Apv
      | VideoCodec::Arbc
      | VideoCodec::Argo
      | VideoCodec::Asv1
      | VideoCodec::Asv2
      | VideoCodec::Aura
      | VideoCodec::Aura2
      | VideoCodec::Av1
      | VideoCodec::Avrn
      | VideoCodec::Avrp
      | VideoCodec::Avs
      | VideoCodec::Avs2
      | VideoCodec::Avs3
      | VideoCodec::Avui
      | VideoCodec::Bethsoftvid
      | VideoCodec::Bfi
      | VideoCodec::Binkvideo
      | VideoCodec::Bintext
      | VideoCodec::Bitpacked
      | VideoCodec::Bmp
      | VideoCodec::BmvVideo
      | VideoCodec::BrenderPix
      | VideoCodec::C93
      | VideoCodec::Cavs
      | VideoCodec::Cdgraphics
      | VideoCodec::Cdtoons
      | VideoCodec::Cdxl
      | VideoCodec::Cfhd
      | VideoCodec::Cinepak
      | VideoCodec::Clearvideo
      | VideoCodec::Cljr
      | VideoCodec::Cllc
      | VideoCodec::Cmv
      | VideoCodec::Cpia
      | VideoCodec::Cri
      | VideoCodec::Cscd
      | VideoCodec::Cyuv
      | VideoCodec::Daala
      | VideoCodec::Dds
      | VideoCodec::Dfa
      | VideoCodec::Dirac
      | VideoCodec::Dnxhd
      | VideoCodec::Dnxuc
      | VideoCodec::Dpx
      | VideoCodec::Dsicinvideo
      | VideoCodec::Dvvideo
      | VideoCodec::Dxa
      | VideoCodec::Dxtory
      | VideoCodec::Dxv
      | VideoCodec::Escape124
      | VideoCodec::Escape130
      | VideoCodec::Evc
      | VideoCodec::Exr
      | VideoCodec::Ffv1
      | VideoCodec::Ffvhuff
      | VideoCodec::Fic
      | VideoCodec::Fits
      | VideoCodec::Flashsv
      | VideoCodec::Flashsv2
      | VideoCodec::Flic
      | VideoCodec::Flv1
      | VideoCodec::Fmvc
      | VideoCodec::Fraps
      | VideoCodec::Frwu
      | VideoCodec::G2m
      | VideoCodec::Gdv
      | VideoCodec::Gem
      | VideoCodec::Gif
      | VideoCodec::H261
      | VideoCodec::H263
      | VideoCodec::H263i
      | VideoCodec::H263p
      | VideoCodec::H264
      | VideoCodec::Hap
      | VideoCodec::Hdr
      | VideoCodec::Hevc
      | VideoCodec::Hnm4video
      | VideoCodec::HqHqa
      | VideoCodec::Hqx
      | VideoCodec::Huffyuv
      | VideoCodec::Hymt
      | VideoCodec::Idcin
      | VideoCodec::Idf
      | VideoCodec::IffIlbm
      | VideoCodec::Imm4
      | VideoCodec::Imm5
      | VideoCodec::Indeo2
      | VideoCodec::Indeo3
      | VideoCodec::Indeo4
      | VideoCodec::Indeo5
      | VideoCodec::Interplayvideo
      | VideoCodec::Ipu
      | VideoCodec::Jpeg2000
      | VideoCodec::Jpegls
      | VideoCodec::Jpegxl
      | VideoCodec::JpegxlAnim
      | VideoCodec::Jpegxs
      | VideoCodec::Jv
      | VideoCodec::Kgv1
      | VideoCodec::Kmvc
      | VideoCodec::Lagarith
      | VideoCodec::Lcevc
      | VideoCodec::Lead
      | VideoCodec::Ljpeg
      | VideoCodec::Loco
      | VideoCodec::Lscr
      | VideoCodec::M101
      | VideoCodec::Mad
      | VideoCodec::Magicyuv
      | VideoCodec::Mdec
      | VideoCodec::Media100
      | VideoCodec::Mimic
      | VideoCodec::Mjpeg
      | VideoCodec::Mjpegb
      | VideoCodec::Mmvideo
      | VideoCodec::Mobiclip
      | VideoCodec::Motionpixels
      | VideoCodec::Mpeg1video
      | VideoCodec::Mpeg2video
      | VideoCodec::Mpeg4
      | VideoCodec::Msa1
      | VideoCodec::Mscc
      | VideoCodec::Msmpeg4v1
      | VideoCodec::Msmpeg4v2
      | VideoCodec::Msmpeg4v3
      | VideoCodec::Msp2
      | VideoCodec::Msrle
      | VideoCodec::Mss1
      | VideoCodec::Mss2
      | VideoCodec::Msvideo1
      | VideoCodec::Mszh
      | VideoCodec::Mts2
      | VideoCodec::Mv30
      | VideoCodec::Mvc1
      | VideoCodec::Mvc2
      | VideoCodec::Mvdv
      | VideoCodec::Mvha
      | VideoCodec::Mwsc
      | VideoCodec::Mxpeg
      | VideoCodec::Notchlc
      | VideoCodec::Nuv
      | VideoCodec::PafVideo
      | VideoCodec::Pam
      | VideoCodec::Pbm
      | VideoCodec::Pcx
      | VideoCodec::Pdv
      | VideoCodec::Pfm
      | VideoCodec::Pgm
      | VideoCodec::Pgmyuv
      | VideoCodec::Pgx
      | VideoCodec::Phm
      | VideoCodec::Photocd
      | VideoCodec::Pictor
      | VideoCodec::Pixlet
      | VideoCodec::Png
      | VideoCodec::Ppm
      | VideoCodec::Prores
      | VideoCodec::ProresRaw
      | VideoCodec::Prosumer
      | VideoCodec::Psd
      | VideoCodec::Ptx
      | VideoCodec::Qdraw
      | VideoCodec::Qoi
      | VideoCodec::Qpeg
      | VideoCodec::Qtrle
      | VideoCodec::R10k
      | VideoCodec::R210
      | VideoCodec::Rasc
      | VideoCodec::Rawvideo
      | VideoCodec::Rl2
      | VideoCodec::Roq
      | VideoCodec::Rpza
      | VideoCodec::Rscc
      | VideoCodec::Rtv1
      | VideoCodec::Rv10
      | VideoCodec::Rv20
      | VideoCodec::Rv30
      | VideoCodec::Rv40
      | VideoCodec::Rv60
      | VideoCodec::Sanm
      | VideoCodec::Scpr
      | VideoCodec::Screenpresso
      | VideoCodec::Sga
      | VideoCodec::Sgi
      | VideoCodec::Sgirle
      | VideoCodec::Sheervideo
      | VideoCodec::SimbiosisImx
      | VideoCodec::Smackvideo
      | VideoCodec::Smc
      | VideoCodec::Smvjpeg
      | VideoCodec::Snow
      | VideoCodec::Sp5x
      | VideoCodec::Speedhq
      | VideoCodec::Srgc
      | VideoCodec::Sunrast
      | VideoCodec::Svg
      | VideoCodec::Svq1
      | VideoCodec::Svq3
      | VideoCodec::Targa
      | VideoCodec::TargaY216
      | VideoCodec::Tdsc
      | VideoCodec::Tgq
      | VideoCodec::Tgv
      | VideoCodec::Theora
      | VideoCodec::Thp
      | VideoCodec::Tiertexseqvideo
      | VideoCodec::Tiff
      | VideoCodec::Tmv
      | VideoCodec::Tqi
      | VideoCodec::Truemotion1
      | VideoCodec::Truemotion2
      | VideoCodec::Truemotion2rt
      | VideoCodec::Tscc
      | VideoCodec::Tscc2
      | VideoCodec::Txd
      | VideoCodec::Ulti
      | VideoCodec::Utvideo
      | VideoCodec::V210
      | VideoCodec::V210x
      | VideoCodec::Vb
      | VideoCodec::Vble
      | VideoCodec::Vbn
      | VideoCodec::Vc1
      | VideoCodec::Vc1image
      | VideoCodec::Vcr1
      | VideoCodec::Vixl
      | VideoCodec::Vmdvideo
      | VideoCodec::Vmix
      | VideoCodec::Vmnc
      | VideoCodec::Vnull
      | VideoCodec::Vp3
      | VideoCodec::Vp4
      | VideoCodec::Vp5
      | VideoCodec::Vp6
      | VideoCodec::Vp6a
      | VideoCodec::Vp6f
      | VideoCodec::Vp7
      | VideoCodec::Vp8
      | VideoCodec::Vp9
      | VideoCodec::Vqc
      | VideoCodec::Vvc
      | VideoCodec::Wbmp
      | VideoCodec::Wcmv
      | VideoCodec::Webp
      | VideoCodec::WebpAnim
      | VideoCodec::Wmv1
      | VideoCodec::Wmv2
      | VideoCodec::Wmv3
      | VideoCodec::Wmv3image
      | VideoCodec::Wnv1
      | VideoCodec::WrappedAvframe
      | VideoCodec::WsVqa
      | VideoCodec::XanWc3
      | VideoCodec::XanWc4
      | VideoCodec::Xbin
      | VideoCodec::Xbm
      | VideoCodec::Xface
      | VideoCodec::Xpm
      | VideoCodec::Xwd
      | VideoCodec::Y41p
      | VideoCodec::Ylc
      | VideoCodec::Yop
      | VideoCodec::Yuv4
      | VideoCodec::Zerocodec
      | VideoCodec::Zlib
      | VideoCodec::Zmbv => {}
      VideoCodec::Other(_) => {}
    }
  }
};
impl FromStr for VideoCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name, case-insensitively; unknown
  /// values land in [`Self::Other`] (infallible, lossless), carrying
  /// the caller's spelling verbatim.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"012v" => Self::N012v,
      b"4xm" => Self::N4xm,
      b"8bps" => Self::N8bps,
      b"a64_multi" => Self::A64Multi,
      b"a64_multi5" => Self::A64Multi5,
      b"aasc" => Self::Aasc,
      b"agm" => Self::Agm,
      b"aic" => Self::Aic,
      b"alias_pix" => Self::AliasPix,
      b"amv" => Self::Amv,
      b"anm" => Self::Anm,
      b"ansi" => Self::Ansi,
      b"apng" => Self::Apng,
      b"apv" => Self::Apv,
      b"arbc" => Self::Arbc,
      b"argo" => Self::Argo,
      b"asv1" => Self::Asv1,
      b"asv2" => Self::Asv2,
      b"aura" => Self::Aura,
      b"aura2" => Self::Aura2,
      b"av1" => Self::Av1,
      b"avrn" => Self::Avrn,
      b"avrp" => Self::Avrp,
      b"avs" => Self::Avs,
      b"avs2" => Self::Avs2,
      b"avs3" => Self::Avs3,
      b"avui" => Self::Avui,
      b"bethsoftvid" => Self::Bethsoftvid,
      b"bfi" => Self::Bfi,
      b"binkvideo" => Self::Binkvideo,
      b"bintext" => Self::Bintext,
      b"bitpacked" => Self::Bitpacked,
      b"bmp" => Self::Bmp,
      b"bmv_video" => Self::BmvVideo,
      b"brender_pix" => Self::BrenderPix,
      b"c93" => Self::C93,
      b"cavs" => Self::Cavs,
      b"cdgraphics" => Self::Cdgraphics,
      b"cdtoons" => Self::Cdtoons,
      b"cdxl" => Self::Cdxl,
      b"cfhd" => Self::Cfhd,
      b"cinepak" => Self::Cinepak,
      b"clearvideo" => Self::Clearvideo,
      b"cljr" => Self::Cljr,
      b"cllc" => Self::Cllc,
      b"cmv" => Self::Cmv,
      b"cpia" => Self::Cpia,
      b"cri" => Self::Cri,
      b"cscd" => Self::Cscd,
      b"cyuv" => Self::Cyuv,
      b"daala" => Self::Daala,
      b"dds" => Self::Dds,
      b"dfa" => Self::Dfa,
      b"dirac" => Self::Dirac,
      b"dnxhd" => Self::Dnxhd,
      b"dnxuc" => Self::Dnxuc,
      b"dpx" => Self::Dpx,
      b"dsicinvideo" => Self::Dsicinvideo,
      b"dvvideo" => Self::Dvvideo,
      b"dxa" => Self::Dxa,
      b"dxtory" => Self::Dxtory,
      b"dxv" => Self::Dxv,
      b"escape124" => Self::Escape124,
      b"escape130" => Self::Escape130,
      b"evc" => Self::Evc,
      b"exr" => Self::Exr,
      b"ffv1" => Self::Ffv1,
      b"ffvhuff" => Self::Ffvhuff,
      b"fic" => Self::Fic,
      b"fits" => Self::Fits,
      b"flashsv" => Self::Flashsv,
      b"flashsv2" => Self::Flashsv2,
      b"flic" => Self::Flic,
      b"flv1" => Self::Flv1,
      b"fmvc" => Self::Fmvc,
      b"fraps" => Self::Fraps,
      b"frwu" => Self::Frwu,
      b"g2m" => Self::G2m,
      b"gdv" => Self::Gdv,
      b"gem" => Self::Gem,
      b"gif" => Self::Gif,
      b"h261" => Self::H261,
      b"h263" => Self::H263,
      b"h263i" => Self::H263i,
      b"h263p" => Self::H263p,
      b"h264" => Self::H264,
      b"hap" => Self::Hap,
      b"hdr" => Self::Hdr,
      b"hevc" => Self::Hevc,
      b"hnm4video" => Self::Hnm4video,
      b"hq_hqa" => Self::HqHqa,
      b"hqx" => Self::Hqx,
      b"huffyuv" => Self::Huffyuv,
      b"hymt" => Self::Hymt,
      b"idcin" => Self::Idcin,
      b"idf" => Self::Idf,
      b"iff_ilbm" => Self::IffIlbm,
      b"imm4" => Self::Imm4,
      b"imm5" => Self::Imm5,
      b"indeo2" => Self::Indeo2,
      b"indeo3" => Self::Indeo3,
      b"indeo4" => Self::Indeo4,
      b"indeo5" => Self::Indeo5,
      b"interplayvideo" => Self::Interplayvideo,
      b"ipu" => Self::Ipu,
      b"jpeg2000" => Self::Jpeg2000,
      b"jpegls" => Self::Jpegls,
      b"jpegxl" => Self::Jpegxl,
      b"jpegxl_anim" => Self::JpegxlAnim,
      b"jpegxs" => Self::Jpegxs,
      b"jv" => Self::Jv,
      b"kgv1" => Self::Kgv1,
      b"kmvc" => Self::Kmvc,
      b"lagarith" => Self::Lagarith,
      b"lcevc" => Self::Lcevc,
      b"lead" => Self::Lead,
      b"ljpeg" => Self::Ljpeg,
      b"loco" => Self::Loco,
      b"lscr" => Self::Lscr,
      b"m101" => Self::M101,
      b"mad" => Self::Mad,
      b"magicyuv" => Self::Magicyuv,
      b"mdec" => Self::Mdec,
      b"media100" => Self::Media100,
      b"mimic" => Self::Mimic,
      b"mjpeg" => Self::Mjpeg,
      b"mjpegb" => Self::Mjpegb,
      b"mmvideo" => Self::Mmvideo,
      b"mobiclip" => Self::Mobiclip,
      b"motionpixels" => Self::Motionpixels,
      b"mpeg1video" => Self::Mpeg1video,
      b"mpeg2video" => Self::Mpeg2video,
      b"mpeg4" => Self::Mpeg4,
      b"msa1" => Self::Msa1,
      b"mscc" => Self::Mscc,
      b"msmpeg4v1" => Self::Msmpeg4v1,
      b"msmpeg4v2" => Self::Msmpeg4v2,
      b"msmpeg4v3" => Self::Msmpeg4v3,
      b"msp2" => Self::Msp2,
      b"msrle" => Self::Msrle,
      b"mss1" => Self::Mss1,
      b"mss2" => Self::Mss2,
      b"msvideo1" => Self::Msvideo1,
      b"mszh" => Self::Mszh,
      b"mts2" => Self::Mts2,
      b"mv30" => Self::Mv30,
      b"mvc1" => Self::Mvc1,
      b"mvc2" => Self::Mvc2,
      b"mvdv" => Self::Mvdv,
      b"mvha" => Self::Mvha,
      b"mwsc" => Self::Mwsc,
      b"mxpeg" => Self::Mxpeg,
      b"notchlc" => Self::Notchlc,
      b"nuv" => Self::Nuv,
      b"paf_video" => Self::PafVideo,
      b"pam" => Self::Pam,
      b"pbm" => Self::Pbm,
      b"pcx" => Self::Pcx,
      b"pdv" => Self::Pdv,
      b"pfm" => Self::Pfm,
      b"pgm" => Self::Pgm,
      b"pgmyuv" => Self::Pgmyuv,
      b"pgx" => Self::Pgx,
      b"phm" => Self::Phm,
      b"photocd" => Self::Photocd,
      b"pictor" => Self::Pictor,
      b"pixlet" => Self::Pixlet,
      b"png" => Self::Png,
      b"ppm" => Self::Ppm,
      b"prores" => Self::Prores,
      b"prores_raw" => Self::ProresRaw,
      b"prosumer" => Self::Prosumer,
      b"psd" => Self::Psd,
      b"ptx" => Self::Ptx,
      b"qdraw" => Self::Qdraw,
      b"qoi" => Self::Qoi,
      b"qpeg" => Self::Qpeg,
      b"qtrle" => Self::Qtrle,
      b"r10k" => Self::R10k,
      b"r210" => Self::R210,
      b"rasc" => Self::Rasc,
      b"rawvideo" => Self::Rawvideo,
      b"rl2" => Self::Rl2,
      b"roq" => Self::Roq,
      b"rpza" => Self::Rpza,
      b"rscc" => Self::Rscc,
      b"rtv1" => Self::Rtv1,
      b"rv10" => Self::Rv10,
      b"rv20" => Self::Rv20,
      b"rv30" => Self::Rv30,
      b"rv40" => Self::Rv40,
      b"rv60" => Self::Rv60,
      b"sanm" => Self::Sanm,
      b"scpr" => Self::Scpr,
      b"screenpresso" => Self::Screenpresso,
      b"sga" => Self::Sga,
      b"sgi" => Self::Sgi,
      b"sgirle" => Self::Sgirle,
      b"sheervideo" => Self::Sheervideo,
      b"simbiosis_imx" => Self::SimbiosisImx,
      b"smackvideo" => Self::Smackvideo,
      b"smc" => Self::Smc,
      b"smvjpeg" => Self::Smvjpeg,
      b"snow" => Self::Snow,
      b"sp5x" => Self::Sp5x,
      b"speedhq" => Self::Speedhq,
      b"srgc" => Self::Srgc,
      b"sunrast" => Self::Sunrast,
      b"svg" => Self::Svg,
      b"svq1" => Self::Svq1,
      b"svq3" => Self::Svq3,
      b"targa" => Self::Targa,
      b"targa_y216" => Self::TargaY216,
      b"tdsc" => Self::Tdsc,
      b"tgq" => Self::Tgq,
      b"tgv" => Self::Tgv,
      b"theora" => Self::Theora,
      b"thp" => Self::Thp,
      b"tiertexseqvideo" => Self::Tiertexseqvideo,
      b"tiff" => Self::Tiff,
      b"tmv" => Self::Tmv,
      b"tqi" => Self::Tqi,
      b"truemotion1" => Self::Truemotion1,
      b"truemotion2" => Self::Truemotion2,
      b"truemotion2rt" => Self::Truemotion2rt,
      b"tscc" => Self::Tscc,
      b"tscc2" => Self::Tscc2,
      b"txd" => Self::Txd,
      b"ulti" => Self::Ulti,
      b"utvideo" => Self::Utvideo,
      b"v210" => Self::V210,
      b"v210x" => Self::V210x,
      b"vb" => Self::Vb,
      b"vble" => Self::Vble,
      b"vbn" => Self::Vbn,
      b"vc1" => Self::Vc1,
      b"vc1image" => Self::Vc1image,
      b"vcr1" => Self::Vcr1,
      b"vixl" => Self::Vixl,
      b"vmdvideo" => Self::Vmdvideo,
      b"vmix" => Self::Vmix,
      b"vmnc" => Self::Vmnc,
      b"vnull" => Self::Vnull,
      b"vp3" => Self::Vp3,
      b"vp4" => Self::Vp4,
      b"vp5" => Self::Vp5,
      b"vp6" => Self::Vp6,
      b"vp6a" => Self::Vp6a,
      b"vp6f" => Self::Vp6f,
      b"vp7" => Self::Vp7,
      b"vp8" => Self::Vp8,
      b"vp9" => Self::Vp9,
      b"vqc" => Self::Vqc,
      b"vvc" => Self::Vvc,
      b"wbmp" => Self::Wbmp,
      b"wcmv" => Self::Wcmv,
      b"webp" => Self::Webp,
      b"webp_anim" => Self::WebpAnim,
      b"wmv1" => Self::Wmv1,
      b"wmv2" => Self::Wmv2,
      b"wmv3" => Self::Wmv3,
      b"wmv3image" => Self::Wmv3image,
      b"wnv1" => Self::Wnv1,
      b"wrapped_avframe" => Self::WrappedAvframe,
      b"ws_vqa" => Self::WsVqa,
      b"xan_wc3" => Self::XanWc3,
      b"xan_wc4" => Self::XanWc4,
      b"xbin" => Self::Xbin,
      b"xbm" => Self::Xbm,
      b"xface" => Self::Xface,
      b"xpm" => Self::Xpm,
      b"xwd" => Self::Xwd,
      b"y41p" => Self::Y41p,
      b"ylc" => Self::Ylc,
      b"yop" => Self::Yop,
      b"yuv4" => Self::Yuv4,
      b"zerocodec" => Self::Zerocodec,
      b"zlib" => Self::Zlib,
      b"zmbv" => Self::Zmbv,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}
/** Audio codec family — every codec FFmpeg n9.0 knows under media type `audio`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::audio_codec")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum AudioCodec {
  /// FFmpeg `"4gv"`.
  N4gv,
  /// FFmpeg `"8svx_exp"`.
  N8svxExp,
  /// FFmpeg `"8svx_fib"`.
  N8svxFib,
  /// FFmpeg `"aac"`.
  Aac,
  /// FFmpeg `"aac_latm"`.
  AacLatm,
  /// FFmpeg `"ac3"`.
  Ac3,
  /// FFmpeg `"ac4"`.
  Ac4,
  /// FFmpeg `"acelp.kelvin"`.
  AcelpKelvin,
  /// FFmpeg `"adpcm_4xm"`.
  Adpcm4xm,
  /// FFmpeg `"adpcm_adx"`.
  AdpcmAdx,
  /// FFmpeg `"adpcm_afc"`.
  AdpcmAfc,
  /// FFmpeg `"adpcm_agm"`.
  AdpcmAgm,
  /// FFmpeg `"adpcm_aica"`.
  AdpcmAica,
  /// FFmpeg `"adpcm_argo"`.
  AdpcmArgo,
  /// FFmpeg `"adpcm_circus"`.
  AdpcmCircus,
  /// FFmpeg `"adpcm_ct"`.
  AdpcmCt,
  /// FFmpeg `"adpcm_dtk"`.
  AdpcmDtk,
  /// FFmpeg `"adpcm_ea"`.
  AdpcmEa,
  /// FFmpeg `"adpcm_ea_maxis_xa"`.
  AdpcmEaMaxisXa,
  /// FFmpeg `"adpcm_ea_r1"`.
  AdpcmEaR1,
  /// FFmpeg `"adpcm_ea_r2"`.
  AdpcmEaR2,
  /// FFmpeg `"adpcm_ea_r3"`.
  AdpcmEaR3,
  /// FFmpeg `"adpcm_ea_xas"`.
  AdpcmEaXas,
  /// FFmpeg `"adpcm_g722"`.
  AdpcmG722,
  /// FFmpeg `"adpcm_g726"`.
  AdpcmG726,
  /// FFmpeg `"adpcm_g726le"`.
  AdpcmG726le,
  /// FFmpeg `"adpcm_ima_acorn"`.
  AdpcmImaAcorn,
  /// FFmpeg `"adpcm_ima_alp"`.
  AdpcmImaAlp,
  /// FFmpeg `"adpcm_ima_amv"`.
  AdpcmImaAmv,
  /// FFmpeg `"adpcm_ima_apc"`.
  AdpcmImaApc,
  /// FFmpeg `"adpcm_ima_apm"`.
  AdpcmImaApm,
  /// FFmpeg `"adpcm_ima_cunning"`.
  AdpcmImaCunning,
  /// FFmpeg `"adpcm_ima_dat4"`.
  AdpcmImaDat4,
  /// FFmpeg `"adpcm_ima_dk3"`.
  AdpcmImaDk3,
  /// FFmpeg `"adpcm_ima_dk4"`.
  AdpcmImaDk4,
  /// FFmpeg `"adpcm_ima_ea_eacs"`.
  AdpcmImaEaEacs,
  /// FFmpeg `"adpcm_ima_ea_sead"`.
  AdpcmImaEaSead,
  /// FFmpeg `"adpcm_ima_escape"`.
  AdpcmImaEscape,
  /// FFmpeg `"adpcm_ima_hvqm2"`.
  AdpcmImaHvqm2,
  /// FFmpeg `"adpcm_ima_hvqm4"`.
  AdpcmImaHvqm4,
  /// FFmpeg `"adpcm_ima_iss"`.
  AdpcmImaIss,
  /// FFmpeg `"adpcm_ima_magix"`.
  AdpcmImaMagix,
  /// FFmpeg `"adpcm_ima_moflex"`.
  AdpcmImaMoflex,
  /// FFmpeg `"adpcm_ima_mtf"`.
  AdpcmImaMtf,
  /// FFmpeg `"adpcm_ima_oki"`.
  AdpcmImaOki,
  /// FFmpeg `"adpcm_ima_pda"`.
  AdpcmImaPda,
  /// FFmpeg `"adpcm_ima_qt"`.
  AdpcmImaQt,
  /// FFmpeg `"adpcm_ima_rad"`.
  AdpcmImaRad,
  /// FFmpeg `"adpcm_ima_smjpeg"`.
  AdpcmImaSmjpeg,
  /// FFmpeg `"adpcm_ima_ssi"`.
  AdpcmImaSsi,
  /// FFmpeg `"adpcm_ima_wav"`.
  AdpcmImaWav,
  /// FFmpeg `"adpcm_ima_ws"`.
  AdpcmImaWs,
  /// FFmpeg `"adpcm_ima_xbox"`.
  AdpcmImaXbox,
  /// FFmpeg `"adpcm_ms"`.
  AdpcmMs,
  /// FFmpeg `"adpcm_mtaf"`.
  AdpcmMtaf,
  /// FFmpeg `"adpcm_n64"`.
  AdpcmN64,
  /// FFmpeg `"adpcm_psx"`.
  AdpcmPsx,
  /// FFmpeg `"adpcm_psxc"`.
  AdpcmPsxc,
  /// FFmpeg `"adpcm_sanyo"`.
  AdpcmSanyo,
  /// FFmpeg `"adpcm_sbpro_2"`.
  AdpcmSbpro2,
  /// FFmpeg `"adpcm_sbpro_3"`.
  AdpcmSbpro3,
  /// FFmpeg `"adpcm_sbpro_4"`.
  AdpcmSbpro4,
  /// FFmpeg `"adpcm_swf"`.
  AdpcmSwf,
  /// FFmpeg `"adpcm_thp"`.
  AdpcmThp,
  /// FFmpeg `"adpcm_thp_le"`.
  AdpcmThpLe,
  /// FFmpeg `"adpcm_vima"`.
  AdpcmVima,
  /// FFmpeg `"adpcm_xa"`.
  AdpcmXa,
  /// FFmpeg `"adpcm_xmd"`.
  AdpcmXmd,
  /// FFmpeg `"adpcm_yamaha"`.
  AdpcmYamaha,
  /// FFmpeg `"adpcm_zork"`.
  AdpcmZork,
  /// FFmpeg `"ahx"`.
  Ahx,
  /// FFmpeg `"alac"`.
  Alac,
  /// FFmpeg `"amr_nb"`.
  AmrNb,
  /// FFmpeg `"amr_wb"`.
  AmrWb,
  /// FFmpeg `"anull"`.
  Anull,
  /// FFmpeg `"apac"`.
  Apac,
  /// FFmpeg `"ape"`.
  Ape,
  /// FFmpeg `"apple_apac"`.
  AppleApac,
  /// FFmpeg `"aptx"`.
  Aptx,
  /// FFmpeg `"aptx_hd"`.
  AptxHd,
  /// FFmpeg `"atrac1"`.
  Atrac1,
  /// FFmpeg `"atrac3"`.
  Atrac3,
  /// FFmpeg `"atrac3al"`.
  Atrac3al,
  /// FFmpeg `"atrac3p"`.
  Atrac3p,
  /// FFmpeg `"atrac3pal"`.
  Atrac3pal,
  /// FFmpeg `"atrac9"`.
  Atrac9,
  /// FFmpeg `"avc"`.
  Avc,
  /// FFmpeg `"binkaudio_dct"`.
  BinkaudioDct,
  /// FFmpeg `"binkaudio_rdft"`.
  BinkaudioRdft,
  /// FFmpeg `"bmv_audio"`.
  BmvAudio,
  /// FFmpeg `"bonk"`.
  Bonk,
  /// FFmpeg `"cbd2_dpcm"`.
  Cbd2Dpcm,
  /// FFmpeg `"celt"`.
  Celt,
  /// FFmpeg `"codec2"`.
  Codec2,
  /// FFmpeg `"comfortnoise"`.
  Comfortnoise,
  /// FFmpeg `"cook"`.
  Cook,
  /// FFmpeg `"derf_dpcm"`.
  DerfDpcm,
  /// FFmpeg `"dfpwm"`.
  Dfpwm,
  /// FFmpeg `"dolby_e"`.
  DolbyE,
  /// FFmpeg `"dsd_lsbf"`.
  DsdLsbf,
  /// FFmpeg `"dsd_lsbf_planar"`.
  DsdLsbfPlanar,
  /// FFmpeg `"dsd_msbf"`.
  DsdMsbf,
  /// FFmpeg `"dsd_msbf_planar"`.
  DsdMsbfPlanar,
  /// FFmpeg `"dsicinaudio"`.
  Dsicinaudio,
  /// FFmpeg `"dss_sp"`.
  DssSp,
  /// FFmpeg `"dst"`.
  Dst,
  /// FFmpeg `"dts"`.
  Dts,
  /// FFmpeg `"dvaudio"`.
  Dvaudio,
  /// FFmpeg `"eac3"`.
  Eac3,
  /// FFmpeg `"evrc"`.
  Evrc,
  /// FFmpeg `"fastaudio"`.
  Fastaudio,
  /// FFmpeg `"flac"`.
  Flac,
  /// FFmpeg `"ftr"`.
  Ftr,
  /// FFmpeg `"g723_1"`.
  G7231,
  /// FFmpeg `"g728"`.
  G728,
  /// FFmpeg `"g729"`.
  G729,
  /// FFmpeg `"gremlin_dpcm"`.
  GremlinDpcm,
  /// FFmpeg `"gsm"`.
  Gsm,
  /// FFmpeg `"gsm_ms"`.
  GsmMs,
  /// FFmpeg `"hca"`.
  Hca,
  /// FFmpeg `"hcom"`.
  Hcom,
  /// FFmpeg `"iac"`.
  Iac,
  /// FFmpeg `"ilbc"`.
  Ilbc,
  /// FFmpeg `"imc"`.
  Imc,
  /// FFmpeg `"interplay_dpcm"`.
  InterplayDpcm,
  /// FFmpeg `"interplayacm"`.
  Interplayacm,
  /// FFmpeg `"lc3"`.
  Lc3,
  /// FFmpeg `"mace3"`.
  Mace3,
  /// FFmpeg `"mace6"`.
  Mace6,
  /// FFmpeg `"metasound"`.
  Metasound,
  /// FFmpeg `"misc4"`.
  Misc4,
  /// FFmpeg `"mlp"`.
  Mlp,
  /// FFmpeg `"mp1"`.
  Mp1,
  /// FFmpeg `"mp2"`.
  Mp2,
  /// FFmpeg `"mp3"`.
  Mp3,
  /// FFmpeg `"mp3adu"`.
  Mp3adu,
  /// FFmpeg `"mp3on4"`.
  Mp3on4,
  /// FFmpeg `"mp4als"`.
  Mp4als,
  /// FFmpeg `"mpegh_3d_audio"`.
  Mpegh3dAudio,
  /// FFmpeg `"msnsiren"`.
  Msnsiren,
  /// FFmpeg `"musepack7"`.
  Musepack7,
  /// FFmpeg `"musepack8"`.
  Musepack8,
  /// FFmpeg `"nellymoser"`.
  Nellymoser,
  /// FFmpeg `"opus"`.
  Opus,
  /// FFmpeg `"osq"`.
  Osq,
  /// FFmpeg `"paf_audio"`.
  PafAudio,
  /// FFmpeg `"pcm_alaw"`.
  PcmAlaw,
  /// FFmpeg `"pcm_bluray"`.
  PcmBluray,
  /// FFmpeg `"pcm_dvd"`.
  PcmDvd,
  /// FFmpeg `"pcm_f16le"`.
  PcmF16le,
  /// FFmpeg `"pcm_f24le"`.
  PcmF24le,
  /// FFmpeg `"pcm_f32be"`.
  PcmF32be,
  /// FFmpeg `"pcm_f32le"`.
  PcmF32le,
  /// FFmpeg `"pcm_f64be"`.
  PcmF64be,
  /// FFmpeg `"pcm_f64le"`.
  PcmF64le,
  /// FFmpeg `"pcm_lxf"`.
  PcmLxf,
  /// FFmpeg `"pcm_mulaw"`.
  PcmMulaw,
  /// FFmpeg `"pcm_s16be"`.
  PcmS16be,
  /// FFmpeg `"pcm_s16be_planar"`.
  PcmS16bePlanar,
  /// FFmpeg `"pcm_s16le"`.
  PcmS16le,
  /// FFmpeg `"pcm_s16le_planar"`.
  PcmS16lePlanar,
  /// FFmpeg `"pcm_s24be"`.
  PcmS24be,
  /// FFmpeg `"pcm_s24daud"`.
  PcmS24daud,
  /// FFmpeg `"pcm_s24le"`.
  PcmS24le,
  /// FFmpeg `"pcm_s24le_planar"`.
  PcmS24lePlanar,
  /// FFmpeg `"pcm_s32be"`.
  PcmS32be,
  /// FFmpeg `"pcm_s32le"`.
  PcmS32le,
  /// FFmpeg `"pcm_s32le_planar"`.
  PcmS32lePlanar,
  /// FFmpeg `"pcm_s64be"`.
  PcmS64be,
  /// FFmpeg `"pcm_s64le"`.
  PcmS64le,
  /// FFmpeg `"pcm_s8"`.
  PcmS8,
  /// FFmpeg `"pcm_s8_planar"`.
  PcmS8Planar,
  /// FFmpeg `"pcm_sga"`.
  PcmSga,
  /// FFmpeg `"pcm_u16be"`.
  PcmU16be,
  /// FFmpeg `"pcm_u16le"`.
  PcmU16le,
  /// FFmpeg `"pcm_u24be"`.
  PcmU24be,
  /// FFmpeg `"pcm_u24le"`.
  PcmU24le,
  /// FFmpeg `"pcm_u32be"`.
  PcmU32be,
  /// FFmpeg `"pcm_u32le"`.
  PcmU32le,
  /// FFmpeg `"pcm_u8"`.
  PcmU8,
  /// FFmpeg `"pcm_vidc"`.
  PcmVidc,
  /// FFmpeg `"qcelp"`.
  Qcelp,
  /// FFmpeg `"qdm2"`.
  Qdm2,
  /// FFmpeg `"qdmc"`.
  Qdmc,
  /// FFmpeg `"qoa"`.
  Qoa,
  /// FFmpeg `"ra_144"`.
  Ra144,
  /// FFmpeg `"ra_288"`.
  Ra288,
  /// FFmpeg `"ralf"`.
  Ralf,
  /// FFmpeg `"rka"`.
  Rka,
  /// FFmpeg `"roq_dpcm"`.
  RoqDpcm,
  /// FFmpeg `"s302m"`.
  S302m,
  /// FFmpeg `"sbc"`.
  Sbc,
  /// FFmpeg `"sdx2_dpcm"`.
  Sdx2Dpcm,
  /// FFmpeg `"shorten"`.
  Shorten,
  /// FFmpeg `"sipr"`.
  Sipr,
  /// FFmpeg `"siren"`.
  Siren,
  /// FFmpeg `"smackaudio"`.
  Smackaudio,
  /// FFmpeg `"smv"`.
  Smv,
  /// FFmpeg `"sol_dpcm"`.
  SolDpcm,
  /// FFmpeg `"sonic"`.
  Sonic,
  /// FFmpeg `"sonicls"`.
  Sonicls,
  /// FFmpeg `"speex"`.
  Speex,
  /// FFmpeg `"tak"`.
  Tak,
  /// FFmpeg `"truehd"`.
  Truehd,
  /// FFmpeg `"truespeech"`.
  Truespeech,
  /// FFmpeg `"tta"`.
  Tta,
  /// FFmpeg `"twinvq"`.
  Twinvq,
  /// FFmpeg `"vmdaudio"`.
  Vmdaudio,
  /// FFmpeg `"vorbis"`.
  Vorbis,
  /// FFmpeg `"wady_dpcm"`.
  WadyDpcm,
  /// FFmpeg `"wavarc"`.
  Wavarc,
  /// FFmpeg `"wavesynth"`.
  Wavesynth,
  /// FFmpeg `"wavpack"`.
  Wavpack,
  /// FFmpeg `"westwood_snd1"`.
  WestwoodSnd1,
  /// FFmpeg `"wmalossless"`.
  Wmalossless,
  /// FFmpeg `"wmapro"`.
  Wmapro,
  /// FFmpeg `"wmav1"`.
  Wmav1,
  /// FFmpeg `"wmav2"`.
  Wmav2,
  /// FFmpeg `"wmavoice"`.
  Wmavoice,
  /// FFmpeg `"xan_dpcm"`.
  XanDpcm,
  /// FFmpeg `"xma1"`.
  Xma1,
  /// FFmpeg `"xma2"`.
  Xma2,
  /// A codec not enumerated above — carries the FFmpeg short name
  /// verbatim.
  Other(Utf8Bytes),
}
impl AudioCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::N4gv => "4gv",
      Self::N8svxExp => "8svx_exp",
      Self::N8svxFib => "8svx_fib",
      Self::Aac => "aac",
      Self::AacLatm => "aac_latm",
      Self::Ac3 => "ac3",
      Self::Ac4 => "ac4",
      Self::AcelpKelvin => "acelp.kelvin",
      Self::Adpcm4xm => "adpcm_4xm",
      Self::AdpcmAdx => "adpcm_adx",
      Self::AdpcmAfc => "adpcm_afc",
      Self::AdpcmAgm => "adpcm_agm",
      Self::AdpcmAica => "adpcm_aica",
      Self::AdpcmArgo => "adpcm_argo",
      Self::AdpcmCircus => "adpcm_circus",
      Self::AdpcmCt => "adpcm_ct",
      Self::AdpcmDtk => "adpcm_dtk",
      Self::AdpcmEa => "adpcm_ea",
      Self::AdpcmEaMaxisXa => "adpcm_ea_maxis_xa",
      Self::AdpcmEaR1 => "adpcm_ea_r1",
      Self::AdpcmEaR2 => "adpcm_ea_r2",
      Self::AdpcmEaR3 => "adpcm_ea_r3",
      Self::AdpcmEaXas => "adpcm_ea_xas",
      Self::AdpcmG722 => "adpcm_g722",
      Self::AdpcmG726 => "adpcm_g726",
      Self::AdpcmG726le => "adpcm_g726le",
      Self::AdpcmImaAcorn => "adpcm_ima_acorn",
      Self::AdpcmImaAlp => "adpcm_ima_alp",
      Self::AdpcmImaAmv => "adpcm_ima_amv",
      Self::AdpcmImaApc => "adpcm_ima_apc",
      Self::AdpcmImaApm => "adpcm_ima_apm",
      Self::AdpcmImaCunning => "adpcm_ima_cunning",
      Self::AdpcmImaDat4 => "adpcm_ima_dat4",
      Self::AdpcmImaDk3 => "adpcm_ima_dk3",
      Self::AdpcmImaDk4 => "adpcm_ima_dk4",
      Self::AdpcmImaEaEacs => "adpcm_ima_ea_eacs",
      Self::AdpcmImaEaSead => "adpcm_ima_ea_sead",
      Self::AdpcmImaEscape => "adpcm_ima_escape",
      Self::AdpcmImaHvqm2 => "adpcm_ima_hvqm2",
      Self::AdpcmImaHvqm4 => "adpcm_ima_hvqm4",
      Self::AdpcmImaIss => "adpcm_ima_iss",
      Self::AdpcmImaMagix => "adpcm_ima_magix",
      Self::AdpcmImaMoflex => "adpcm_ima_moflex",
      Self::AdpcmImaMtf => "adpcm_ima_mtf",
      Self::AdpcmImaOki => "adpcm_ima_oki",
      Self::AdpcmImaPda => "adpcm_ima_pda",
      Self::AdpcmImaQt => "adpcm_ima_qt",
      Self::AdpcmImaRad => "adpcm_ima_rad",
      Self::AdpcmImaSmjpeg => "adpcm_ima_smjpeg",
      Self::AdpcmImaSsi => "adpcm_ima_ssi",
      Self::AdpcmImaWav => "adpcm_ima_wav",
      Self::AdpcmImaWs => "adpcm_ima_ws",
      Self::AdpcmImaXbox => "adpcm_ima_xbox",
      Self::AdpcmMs => "adpcm_ms",
      Self::AdpcmMtaf => "adpcm_mtaf",
      Self::AdpcmN64 => "adpcm_n64",
      Self::AdpcmPsx => "adpcm_psx",
      Self::AdpcmPsxc => "adpcm_psxc",
      Self::AdpcmSanyo => "adpcm_sanyo",
      Self::AdpcmSbpro2 => "adpcm_sbpro_2",
      Self::AdpcmSbpro3 => "adpcm_sbpro_3",
      Self::AdpcmSbpro4 => "adpcm_sbpro_4",
      Self::AdpcmSwf => "adpcm_swf",
      Self::AdpcmThp => "adpcm_thp",
      Self::AdpcmThpLe => "adpcm_thp_le",
      Self::AdpcmVima => "adpcm_vima",
      Self::AdpcmXa => "adpcm_xa",
      Self::AdpcmXmd => "adpcm_xmd",
      Self::AdpcmYamaha => "adpcm_yamaha",
      Self::AdpcmZork => "adpcm_zork",
      Self::Ahx => "ahx",
      Self::Alac => "alac",
      Self::AmrNb => "amr_nb",
      Self::AmrWb => "amr_wb",
      Self::Anull => "anull",
      Self::Apac => "apac",
      Self::Ape => "ape",
      Self::AppleApac => "apple_apac",
      Self::Aptx => "aptx",
      Self::AptxHd => "aptx_hd",
      Self::Atrac1 => "atrac1",
      Self::Atrac3 => "atrac3",
      Self::Atrac3al => "atrac3al",
      Self::Atrac3p => "atrac3p",
      Self::Atrac3pal => "atrac3pal",
      Self::Atrac9 => "atrac9",
      Self::Avc => "avc",
      Self::BinkaudioDct => "binkaudio_dct",
      Self::BinkaudioRdft => "binkaudio_rdft",
      Self::BmvAudio => "bmv_audio",
      Self::Bonk => "bonk",
      Self::Cbd2Dpcm => "cbd2_dpcm",
      Self::Celt => "celt",
      Self::Codec2 => "codec2",
      Self::Comfortnoise => "comfortnoise",
      Self::Cook => "cook",
      Self::DerfDpcm => "derf_dpcm",
      Self::Dfpwm => "dfpwm",
      Self::DolbyE => "dolby_e",
      Self::DsdLsbf => "dsd_lsbf",
      Self::DsdLsbfPlanar => "dsd_lsbf_planar",
      Self::DsdMsbf => "dsd_msbf",
      Self::DsdMsbfPlanar => "dsd_msbf_planar",
      Self::Dsicinaudio => "dsicinaudio",
      Self::DssSp => "dss_sp",
      Self::Dst => "dst",
      Self::Dts => "dts",
      Self::Dvaudio => "dvaudio",
      Self::Eac3 => "eac3",
      Self::Evrc => "evrc",
      Self::Fastaudio => "fastaudio",
      Self::Flac => "flac",
      Self::Ftr => "ftr",
      Self::G7231 => "g723_1",
      Self::G728 => "g728",
      Self::G729 => "g729",
      Self::GremlinDpcm => "gremlin_dpcm",
      Self::Gsm => "gsm",
      Self::GsmMs => "gsm_ms",
      Self::Hca => "hca",
      Self::Hcom => "hcom",
      Self::Iac => "iac",
      Self::Ilbc => "ilbc",
      Self::Imc => "imc",
      Self::InterplayDpcm => "interplay_dpcm",
      Self::Interplayacm => "interplayacm",
      Self::Lc3 => "lc3",
      Self::Mace3 => "mace3",
      Self::Mace6 => "mace6",
      Self::Metasound => "metasound",
      Self::Misc4 => "misc4",
      Self::Mlp => "mlp",
      Self::Mp1 => "mp1",
      Self::Mp2 => "mp2",
      Self::Mp3 => "mp3",
      Self::Mp3adu => "mp3adu",
      Self::Mp3on4 => "mp3on4",
      Self::Mp4als => "mp4als",
      Self::Mpegh3dAudio => "mpegh_3d_audio",
      Self::Msnsiren => "msnsiren",
      Self::Musepack7 => "musepack7",
      Self::Musepack8 => "musepack8",
      Self::Nellymoser => "nellymoser",
      Self::Opus => "opus",
      Self::Osq => "osq",
      Self::PafAudio => "paf_audio",
      Self::PcmAlaw => "pcm_alaw",
      Self::PcmBluray => "pcm_bluray",
      Self::PcmDvd => "pcm_dvd",
      Self::PcmF16le => "pcm_f16le",
      Self::PcmF24le => "pcm_f24le",
      Self::PcmF32be => "pcm_f32be",
      Self::PcmF32le => "pcm_f32le",
      Self::PcmF64be => "pcm_f64be",
      Self::PcmF64le => "pcm_f64le",
      Self::PcmLxf => "pcm_lxf",
      Self::PcmMulaw => "pcm_mulaw",
      Self::PcmS16be => "pcm_s16be",
      Self::PcmS16bePlanar => "pcm_s16be_planar",
      Self::PcmS16le => "pcm_s16le",
      Self::PcmS16lePlanar => "pcm_s16le_planar",
      Self::PcmS24be => "pcm_s24be",
      Self::PcmS24daud => "pcm_s24daud",
      Self::PcmS24le => "pcm_s24le",
      Self::PcmS24lePlanar => "pcm_s24le_planar",
      Self::PcmS32be => "pcm_s32be",
      Self::PcmS32le => "pcm_s32le",
      Self::PcmS32lePlanar => "pcm_s32le_planar",
      Self::PcmS64be => "pcm_s64be",
      Self::PcmS64le => "pcm_s64le",
      Self::PcmS8 => "pcm_s8",
      Self::PcmS8Planar => "pcm_s8_planar",
      Self::PcmSga => "pcm_sga",
      Self::PcmU16be => "pcm_u16be",
      Self::PcmU16le => "pcm_u16le",
      Self::PcmU24be => "pcm_u24be",
      Self::PcmU24le => "pcm_u24le",
      Self::PcmU32be => "pcm_u32be",
      Self::PcmU32le => "pcm_u32le",
      Self::PcmU8 => "pcm_u8",
      Self::PcmVidc => "pcm_vidc",
      Self::Qcelp => "qcelp",
      Self::Qdm2 => "qdm2",
      Self::Qdmc => "qdmc",
      Self::Qoa => "qoa",
      Self::Ra144 => "ra_144",
      Self::Ra288 => "ra_288",
      Self::Ralf => "ralf",
      Self::Rka => "rka",
      Self::RoqDpcm => "roq_dpcm",
      Self::S302m => "s302m",
      Self::Sbc => "sbc",
      Self::Sdx2Dpcm => "sdx2_dpcm",
      Self::Shorten => "shorten",
      Self::Sipr => "sipr",
      Self::Siren => "siren",
      Self::Smackaudio => "smackaudio",
      Self::Smv => "smv",
      Self::SolDpcm => "sol_dpcm",
      Self::Sonic => "sonic",
      Self::Sonicls => "sonicls",
      Self::Speex => "speex",
      Self::Tak => "tak",
      Self::Truehd => "truehd",
      Self::Truespeech => "truespeech",
      Self::Tta => "tta",
      Self::Twinvq => "twinvq",
      Self::Vmdaudio => "vmdaudio",
      Self::Vorbis => "vorbis",
      Self::WadyDpcm => "wady_dpcm",
      Self::Wavarc => "wavarc",
      Self::Wavesynth => "wavesynth",
      Self::Wavpack => "wavpack",
      Self::WestwoodSnd1 => "westwood_snd1",
      Self::Wmalossless => "wmalossless",
      Self::Wmapro => "wmapro",
      Self::Wmav1 => "wmav1",
      Self::Wmav2 => "wmav2",
      Self::Wmavoice => "wmavoice",
      Self::XanDpcm => "xan_dpcm",
      Self::Xma1 => "xma1",
      Self::Xma2 => "xma2",
      Self::Other(s) => s.as_str(),
    }
  }
  /// The open escape for a codec name FFmpeg n9.0 does not carry.
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
}
impl AudioCodec {
  /// Every audio codec this vocabulary names, in declaration order.
  ///
  /// A slice rather than an array: how many codecs this build carries
  /// is a fact about the vendored FFmpeg table, not part of the type,
  /// so a regeneration that adds one stays a minor change.
  ///
  /// [`Self::Other`] is not a member. The roster answers "which names
  /// does this build know", and the escape is precisely the arm that
  /// carries a name it does not.
  pub const ROSTER: &'static [Self] = &[
    Self::N4gv,
    Self::N8svxExp,
    Self::N8svxFib,
    Self::Aac,
    Self::AacLatm,
    Self::Ac3,
    Self::Ac4,
    Self::AcelpKelvin,
    Self::Adpcm4xm,
    Self::AdpcmAdx,
    Self::AdpcmAfc,
    Self::AdpcmAgm,
    Self::AdpcmAica,
    Self::AdpcmArgo,
    Self::AdpcmCircus,
    Self::AdpcmCt,
    Self::AdpcmDtk,
    Self::AdpcmEa,
    Self::AdpcmEaMaxisXa,
    Self::AdpcmEaR1,
    Self::AdpcmEaR2,
    Self::AdpcmEaR3,
    Self::AdpcmEaXas,
    Self::AdpcmG722,
    Self::AdpcmG726,
    Self::AdpcmG726le,
    Self::AdpcmImaAcorn,
    Self::AdpcmImaAlp,
    Self::AdpcmImaAmv,
    Self::AdpcmImaApc,
    Self::AdpcmImaApm,
    Self::AdpcmImaCunning,
    Self::AdpcmImaDat4,
    Self::AdpcmImaDk3,
    Self::AdpcmImaDk4,
    Self::AdpcmImaEaEacs,
    Self::AdpcmImaEaSead,
    Self::AdpcmImaEscape,
    Self::AdpcmImaHvqm2,
    Self::AdpcmImaHvqm4,
    Self::AdpcmImaIss,
    Self::AdpcmImaMagix,
    Self::AdpcmImaMoflex,
    Self::AdpcmImaMtf,
    Self::AdpcmImaOki,
    Self::AdpcmImaPda,
    Self::AdpcmImaQt,
    Self::AdpcmImaRad,
    Self::AdpcmImaSmjpeg,
    Self::AdpcmImaSsi,
    Self::AdpcmImaWav,
    Self::AdpcmImaWs,
    Self::AdpcmImaXbox,
    Self::AdpcmMs,
    Self::AdpcmMtaf,
    Self::AdpcmN64,
    Self::AdpcmPsx,
    Self::AdpcmPsxc,
    Self::AdpcmSanyo,
    Self::AdpcmSbpro2,
    Self::AdpcmSbpro3,
    Self::AdpcmSbpro4,
    Self::AdpcmSwf,
    Self::AdpcmThp,
    Self::AdpcmThpLe,
    Self::AdpcmVima,
    Self::AdpcmXa,
    Self::AdpcmXmd,
    Self::AdpcmYamaha,
    Self::AdpcmZork,
    Self::Ahx,
    Self::Alac,
    Self::AmrNb,
    Self::AmrWb,
    Self::Anull,
    Self::Apac,
    Self::Ape,
    Self::AppleApac,
    Self::Aptx,
    Self::AptxHd,
    Self::Atrac1,
    Self::Atrac3,
    Self::Atrac3al,
    Self::Atrac3p,
    Self::Atrac3pal,
    Self::Atrac9,
    Self::Avc,
    Self::BinkaudioDct,
    Self::BinkaudioRdft,
    Self::BmvAudio,
    Self::Bonk,
    Self::Cbd2Dpcm,
    Self::Celt,
    Self::Codec2,
    Self::Comfortnoise,
    Self::Cook,
    Self::DerfDpcm,
    Self::Dfpwm,
    Self::DolbyE,
    Self::DsdLsbf,
    Self::DsdLsbfPlanar,
    Self::DsdMsbf,
    Self::DsdMsbfPlanar,
    Self::Dsicinaudio,
    Self::DssSp,
    Self::Dst,
    Self::Dts,
    Self::Dvaudio,
    Self::Eac3,
    Self::Evrc,
    Self::Fastaudio,
    Self::Flac,
    Self::Ftr,
    Self::G7231,
    Self::G728,
    Self::G729,
    Self::GremlinDpcm,
    Self::Gsm,
    Self::GsmMs,
    Self::Hca,
    Self::Hcom,
    Self::Iac,
    Self::Ilbc,
    Self::Imc,
    Self::InterplayDpcm,
    Self::Interplayacm,
    Self::Lc3,
    Self::Mace3,
    Self::Mace6,
    Self::Metasound,
    Self::Misc4,
    Self::Mlp,
    Self::Mp1,
    Self::Mp2,
    Self::Mp3,
    Self::Mp3adu,
    Self::Mp3on4,
    Self::Mp4als,
    Self::Mpegh3dAudio,
    Self::Msnsiren,
    Self::Musepack7,
    Self::Musepack8,
    Self::Nellymoser,
    Self::Opus,
    Self::Osq,
    Self::PafAudio,
    Self::PcmAlaw,
    Self::PcmBluray,
    Self::PcmDvd,
    Self::PcmF16le,
    Self::PcmF24le,
    Self::PcmF32be,
    Self::PcmF32le,
    Self::PcmF64be,
    Self::PcmF64le,
    Self::PcmLxf,
    Self::PcmMulaw,
    Self::PcmS16be,
    Self::PcmS16bePlanar,
    Self::PcmS16le,
    Self::PcmS16lePlanar,
    Self::PcmS24be,
    Self::PcmS24daud,
    Self::PcmS24le,
    Self::PcmS24lePlanar,
    Self::PcmS32be,
    Self::PcmS32le,
    Self::PcmS32lePlanar,
    Self::PcmS64be,
    Self::PcmS64le,
    Self::PcmS8,
    Self::PcmS8Planar,
    Self::PcmSga,
    Self::PcmU16be,
    Self::PcmU16le,
    Self::PcmU24be,
    Self::PcmU24le,
    Self::PcmU32be,
    Self::PcmU32le,
    Self::PcmU8,
    Self::PcmVidc,
    Self::Qcelp,
    Self::Qdm2,
    Self::Qdmc,
    Self::Qoa,
    Self::Ra144,
    Self::Ra288,
    Self::Ralf,
    Self::Rka,
    Self::RoqDpcm,
    Self::S302m,
    Self::Sbc,
    Self::Sdx2Dpcm,
    Self::Shorten,
    Self::Sipr,
    Self::Siren,
    Self::Smackaudio,
    Self::Smv,
    Self::SolDpcm,
    Self::Sonic,
    Self::Sonicls,
    Self::Speex,
    Self::Tak,
    Self::Truehd,
    Self::Truespeech,
    Self::Tta,
    Self::Twinvq,
    Self::Vmdaudio,
    Self::Vorbis,
    Self::WadyDpcm,
    Self::Wavarc,
    Self::Wavesynth,
    Self::Wavpack,
    Self::WestwoodSnd1,
    Self::Wmalossless,
    Self::Wmapro,
    Self::Wmav1,
    Self::Wmav2,
    Self::Wmavoice,
    Self::XanDpcm,
    Self::Xma1,
    Self::Xma2,
  ];
}
const _: () = {
  #[allow(dead_code)]
  fn every_variant_is_rostered(v: &AudioCodec) {
    match v {
      AudioCodec::N4gv
      | AudioCodec::N8svxExp
      | AudioCodec::N8svxFib
      | AudioCodec::Aac
      | AudioCodec::AacLatm
      | AudioCodec::Ac3
      | AudioCodec::Ac4
      | AudioCodec::AcelpKelvin
      | AudioCodec::Adpcm4xm
      | AudioCodec::AdpcmAdx
      | AudioCodec::AdpcmAfc
      | AudioCodec::AdpcmAgm
      | AudioCodec::AdpcmAica
      | AudioCodec::AdpcmArgo
      | AudioCodec::AdpcmCircus
      | AudioCodec::AdpcmCt
      | AudioCodec::AdpcmDtk
      | AudioCodec::AdpcmEa
      | AudioCodec::AdpcmEaMaxisXa
      | AudioCodec::AdpcmEaR1
      | AudioCodec::AdpcmEaR2
      | AudioCodec::AdpcmEaR3
      | AudioCodec::AdpcmEaXas
      | AudioCodec::AdpcmG722
      | AudioCodec::AdpcmG726
      | AudioCodec::AdpcmG726le
      | AudioCodec::AdpcmImaAcorn
      | AudioCodec::AdpcmImaAlp
      | AudioCodec::AdpcmImaAmv
      | AudioCodec::AdpcmImaApc
      | AudioCodec::AdpcmImaApm
      | AudioCodec::AdpcmImaCunning
      | AudioCodec::AdpcmImaDat4
      | AudioCodec::AdpcmImaDk3
      | AudioCodec::AdpcmImaDk4
      | AudioCodec::AdpcmImaEaEacs
      | AudioCodec::AdpcmImaEaSead
      | AudioCodec::AdpcmImaEscape
      | AudioCodec::AdpcmImaHvqm2
      | AudioCodec::AdpcmImaHvqm4
      | AudioCodec::AdpcmImaIss
      | AudioCodec::AdpcmImaMagix
      | AudioCodec::AdpcmImaMoflex
      | AudioCodec::AdpcmImaMtf
      | AudioCodec::AdpcmImaOki
      | AudioCodec::AdpcmImaPda
      | AudioCodec::AdpcmImaQt
      | AudioCodec::AdpcmImaRad
      | AudioCodec::AdpcmImaSmjpeg
      | AudioCodec::AdpcmImaSsi
      | AudioCodec::AdpcmImaWav
      | AudioCodec::AdpcmImaWs
      | AudioCodec::AdpcmImaXbox
      | AudioCodec::AdpcmMs
      | AudioCodec::AdpcmMtaf
      | AudioCodec::AdpcmN64
      | AudioCodec::AdpcmPsx
      | AudioCodec::AdpcmPsxc
      | AudioCodec::AdpcmSanyo
      | AudioCodec::AdpcmSbpro2
      | AudioCodec::AdpcmSbpro3
      | AudioCodec::AdpcmSbpro4
      | AudioCodec::AdpcmSwf
      | AudioCodec::AdpcmThp
      | AudioCodec::AdpcmThpLe
      | AudioCodec::AdpcmVima
      | AudioCodec::AdpcmXa
      | AudioCodec::AdpcmXmd
      | AudioCodec::AdpcmYamaha
      | AudioCodec::AdpcmZork
      | AudioCodec::Ahx
      | AudioCodec::Alac
      | AudioCodec::AmrNb
      | AudioCodec::AmrWb
      | AudioCodec::Anull
      | AudioCodec::Apac
      | AudioCodec::Ape
      | AudioCodec::AppleApac
      | AudioCodec::Aptx
      | AudioCodec::AptxHd
      | AudioCodec::Atrac1
      | AudioCodec::Atrac3
      | AudioCodec::Atrac3al
      | AudioCodec::Atrac3p
      | AudioCodec::Atrac3pal
      | AudioCodec::Atrac9
      | AudioCodec::Avc
      | AudioCodec::BinkaudioDct
      | AudioCodec::BinkaudioRdft
      | AudioCodec::BmvAudio
      | AudioCodec::Bonk
      | AudioCodec::Cbd2Dpcm
      | AudioCodec::Celt
      | AudioCodec::Codec2
      | AudioCodec::Comfortnoise
      | AudioCodec::Cook
      | AudioCodec::DerfDpcm
      | AudioCodec::Dfpwm
      | AudioCodec::DolbyE
      | AudioCodec::DsdLsbf
      | AudioCodec::DsdLsbfPlanar
      | AudioCodec::DsdMsbf
      | AudioCodec::DsdMsbfPlanar
      | AudioCodec::Dsicinaudio
      | AudioCodec::DssSp
      | AudioCodec::Dst
      | AudioCodec::Dts
      | AudioCodec::Dvaudio
      | AudioCodec::Eac3
      | AudioCodec::Evrc
      | AudioCodec::Fastaudio
      | AudioCodec::Flac
      | AudioCodec::Ftr
      | AudioCodec::G7231
      | AudioCodec::G728
      | AudioCodec::G729
      | AudioCodec::GremlinDpcm
      | AudioCodec::Gsm
      | AudioCodec::GsmMs
      | AudioCodec::Hca
      | AudioCodec::Hcom
      | AudioCodec::Iac
      | AudioCodec::Ilbc
      | AudioCodec::Imc
      | AudioCodec::InterplayDpcm
      | AudioCodec::Interplayacm
      | AudioCodec::Lc3
      | AudioCodec::Mace3
      | AudioCodec::Mace6
      | AudioCodec::Metasound
      | AudioCodec::Misc4
      | AudioCodec::Mlp
      | AudioCodec::Mp1
      | AudioCodec::Mp2
      | AudioCodec::Mp3
      | AudioCodec::Mp3adu
      | AudioCodec::Mp3on4
      | AudioCodec::Mp4als
      | AudioCodec::Mpegh3dAudio
      | AudioCodec::Msnsiren
      | AudioCodec::Musepack7
      | AudioCodec::Musepack8
      | AudioCodec::Nellymoser
      | AudioCodec::Opus
      | AudioCodec::Osq
      | AudioCodec::PafAudio
      | AudioCodec::PcmAlaw
      | AudioCodec::PcmBluray
      | AudioCodec::PcmDvd
      | AudioCodec::PcmF16le
      | AudioCodec::PcmF24le
      | AudioCodec::PcmF32be
      | AudioCodec::PcmF32le
      | AudioCodec::PcmF64be
      | AudioCodec::PcmF64le
      | AudioCodec::PcmLxf
      | AudioCodec::PcmMulaw
      | AudioCodec::PcmS16be
      | AudioCodec::PcmS16bePlanar
      | AudioCodec::PcmS16le
      | AudioCodec::PcmS16lePlanar
      | AudioCodec::PcmS24be
      | AudioCodec::PcmS24daud
      | AudioCodec::PcmS24le
      | AudioCodec::PcmS24lePlanar
      | AudioCodec::PcmS32be
      | AudioCodec::PcmS32le
      | AudioCodec::PcmS32lePlanar
      | AudioCodec::PcmS64be
      | AudioCodec::PcmS64le
      | AudioCodec::PcmS8
      | AudioCodec::PcmS8Planar
      | AudioCodec::PcmSga
      | AudioCodec::PcmU16be
      | AudioCodec::PcmU16le
      | AudioCodec::PcmU24be
      | AudioCodec::PcmU24le
      | AudioCodec::PcmU32be
      | AudioCodec::PcmU32le
      | AudioCodec::PcmU8
      | AudioCodec::PcmVidc
      | AudioCodec::Qcelp
      | AudioCodec::Qdm2
      | AudioCodec::Qdmc
      | AudioCodec::Qoa
      | AudioCodec::Ra144
      | AudioCodec::Ra288
      | AudioCodec::Ralf
      | AudioCodec::Rka
      | AudioCodec::RoqDpcm
      | AudioCodec::S302m
      | AudioCodec::Sbc
      | AudioCodec::Sdx2Dpcm
      | AudioCodec::Shorten
      | AudioCodec::Sipr
      | AudioCodec::Siren
      | AudioCodec::Smackaudio
      | AudioCodec::Smv
      | AudioCodec::SolDpcm
      | AudioCodec::Sonic
      | AudioCodec::Sonicls
      | AudioCodec::Speex
      | AudioCodec::Tak
      | AudioCodec::Truehd
      | AudioCodec::Truespeech
      | AudioCodec::Tta
      | AudioCodec::Twinvq
      | AudioCodec::Vmdaudio
      | AudioCodec::Vorbis
      | AudioCodec::WadyDpcm
      | AudioCodec::Wavarc
      | AudioCodec::Wavesynth
      | AudioCodec::Wavpack
      | AudioCodec::WestwoodSnd1
      | AudioCodec::Wmalossless
      | AudioCodec::Wmapro
      | AudioCodec::Wmav1
      | AudioCodec::Wmav2
      | AudioCodec::Wmavoice
      | AudioCodec::XanDpcm
      | AudioCodec::Xma1
      | AudioCodec::Xma2 => {}
      AudioCodec::Other(_) => {}
    }
  }
};
impl FromStr for AudioCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name, case-insensitively; unknown
  /// values land in [`Self::Other`] (infallible, lossless), carrying
  /// the caller's spelling verbatim.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"4gv" => Self::N4gv,
      b"8svx_exp" => Self::N8svxExp,
      b"8svx_fib" => Self::N8svxFib,
      b"aac" => Self::Aac,
      b"aac_latm" => Self::AacLatm,
      b"ac3" => Self::Ac3,
      b"ac4" => Self::Ac4,
      b"acelp.kelvin" => Self::AcelpKelvin,
      b"adpcm_4xm" => Self::Adpcm4xm,
      b"adpcm_adx" => Self::AdpcmAdx,
      b"adpcm_afc" => Self::AdpcmAfc,
      b"adpcm_agm" => Self::AdpcmAgm,
      b"adpcm_aica" => Self::AdpcmAica,
      b"adpcm_argo" => Self::AdpcmArgo,
      b"adpcm_circus" => Self::AdpcmCircus,
      b"adpcm_ct" => Self::AdpcmCt,
      b"adpcm_dtk" => Self::AdpcmDtk,
      b"adpcm_ea" => Self::AdpcmEa,
      b"adpcm_ea_maxis_xa" => Self::AdpcmEaMaxisXa,
      b"adpcm_ea_r1" => Self::AdpcmEaR1,
      b"adpcm_ea_r2" => Self::AdpcmEaR2,
      b"adpcm_ea_r3" => Self::AdpcmEaR3,
      b"adpcm_ea_xas" => Self::AdpcmEaXas,
      b"adpcm_g722" => Self::AdpcmG722,
      b"adpcm_g726" => Self::AdpcmG726,
      b"adpcm_g726le" => Self::AdpcmG726le,
      b"adpcm_ima_acorn" => Self::AdpcmImaAcorn,
      b"adpcm_ima_alp" => Self::AdpcmImaAlp,
      b"adpcm_ima_amv" => Self::AdpcmImaAmv,
      b"adpcm_ima_apc" => Self::AdpcmImaApc,
      b"adpcm_ima_apm" => Self::AdpcmImaApm,
      b"adpcm_ima_cunning" => Self::AdpcmImaCunning,
      b"adpcm_ima_dat4" => Self::AdpcmImaDat4,
      b"adpcm_ima_dk3" => Self::AdpcmImaDk3,
      b"adpcm_ima_dk4" => Self::AdpcmImaDk4,
      b"adpcm_ima_ea_eacs" => Self::AdpcmImaEaEacs,
      b"adpcm_ima_ea_sead" => Self::AdpcmImaEaSead,
      b"adpcm_ima_escape" => Self::AdpcmImaEscape,
      b"adpcm_ima_hvqm2" => Self::AdpcmImaHvqm2,
      b"adpcm_ima_hvqm4" => Self::AdpcmImaHvqm4,
      b"adpcm_ima_iss" => Self::AdpcmImaIss,
      b"adpcm_ima_magix" => Self::AdpcmImaMagix,
      b"adpcm_ima_moflex" => Self::AdpcmImaMoflex,
      b"adpcm_ima_mtf" => Self::AdpcmImaMtf,
      b"adpcm_ima_oki" => Self::AdpcmImaOki,
      b"adpcm_ima_pda" => Self::AdpcmImaPda,
      b"adpcm_ima_qt" => Self::AdpcmImaQt,
      b"adpcm_ima_rad" => Self::AdpcmImaRad,
      b"adpcm_ima_smjpeg" => Self::AdpcmImaSmjpeg,
      b"adpcm_ima_ssi" => Self::AdpcmImaSsi,
      b"adpcm_ima_wav" => Self::AdpcmImaWav,
      b"adpcm_ima_ws" => Self::AdpcmImaWs,
      b"adpcm_ima_xbox" => Self::AdpcmImaXbox,
      b"adpcm_ms" => Self::AdpcmMs,
      b"adpcm_mtaf" => Self::AdpcmMtaf,
      b"adpcm_n64" => Self::AdpcmN64,
      b"adpcm_psx" => Self::AdpcmPsx,
      b"adpcm_psxc" => Self::AdpcmPsxc,
      b"adpcm_sanyo" => Self::AdpcmSanyo,
      b"adpcm_sbpro_2" => Self::AdpcmSbpro2,
      b"adpcm_sbpro_3" => Self::AdpcmSbpro3,
      b"adpcm_sbpro_4" => Self::AdpcmSbpro4,
      b"adpcm_swf" => Self::AdpcmSwf,
      b"adpcm_thp" => Self::AdpcmThp,
      b"adpcm_thp_le" => Self::AdpcmThpLe,
      b"adpcm_vima" => Self::AdpcmVima,
      b"adpcm_xa" => Self::AdpcmXa,
      b"adpcm_xmd" => Self::AdpcmXmd,
      b"adpcm_yamaha" => Self::AdpcmYamaha,
      b"adpcm_zork" => Self::AdpcmZork,
      b"ahx" => Self::Ahx,
      b"alac" => Self::Alac,
      b"amr_nb" => Self::AmrNb,
      b"amr_wb" => Self::AmrWb,
      b"anull" => Self::Anull,
      b"apac" => Self::Apac,
      b"ape" => Self::Ape,
      b"apple_apac" => Self::AppleApac,
      b"aptx" => Self::Aptx,
      b"aptx_hd" => Self::AptxHd,
      b"atrac1" => Self::Atrac1,
      b"atrac3" => Self::Atrac3,
      b"atrac3al" => Self::Atrac3al,
      b"atrac3p" => Self::Atrac3p,
      b"atrac3pal" => Self::Atrac3pal,
      b"atrac9" => Self::Atrac9,
      b"avc" => Self::Avc,
      b"binkaudio_dct" => Self::BinkaudioDct,
      b"binkaudio_rdft" => Self::BinkaudioRdft,
      b"bmv_audio" => Self::BmvAudio,
      b"bonk" => Self::Bonk,
      b"cbd2_dpcm" => Self::Cbd2Dpcm,
      b"celt" => Self::Celt,
      b"codec2" => Self::Codec2,
      b"comfortnoise" => Self::Comfortnoise,
      b"cook" => Self::Cook,
      b"derf_dpcm" => Self::DerfDpcm,
      b"dfpwm" => Self::Dfpwm,
      b"dolby_e" => Self::DolbyE,
      b"dsd_lsbf" => Self::DsdLsbf,
      b"dsd_lsbf_planar" => Self::DsdLsbfPlanar,
      b"dsd_msbf" => Self::DsdMsbf,
      b"dsd_msbf_planar" => Self::DsdMsbfPlanar,
      b"dsicinaudio" => Self::Dsicinaudio,
      b"dss_sp" => Self::DssSp,
      b"dst" => Self::Dst,
      b"dts" => Self::Dts,
      b"dvaudio" => Self::Dvaudio,
      b"eac3" => Self::Eac3,
      b"evrc" => Self::Evrc,
      b"fastaudio" => Self::Fastaudio,
      b"flac" => Self::Flac,
      b"ftr" => Self::Ftr,
      b"g723_1" => Self::G7231,
      b"g728" => Self::G728,
      b"g729" => Self::G729,
      b"gremlin_dpcm" => Self::GremlinDpcm,
      b"gsm" => Self::Gsm,
      b"gsm_ms" => Self::GsmMs,
      b"hca" => Self::Hca,
      b"hcom" => Self::Hcom,
      b"iac" => Self::Iac,
      b"ilbc" => Self::Ilbc,
      b"imc" => Self::Imc,
      b"interplay_dpcm" => Self::InterplayDpcm,
      b"interplayacm" => Self::Interplayacm,
      b"lc3" => Self::Lc3,
      b"mace3" => Self::Mace3,
      b"mace6" => Self::Mace6,
      b"metasound" => Self::Metasound,
      b"misc4" => Self::Misc4,
      b"mlp" => Self::Mlp,
      b"mp1" => Self::Mp1,
      b"mp2" => Self::Mp2,
      b"mp3" => Self::Mp3,
      b"mp3adu" => Self::Mp3adu,
      b"mp3on4" => Self::Mp3on4,
      b"mp4als" => Self::Mp4als,
      b"mpegh_3d_audio" => Self::Mpegh3dAudio,
      b"msnsiren" => Self::Msnsiren,
      b"musepack7" => Self::Musepack7,
      b"musepack8" => Self::Musepack8,
      b"nellymoser" => Self::Nellymoser,
      b"opus" => Self::Opus,
      b"osq" => Self::Osq,
      b"paf_audio" => Self::PafAudio,
      b"pcm_alaw" => Self::PcmAlaw,
      b"pcm_bluray" => Self::PcmBluray,
      b"pcm_dvd" => Self::PcmDvd,
      b"pcm_f16le" => Self::PcmF16le,
      b"pcm_f24le" => Self::PcmF24le,
      b"pcm_f32be" => Self::PcmF32be,
      b"pcm_f32le" => Self::PcmF32le,
      b"pcm_f64be" => Self::PcmF64be,
      b"pcm_f64le" => Self::PcmF64le,
      b"pcm_lxf" => Self::PcmLxf,
      b"pcm_mulaw" => Self::PcmMulaw,
      b"pcm_s16be" => Self::PcmS16be,
      b"pcm_s16be_planar" => Self::PcmS16bePlanar,
      b"pcm_s16le" => Self::PcmS16le,
      b"pcm_s16le_planar" => Self::PcmS16lePlanar,
      b"pcm_s24be" => Self::PcmS24be,
      b"pcm_s24daud" => Self::PcmS24daud,
      b"pcm_s24le" => Self::PcmS24le,
      b"pcm_s24le_planar" => Self::PcmS24lePlanar,
      b"pcm_s32be" => Self::PcmS32be,
      b"pcm_s32le" => Self::PcmS32le,
      b"pcm_s32le_planar" => Self::PcmS32lePlanar,
      b"pcm_s64be" => Self::PcmS64be,
      b"pcm_s64le" => Self::PcmS64le,
      b"pcm_s8" => Self::PcmS8,
      b"pcm_s8_planar" => Self::PcmS8Planar,
      b"pcm_sga" => Self::PcmSga,
      b"pcm_u16be" => Self::PcmU16be,
      b"pcm_u16le" => Self::PcmU16le,
      b"pcm_u24be" => Self::PcmU24be,
      b"pcm_u24le" => Self::PcmU24le,
      b"pcm_u32be" => Self::PcmU32be,
      b"pcm_u32le" => Self::PcmU32le,
      b"pcm_u8" => Self::PcmU8,
      b"pcm_vidc" => Self::PcmVidc,
      b"qcelp" => Self::Qcelp,
      b"qdm2" => Self::Qdm2,
      b"qdmc" => Self::Qdmc,
      b"qoa" => Self::Qoa,
      b"ra_144" => Self::Ra144,
      b"ra_288" => Self::Ra288,
      b"ralf" => Self::Ralf,
      b"rka" => Self::Rka,
      b"roq_dpcm" => Self::RoqDpcm,
      b"s302m" => Self::S302m,
      b"sbc" => Self::Sbc,
      b"sdx2_dpcm" => Self::Sdx2Dpcm,
      b"shorten" => Self::Shorten,
      b"sipr" => Self::Sipr,
      b"siren" => Self::Siren,
      b"smackaudio" => Self::Smackaudio,
      b"smv" => Self::Smv,
      b"sol_dpcm" => Self::SolDpcm,
      b"sonic" => Self::Sonic,
      b"sonicls" => Self::Sonicls,
      b"speex" => Self::Speex,
      b"tak" => Self::Tak,
      b"truehd" => Self::Truehd,
      b"truespeech" => Self::Truespeech,
      b"tta" => Self::Tta,
      b"twinvq" => Self::Twinvq,
      b"vmdaudio" => Self::Vmdaudio,
      b"vorbis" => Self::Vorbis,
      b"wady_dpcm" => Self::WadyDpcm,
      b"wavarc" => Self::Wavarc,
      b"wavesynth" => Self::Wavesynth,
      b"wavpack" => Self::Wavpack,
      b"westwood_snd1" => Self::WestwoodSnd1,
      b"wmalossless" => Self::Wmalossless,
      b"wmapro" => Self::Wmapro,
      b"wmav1" => Self::Wmav1,
      b"wmav2" => Self::Wmav2,
      b"wmavoice" => Self::Wmavoice,
      b"xan_dpcm" => Self::XanDpcm,
      b"xma1" => Self::Xma1,
      b"xma2" => Self::Xma2,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}
/** Subtitle codec family — every codec FFmpeg n9.0 knows under media type `subtitle`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::subtitle_codec")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum SubtitleCodec {
  /// FFmpeg `"arib_caption"`.
  AribCaption,
  /// FFmpeg `"ass"`.
  Ass,
  /// FFmpeg `"dvb_subtitle"`.
  DvbSubtitle,
  /// FFmpeg `"dvb_teletext"`.
  DvbTeletext,
  /// FFmpeg `"dvd_subtitle"`.
  DvdSubtitle,
  /// FFmpeg `"eia_608"`.
  Eia608,
  /// FFmpeg `"hdmv_pgs_subtitle"`.
  HdmvPgsSubtitle,
  /// FFmpeg `"hdmv_text_subtitle"`.
  HdmvTextSubtitle,
  /// FFmpeg `"ivtv_vbi"`.
  IvtvVbi,
  /// FFmpeg `"jacosub"`.
  Jacosub,
  /// FFmpeg `"microdvd"`.
  Microdvd,
  /// FFmpeg `"mov_text"`.
  MovText,
  /// FFmpeg `"mpl2"`.
  Mpl2,
  /// FFmpeg `"pjs"`.
  Pjs,
  /// FFmpeg `"realtext"`.
  Realtext,
  /// FFmpeg `"sami"`.
  Sami,
  /// FFmpeg `"srt"`.
  Srt,
  /// FFmpeg `"ssa"`.
  Ssa,
  /// FFmpeg `"stl"`.
  Stl,
  /// FFmpeg `"subrip"`.
  Subrip,
  /// FFmpeg `"subviewer"`.
  Subviewer,
  /// FFmpeg `"subviewer1"`.
  Subviewer1,
  /// FFmpeg `"text"`.
  Text,
  /// FFmpeg `"ttml"`.
  Ttml,
  /// FFmpeg `"vplayer"`.
  Vplayer,
  /// FFmpeg `"webvtt"`.
  Webvtt,
  /// FFmpeg `"xsub"`.
  Xsub,
  /// A codec not enumerated above — carries the FFmpeg short name
  /// verbatim.
  Other(Utf8Bytes),
}
impl SubtitleCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::AribCaption => "arib_caption",
      Self::Ass => "ass",
      Self::DvbSubtitle => "dvb_subtitle",
      Self::DvbTeletext => "dvb_teletext",
      Self::DvdSubtitle => "dvd_subtitle",
      Self::Eia608 => "eia_608",
      Self::HdmvPgsSubtitle => "hdmv_pgs_subtitle",
      Self::HdmvTextSubtitle => "hdmv_text_subtitle",
      Self::IvtvVbi => "ivtv_vbi",
      Self::Jacosub => "jacosub",
      Self::Microdvd => "microdvd",
      Self::MovText => "mov_text",
      Self::Mpl2 => "mpl2",
      Self::Pjs => "pjs",
      Self::Realtext => "realtext",
      Self::Sami => "sami",
      Self::Srt => "srt",
      Self::Ssa => "ssa",
      Self::Stl => "stl",
      Self::Subrip => "subrip",
      Self::Subviewer => "subviewer",
      Self::Subviewer1 => "subviewer1",
      Self::Text => "text",
      Self::Ttml => "ttml",
      Self::Vplayer => "vplayer",
      Self::Webvtt => "webvtt",
      Self::Xsub => "xsub",
      Self::Other(s) => s.as_str(),
    }
  }
  /// The open escape for a codec name FFmpeg n9.0 does not carry.
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
  /// Is this a **bitmap** (image-based) subtitle codec, requiring an
  /// OCR pipeline stage to extract searchable text?
  ///
  /// - `Some(true)`: matches FFmpeg's `AV_CODEC_PROP_BITMAP_SUB` flag.
  /// - `Some(false)`: a known FFmpeg subtitle codec without
  ///   `AV_CODEC_PROP_BITMAP_SUB` (text codecs and teletext/VBI-style
  ///   codecs that carry no `.props` at all in FFmpeg n9.0).
  /// - `None`: [`Self::Other`] — the codec name is not in the vendored
  ///   FFmpeg table, so we cannot consult `.props`.
  ///
  /// (4 bitmap / 23 non-bitmap variant(s) per FFmpeg n9.0).
  pub fn is_image_based(&self) -> Option<bool> {
    match self {
      Self::DvbSubtitle | Self::DvdSubtitle | Self::HdmvPgsSubtitle | Self::Xsub => Some(true),
      Self::AribCaption
      | Self::Ass
      | Self::DvbTeletext
      | Self::Eia608
      | Self::HdmvTextSubtitle
      | Self::IvtvVbi
      | Self::Jacosub
      | Self::Microdvd
      | Self::MovText
      | Self::Mpl2
      | Self::Pjs
      | Self::Realtext
      | Self::Sami
      | Self::Srt
      | Self::Ssa
      | Self::Stl
      | Self::Subrip
      | Self::Subviewer
      | Self::Subviewer1
      | Self::Text
      | Self::Ttml
      | Self::Vplayer
      | Self::Webvtt => Some(false),
      Self::Other(_) => None,
    }
  }
}
impl SubtitleCodec {
  /// Every subtitle codec this vocabulary names, in declaration order.
  ///
  /// A slice rather than an array: how many codecs this build carries
  /// is a fact about the vendored FFmpeg table, not part of the type,
  /// so a regeneration that adds one stays a minor change.
  ///
  /// [`Self::Other`] is not a member. The roster answers "which names
  /// does this build know", and the escape is precisely the arm that
  /// carries a name it does not.
  pub const ROSTER: &'static [Self] = &[
    Self::AribCaption,
    Self::Ass,
    Self::DvbSubtitle,
    Self::DvbTeletext,
    Self::DvdSubtitle,
    Self::Eia608,
    Self::HdmvPgsSubtitle,
    Self::HdmvTextSubtitle,
    Self::IvtvVbi,
    Self::Jacosub,
    Self::Microdvd,
    Self::MovText,
    Self::Mpl2,
    Self::Pjs,
    Self::Realtext,
    Self::Sami,
    Self::Srt,
    Self::Ssa,
    Self::Stl,
    Self::Subrip,
    Self::Subviewer,
    Self::Subviewer1,
    Self::Text,
    Self::Ttml,
    Self::Vplayer,
    Self::Webvtt,
    Self::Xsub,
  ];
}
const _: () = {
  #[allow(dead_code)]
  fn every_variant_is_rostered(v: &SubtitleCodec) {
    match v {
      SubtitleCodec::AribCaption
      | SubtitleCodec::Ass
      | SubtitleCodec::DvbSubtitle
      | SubtitleCodec::DvbTeletext
      | SubtitleCodec::DvdSubtitle
      | SubtitleCodec::Eia608
      | SubtitleCodec::HdmvPgsSubtitle
      | SubtitleCodec::HdmvTextSubtitle
      | SubtitleCodec::IvtvVbi
      | SubtitleCodec::Jacosub
      | SubtitleCodec::Microdvd
      | SubtitleCodec::MovText
      | SubtitleCodec::Mpl2
      | SubtitleCodec::Pjs
      | SubtitleCodec::Realtext
      | SubtitleCodec::Sami
      | SubtitleCodec::Srt
      | SubtitleCodec::Ssa
      | SubtitleCodec::Stl
      | SubtitleCodec::Subrip
      | SubtitleCodec::Subviewer
      | SubtitleCodec::Subviewer1
      | SubtitleCodec::Text
      | SubtitleCodec::Ttml
      | SubtitleCodec::Vplayer
      | SubtitleCodec::Webvtt
      | SubtitleCodec::Xsub => {}
      SubtitleCodec::Other(_) => {}
    }
  }
};
impl FromStr for SubtitleCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name, case-insensitively; unknown
  /// values land in [`Self::Other`] (infallible, lossless), carrying
  /// the caller's spelling verbatim.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"arib_caption" => Self::AribCaption,
      b"ass" => Self::Ass,
      b"dvb_subtitle" => Self::DvbSubtitle,
      b"dvb_teletext" => Self::DvbTeletext,
      b"dvd_subtitle" => Self::DvdSubtitle,
      b"eia_608" => Self::Eia608,
      b"hdmv_pgs_subtitle" => Self::HdmvPgsSubtitle,
      b"hdmv_text_subtitle" => Self::HdmvTextSubtitle,
      b"ivtv_vbi" => Self::IvtvVbi,
      b"jacosub" => Self::Jacosub,
      b"microdvd" => Self::Microdvd,
      b"mov_text" => Self::MovText,
      b"mpl2" => Self::Mpl2,
      b"pjs" => Self::Pjs,
      b"realtext" => Self::Realtext,
      b"sami" => Self::Sami,
      b"srt" => Self::Srt,
      b"ssa" => Self::Ssa,
      b"stl" => Self::Stl,
      b"subrip" => Self::Subrip,
      b"subviewer" => Self::Subviewer,
      b"subviewer1" => Self::Subviewer1,
      b"text" => Self::Text,
      b"ttml" => Self::Ttml,
      b"vplayer" => Self::Vplayer,
      b"webvtt" => Self::Webvtt,
      b"xsub" => Self::Xsub,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}
/** Data codec family — every codec FFmpeg n9.0 knows under media type `data`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::data_codec")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum DataCodec {
  /// FFmpeg `"bin_data"`.
  BinData,
  /// FFmpeg `"dvd_nav_packet"`.
  DvdNavPacket,
  /// FFmpeg `"epg"`.
  Epg,
  /// FFmpeg `"klv"`.
  Klv,
  /// FFmpeg `"mpegts"`.
  Mpegts,
  /// FFmpeg `"otf"`.
  Otf,
  /// FFmpeg `"scte_35"`.
  Scte35,
  /// FFmpeg `"smpte_2038"`.
  Smpte2038,
  /// FFmpeg `"smpte_436m_anc"`.
  Smpte436mAnc,
  /// FFmpeg `"timed_id3"`.
  TimedId3,
  /// FFmpeg `"ttf"`.
  Ttf,
  /// A codec not enumerated above — carries the FFmpeg short name
  /// verbatim.
  Other(Utf8Bytes),
}
impl DataCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::BinData => "bin_data",
      Self::DvdNavPacket => "dvd_nav_packet",
      Self::Epg => "epg",
      Self::Klv => "klv",
      Self::Mpegts => "mpegts",
      Self::Otf => "otf",
      Self::Scte35 => "scte_35",
      Self::Smpte2038 => "smpte_2038",
      Self::Smpte436mAnc => "smpte_436m_anc",
      Self::TimedId3 => "timed_id3",
      Self::Ttf => "ttf",
      Self::Other(s) => s.as_str(),
    }
  }
  /// The open escape for a codec name FFmpeg n9.0 does not carry.
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
}
impl DataCodec {
  /// Every data codec this vocabulary names, in declaration order.
  ///
  /// A slice rather than an array: how many codecs this build carries
  /// is a fact about the vendored FFmpeg table, not part of the type,
  /// so a regeneration that adds one stays a minor change.
  ///
  /// [`Self::Other`] is not a member. The roster answers "which names
  /// does this build know", and the escape is precisely the arm that
  /// carries a name it does not.
  pub const ROSTER: &'static [Self] = &[
    Self::BinData,
    Self::DvdNavPacket,
    Self::Epg,
    Self::Klv,
    Self::Mpegts,
    Self::Otf,
    Self::Scte35,
    Self::Smpte2038,
    Self::Smpte436mAnc,
    Self::TimedId3,
    Self::Ttf,
  ];
}
const _: () = {
  #[allow(dead_code)]
  fn every_variant_is_rostered(v: &DataCodec) {
    match v {
      DataCodec::BinData
      | DataCodec::DvdNavPacket
      | DataCodec::Epg
      | DataCodec::Klv
      | DataCodec::Mpegts
      | DataCodec::Otf
      | DataCodec::Scte35
      | DataCodec::Smpte2038
      | DataCodec::Smpte436mAnc
      | DataCodec::TimedId3
      | DataCodec::Ttf => {}
      DataCodec::Other(_) => {}
    }
  }
};
impl FromStr for DataCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name, case-insensitively; unknown
  /// values land in [`Self::Other`] (infallible, lossless), carrying
  /// the caller's spelling verbatim.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"bin_data" => Self::BinData,
      b"dvd_nav_packet" => Self::DvdNavPacket,
      b"epg" => Self::Epg,
      b"klv" => Self::Klv,
      b"mpegts" => Self::Mpegts,
      b"otf" => Self::Otf,
      b"scte_35" => Self::Scte35,
      b"smpte_2038" => Self::Smpte2038,
      b"smpte_436m_anc" => Self::Smpte436mAnc,
      b"timed_id3" => Self::TimedId3,
      b"ttf" => Self::Ttf,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}
/** Attachment codec family — the FFmpeg codec ids `libavformat/matroskadec.c`'s `mkv_mime_tags` table assigns to an `AVMEDIA_TYPE_ATTACHMENT` stream (`ATTACHMENT_CODECS`; see its doc comment for the full census — `libavcodec/codec_desc.c` has no `AVMEDIA_TYPE_ATTACHMENT` media type to enumerate here the way `DataCodec` and the other vendored enums are).

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(Utf8Bytes)` arm is the lossless escape for an attachment codec id this list does not (yet) name.*/
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::attachment_codec")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum AttachmentCodec {
  /// FFmpeg `"bin_data"`.
  BinData,
  /// FFmpeg `"otf"`.
  Otf,
  /// FFmpeg `"ttf"`.
  Ttf,
  /// A codec not enumerated above — carries the FFmpeg short name
  /// verbatim.
  Other(Utf8Bytes),
}
impl AttachmentCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::BinData => "bin_data",
      Self::Otf => "otf",
      Self::Ttf => "ttf",
      Self::Other(s) => s.as_str(),
    }
  }
  /// The open escape for a codec id not in `ATTACHMENT_CODECS`.
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
}
impl AttachmentCodec {
  /// Every attachment codec this vocabulary names, in declaration order.
  ///
  /// A slice rather than an array: how many codecs this build carries
  /// is a fact about the vendored FFmpeg table, not part of the type,
  /// so a regeneration that adds one stays a minor change.
  ///
  /// [`Self::Other`] is not a member. The roster answers "which names
  /// does this build know", and the escape is precisely the arm that
  /// carries a name it does not.
  pub const ROSTER: &'static [Self] = &[Self::BinData, Self::Otf, Self::Ttf];
}
const _: () = {
  #[allow(dead_code)]
  fn every_variant_is_rostered(v: &AttachmentCodec) {
    match v {
      AttachmentCodec::BinData | AttachmentCodec::Otf | AttachmentCodec::Ttf => {}
      AttachmentCodec::Other(_) => {}
    }
  }
};
impl FromStr for AttachmentCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name, case-insensitively; unknown
  /// values land in [`Self::Other`] (infallible, lossless), carrying
  /// the caller's spelling verbatim.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    let folded = crate::parse::lookup(crate::parse::Case::Insensitive, s, &mut buf);
    Ok(match folded {
      b"bin_data" => Self::BinData,
      b"otf" => Self::Otf,
      b"ttf" => Self::Ttf,
      _ => Self::Other(Utf8Bytes::from(s)),
    })
  }
}
#[cfg(test)]
mod tests;
