//! Stream-descriptor **codec** vocabulary for video, audio, and subtitle
//! tracks.
//!
//! **Generated** from `xtask/vendor/ffmpeg-codecs.txt` (FFmpeg n8.1
//! `libavcodec/codec_desc.c`) by `cargo xtask gen-codec`. Every codec
//! FFmpeg knows under media types `video` / `audio` / `subtitle` has
//! a named variant here; the `Other(SmolStr)` arm remains a lossless
//! escape for codecs added in a future FFmpeg release before this file
//! is regenerated.
//!
//! Regenerate in two steps:
//! 1. `cargo xtask sync`       — refreshes the vendored table.
//! 2. `cargo xtask gen-codec`  — regenerates this file from it.
//!
//! `cargo xtask check` verifies every named variant's canonical string
//! exists in the vendored table — CI gate against drift.
#![allow(non_camel_case_types, non_snake_case)]
use core::str::FromStr;
use derive_more::{Display, IsVariant};
use smol_str::SmolStr;
/** Video codec family — every codec FFmpeg n8.1 knows under media type `video`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(SmolStr)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum VideoCodec {
  /// FFmpeg `"012v"`.
  _012v,
  /// FFmpeg `"4xm"`.
  _4xm,
  /// FFmpeg `"8bps"`.
  _8bps,
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
  /// FFmpeg `"v308"`.
  V308,
  /// FFmpeg `"v408"`.
  V408,
  /// FFmpeg `"v410"`.
  V410,
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
  Other(SmolStr),
}
impl VideoCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::_012v => "012v",
      Self::_4xm => "4xm",
      Self::_8bps => "8bps",
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
      Self::V308 => "v308",
      Self::V408 => "v408",
      Self::V410 => "v410",
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
}
impl FromStr for VideoCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "012v" => Self::_012v,
      "4xm" => Self::_4xm,
      "8bps" => Self::_8bps,
      "a64_multi" => Self::A64Multi,
      "a64_multi5" => Self::A64Multi5,
      "aasc" => Self::Aasc,
      "agm" => Self::Agm,
      "aic" => Self::Aic,
      "alias_pix" => Self::AliasPix,
      "amv" => Self::Amv,
      "anm" => Self::Anm,
      "ansi" => Self::Ansi,
      "apng" => Self::Apng,
      "apv" => Self::Apv,
      "arbc" => Self::Arbc,
      "argo" => Self::Argo,
      "asv1" => Self::Asv1,
      "asv2" => Self::Asv2,
      "aura" => Self::Aura,
      "aura2" => Self::Aura2,
      "av1" => Self::Av1,
      "avrn" => Self::Avrn,
      "avrp" => Self::Avrp,
      "avs" => Self::Avs,
      "avs2" => Self::Avs2,
      "avs3" => Self::Avs3,
      "avui" => Self::Avui,
      "bethsoftvid" => Self::Bethsoftvid,
      "bfi" => Self::Bfi,
      "binkvideo" => Self::Binkvideo,
      "bintext" => Self::Bintext,
      "bitpacked" => Self::Bitpacked,
      "bmp" => Self::Bmp,
      "bmv_video" => Self::BmvVideo,
      "brender_pix" => Self::BrenderPix,
      "c93" => Self::C93,
      "cavs" => Self::Cavs,
      "cdgraphics" => Self::Cdgraphics,
      "cdtoons" => Self::Cdtoons,
      "cdxl" => Self::Cdxl,
      "cfhd" => Self::Cfhd,
      "cinepak" => Self::Cinepak,
      "clearvideo" => Self::Clearvideo,
      "cljr" => Self::Cljr,
      "cllc" => Self::Cllc,
      "cmv" => Self::Cmv,
      "cpia" => Self::Cpia,
      "cri" => Self::Cri,
      "cscd" => Self::Cscd,
      "cyuv" => Self::Cyuv,
      "daala" => Self::Daala,
      "dds" => Self::Dds,
      "dfa" => Self::Dfa,
      "dirac" => Self::Dirac,
      "dnxhd" => Self::Dnxhd,
      "dnxuc" => Self::Dnxuc,
      "dpx" => Self::Dpx,
      "dsicinvideo" => Self::Dsicinvideo,
      "dvvideo" => Self::Dvvideo,
      "dxa" => Self::Dxa,
      "dxtory" => Self::Dxtory,
      "dxv" => Self::Dxv,
      "escape124" => Self::Escape124,
      "escape130" => Self::Escape130,
      "evc" => Self::Evc,
      "exr" => Self::Exr,
      "ffv1" => Self::Ffv1,
      "ffvhuff" => Self::Ffvhuff,
      "fic" => Self::Fic,
      "fits" => Self::Fits,
      "flashsv" => Self::Flashsv,
      "flashsv2" => Self::Flashsv2,
      "flic" => Self::Flic,
      "flv1" => Self::Flv1,
      "fmvc" => Self::Fmvc,
      "fraps" => Self::Fraps,
      "frwu" => Self::Frwu,
      "g2m" => Self::G2m,
      "gdv" => Self::Gdv,
      "gem" => Self::Gem,
      "gif" => Self::Gif,
      "h261" => Self::H261,
      "h263" => Self::H263,
      "h263i" => Self::H263i,
      "h263p" => Self::H263p,
      "h264" => Self::H264,
      "hap" => Self::Hap,
      "hdr" => Self::Hdr,
      "hevc" => Self::Hevc,
      "hnm4video" => Self::Hnm4video,
      "hq_hqa" => Self::HqHqa,
      "hqx" => Self::Hqx,
      "huffyuv" => Self::Huffyuv,
      "hymt" => Self::Hymt,
      "idcin" => Self::Idcin,
      "idf" => Self::Idf,
      "iff_ilbm" => Self::IffIlbm,
      "imm4" => Self::Imm4,
      "imm5" => Self::Imm5,
      "indeo2" => Self::Indeo2,
      "indeo3" => Self::Indeo3,
      "indeo4" => Self::Indeo4,
      "indeo5" => Self::Indeo5,
      "interplayvideo" => Self::Interplayvideo,
      "ipu" => Self::Ipu,
      "jpeg2000" => Self::Jpeg2000,
      "jpegls" => Self::Jpegls,
      "jpegxl" => Self::Jpegxl,
      "jpegxl_anim" => Self::JpegxlAnim,
      "jpegxs" => Self::Jpegxs,
      "jv" => Self::Jv,
      "kgv1" => Self::Kgv1,
      "kmvc" => Self::Kmvc,
      "lagarith" => Self::Lagarith,
      "lcevc" => Self::Lcevc,
      "lead" => Self::Lead,
      "ljpeg" => Self::Ljpeg,
      "loco" => Self::Loco,
      "lscr" => Self::Lscr,
      "m101" => Self::M101,
      "mad" => Self::Mad,
      "magicyuv" => Self::Magicyuv,
      "mdec" => Self::Mdec,
      "media100" => Self::Media100,
      "mimic" => Self::Mimic,
      "mjpeg" => Self::Mjpeg,
      "mjpegb" => Self::Mjpegb,
      "mmvideo" => Self::Mmvideo,
      "mobiclip" => Self::Mobiclip,
      "motionpixels" => Self::Motionpixels,
      "mpeg1video" => Self::Mpeg1video,
      "mpeg2video" => Self::Mpeg2video,
      "mpeg4" => Self::Mpeg4,
      "msa1" => Self::Msa1,
      "mscc" => Self::Mscc,
      "msmpeg4v1" => Self::Msmpeg4v1,
      "msmpeg4v2" => Self::Msmpeg4v2,
      "msmpeg4v3" => Self::Msmpeg4v3,
      "msp2" => Self::Msp2,
      "msrle" => Self::Msrle,
      "mss1" => Self::Mss1,
      "mss2" => Self::Mss2,
      "msvideo1" => Self::Msvideo1,
      "mszh" => Self::Mszh,
      "mts2" => Self::Mts2,
      "mv30" => Self::Mv30,
      "mvc1" => Self::Mvc1,
      "mvc2" => Self::Mvc2,
      "mvdv" => Self::Mvdv,
      "mvha" => Self::Mvha,
      "mwsc" => Self::Mwsc,
      "mxpeg" => Self::Mxpeg,
      "notchlc" => Self::Notchlc,
      "nuv" => Self::Nuv,
      "paf_video" => Self::PafVideo,
      "pam" => Self::Pam,
      "pbm" => Self::Pbm,
      "pcx" => Self::Pcx,
      "pdv" => Self::Pdv,
      "pfm" => Self::Pfm,
      "pgm" => Self::Pgm,
      "pgmyuv" => Self::Pgmyuv,
      "pgx" => Self::Pgx,
      "phm" => Self::Phm,
      "photocd" => Self::Photocd,
      "pictor" => Self::Pictor,
      "pixlet" => Self::Pixlet,
      "png" => Self::Png,
      "ppm" => Self::Ppm,
      "prores" => Self::Prores,
      "prores_raw" => Self::ProresRaw,
      "prosumer" => Self::Prosumer,
      "psd" => Self::Psd,
      "ptx" => Self::Ptx,
      "qdraw" => Self::Qdraw,
      "qoi" => Self::Qoi,
      "qpeg" => Self::Qpeg,
      "qtrle" => Self::Qtrle,
      "r10k" => Self::R10k,
      "r210" => Self::R210,
      "rasc" => Self::Rasc,
      "rawvideo" => Self::Rawvideo,
      "rl2" => Self::Rl2,
      "roq" => Self::Roq,
      "rpza" => Self::Rpza,
      "rscc" => Self::Rscc,
      "rtv1" => Self::Rtv1,
      "rv10" => Self::Rv10,
      "rv20" => Self::Rv20,
      "rv30" => Self::Rv30,
      "rv40" => Self::Rv40,
      "rv60" => Self::Rv60,
      "sanm" => Self::Sanm,
      "scpr" => Self::Scpr,
      "screenpresso" => Self::Screenpresso,
      "sga" => Self::Sga,
      "sgi" => Self::Sgi,
      "sgirle" => Self::Sgirle,
      "sheervideo" => Self::Sheervideo,
      "simbiosis_imx" => Self::SimbiosisImx,
      "smackvideo" => Self::Smackvideo,
      "smc" => Self::Smc,
      "smvjpeg" => Self::Smvjpeg,
      "snow" => Self::Snow,
      "sp5x" => Self::Sp5x,
      "speedhq" => Self::Speedhq,
      "srgc" => Self::Srgc,
      "sunrast" => Self::Sunrast,
      "svg" => Self::Svg,
      "svq1" => Self::Svq1,
      "svq3" => Self::Svq3,
      "targa" => Self::Targa,
      "targa_y216" => Self::TargaY216,
      "tdsc" => Self::Tdsc,
      "tgq" => Self::Tgq,
      "tgv" => Self::Tgv,
      "theora" => Self::Theora,
      "thp" => Self::Thp,
      "tiertexseqvideo" => Self::Tiertexseqvideo,
      "tiff" => Self::Tiff,
      "tmv" => Self::Tmv,
      "tqi" => Self::Tqi,
      "truemotion1" => Self::Truemotion1,
      "truemotion2" => Self::Truemotion2,
      "truemotion2rt" => Self::Truemotion2rt,
      "tscc" => Self::Tscc,
      "tscc2" => Self::Tscc2,
      "txd" => Self::Txd,
      "ulti" => Self::Ulti,
      "utvideo" => Self::Utvideo,
      "v210" => Self::V210,
      "v210x" => Self::V210x,
      "v308" => Self::V308,
      "v408" => Self::V408,
      "v410" => Self::V410,
      "vb" => Self::Vb,
      "vble" => Self::Vble,
      "vbn" => Self::Vbn,
      "vc1" => Self::Vc1,
      "vc1image" => Self::Vc1image,
      "vcr1" => Self::Vcr1,
      "vixl" => Self::Vixl,
      "vmdvideo" => Self::Vmdvideo,
      "vmix" => Self::Vmix,
      "vmnc" => Self::Vmnc,
      "vnull" => Self::Vnull,
      "vp3" => Self::Vp3,
      "vp4" => Self::Vp4,
      "vp5" => Self::Vp5,
      "vp6" => Self::Vp6,
      "vp6a" => Self::Vp6a,
      "vp6f" => Self::Vp6f,
      "vp7" => Self::Vp7,
      "vp8" => Self::Vp8,
      "vp9" => Self::Vp9,
      "vqc" => Self::Vqc,
      "vvc" => Self::Vvc,
      "wbmp" => Self::Wbmp,
      "wcmv" => Self::Wcmv,
      "webp" => Self::Webp,
      "wmv1" => Self::Wmv1,
      "wmv2" => Self::Wmv2,
      "wmv3" => Self::Wmv3,
      "wmv3image" => Self::Wmv3image,
      "wnv1" => Self::Wnv1,
      "wrapped_avframe" => Self::WrappedAvframe,
      "ws_vqa" => Self::WsVqa,
      "xan_wc3" => Self::XanWc3,
      "xan_wc4" => Self::XanWc4,
      "xbin" => Self::Xbin,
      "xbm" => Self::Xbm,
      "xface" => Self::Xface,
      "xpm" => Self::Xpm,
      "xwd" => Self::Xwd,
      "y41p" => Self::Y41p,
      "ylc" => Self::Ylc,
      "yop" => Self::Yop,
      "yuv4" => Self::Yuv4,
      "zerocodec" => Self::Zerocodec,
      "zlib" => Self::Zlib,
      "zmbv" => Self::Zmbv,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}
/** Audio codec family — every codec FFmpeg n8.1 knows under media type `audio`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(SmolStr)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum AudioCodec {
  /// FFmpeg `"4gv"`.
  _4gv,
  /// FFmpeg `"8svx_exp"`.
  _8svxExp,
  /// FFmpeg `"8svx_fib"`.
  _8svxFib,
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
  Other(SmolStr),
}
impl AudioCodec {
  /// Canonical FFmpeg short name (matches `ffmpeg -codecs` column 2).
  pub fn as_str(&self) -> &str {
    match self {
      Self::_4gv => "4gv",
      Self::_8svxExp => "8svx_exp",
      Self::_8svxFib => "8svx_fib",
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
}
impl FromStr for AudioCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "4gv" => Self::_4gv,
      "8svx_exp" => Self::_8svxExp,
      "8svx_fib" => Self::_8svxFib,
      "aac" => Self::Aac,
      "aac_latm" => Self::AacLatm,
      "ac3" => Self::Ac3,
      "ac4" => Self::Ac4,
      "acelp.kelvin" => Self::AcelpKelvin,
      "adpcm_4xm" => Self::Adpcm4xm,
      "adpcm_adx" => Self::AdpcmAdx,
      "adpcm_afc" => Self::AdpcmAfc,
      "adpcm_agm" => Self::AdpcmAgm,
      "adpcm_aica" => Self::AdpcmAica,
      "adpcm_argo" => Self::AdpcmArgo,
      "adpcm_circus" => Self::AdpcmCircus,
      "adpcm_ct" => Self::AdpcmCt,
      "adpcm_dtk" => Self::AdpcmDtk,
      "adpcm_ea" => Self::AdpcmEa,
      "adpcm_ea_maxis_xa" => Self::AdpcmEaMaxisXa,
      "adpcm_ea_r1" => Self::AdpcmEaR1,
      "adpcm_ea_r2" => Self::AdpcmEaR2,
      "adpcm_ea_r3" => Self::AdpcmEaR3,
      "adpcm_ea_xas" => Self::AdpcmEaXas,
      "adpcm_g722" => Self::AdpcmG722,
      "adpcm_g726" => Self::AdpcmG726,
      "adpcm_g726le" => Self::AdpcmG726le,
      "adpcm_ima_acorn" => Self::AdpcmImaAcorn,
      "adpcm_ima_alp" => Self::AdpcmImaAlp,
      "adpcm_ima_amv" => Self::AdpcmImaAmv,
      "adpcm_ima_apc" => Self::AdpcmImaApc,
      "adpcm_ima_apm" => Self::AdpcmImaApm,
      "adpcm_ima_cunning" => Self::AdpcmImaCunning,
      "adpcm_ima_dat4" => Self::AdpcmImaDat4,
      "adpcm_ima_dk3" => Self::AdpcmImaDk3,
      "adpcm_ima_dk4" => Self::AdpcmImaDk4,
      "adpcm_ima_ea_eacs" => Self::AdpcmImaEaEacs,
      "adpcm_ima_ea_sead" => Self::AdpcmImaEaSead,
      "adpcm_ima_escape" => Self::AdpcmImaEscape,
      "adpcm_ima_hvqm2" => Self::AdpcmImaHvqm2,
      "adpcm_ima_hvqm4" => Self::AdpcmImaHvqm4,
      "adpcm_ima_iss" => Self::AdpcmImaIss,
      "adpcm_ima_magix" => Self::AdpcmImaMagix,
      "adpcm_ima_moflex" => Self::AdpcmImaMoflex,
      "adpcm_ima_mtf" => Self::AdpcmImaMtf,
      "adpcm_ima_oki" => Self::AdpcmImaOki,
      "adpcm_ima_pda" => Self::AdpcmImaPda,
      "adpcm_ima_qt" => Self::AdpcmImaQt,
      "adpcm_ima_rad" => Self::AdpcmImaRad,
      "adpcm_ima_smjpeg" => Self::AdpcmImaSmjpeg,
      "adpcm_ima_ssi" => Self::AdpcmImaSsi,
      "adpcm_ima_wav" => Self::AdpcmImaWav,
      "adpcm_ima_ws" => Self::AdpcmImaWs,
      "adpcm_ima_xbox" => Self::AdpcmImaXbox,
      "adpcm_ms" => Self::AdpcmMs,
      "adpcm_mtaf" => Self::AdpcmMtaf,
      "adpcm_n64" => Self::AdpcmN64,
      "adpcm_psx" => Self::AdpcmPsx,
      "adpcm_psxc" => Self::AdpcmPsxc,
      "adpcm_sanyo" => Self::AdpcmSanyo,
      "adpcm_sbpro_2" => Self::AdpcmSbpro2,
      "adpcm_sbpro_3" => Self::AdpcmSbpro3,
      "adpcm_sbpro_4" => Self::AdpcmSbpro4,
      "adpcm_swf" => Self::AdpcmSwf,
      "adpcm_thp" => Self::AdpcmThp,
      "adpcm_thp_le" => Self::AdpcmThpLe,
      "adpcm_vima" => Self::AdpcmVima,
      "adpcm_xa" => Self::AdpcmXa,
      "adpcm_xmd" => Self::AdpcmXmd,
      "adpcm_yamaha" => Self::AdpcmYamaha,
      "adpcm_zork" => Self::AdpcmZork,
      "ahx" => Self::Ahx,
      "alac" => Self::Alac,
      "amr_nb" => Self::AmrNb,
      "amr_wb" => Self::AmrWb,
      "anull" => Self::Anull,
      "apac" => Self::Apac,
      "ape" => Self::Ape,
      "aptx" => Self::Aptx,
      "aptx_hd" => Self::AptxHd,
      "atrac1" => Self::Atrac1,
      "atrac3" => Self::Atrac3,
      "atrac3al" => Self::Atrac3al,
      "atrac3p" => Self::Atrac3p,
      "atrac3pal" => Self::Atrac3pal,
      "atrac9" => Self::Atrac9,
      "avc" => Self::Avc,
      "binkaudio_dct" => Self::BinkaudioDct,
      "binkaudio_rdft" => Self::BinkaudioRdft,
      "bmv_audio" => Self::BmvAudio,
      "bonk" => Self::Bonk,
      "cbd2_dpcm" => Self::Cbd2Dpcm,
      "celt" => Self::Celt,
      "codec2" => Self::Codec2,
      "comfortnoise" => Self::Comfortnoise,
      "cook" => Self::Cook,
      "derf_dpcm" => Self::DerfDpcm,
      "dfpwm" => Self::Dfpwm,
      "dolby_e" => Self::DolbyE,
      "dsd_lsbf" => Self::DsdLsbf,
      "dsd_lsbf_planar" => Self::DsdLsbfPlanar,
      "dsd_msbf" => Self::DsdMsbf,
      "dsd_msbf_planar" => Self::DsdMsbfPlanar,
      "dsicinaudio" => Self::Dsicinaudio,
      "dss_sp" => Self::DssSp,
      "dst" => Self::Dst,
      "dts" => Self::Dts,
      "dvaudio" => Self::Dvaudio,
      "eac3" => Self::Eac3,
      "evrc" => Self::Evrc,
      "fastaudio" => Self::Fastaudio,
      "flac" => Self::Flac,
      "ftr" => Self::Ftr,
      "g723_1" => Self::G7231,
      "g728" => Self::G728,
      "g729" => Self::G729,
      "gremlin_dpcm" => Self::GremlinDpcm,
      "gsm" => Self::Gsm,
      "gsm_ms" => Self::GsmMs,
      "hca" => Self::Hca,
      "hcom" => Self::Hcom,
      "iac" => Self::Iac,
      "ilbc" => Self::Ilbc,
      "imc" => Self::Imc,
      "interplay_dpcm" => Self::InterplayDpcm,
      "interplayacm" => Self::Interplayacm,
      "lc3" => Self::Lc3,
      "mace3" => Self::Mace3,
      "mace6" => Self::Mace6,
      "metasound" => Self::Metasound,
      "misc4" => Self::Misc4,
      "mlp" => Self::Mlp,
      "mp1" => Self::Mp1,
      "mp2" => Self::Mp2,
      "mp3" => Self::Mp3,
      "mp3adu" => Self::Mp3adu,
      "mp3on4" => Self::Mp3on4,
      "mp4als" => Self::Mp4als,
      "mpegh_3d_audio" => Self::Mpegh3dAudio,
      "msnsiren" => Self::Msnsiren,
      "musepack7" => Self::Musepack7,
      "musepack8" => Self::Musepack8,
      "nellymoser" => Self::Nellymoser,
      "opus" => Self::Opus,
      "osq" => Self::Osq,
      "paf_audio" => Self::PafAudio,
      "pcm_alaw" => Self::PcmAlaw,
      "pcm_bluray" => Self::PcmBluray,
      "pcm_dvd" => Self::PcmDvd,
      "pcm_f16le" => Self::PcmF16le,
      "pcm_f24le" => Self::PcmF24le,
      "pcm_f32be" => Self::PcmF32be,
      "pcm_f32le" => Self::PcmF32le,
      "pcm_f64be" => Self::PcmF64be,
      "pcm_f64le" => Self::PcmF64le,
      "pcm_lxf" => Self::PcmLxf,
      "pcm_mulaw" => Self::PcmMulaw,
      "pcm_s16be" => Self::PcmS16be,
      "pcm_s16be_planar" => Self::PcmS16bePlanar,
      "pcm_s16le" => Self::PcmS16le,
      "pcm_s16le_planar" => Self::PcmS16lePlanar,
      "pcm_s24be" => Self::PcmS24be,
      "pcm_s24daud" => Self::PcmS24daud,
      "pcm_s24le" => Self::PcmS24le,
      "pcm_s24le_planar" => Self::PcmS24lePlanar,
      "pcm_s32be" => Self::PcmS32be,
      "pcm_s32le" => Self::PcmS32le,
      "pcm_s32le_planar" => Self::PcmS32lePlanar,
      "pcm_s64be" => Self::PcmS64be,
      "pcm_s64le" => Self::PcmS64le,
      "pcm_s8" => Self::PcmS8,
      "pcm_s8_planar" => Self::PcmS8Planar,
      "pcm_sga" => Self::PcmSga,
      "pcm_u16be" => Self::PcmU16be,
      "pcm_u16le" => Self::PcmU16le,
      "pcm_u24be" => Self::PcmU24be,
      "pcm_u24le" => Self::PcmU24le,
      "pcm_u32be" => Self::PcmU32be,
      "pcm_u32le" => Self::PcmU32le,
      "pcm_u8" => Self::PcmU8,
      "pcm_vidc" => Self::PcmVidc,
      "qcelp" => Self::Qcelp,
      "qdm2" => Self::Qdm2,
      "qdmc" => Self::Qdmc,
      "qoa" => Self::Qoa,
      "ra_144" => Self::Ra144,
      "ra_288" => Self::Ra288,
      "ralf" => Self::Ralf,
      "rka" => Self::Rka,
      "roq_dpcm" => Self::RoqDpcm,
      "s302m" => Self::S302m,
      "sbc" => Self::Sbc,
      "sdx2_dpcm" => Self::Sdx2Dpcm,
      "shorten" => Self::Shorten,
      "sipr" => Self::Sipr,
      "siren" => Self::Siren,
      "smackaudio" => Self::Smackaudio,
      "smv" => Self::Smv,
      "sol_dpcm" => Self::SolDpcm,
      "sonic" => Self::Sonic,
      "sonicls" => Self::Sonicls,
      "speex" => Self::Speex,
      "tak" => Self::Tak,
      "truehd" => Self::Truehd,
      "truespeech" => Self::Truespeech,
      "tta" => Self::Tta,
      "twinvq" => Self::Twinvq,
      "vmdaudio" => Self::Vmdaudio,
      "vorbis" => Self::Vorbis,
      "wady_dpcm" => Self::WadyDpcm,
      "wavarc" => Self::Wavarc,
      "wavesynth" => Self::Wavesynth,
      "wavpack" => Self::Wavpack,
      "westwood_snd1" => Self::WestwoodSnd1,
      "wmalossless" => Self::Wmalossless,
      "wmapro" => Self::Wmapro,
      "wmav1" => Self::Wmav1,
      "wmav2" => Self::Wmav2,
      "wmavoice" => Self::Wmavoice,
      "xan_dpcm" => Self::XanDpcm,
      "xma1" => Self::Xma1,
      "xma2" => Self::Xma2,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}
/** Subtitle codec family — every codec FFmpeg n8.1 knows under media type `subtitle`.

`#[non_exhaustive]` keeps future additions non-breaking; the `Other(SmolStr)` arm is the lossless escape for codecs added upstream before this file is regenerated.*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
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
  Other(SmolStr),
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
  /// Is this a **bitmap** (image-based) subtitle codec, requiring an
  /// OCR pipeline stage to extract searchable text?
  ///
  /// - `Some(true)`: matches FFmpeg's `AV_CODEC_PROP_BITMAP_SUB` flag.
  /// - `Some(false)`: a known FFmpeg subtitle codec without
  ///   `AV_CODEC_PROP_BITMAP_SUB` (text codecs and teletext/VBI-style
  ///   codecs that carry no `.props` at all in FFmpeg n8.1).
  /// - `None`: [`Self::Other`] — the codec name is not in the vendored
  ///   FFmpeg table, so we cannot consult `.props`.
  ///
  /// (4 bitmap / 23 non-bitmap variant(s) per FFmpeg n8.1).
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
impl FromStr for SubtitleCodec {
  type Err = core::convert::Infallible;
  /// Recognise an FFmpeg codec short name; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s {
      "arib_caption" => Self::AribCaption,
      "ass" => Self::Ass,
      "dvb_subtitle" => Self::DvbSubtitle,
      "dvb_teletext" => Self::DvbTeletext,
      "dvd_subtitle" => Self::DvdSubtitle,
      "eia_608" => Self::Eia608,
      "hdmv_pgs_subtitle" => Self::HdmvPgsSubtitle,
      "hdmv_text_subtitle" => Self::HdmvTextSubtitle,
      "ivtv_vbi" => Self::IvtvVbi,
      "jacosub" => Self::Jacosub,
      "microdvd" => Self::Microdvd,
      "mov_text" => Self::MovText,
      "mpl2" => Self::Mpl2,
      "pjs" => Self::Pjs,
      "realtext" => Self::Realtext,
      "sami" => Self::Sami,
      "srt" => Self::Srt,
      "ssa" => Self::Ssa,
      "stl" => Self::Stl,
      "subrip" => Self::Subrip,
      "subviewer" => Self::Subviewer,
      "subviewer1" => Self::Subviewer1,
      "text" => Self::Text,
      "ttml" => Self::Ttml,
      "vplayer" => Self::Vplayer,
      "webvtt" => Self::Webvtt,
      "xsub" => Self::Xsub,
      other => Self::Other(SmolStr::new(other)),
    })
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  const VENDOR: &str = include_str!("../../xtask/vendor/ffmpeg-codecs.txt");
  fn vendored_of(media: &'static str) -> impl Iterator<Item = &'static str> {
    VENDOR.lines().filter_map(move |l| {
      let l = l.trim();
      if l.is_empty() || l.starts_with('#') {
        return None;
      }
      let mut it = l.split_whitespace();
      match (it.next(), it.next()) {
        (Some(m), Some(n)) if m == media => Some(n),
        _ => None,
      }
    })
  }
  #[test]
  fn every_video_codec_round_trips_to_named_variant() {
    let mut n = 0usize;
    for name in vendored_of("video") {
      let c: VideoCodec = name.parse().unwrap();
      assert!(
        !c.is_other(),
        "video `{name}` should parse to a named variant"
      );
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
      assert!(
        !c.is_other(),
        "audio `{name}` should parse to a named variant"
      );
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
      assert!(
        !c.is_other(),
        "subtitle `{name}` should parse to a named variant"
      );
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
      assert_eq!(
        c.is_image_based(),
        Some(true),
        "`{n}` should be image-based"
      );
    }
    for n in [
      "subrip", "ass", "ssa", "webvtt", "mov_text", "ttml", "microdvd",
    ] {
      let c: SubtitleCodec = n.parse().unwrap();
      assert_eq!(
        c.is_image_based(),
        Some(false),
        "`{n}` should NOT be image-based"
      );
    }
  }
  #[test]
  fn subtitle_image_based_is_unknown_for_other() {
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
