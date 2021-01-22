use std::convert::TryInto;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::number::complete::be_u32;

use crate::amf3::read::AMF3Decoder;
use crate::types::{AMFVersion, Header, Lso};
use crate::amf0;

const HEADER_VERSION: [u8; 2] = [0x00, 0xbf];
const HEADER_SIGNATURE: [u8; 10] = [0x54, 0x43, 0x53, 0x4f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
const PADDING: [u8; 1] = [0x00];

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

/// The main entry point of decoding a LSO file
/// Example of use
/// ```
/// use std::fs::File;
/// use std::io::Read;
/// use flash_lso::read::Reader;
/// let mut x = File::open("tests/sol/AS2-Demo.sol").expect("Couldn't open file");
/// let mut data = Vec::new();
/// let _ = x.read_to_end(&mut data).expect("Unable to read file");
/// let d = Reader::default().parse(&data).expect("Failed to parse lso file");
/// println!("{:#?}", d);
/// ```
/// }
#[derive(Default)]
pub struct Reader {
    #[cfg(feature = "amf3")]
    /// Handles reading Value::AMF3() wrapped types
    pub amf3_decoder: AMF3Decoder,
}

impl Reader {
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
    pub fn parse<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Lso> {
        let (i, header) = self.parse_header(i)?;
        match header.format_version {
            AMFVersion::AMF0 => {
                let (i, body) = amf0::read::parse_body(i)?;
                Ok((i, Lso { header, body }))
            }
            AMFVersion::AMF3 => {
                let (i, body) = self.amf3_decoder.parse_body(i)?;
                Ok((i, Lso { header, body }))
            }
        }
    }
}
