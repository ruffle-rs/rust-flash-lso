use std::convert::TryInto;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::number::complete::be_u32;

use crate::amf0;
use crate::amf0::read::AMF0Decoder;
use crate::amf3::read::AMF3Decoder;
use crate::errors::Error;
use crate::nom_utils::AMFResult;
use crate::types::{AMFVersion, Header, Lso};
use nom::combinator::all_consuming;

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
    /// Handles reading Amf3 data
    pub amf3_decoder: AMF3Decoder,
    
    /// Handles reading Amf0 data
    pub amf0_decoder: AMF0Decoder,
}

impl Reader {
    fn parse_header<'a>(&self, i: &'a [u8]) -> AMFResult<'a, Header> {
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

    /// Read a given buffer as an Lso
    ///
    /// Unlike parse, this function will not error if the entire slice isn't consumed
    /// and will return the data that was not parsed
    pub fn parse_incomplete<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Lso> {
        let (i, header) = self.parse_header(i)?;
        match header.format_version {
            AMFVersion::AMF0 => {
                let (i, body) = self.amf0_decoder.parse_body(i)?;
                Ok((i, Lso { header, body }))
            }

            AMFVersion::AMF3 => {
                let (i, body) = self.amf3_decoder.parse_body(i)?;
                Ok((i, Lso { header, body }))
            }
        }
    }

    /// Read a given slice as an Lso
    ///
    /// This function will return an error if the slice could not be parsed or if the entire slice
    /// was not consumed
    pub fn parse<'a>(&mut self, i: &'a [u8]) -> Result<Lso, nom::Err<Error<'a>>> {
        let (_, lso) = all_consuming(|i| self.parse_incomplete(i))(i)?;
        Ok(lso)
    }
}
