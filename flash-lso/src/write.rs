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

/// Handles writing a given LSO
#[derive(Default)]
pub struct Writer {
    #[cfg(feature = "amf3")]
    /// The encoder used for writing Value::AMF3() wrapped types
    pub amf3_encoder: AMF3Encoder,
}

impl Writer {
    /// Write a given LSO
    pub fn write_full<'a, 'b: 'a, W: Write + 'a>(
        &'a mut self,
        lso: &'b Lso,
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
pub fn write_to_bytes(lso: &Lso) -> Vec<u8> {
    let v = vec![];

    let mut s = Writer::default();
    let serialise = s.write_full(lso);
    let (buffer, _size) = gen(serialise, v).unwrap();
    buffer
}
