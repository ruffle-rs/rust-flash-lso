#![type_length_limit = "1193167"]

const HEADER_VERSION: [u8; 2] = [0x00, 0xbf];
const HEADER_SIGNATURE: [u8; 10] = [0x54, 0x43, 0x53, 0x4f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
const PADDING: [u8; 1] = [0x00];

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

pub mod amf0;
pub mod amf3;
mod element_cache;
pub mod types;

use crate::amf3::AMF3Decoder;
use crate::types::{AMFVersion, Sol, SolHeader};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::number::complete::be_u32;
use nom::IResult;
use std::convert::TryInto;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "flex")]
pub mod flex;

/// The main entry point of decoding a SOL file
/// Example of use
/// ```
/// use std::fs::File;
/// use std::io::Read;
/// use flash_lso::LSODeserializer;
/// let mut x = File::open("tests/sol/AS2-Demo.sol").expect("Couldn't open file");
/// let mut data = Vec::new();
/// let _ = x.read_to_end(&mut data).expect("Unable to read file");
/// let d = LSODeserializer::default().parse_full(&data).expect("Failed to parse lso file");
/// println!("{:#?}", d);
/// ```
/// }
#[derive(Default)]
pub struct LSODeserializer {
    pub amf3_decoder: AMF3Decoder,
}

impl LSODeserializer {
    pub fn parse_header<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolHeader> {
        let (i, _) = tag(HEADER_VERSION)(i)?;
        let (i, l) = be_u32(i)?;
        let (i, _) = tag(HEADER_SIGNATURE)(i)?;

        let (i, name) = amf0::decoder::parse_string(i)?;

        let (i, _) = tag(PADDING)(i)?;
        let (i, _) = tag(PADDING)(i)?;
        let (i, _) = tag(PADDING)(i)?;

        let (i, version) = alt((tag(&[FORMAT_VERSION_AMF0]), tag(&[FORMAT_VERSION_AMF3])))(i)?;

        // This unwrap can't fail because of the alt above
        let format_version: AMFVersion = version[0].try_into().unwrap();

        Ok((
            i,
            SolHeader {
                length: l,
                name: name.to_string(),
                format_version,
            },
        ))
    }

    pub fn parse_full<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Sol> {
        let (i, header) = self.parse_header(i)?;
        match header.format_version {
            AMFVersion::AMF0 => {
                let (i, body) = amf0::decoder::parse_body(i)?;
                Ok((i, Sol { header, body }))
            }
            AMFVersion::AMF3 => {
                let (i, body) = self.amf3_decoder.parse_body(i)?;
                Ok((i, Sol { header, body }))
            }
        }
    }
}

pub mod encoder {
    use crate::types::{AMFVersion, Sol, SolHeader};
    use crate::{
        FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3, HEADER_SIGNATURE, HEADER_VERSION, PADDING,
    };
    use cookie_factory::bytes::{be_u16, be_u32};
    use cookie_factory::SerializeFn;
    use std::io::Write;

    use crate::amf0::encoder as amf0_encoder;
    use crate::amf3::encoder::AMF3Encoder;
    use cookie_factory::combinator::cond;
    use cookie_factory::combinator::slice;
    use cookie_factory::combinator::string;
    use cookie_factory::gen;
    use cookie_factory::sequence::tuple;

    #[derive(Default)]
    pub struct LSOSerializer {
        pub amf3_encoder: AMF3Encoder,
    }

    impl LSOSerializer {
        pub fn write_full<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            lso: &'b Sol,
        ) -> impl SerializeFn<W> + 'a {
            let amf0 = cond(
                lso.header.format_version == AMFVersion::AMF0,
                amf0_encoder::write_body(&lso.body),
            );
            let amf3 = cond(
                lso.header.format_version == AMFVersion::AMF3,
                self.amf3_encoder.write_body(&lso.body),
            );

            tuple((write_header(&lso.header), amf0, amf3))
        }
    }

    pub fn write_string<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((be_u16(s.len() as u16), string(s)))
    }

    pub fn write_header<'a, 'b: 'a, W: Write + 'a>(
        header: &'b SolHeader,
    ) -> impl SerializeFn<W> + 'a {
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

    pub fn write_to_bytes(lso: &Sol) -> Vec<u8> {
        let v = vec![];

        let s = LSOSerializer::default();
        let serialise = s.write_full(lso);
        let (buffer, _size) = gen(serialise, v).unwrap();
        buffer
    }
}
