use std::convert::TryInto;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::number::complete::{be_u16, be_u32, be_u8};

use crate::amf0;
use crate::amf0::read::AMF0Decoder;
use crate::errors::Error;
use crate::nom_utils::AMFResult;
use crate::packet::{Header, Message, Packet};
use crate::types::AMFVersion;
use nom::combinator::all_consuming;
use nom::multi::length_count;

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

fn parse_header(i: &[u8]) -> AMFResult<'_, Header> {
    let (i, name) = amf0::read::parse_string(i)?;
    let (i, must_understand) = be_u8(i)?;
    let (i, _length) = be_u32(i)?;
    let (i, value) = AMF0Decoder::default().parse_single_element(i)?;

    Ok((
        i,
        Header {
            name: name.to_string(),
            must_understand: must_understand != 0,
            value,
        },
    ))
}

fn parse_message(i: &[u8]) -> AMFResult<'_, Message> {
    let (i, target_uri) = amf0::read::parse_string(i)?;
    let (i, response_uri) = amf0::read::parse_string(i)?;
    let (i, _length) = be_u32(i)?;
    let (i, contents) = AMF0Decoder::default().parse_single_element(i)?;

    Ok((
        i,
        Message {
            target_uri: target_uri.to_string(),
            response_uri: response_uri.to_string(),
            contents,
        },
    ))
}

/// Read a given buffer as a packet
///
/// Unlike parse, this function will not error if the entire slice isn't consumed
/// and will return the data that was not parsed
pub fn parse_incomplete(i: &[u8]) -> AMFResult<'_, Packet> {
    let (i, _) = tag(&[0u8])(i)?;
    let (i, version) = alt((tag(&[FORMAT_VERSION_AMF0]), tag(&[FORMAT_VERSION_AMF3])))(i)?;
    // This unwrap can't fail because of the alt above
    let version: AMFVersion = version[0].try_into().unwrap();

    let (i, headers) = length_count(be_u16, parse_header)(i)?;
    let (i, messages) = length_count(be_u16, parse_message)(i)?;

    Ok((
        i,
        Packet {
            version,
            headers,
            messages,
        },
    ))
}

/// Read a given slice as a packet
///
/// This function will return an error if the slice could not be parsed or if the entire slice
/// was not consumed
pub fn parse(i: &[u8]) -> Result<Packet, nom::Err<Error<'_>>> {
    let (_, packet) = all_consuming(|i| parse_incomplete(i))(i)?;
    Ok(packet)
}
