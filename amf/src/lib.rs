const HEADER_VERSION: [u8; 2] = [0x00, 0xbf];
const HEADER_SIGNATURE: [u8; 10] = [0x54, 0x43, 0x53, 0x4f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
const PADDING: [u8; 1] = [0x00];

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

pub mod amf0;
pub mod amf3;
pub mod types;

use crate::types::{Sol, SolHeader};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{make_error, ErrorKind};
use nom::number::complete::be_u32;
use nom::Err;
use nom::IResult;
use std::convert::TryInto;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// The main entry point of decoding a SOL file
/// Example of use
/// ```
/// use std::fs::File;
/// use std::io::Read;
/// use amf::SolDeserializer;
/// fn main() {
///     let mut x = File::open(path).expect("Couldn't open file");
///     let mut data = Vec::new();
///     let _ = x.read_to_end(&mut data).expect("Unable to read file");
///     let d = SolDeserializer::default().parse_full(&data).expect("Failed to parse sol file");
///     println!("{:#?}", d);
/// }
/// ```
/// }
#[derive(Default)]
pub struct SolDeserializer {}

impl SolDeserializer {
    pub fn parse_version<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], [u8; 2]> {
        map(tag(HEADER_VERSION), |v: &[u8]| v.try_into().unwrap())(i)
    }

    pub fn parse_signature<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], [u8; 10]> {
        map(tag(HEADER_SIGNATURE), |sig: &[u8]| sig.try_into().unwrap())(i)
    }

    pub fn parse_header<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolHeader> {
        let (i, v) = self.parse_version(i)?;
        let (i, l) = be_u32(i)?;
        let (i, sig) = self.parse_signature(i)?;

        let (i, name) = amf0::parse_string(i)?;

        let (i, _) = tag(PADDING)(i)?;
        let (i, _) = tag(PADDING)(i)?;
        let (i, _) = tag(PADDING)(i)?;

        let (i, format_version) = map(
            alt((tag(&[FORMAT_VERSION_AMF0]), tag(&[FORMAT_VERSION_AMF3]))),
            |v: &[u8]| v[0],
        )(i)?;

        Ok((
            i,
            SolHeader {
                version: v,
                length: l,
                signature: sig,
                name: name.to_string(),
                format_version,
            },
        ))
    }

    pub fn parse_full<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Sol> {
        let (i, header) = self.parse_header(i)?;
        match header.format_version {
            FORMAT_VERSION_AMF0 => {
                let (i, body) = amf0::parse_body(i)?;
                Ok((i, Sol { header, body }))
            }
            FORMAT_VERSION_AMF3 => {
                let decoder = amf3::AMF3Decoder::default();
                let (i, body) = decoder.parse_body(i)?;
                Ok((i, Sol { header, body }))
            }
            _ => Err(Err::Error(make_error(i, ErrorKind::Digit))),
        }
    }
}
