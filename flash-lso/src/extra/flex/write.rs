//! Handles encoding of flex types

use crate::amf3::custom_encoder::CustomEncoder;
use crate::amf3::write::AMF3Encoder;
use crate::extra::flex::{
    BODY_FLAG, CLIENT_ID_BYTES_FLAG, CLIENT_ID_FLAG, CORRELATION_ID_BYTES_FLAG,
    CORRELATION_ID_FLAG, DESTINATION_ID_FLAG, HEADERS_FLAG, MESSAGE_ID_BYTES_FLAG, MESSAGE_ID_FLAG,
    NEXT_FLAG, OPERATION_FLAG, TIMESTAMP_FLAG, TTL_FLAG,
};
use crate::types::{ClassDefinition, Element};
use crate::write::WriteExt;
use std::io::Write;

struct ArrayCollection;

impl CustomEncoder for ArrayCollection {
    fn encode(
        &self,
        elements: &[Element],
        _class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        self.do_encode(&mut v, elements, encoder).unwrap();
        v
    }
}

impl ArrayCollection {
    fn do_encode<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        elements: &'b [Element],
        encoder: &'a AMF3Encoder,
    ) -> std::io::Result<()> {
        let data = elements.first().unwrap();
        encoder.write_value_element(writer, &data.value)
    }
}

struct ObjectProxy;

impl CustomEncoder for ObjectProxy {
    fn encode(
        &self,
        elements: &[Element],
        _class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        self.do_encode(&mut v, elements, encoder).unwrap();
        v
    }
}

impl ObjectProxy {
    fn do_encode<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        elements: &'b [Element],
        encoder: &'a AMF3Encoder,
    ) -> std::io::Result<()> {
        let data = elements.first().unwrap();
        encoder.write_value_element(writer, &data.value)
    }
}

fn write_flags<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, flags: &'a [u8]) -> std::io::Result<()> {
    for (index, flag) in flags.iter().enumerate() {
        if index == flags.len() {
            writer.write_u8(*flag & !NEXT_FLAG)?;
        } else {
            writer.write_u8(*flag | NEXT_FLAG)?;
        }
    }
    Ok(())
}

struct AbstractMessage;

impl CustomEncoder for AbstractMessage {
    fn encode(
        &self,
        elements: &[Element],
        _class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        write_abstract_message(&mut v, elements, encoder).unwrap();
        v
    }
}

fn write_abstract_message<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    elements: &'b [Element],
    encoder: &'a AMF3Encoder,
) -> std::io::Result<()> {
    let mut flags = Vec::new();
    let mut new_elements = Vec::new();
    {
        let mut flag = 0;

        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "body")
            .map(|e| e.value.clone())
        {
            flag |= BODY_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "client_id")
            .map(|e| e.value.clone())
        {
            flag |= CLIENT_ID_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "destination")
            .map(|e| e.value.clone())
        {
            flag |= DESTINATION_ID_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "headers")
            .map(|e| e.value.clone())
        {
            flag |= HEADERS_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "message_id")
            .map(|e| e.value.clone())
        {
            flag |= MESSAGE_ID_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "timestamp")
            .map(|e| e.value.clone())
        {
            flag |= TIMESTAMP_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "ttl")
            .map(|e| e.value.clone())
        {
            flag |= TTL_FLAG;
            new_elements.push(v);
        }

        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "children_1")
            .map(|e| e.value.clone())
        {
            flag |= 0b0100_0000;
            new_elements.push(v);
        }

        flags.push(flag);
    }
    {
        let mut flag = 0;

        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "client_id_bytes")
            .map(|e| e.value.clone())
        {
            flag |= CLIENT_ID_BYTES_FLAG;
            new_elements.push(v);
        }
        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "message_id_bytes")
            .map(|e| e.value.clone())
        {
            flag |= MESSAGE_ID_BYTES_FLAG;
            new_elements.push(v);
        }

        for n in 2..7 {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_{n}"))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            }
        }

        flags.push(flag);
    }

    {
        let mut n = 0;
        let mut base = 8;
        let mut flag = 0;
        loop {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_{}", n + base))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            } else {
                if flag != 0 {
                    flags.push(flag);
                }
                break;
            }

            n += 1;
            if n > 7 {
                n = 0;
                base += 7;
                flags.push(flag);
                flag = 0;
            }
        }
    }

    write_flags(writer, &flags)?;
    for v in new_elements {
        encoder.write_value_element(writer, &v)?;
    }

    Ok(())
}

struct AsyncMessage;

impl CustomEncoder for AsyncMessage {
    fn encode(
        &self,
        elements: &[Element],
        _class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        write_async_message(&mut v, elements, encoder).unwrap();
        v
    }
}

