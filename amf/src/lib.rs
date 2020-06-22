#![warn(
clippy::all,
clippy::restriction,
// clippy::pedantic,
// clippy::nursery,
// clippy::cargo,
)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::unimplemented)]
#![allow(clippy::unwrap)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::implicit_return, clippy::missing_docs_in_private_items)]

const HEADER_VERSION: [u8; 2] = [0x00, 0xbf];
const HEADER_SIGNATURE: [u8; 10] = [0x54, 0x43, 0x53, 0x4f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
const PADDING: [u8; 1] = [0x00];

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

pub mod amf0;
pub mod amf3;
pub mod types;

use crate::types::Sol;
use nom::IResult;
use nom::error::{make_error, ErrorKind};
//TODO: look at do_parse!

#[macro_use]
extern crate lazy_static;

pub fn parse_full(i: &[u8]) -> IResult<&[u8], Sol> {
    let (i, header) = amf0::parse_header(i)?;
    match header.format_version {
        FORMAT_VERSION_AMF0 => {
            let (i, body) = amf0::parse_body(i)?;
            Ok((i, Sol { header, body }))
        }
        FORMAT_VERSION_AMF3 => {
            let (i, body) = amf3::parse_body(i)?;
            Ok((i, Sol { header, body }))
        }
        _ => unimplemented!(),
    }
}
#[cfg(test)]
mod tests {}
