// Cluster B — closed FFmpeg-coded enums w/ lossless `from_u32`, colour /
// pixel-format / frame geometry / disposition structs, frame coded enums.
//
// Owned types (verify each before writing):
//   COLOUR enums via `from_u32`:
//     - color::{Matrix, Primaries, Transfer, DynamicRange, ChromaLocation,
//                DcpTargetGamut}
//   PIXEL FORMAT:
//     - pixel_format::PixelFormat (from_u32)
//   FRAME coded enums via `from_u32`:
//     - frame::{Rotation, FieldOrder, StereoMode}
//   SUBTITLE / AUDIO / DISPOSITION coded enums via `from_u32`:
//     - subtitle::TrackOrigin
//     - audio::BitRateMode
//     - disposition::TrackDisposition
//   COLOUR structs (via public `new`):
//     - color::{Info, ContentLightLevel, ChromaCoord, MasteringDisplay,
//                HdrStaticMetadata, DolbyVisionConfig}
//   FRAME structs (via public `new`, watch NonZeroU32 for Rational denom):
//     - frame::{Dimensions, Rect, Rational, SampleAspectRatio, FrameRate}
//
// Use `super::arb_via_code!(Matrix, Primaries, ...)` for the coded-enum batch
// and hand-write each struct's impl via its constructor.