fn write_async_message<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    elements: &'b [Element],
    encoder: &'a AMF3Encoder,
) -> std::io::Result<()> {
    let mut flags = Vec::new();
    let mut new_elements = Vec::new();
    {
        let mut flag = 0;

        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "correlation_id")
            .map(|e| e.value.clone())
        {
            flag |= CORRELATION_ID_FLAG;
            new_elements.push(v);
        }

        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "correlation_id_bytes")
            .map(|e| e.value.clone())
        {
            flag |= CORRELATION_ID_BYTES_FLAG;
            new_elements.push(v);
        }

        for n in 2..7 {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_async_{n}"))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            }
        }

        flags.push(flag);
    }
    {
        let mut n = 0;
        let mut base = 7;
        let mut flag = 0;
        loop {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_async_{}", n + base))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            } else {
                if flag != 0 {
                    flags.push(flag);
                }
                break;
            }

            n += 1;
            if n > 7 {
                n = 0;
                base += 7;
                flags.push(flag);
                flag = 0;
            }
        }
    }

    write_abstract_message(writer, elements, encoder)?;
    write_flags(writer, &flags)?;
    for v in new_elements {
        encoder.write_value_element(writer, &v)?;
    }

    Ok(())
}

struct AcknowledgeMessage;

impl CustomEncoder for AcknowledgeMessage {
    fn encode(
        &self,
        elements: &[Element],
        _class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        write_acknowledge_message(&mut v, elements, encoder).unwrap();
        v
    }
}

fn write_acknowledge_message<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    elements: &'b [Element],
    encoder: &'a AMF3Encoder,
) -> std::io::Result<()> {
    let mut flags = Vec::new();
    let mut new_elements = Vec::new();
    {
        let mut n = 0;
        let mut base = 0;
        let mut flag = 0;
        loop {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_acknowledge_{}", n + base))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            } else {
                if flag != 0 {
                    flags.push(flag);
                }
                break;
            }

            n += 1;
            if n > 7 {
                n = 0;
                base += 7;
                flags.push(flag);
                flag = 0;
            }
        }
    }

    write_async_message(writer, elements, encoder)?;
    write_flags(writer, &flags)?;
    for v in new_elements {
        encoder.write_value_element(writer, &v)?;
    }
    Ok(())
}

struct CommandMessage;

impl CustomEncoder for CommandMessage {
    fn encode(
        &self,
        elements: &[Element],
        _class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        write_command_message(&mut v, elements, encoder).unwrap();
        v
    }
}

fn write_command_message<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    elements: &'b [Element],
    encoder: &'a AMF3Encoder,
) -> std::io::Result<()> {
    let mut flags = Vec::new();
    let mut new_elements = Vec::new();
    {
        let mut flag = 0;

        if let Some(v) = elements
            .iter()
            .find(|e| e.name == "operation")
            .map(|e| e.value.clone())
        {
            flag |= OPERATION_FLAG;
            new_elements.push(v);
        }

        for n in 1..7 {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_command_{n}"))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            }
        }

        flags.push(flag);
    }
    {
        let mut n = 0;
        let mut base = 8;
        let mut flag = 0;
        loop {
            if let Some(v) = elements
                .iter()
                .find(|e| e.name == format!("children_command_{}", n + base))
                .map(|e| e.value.clone())
            {
                flag |= 0b1 << n;
                new_elements.push(v);
            } else {
                if flag != 0 {
                    flags.push(flag);
                }
                break;
            }

            n += 1;
            if n > 7 {
                n = 0;
                base += 7;
                flags.push(flag);
                flag = 0;
            }
        }
    }

    write_async_message(writer, elements, encoder)?;
    write_flags(writer, &flags)?;
    for v in new_elements {
        encoder.write_value_element(writer, &v)?;
    }
    Ok(())
}

/// Register the flex encoders into the given AMF3Encoder
#[inline]
pub fn register_encoders(encoder: &mut AMF3Encoder) {
    encoder.external_encoders.insert(
        "flex.messaging.io.ArrayCollection".to_string(),
        Box::new(ArrayCollection {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.ArrayList".to_string(),
        Box::new(ArrayCollection {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.AbstractMessage".to_string(),
        Box::new(AbstractMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.AsyncMessage".to_string(),
        Box::new(AsyncMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.AsyncMessageExt".to_string(),
        Box::new(AsyncMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.AcknowledgeMessage".to_string(),
        Box::new(AcknowledgeMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.AcknowledgeMessageExt".to_string(),
        Box::new(AcknowledgeMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.CommandMessage".to_string(),
        Box::new(CommandMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.CommandMessageExt".to_string(),
        Box::new(CommandMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.ErrorMessage".to_string(),
        Box::new(AcknowledgeMessage {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.ObjectProxy".to_string(),
        Box::new(ObjectProxy {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.ManagedObjectProxy".to_string(),
        Box::new(ObjectProxy {}),
    );

    encoder.external_encoders.insert(
        "flex.messaging.io.SerializationProxy".to_string(),
        Box::new(ObjectProxy {}),
    );
}
