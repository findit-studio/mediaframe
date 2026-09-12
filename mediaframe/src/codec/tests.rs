use super::*;
use ::std::string::ToString;
/// Every `(media_type, FFmpeg short name)` pair this module was
/// generated from — embedded at codegen so the test suite stays
/// self-contained when `mediaframe` is packaged for crates.io.
const VENDORED_PAIRS: &[(&str, &str)] = &[
  ("video", "012v"),
  ("video", "4xm"),
  ("video", "8bps"),
  ("video", "a64_multi"),
  ("video", "a64_multi5"),
  ("video", "aasc"),
  ("video", "agm"),
  ("video", "aic"),
  ("video", "alias_pix"),
  ("video", "amv"),
  ("video", "anm"),
  ("video", "ansi"),
  ("video", "apng"),
  ("video", "apv"),
  ("video", "arbc"),
  ("video", "argo"),
  ("video", "asv1"),
  ("video", "asv2"),
  ("video", "aura"),
  ("video", "aura2"),
  ("video", "av1"),
  ("video", "avrn"),
  ("video", "avrp"),
  ("video", "avs"),
  ("video", "avs2"),
  ("video", "avs3"),
  ("video", "avui"),
  ("video", "bethsoftvid"),
  ("video", "bfi"),
  ("video", "binkvideo"),
  ("video", "bintext"),
  ("video", "bitpacked"),
  ("video", "bmp"),
  ("video", "bmv_video"),
  ("video", "brender_pix"),
  ("video", "c93"),
  ("video", "cavs"),
  ("video", "cdgraphics"),
  ("video", "cdtoons"),
  ("video", "cdxl"),
  ("video", "cfhd"),
  ("video", "cinepak"),
  ("video", "clearvideo"),
  ("video", "cljr"),
  ("video", "cllc"),
  ("video", "cmv"),
  ("video", "cpia"),
  ("video", "cri"),
  ("video", "cscd"),
  ("video", "cyuv"),
  ("video", "daala"),
  ("video", "dds"),
  ("video", "dfa"),
  ("video", "dirac"),
  ("video", "dnxhd"),
  ("video", "dnxuc"),
  ("video", "dpx"),
  ("video", "dsicinvideo"),
  ("video", "dvvideo"),
  ("video", "dxa"),
  ("video", "dxtory"),
  ("video", "dxv"),
  ("video", "escape124"),
  ("video", "escape130"),
  ("video", "evc"),
  ("video", "exr"),
  ("video", "ffv1"),
  ("video", "ffvhuff"),
  ("video", "fic"),
  ("video", "fits"),
  ("video", "flashsv"),
  ("video", "flashsv2"),
  ("video", "flic"),
  ("video", "flv1"),
  ("video", "fmvc"),
  ("video", "fraps"),
  ("video", "frwu"),
  ("video", "g2m"),
  ("video", "gdv"),
  ("video", "gem"),
  ("video", "gif"),
  ("video", "h261"),
  ("video", "h263"),
  ("video", "h263i"),
  ("video", "h263p"),
  ("video", "h264"),
  ("video", "hap"),
  ("video", "hdr"),
  ("video", "hevc"),
  ("video", "hnm4video"),
  ("video", "hq_hqa"),
  ("video", "hqx"),
  ("video", "huffyuv"),
  ("video", "hymt"),
  ("video", "idcin"),
  ("video", "idf"),
  ("video", "iff_ilbm"),
  ("video", "imm4"),
  ("video", "imm5"),
  ("video", "indeo2"),
  ("video", "indeo3"),
  ("video", "indeo4"),
  ("video", "indeo5"),
  ("video", "interplayvideo"),
  ("video", "ipu"),
  ("video", "jpeg2000"),
  ("video", "jpegls"),
  ("video", "jpegxl"),
  ("video", "jpegxl_anim"),
  ("video", "jpegxs"),
  ("video", "jv"),
  ("video", "kgv1"),
  ("video", "kmvc"),
  ("video", "lagarith"),
  ("video", "lcevc"),
  ("video", "lead"),
  ("video", "ljpeg"),
  ("video", "loco"),
  ("video", "lscr"),
  ("video", "m101"),
  ("video", "mad"),
  ("video", "magicyuv"),
  ("video", "mdec"),
  ("video", "media100"),
  ("video", "mimic"),
  ("video", "mjpeg"),
  ("video", "mjpegb"),
  ("video", "mmvideo"),
  ("video", "mobiclip"),
  ("video", "motionpixels"),
  ("video", "mpeg1video"),
  ("video", "mpeg2video"),
  ("video", "mpeg4"),
  ("video", "msa1"),
  ("video", "mscc"),
  ("video", "msmpeg4v1"),
  ("video", "msmpeg4v2"),
  ("video", "msmpeg4v3"),
  ("video", "msp2"),
  ("video", "msrle"),
  ("video", "mss1"),
  ("video", "mss2"),
  ("video", "msvideo1"),
  ("video", "mszh"),
  ("video", "mts2"),
  ("video", "mv30"),
  ("video", "mvc1"),
  ("video", "mvc2"),
  ("video", "mvdv"),
  ("video", "mvha"),
  ("video", "mwsc"),
  ("video", "mxpeg"),
  ("video", "notchlc"),
  ("video", "nuv"),
  ("video", "paf_video"),
  ("video", "pam"),
  ("video", "pbm"),
  ("video", "pcx"),
  ("video", "pdv"),
  ("video", "pfm"),
  ("video", "pgm"),
  ("video", "pgmyuv"),
  ("video", "pgx"),
  ("video", "phm"),
  ("video", "photocd"),
  ("video", "pictor"),
  ("video", "pixlet"),
  ("video", "png"),
  ("video", "ppm"),
  ("video", "prores"),
  ("video", "prores_raw"),
  ("video", "prosumer"),
  ("video", "psd"),
  ("video", "ptx"),
  ("video", "qdraw"),
  ("video", "qoi"),
  ("video", "qpeg"),
  ("video", "qtrle"),
  ("video", "r10k"),
  ("video", "r210"),
  ("video", "rasc"),
  ("video", "rawvideo"),
  ("video", "rl2"),
  ("video", "roq"),
  ("video", "rpza"),
  ("video", "rscc"),
  ("video", "rtv1"),
  ("video", "rv10"),
  ("video", "rv20"),
  ("video", "rv30"),
  ("video", "rv40"),
  ("video", "rv60"),
  ("video", "sanm"),
  ("video", "scpr"),
  ("video", "screenpresso"),
  ("video", "sga"),
  ("video", "sgi"),
  ("video", "sgirle"),
  ("video", "sheervideo"),
  ("video", "simbiosis_imx"),
  ("video", "smackvideo"),
  ("video", "smc"),
  ("video", "smvjpeg"),
  ("video", "snow"),
  ("video", "sp5x"),
  ("video", "speedhq"),
  ("video", "srgc"),
  ("video", "sunrast"),
  ("video", "svg"),
  ("video", "svq1"),
  ("video", "svq3"),
  ("video", "targa"),
  ("video", "targa_y216"),
  ("video", "tdsc"),
  ("video", "tgq"),
  ("video", "tgv"),
  ("video", "theora"),
  ("video", "thp"),
  ("video", "tiertexseqvideo"),
  ("video", "tiff"),
  ("video", "tmv"),
  ("video", "tqi"),
  ("video", "truemotion1"),
  ("video", "truemotion2"),
  ("video", "truemotion2rt"),
  ("video", "tscc"),
  ("video", "tscc2"),
  ("video", "txd"),
  ("video", "ulti"),
  ("video", "utvideo"),
  ("video", "v210"),
  ("video", "v210x"),
  ("video", "vb"),
  ("video", "vble"),
  ("video", "vbn"),
  ("video", "vc1"),
  ("video", "vc1image"),
  ("video", "vcr1"),
  ("video", "vixl"),
  ("video", "vmdvideo"),
  ("video", "vmix"),
  ("video", "vmnc"),
  ("video", "vnull"),
  ("video", "vp3"),
  ("video", "vp4"),
  ("video", "vp5"),
  ("video", "vp6"),
  ("video", "vp6a"),
  ("video", "vp6f"),
  ("video", "vp7"),
  ("video", "vp8"),
  ("video", "vp9"),
  ("video", "vqc"),
  ("video", "vvc"),
  ("video", "wbmp"),
  ("video", "wcmv"),
  ("video", "webp"),
  ("video", "webp_anim"),
  ("video", "wmv1"),
  ("video", "wmv2"),
  ("video", "wmv3"),
  ("video", "wmv3image"),
  ("video", "wnv1"),
  ("video", "wrapped_avframe"),
  ("video", "ws_vqa"),
  ("video", "xan_wc3"),
  ("video", "xan_wc4"),
  ("video", "xbin"),
  ("video", "xbm"),
  ("video", "xface"),
  ("video", "xpm"),
  ("video", "xwd"),
  ("video", "y41p"),
  ("video", "ylc"),
  ("video", "yop"),
  ("video", "yuv4"),
  ("video", "zerocodec"),
  ("video", "zlib"),
  ("video", "zmbv"),
  ("audio", "4gv"),
  ("audio", "8svx_exp"),
  ("audio", "8svx_fib"),
  ("audio", "aac"),
  ("audio", "aac_latm"),
  ("audio", "ac3"),
  ("audio", "ac4"),
  ("audio", "acelp.kelvin"),
  ("audio", "adpcm_4xm"),
  ("audio", "adpcm_adx"),
  ("audio", "adpcm_afc"),
  ("audio", "adpcm_agm"),
  ("audio", "adpcm_aica"),
  ("audio", "adpcm_argo"),
  ("audio", "adpcm_circus"),
  ("audio", "adpcm_ct"),
  ("audio", "adpcm_dtk"),
  ("audio", "adpcm_ea"),
  ("audio", "adpcm_ea_maxis_xa"),
  ("audio", "adpcm_ea_r1"),
  ("audio", "adpcm_ea_r2"),
  ("audio", "adpcm_ea_r3"),
  ("audio", "adpcm_ea_xas"),
  ("audio", "adpcm_g722"),
  ("audio", "adpcm_g726"),
  ("audio", "adpcm_g726le"),
  ("audio", "adpcm_ima_acorn"),
  ("audio", "adpcm_ima_alp"),
  ("audio", "adpcm_ima_amv"),
  ("audio", "adpcm_ima_apc"),
  ("audio", "adpcm_ima_apm"),
  ("audio", "adpcm_ima_cunning"),
  ("audio", "adpcm_ima_dat4"),
  ("audio", "adpcm_ima_dk3"),
  ("audio", "adpcm_ima_dk4"),
  ("audio", "adpcm_ima_ea_eacs"),
  ("audio", "adpcm_ima_ea_sead"),
  ("audio", "adpcm_ima_escape"),
  ("audio", "adpcm_ima_hvqm2"),
  ("audio", "adpcm_ima_hvqm4"),
  ("audio", "adpcm_ima_iss"),
  ("audio", "adpcm_ima_magix"),
  ("audio", "adpcm_ima_moflex"),
  ("audio", "adpcm_ima_mtf"),
  ("audio", "adpcm_ima_oki"),
  ("audio", "adpcm_ima_pda"),
  ("audio", "adpcm_ima_qt"),
  ("audio", "adpcm_ima_rad"),
  ("audio", "adpcm_ima_smjpeg"),
  ("audio", "adpcm_ima_ssi"),
  ("audio", "adpcm_ima_wav"),
  ("audio", "adpcm_ima_ws"),
  ("audio", "adpcm_ima_xbox"),
  ("audio", "adpcm_ms"),
  ("audio", "adpcm_mtaf"),
  ("audio", "adpcm_n64"),
  ("audio", "adpcm_psx"),
  ("audio", "adpcm_psxc"),
  ("audio", "adpcm_sanyo"),
  ("audio", "adpcm_sbpro_2"),
  ("audio", "adpcm_sbpro_3"),
  ("audio", "adpcm_sbpro_4"),
  ("audio", "adpcm_swf"),
  ("audio", "adpcm_thp"),
  ("audio", "adpcm_thp_le"),
  ("audio", "adpcm_vima"),
  ("audio", "adpcm_xa"),
  ("audio", "adpcm_xmd"),
  ("audio", "adpcm_yamaha"),
  ("audio", "adpcm_zork"),
  ("audio", "ahx"),
  ("audio", "alac"),
  ("audio", "amr_nb"),
  ("audio", "amr_wb"),
  ("audio", "anull"),
  ("audio", "apac"),
  ("audio", "ape"),
  ("audio", "apple_apac"),
  ("audio", "aptx"),
  ("audio", "aptx_hd"),
  ("audio", "atrac1"),
  ("audio", "atrac3"),
  ("audio", "atrac3al"),
  ("audio", "atrac3p"),
  ("audio", "atrac3pal"),
  ("audio", "atrac9"),
  ("audio", "avc"),
  ("audio", "binkaudio_dct"),
  ("audio", "binkaudio_rdft"),
  ("audio", "bmv_audio"),
  ("audio", "bonk"),
  ("audio", "cbd2_dpcm"),
  ("audio", "celt"),
  ("audio", "codec2"),
  ("audio", "comfortnoise"),
  ("audio", "cook"),
  ("audio", "derf_dpcm"),
  ("audio", "dfpwm"),
  ("audio", "dolby_e"),
  ("audio", "dsd_lsbf"),
  ("audio", "dsd_lsbf_planar"),
  ("audio", "dsd_msbf"),
  ("audio", "dsd_msbf_planar"),
  ("audio", "dsicinaudio"),
  ("audio", "dss_sp"),
  ("audio", "dst"),
  ("audio", "dts"),
  ("audio", "dvaudio"),
  ("audio", "eac3"),
  ("audio", "evrc"),
  ("audio", "fastaudio"),
  ("audio", "flac"),
  ("audio", "ftr"),
  ("audio", "g723_1"),
  ("audio", "g728"),
  ("audio", "g729"),
  ("audio", "gremlin_dpcm"),
  ("audio", "gsm"),
  ("audio", "gsm_ms"),
  ("audio", "hca"),
  ("audio", "hcom"),
  ("audio", "iac"),
  ("audio", "ilbc"),
  ("audio", "imc"),
  ("audio", "interplay_dpcm"),
  ("audio", "interplayacm"),
  ("audio", "lc3"),
  ("audio", "mace3"),
  ("audio", "mace6"),
  ("audio", "metasound"),
  ("audio", "misc4"),
  ("audio", "mlp"),
  ("audio", "mp1"),
  ("audio", "mp2"),
  ("audio", "mp3"),
  ("audio", "mp3adu"),
  ("audio", "mp3on4"),
  ("audio", "mp4als"),
  ("audio", "mpegh_3d_audio"),
  ("audio", "msnsiren"),
  ("audio", "musepack7"),
  ("audio", "musepack8"),
  ("audio", "nellymoser"),
  ("audio", "opus"),
  ("audio", "osq"),
  ("audio", "paf_audio"),
  ("audio", "pcm_alaw"),
  ("audio", "pcm_bluray"),
  ("audio", "pcm_dvd"),
  ("audio", "pcm_f16le"),
  ("audio", "pcm_f24le"),
  ("audio", "pcm_f32be"),
  ("audio", "pcm_f32le"),
  ("audio", "pcm_f64be"),
  ("audio", "pcm_f64le"),
  ("audio", "pcm_lxf"),
  ("audio", "pcm_mulaw"),
  ("audio", "pcm_s16be"),
  ("audio", "pcm_s16be_planar"),
  ("audio", "pcm_s16le"),
  ("audio", "pcm_s16le_planar"),
  ("audio", "pcm_s24be"),
  ("audio", "pcm_s24daud"),
  ("audio", "pcm_s24le"),
  ("audio", "pcm_s24le_planar"),
  ("audio", "pcm_s32be"),
  ("audio", "pcm_s32le"),
  ("audio", "pcm_s32le_planar"),
  ("audio", "pcm_s64be"),
  ("audio", "pcm_s64le"),
  ("audio", "pcm_s8"),
  ("audio", "pcm_s8_planar"),
  ("audio", "pcm_sga"),
  ("audio", "pcm_u16be"),
  ("audio", "pcm_u16le"),
  ("audio", "pcm_u24be"),
  ("audio", "pcm_u24le"),
  ("audio", "pcm_u32be"),
  ("audio", "pcm_u32le"),
  ("audio", "pcm_u8"),
  ("audio", "pcm_vidc"),
  ("audio", "qcelp"),
  ("audio", "qdm2"),
  ("audio", "qdmc"),
  ("audio", "qoa"),
  ("audio", "ra_144"),
  ("audio", "ra_288"),
  ("audio", "ralf"),
  ("audio", "rka"),
  ("audio", "roq_dpcm"),
  ("audio", "s302m"),
  ("audio", "sbc"),
  ("audio", "sdx2_dpcm"),
  ("audio", "shorten"),
  ("audio", "sipr"),
  ("audio", "siren"),
  ("audio", "smackaudio"),
  ("audio", "smv"),
  ("audio", "sol_dpcm"),
  ("audio", "sonic"),
  ("audio", "sonicls"),
  ("audio", "speex"),
  ("audio", "tak"),
  ("audio", "truehd"),
  ("audio", "truespeech"),
  ("audio", "tta"),
  ("audio", "twinvq"),
  ("audio", "vmdaudio"),
  ("audio", "vorbis"),
  ("audio", "wady_dpcm"),
  ("audio", "wavarc"),
  ("audio", "wavesynth"),
  ("audio", "wavpack"),
  ("audio", "westwood_snd1"),
  ("audio", "wmalossless"),
  ("audio", "wmapro"),
  ("audio", "wmav1"),
  ("audio", "wmav2"),
  ("audio", "wmavoice"),
  ("audio", "xan_dpcm"),
  ("audio", "xma1"),
  ("audio", "xma2"),
  ("subtitle", "arib_caption"),
  ("subtitle", "ass"),
  ("subtitle", "dvb_subtitle"),
  ("subtitle", "dvb_teletext"),
  ("subtitle", "dvd_subtitle"),
  ("subtitle", "eia_608"),
  ("subtitle", "hdmv_pgs_subtitle"),
  ("subtitle", "hdmv_text_subtitle"),
  ("subtitle", "ivtv_vbi"),
  ("subtitle", "jacosub"),
  ("subtitle", "microdvd"),
  ("subtitle", "mov_text"),
  ("subtitle", "mpl2"),
  ("subtitle", "pjs"),
  ("subtitle", "realtext"),
  ("subtitle", "sami"),
  ("subtitle", "srt"),
  ("subtitle", "ssa"),
  ("subtitle", "stl"),
  ("subtitle", "subrip"),
  ("subtitle", "subviewer"),
  ("subtitle", "subviewer1"),
  ("subtitle", "text"),
  ("subtitle", "ttml"),
  ("subtitle", "vplayer"),
  ("subtitle", "webvtt"),
  ("subtitle", "xsub"),
  ("data", "bin_data"),
  ("data", "dvd_nav_packet"),
  ("data", "epg"),
  ("data", "klv"),
  ("data", "mpegts"),
  ("data", "otf"),
  ("data", "scte_35"),
  ("data", "smpte_2038"),
  ("data", "smpte_436m_anc"),
  ("data", "timed_id3"),
  ("data", "ttf"),
  ("attachment", "bin_data"),
  ("attachment", "otf"),
  ("attachment", "ttf"),
];
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
fn every_data_codec_round_trips_to_named_variant() {
  let mut n = 0usize;
  for name in vendored_of("data") {
    let c: DataCodec = name.parse().unwrap();
    assert!(
      !c.is_other(),
      "data `{name}` should parse to a named variant"
    );
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
    assert!(
      !c.is_other(),
      "attachment `{name}` should parse to a named variant"
    );
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
    assert!(
      !d.is_other(),
      "`{name}` should also be a named DataCodec variant"
    );
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
  let v: VideoCodec = "VENDOR_Codec".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "VENDOR_Codec");
  assert_eq!(VideoCodec::other("VENDOR_Codec"), v);
  assert_ne!("vendor_codec".parse::<VideoCodec>().unwrap(), v);
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
  check::<VideoCodec>(VideoCodec::ROSTER, "video", 279usize);
  check::<AudioCodec>(AudioCodec::ROSTER, "audio", 222usize);
  check::<SubtitleCodec>(SubtitleCodec::ROSTER, "subtitle", 27usize);
  check::<DataCodec>(DataCodec::ROSTER, "data", 11usize);
  check::<AttachmentCodec>(AttachmentCodec::ROSTER, "attachment", 3usize);
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
