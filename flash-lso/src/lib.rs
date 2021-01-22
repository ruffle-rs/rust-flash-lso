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

use crate::amf3::read::AMF3Decoder;
use crate::types::{AMFVersion, Header, LSO};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::number::complete::be_u32;
use nom::IResult;
use std::convert::TryInto;

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
pub mod amf3;

//TODO: custom error types
// mod errors;

#[cfg(feature = "flex")]
/// Reading and Writing of flex types
pub mod flex;
/// Types used for representing file contents
pub mod types;

mod element_cache;
mod length;
mod nom_utils;

/// The main entry point of decoding a SOL file
/// Example of use
/// ```
/// use std::fs::File;
/// use std::io::Read;
/// use flash_lso::LSODeserializer;
/// let mut x = File::open("tests/sol/AS2-Demo.sol").expect("Couldn't open file");
/// let mut data = Vec::new();
/// let _ = x.read_to_end(&mut data).expect("Unable to read file");
/// let d = LSODeserializer::default().parse(&data).expect("Failed to parse lso file");
/// println!("{:#?}", d);
/// ```
/// }
#[derive(Default)]
pub struct LSODeserializer {
    /// Handles reading Value::AMF3() wrapped types
    pub amf3_decoder: AMF3Decoder,
}

impl LSODeserializer {
    fn parse_header<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Header> {
        let (i, _) = tag(HEADER_VERSION)(i)?;
        let (i, l) = be_u32(i)?;
        let (i, _) = tag(HEADER_SIGNATURE)(i)?;

        let (i, name) = amf0::read::parse_string(i)?;

        let (i, _) = tag(PADDING)(i)?;
        let (i, _) = tag(PADDING)(i)?;
        let (i, _) = tag(PADDING)(i)?;

        let (i, version) = alt((tag(&[FORMAT_VERSION_AMF0]), tag(&[FORMAT_VERSION_AMF3])))(i)?;

        // This unwrap can't fail because of the alt above
        let format_version: AMFVersion = version[0].try_into().unwrap();

        Ok((
            i,
            Header {
                length: l,
                name: name.to_string(),
                format_version,
            },
        ))
    }

    /// Read a given buffer as an LSO
    pub fn parse<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], LSO> {
        let (i, header) = self.parse_header(i)?;
        match header.format_version {
            AMFVersion::AMF0 => {
                let (i, body) = amf0::read::parse_body(i)?;
                Ok((i, LSO { header, body }))
            }
            AMFVersion::AMF3 => {
                let (i, body) = self.amf3_decoder.parse_body(i)?;
                Ok((i, LSO { header, body }))
            }
        }
    }
}

/// Handles encoding of LSO files
pub mod encoder {
    use crate::types::{AMFVersion, Header, LSO};
    use crate::{
        FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3, HEADER_SIGNATURE, HEADER_VERSION, PADDING,
    };
    use cookie_factory::bytes::be_u32;
    use cookie_factory::SerializeFn;
    use std::io::Write;

    use crate::amf3::write::AMF3Encoder;
    use crate::nom_utils::write_string;
    use cookie_factory::combinator::cond;
    use cookie_factory::combinator::slice;
    use cookie_factory::gen;
    use cookie_factory::sequence::tuple;

    /// Handles writing a given LSO
    #[derive(Default)]
    pub struct LSOSerializer {
        /// The encoder used for writing Value::AMF3() wrapped types
        pub amf3_encoder: AMF3Encoder,
    }

    impl LSOSerializer {
        /// Write a given LSO
        pub fn write_full<'a, 'b: 'a, W: Write + 'a>(
            &'a mut self,
            lso: &'b LSO,
        ) -> impl SerializeFn<W> + 'a {
            let amf0 = cond(
                lso.header.format_version == AMFVersion::AMF0,
                crate::amf0::write::write_body(&lso.body),
            );
            let amf3 = cond(
                lso.header.format_version == AMFVersion::AMF3,
                self.amf3_encoder.write_body(&lso.body),
            );

            tuple((write_header(&lso.header), amf0, amf3))
        }
    }

    fn write_header<'a, 'b: 'a, W: Write + 'a>(header: &'b Header) -> impl SerializeFn<W> + 'a {
        tuple((
            slice(HEADER_VERSION),
            be_u32(header.length),
            slice(HEADER_SIGNATURE),
            write_string(&header.name),
            slice(PADDING),
            slice(PADDING),
            slice(PADDING),
            cond(
                header.format_version == AMFVersion::AMF0,
                slice(&[FORMAT_VERSION_AMF0]),
            ),
            cond(
                header.format_version == AMFVersion::AMF3,
                slice(&[FORMAT_VERSION_AMF3]),
            ),
        ))
    }

    /// Write a LSO to a vec of bytes
    pub fn write_to_bytes(lso: &LSO) -> Vec<u8> {
        let v = vec![];

        let mut s = LSOSerializer::default();
        let serialise = s.write_full(lso);
        let (buffer, _size) = gen(serialise, v).unwrap();
        buffer
    }
}
