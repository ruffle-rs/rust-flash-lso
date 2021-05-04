//! Handles writing of LSO files
use std::io::Write;

use cookie_factory::bytes::be_u32;
use cookie_factory::combinator::cond;
use cookie_factory::combinator::slice;
use cookie_factory::gen;
use cookie_factory::sequence::tuple;
use cookie_factory::SerializeFn;

use crate::amf3::write::AMF3Encoder;
use crate::nom_utils::write_string;
use crate::types::{AMFVersion, Header, Lso};
use crate::{FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3, HEADER_SIGNATURE, HEADER_VERSION, PADDING};
use crate::errors::Error;

/// Handles writing a given LSO
#[derive(Default)]
pub struct Writer {
    /// The encoder used for writing Value::AMF3() wrapped types
    pub amf3_encoder: AMF3Encoder,
}

impl Writer {
    /// Write a given LSO
    pub fn write_full<'a, 'b: 'a, W: Write + 'a>(
        &'a mut self,
        lso: &'b mut Lso,
    ) -> impl SerializeFn<W> + 'a {
        let amf0 = cond(
            lso.header.format_version == AMFVersion::AMF0,
            crate::amf0::write::write_body(&lso.body),
        );
        let amf3 = cond(
            lso.header.format_version == AMFVersion::AMF3,
            self.amf3_encoder.write_body(&lso.body),
        );

        let v = Vec::new();
        let serialise = tuple((amf0, amf3));
        let (buffer, size) = gen(serialise, v).unwrap();

        lso.header.length = size as u32 + header_length(&lso.header) as u32;

        tuple((write_header(&lso.header), slice(buffer)))
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

/// Get the serialized length of the header in bytes, this does not include the size of the header length field or the lso version marker
pub fn header_length(header: &Header) -> usize {
    // signature + (name size u16 + name_len) + 3*padding + amf_version_marker
    10 + (2 + header.name.len() + 3 + 1)
}

/// Write a LSO to a vec of bytes
pub fn write_to_bytes<'a>(lso: &mut Lso) -> Result<Vec<u8>, Error<'a>> {
    let v = vec![];

    let mut s = Writer::default();
    let serialise = s.write_full(lso);
    let (buffer, _) = gen(serialise, v)?;
    Ok(buffer)
}
