use crate::PADDING;
/// Support for encoding AMF0
use crate::types::{Element, ObjectId, Reference, Value};
use std::collections::BTreeMap;
use std::io::Write;

use crate::amf0::type_marker::TypeMarker;
use crate::nom_utils::write_string;
use crate::write::WriteExt;
use std::io::Result;

#[cfg(feature = "amf3")]
use crate::amf3::write::AMF3Encoder;

/// Context necessary for emitting AMF0 `Reference` markers.
#[derive(Default)]
pub(crate) struct WriteContext {
    object_table: BTreeMap<ObjectId, u16>,
    next_index: u16,
}

impl WriteContext {
    /// `ObjectId::INVALID` signals "never reference this back"; duplicates keep the first index.
    fn register(&mut self, id: ObjectId) {
        if id == ObjectId::INVALID || self.next_index == u16::MAX {
            return;
        }

        self.object_table.entry(id).or_insert_with(|| {
            let idx = self.next_index;

            self.next_index += 1;

            idx
        });
    }

    fn get(&self, id: ObjectId) -> Option<u16> {
        self.object_table.get(&id).copied()
    }
}

fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, type_: TypeMarker) -> Result<()> {
    writer.write_u8(type_ as u8)
}

fn write_reference_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, r: &Reference) -> Result<()> {
    write_type_marker(writer, TypeMarker::Reference)?;
    writer.write_u16(r.0)?;
    Ok(())
}

fn write_number_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: f64) -> Result<()> {
    write_type_marker(writer, TypeMarker::Number)?;
    writer.write_f64(s)?;
    Ok(())
}

fn write_bool_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: bool) -> Result<()> {
    write_type_marker(writer, TypeMarker::Boolean)?;
    writer.write_u8(u8::from(s))?;
    Ok(())
}

fn write_long_string_content<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: &'b str) -> Result<()> {
    writer.write_u32(s.len() as u32)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

fn write_long_string_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: &'b str) -> Result<()> {
    write_type_marker(writer, TypeMarker::LongString)?;
    write_long_string_content(writer, s)?;
    Ok(())
}

fn write_string_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: &'b str) -> Result<()> {
    write_type_marker(writer, TypeMarker::String)?;
    write_string(writer, s)?;
    Ok(())
}

fn write_object_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    id: ObjectId,
    o: &'b [Element],
) -> Result<()> {
    write_type_marker(writer, TypeMarker::Object)?;

    ctx.register(id);

    for element in o {
        write_element(writer, ctx, element)?;
    }

    writer.write_u16(0)?;
    write_type_marker(writer, TypeMarker::ObjectEnd)?;

    Ok(())
}

fn write_null_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W) -> Result<()> {
    write_type_marker(writer, TypeMarker::Null)
}

fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W) -> Result<()> {
    write_type_marker(writer, TypeMarker::Undefined)
}

fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    id: ObjectId,
    elements: &'b [Value],
) -> Result<()> {
    ctx.register(id);

    write_type_marker(writer, TypeMarker::StrictArray)?;
    writer.write_u32(elements.len() as u32)?;

    for element in elements {
        write_value(writer, ctx, element)?;
    }

    Ok(())
}

fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    id: ObjectId,
    date: f64,
    tz: Option<u16>,
) -> Result<()> {
    ctx.register(id);

    write_type_marker(writer, TypeMarker::Date)?;
    writer.write_f64(date)?;
    writer.write_u16(tz.unwrap_or(0))?;

    Ok(())
}

fn write_unsupported_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W) -> Result<()> {
    write_type_marker(writer, TypeMarker::Unsupported)
}

fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, content: &'b str) -> Result<()> {
    write_type_marker(writer, TypeMarker::Xml)?;
    write_long_string_content(writer, content)?;
    Ok(())
}

fn write_typed_object_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    id: ObjectId,
    name: &'b str,
    elements: &'b [Element],
) -> Result<()> {
    ctx.register(id);

    write_type_marker(writer, TypeMarker::TypedObject)?;
    write_string(writer, name)?;

    for element in elements {
        write_element(writer, ctx, element)?;
    }

    writer.write_u16(0)?;
    write_type_marker(writer, TypeMarker::ObjectEnd)?;

    Ok(())
}

