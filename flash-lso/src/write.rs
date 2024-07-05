//! Handles writing of LSO files
use byteorder::{BigEndian, WriteBytesExt};
use cookie_factory::gen;
use std::io::Write;

use crate::amf3::write::AMF3Encoder;
use crate::errors::Error;
use crate::nom_utils::write_string;
use crate::types::{AMFVersion, Header, Lso};
use crate::{FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3, HEADER_SIGNATURE, HEADER_VERSION, PADDING};

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
        writer: &mut W,
        lso: &'b mut Lso,
    ) -> std::io::Result<()> {
        let (buffer, size) = if lso.header.format_version == AMFVersion::AMF0 {
            let mut buffer = vec![];
            crate::amf0::write::write_body(&mut buffer, &lso.body).unwrap();
            let length = buffer.len() as u64;
            (buffer, length)
        } else {
            let amf3 = self.amf3_encoder.write_body(&lso.body);

            let v = Vec::new();
            gen(amf3, v).unwrap()
        };

        lso.header.length = size as u32 + header_length(&lso.header) as u32;

        write_header(writer, &lso.header)?;
        writer.write_all(&buffer)?;
        Ok(())
    }
}

fn write_header<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    header: &'b Header,
) -> std::io::Result<()> {
    writer.write_all(&HEADER_VERSION)?;
    writer.write_u32::<BigEndian>(header.length)?;
    writer.write_all(&HEADER_SIGNATURE)?;
    write_string(writer, &header.name)?;
    writer.write_all(&PADDING)?;
    writer.write_all(&PADDING)?;
    writer.write_all(&PADDING)?;
    match header.format_version {
        AMFVersion::AMF0 => writer.write_all(&[FORMAT_VERSION_AMF0])?,
        AMFVersion::AMF3 => writer.write_all(&[FORMAT_VERSION_AMF3])?,
    };
    Ok(())
}

/// Get the serialized length of the header in bytes, this does not include the size of the header length field or the lso version marker
pub fn header_length(header: &Header) -> usize {
    // signature + (name size u16 + name_len) + 3*padding + amf_version_marker
    10 + (2 + header.name.len() + 3 + 1)
}

/// Write a LSO to a vec of bytes
pub fn write_to_bytes<'a>(lso: &mut Lso) -> Result<Vec<u8>, Error<'a>> {
    let mut v = vec![];

    let mut s = Writer::default();
    s.write_full(&mut v, lso)
        .map_err(|e| Error::IoError(e.to_string(), e.kind()))?;
    Ok(v)
}
