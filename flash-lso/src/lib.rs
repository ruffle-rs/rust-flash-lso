#![feature(type_alias_impl_trait)]

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
use crate::amf3::AMF3Decoder;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// The main entry point of decoding a SOL file
/// Example of use
/// ```
/// use std::fs::File;
/// use std::io::Read;
/// use flash_lso::LSODeserializer;
/// let mut x = File::open("tests/sol/2.sol").expect("Couldn't open file");
/// let mut data = Vec::new();
/// let _ = x.read_to_end(&mut data).expect("Unable to read file");
/// let d = LSODeserializer::default().parse_full(&data).expect("Failed to parse lso file");
/// println!("{:#?}", d);
/// ```
/// }
#[derive(Default)]
pub struct LSODeserializer {
    amf3_decoder: AMF3Decoder
}

impl LSODeserializer {
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
                let (i, body) = self.amf3_decoder.parse_body(i)?;
                Ok((i, Sol { header, body }))
            }
            _ => Err(Err::Error(make_error(i, ErrorKind::Digit))),
        }
    }
}

pub mod encoder {
    use std::io::Write;
    use crate::types::{SolHeader, Sol};
    use cookie_factory::bytes::{be_u32, be_u16};
    use crate::{PADDING, FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3};
    use cookie_factory::SerializeFn;

    use cookie_factory::combinator::slice;
    use cookie_factory::combinator::string;
    use cookie_factory::combinator::cond;
    use crate::amf0::encoder as amf0_encoder;
    use cookie_factory::sequence::tuple;
    use cookie_factory::gen;

    pub fn write_string<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((be_u16(s.len() as u16), string(s)))
    }

    pub fn write_header<'a, 'b: 'a, W: Write + 'a>(header: &'b SolHeader) -> impl SerializeFn<W> + 'a {
        tuple((slice(header.version),
             be_u32(header.length),
             slice(header.signature),
             write_string(&header.name),
             slice(PADDING),
             slice(PADDING),
             slice(PADDING),
             cond(header.format_version == 0, slice(&[FORMAT_VERSION_AMF0])),
             cond(header.format_version == 3, slice(&[FORMAT_VERSION_AMF3])),
        ))
    }

    pub fn write_full<'a, 'b: 'a, W: Write + 'a>(lso: &'b Sol) -> impl SerializeFn<W> +'a {
       tuple((write_header(&lso.header), amf0_encoder::write_body(&lso.body)))
    }

    pub fn write_to_bytes(lso: &Sol) -> Vec<u8> {
        let v = vec![];
        let (buffer, _size) = gen(write_full(lso), v).unwrap();
        buffer
    }
}