fn write_dense_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    index: usize,
    element: &'b Value,
) -> Result<()> {
    let index_str = index.to_string();

    writer.write_u16(index_str.len() as u16)?;
    writer.write_all(index_str.as_bytes())?;
    write_value(writer, ctx, element)?;

    Ok(())
}

fn write_ecma_array<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    id: ObjectId,
    dense: &'b [Value],
    elements: &'b [Element],
    length: u32,
) -> Result<()> {
    ctx.register(id);

    //TODO: what is the u16 padding
    //TODO: sometimes array length is ignored (u32) sometimes its: elements.len() as u32

    write_type_marker(writer, TypeMarker::ECMAArray)?;
    writer.write_u32(length)?;

    for (idx, value) in dense.iter().enumerate() {
        write_dense_element(writer, ctx, idx, value)?
    }

    for element in elements {
        write_element(writer, ctx, element)?
    }

    writer.write_u16(0)?;
    write_type_marker(writer, TypeMarker::ObjectEnd)?;

    Ok(())
}

pub(crate) fn write_value<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    element: &'b Value,
) -> Result<()> {
    match element {
        Value::Number(n) => write_number_element(writer, *n),
        Value::Bool(b) => write_bool_element(writer, *b),
        Value::String(s) => {
            if s.len() > 65535 {
                write_long_string_element(writer, s)
            } else {
                write_string_element(writer, s)
            }
        }
        Value::Object { id, data } => {
            if let Some(class_def) = &data.class_definition {
                write_typed_object_element(writer, ctx, *id, &class_def.name, &data.elements)
            } else {
                write_object_element(writer, ctx, *id, &data.elements)
            }
        }
        Value::Null => write_null_element(writer),
        Value::Undefined => write_undefined_element(writer),
        Value::StrictArray { id, values } => write_strict_array_element(writer, ctx, *id, values),
        Value::Date { id, data } => {
            write_date_element(writer, ctx, *id, data.time, data.timezone_or_utc)
        }
        Value::Unsupported => write_unsupported_element(writer),
        Value::XML {
            value,
            is_string: _,
        } => write_xml_element(writer, value),
        Value::ECMAArray { id, data } => {
            write_ecma_array(writer, ctx, *id, &data.dense, &data.elements, data.length)
        }
        #[cfg(feature = "amf3")]
        Value::AMF3(e) => {
            write_type_marker(writer, TypeMarker::AMF3)?;
            let encoder = AMF3Encoder::default();
            encoder.write_value_element(writer, e)
        }
        Value::Reference(r) => write_reference_element(writer, r),
        Value::Amf3ObjectReference(id) => {
            if let Some(index) = ctx.get(*id) {
                write_reference_element(writer, &Reference(index))
            } else {
                // Forward reference to a value not yet written — AMF0 can't represent this.
                write_unsupported_element(writer)
            }
        }
        _ => {
            write_unsupported_element(writer) /* Not in amf0, TODO: use the amf3 embedding for every thing else */
        }
    }
}

fn write_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    element: &'b Element,
) -> Result<()> {
    write_string(writer, &element.name)?;
    write_value(writer, ctx, &element.value)?;

    Ok(())
}

fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    ctx: &mut WriteContext,
    element: &'b Element,
) -> Result<()> {
    write_element(writer, ctx, element)?;
    writer.write_all(&PADDING)?;

    Ok(())
}

pub(crate) fn write_body<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    elements: &'b [Element],
) -> Result<()> {
    let mut ctx = WriteContext::default();

    for element in elements {
        write_element_and_padding(writer, &mut ctx, element)?;
    }
    Ok(())
}

/// Serialize a single top-level `Value` with a fresh AMF0 reference table.
pub fn write_single_value<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    value: &'b Value,
) -> Result<()> {
    let mut ctx = WriteContext::default();

    write_value(writer, &mut ctx, value)
}
