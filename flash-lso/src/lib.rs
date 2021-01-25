//! Library for reading and writing the Adobe Flash Local Shared Object (LSO) file format and the contained AMF0/AMF3 data

#![warn(
    anonymous_parameters,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences,
    missing_docs
)]











const HEADER_VERSION: [u8; 2] = [0x00, 0xbf];
const HEADER_SIGNATURE: [u8; 10] = [0x54, 0x43, 0x53, 0x4f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
const PADDING: [u8; 1] = [0x00];

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// Reading and Writing of the AMF0 file format
pub mod amf0;
/// Reading and Writing of the AMF3 file format
#[cfg(feature = "amf3")]
pub mod amf3;

pub mod errors;
mod nom_utils;
pub mod read;
/// Types used for representing file contents
pub mod types;
pub mod write;

pub mod extra;
