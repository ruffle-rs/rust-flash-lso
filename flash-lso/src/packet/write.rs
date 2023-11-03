//! Handles writing of AMF packets

use crate::amf0;
use crate::errors::Error;
use crate::packet::{Header, Message, Packet};
use crate::types::AMFVersion;

fn write_header(
    header: &Header,
    out: &mut Vec<u8>,
    exact_lengths: bool,
) -> Result<(), Error<'static>> {
    // Name
    let name_length = u16::try_from(header.name.len()).map_err(|_| Error::PacketTooLarge)?;
    out.extend(name_length.to_be_bytes());
    out.extend(header.name.as_bytes());

    // Must understand
    if header.must_understand {
        out.push(1);
    } else {
        out.push(0);
    }

    // Value
    let mut value = amf0::write::write_value(&header.value)(vec![].into())?.write;
    if exact_lengths {
        let value_length = u32::try_from(value.len()).map_err(|_| Error::PacketTooLarge)?;
        out.extend(value_length.to_be_bytes());
    } else {
        out.extend(u32::MAX.to_be_bytes());
    }
    out.append(&mut value);

    Ok(())
}

fn write_message(
    message: &Message,
    out: &mut Vec<u8>,
    exact_lengths: bool,
) -> Result<(), Error<'static>> {
    // Target URI
    let target_length =
        u16::try_from(message.target_uri.len()).map_err(|_| Error::PacketTooLarge)?;
    out.extend(target_length.to_be_bytes());
    out.extend(message.target_uri.as_bytes());

    // Response URI
    let response_length =
        u16::try_from(message.response_uri.len()).map_err(|_| Error::PacketTooLarge)?;
    out.extend(response_length.to_be_bytes());
    out.extend(message.response_uri.as_bytes());

    // Contents
    let mut contents = amf0::write::write_value(&message.contents)(vec![].into())?.write;
    if exact_lengths {
        let contents_length = u32::try_from(contents.len()).map_err(|_| Error::PacketTooLarge)?;
        out.extend(contents_length.to_be_bytes());
    } else {
        out.extend(u32::MAX.to_be_bytes());
    }
    out.append(&mut contents);

    Ok(())
}

/// Write a packet to a vec of bytes
pub fn write_to_bytes(packet: &Packet, exact_lengths: bool) -> Result<Vec<u8>, Error<'static>> {
    let mut buffer = vec![];

    // Version
    buffer.push(0);
    match packet.version {
        AMFVersion::AMF0 => buffer.push(0),
        AMFVersion::AMF3 => buffer.push(3),
    }

    // Headers
    let header_count = u16::try_from(packet.headers.len()).map_err(|_| Error::PacketTooLarge)?;
    buffer.extend(header_count.to_be_bytes());
    for header in &packet.headers {
        write_header(header, &mut buffer, exact_lengths)?;
    }

    // Messages
    let message_count = u16::try_from(packet.messages.len()).map_err(|_| Error::PacketTooLarge)?;
    buffer.extend(message_count.to_be_bytes());
    for message in &packet.messages {
        write_message(message, &mut buffer, exact_lengths)?;
    }

    Ok(buffer)
}
