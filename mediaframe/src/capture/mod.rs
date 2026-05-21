//! EXIF / capture-metadata vocabulary — capture device, geographic
//! location (with ISO-6709 parse/format), and (future) capture-time,
//! lens, exposure (ISO/aperture/shutter).
//!
//! Requires the `alloc` feature (`std` implies it) because the
//! constituent types lean on `SmolStr` and `std::string::String` for
//! their text surface.

pub mod device;
pub mod geo;

pub use device::Device;
pub use geo::{GeoLocation, GeoLocationError};
