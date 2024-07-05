/// Support for encoding AMF0
use crate::types::{Element, Reference, Value};
use crate::PADDING;
use std::io::Write;

use crate::amf0::type_marker::TypeMarker;
use crate::amf3::write::AMF3Encoder;
use crate::nom_utils::write_string;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Result;
use std::ops::Deref;
use std::rc::Rc;

fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, type_: TypeMarker) -> Result<()> {
    writer.write_u8(type_ as u8)
}

fn write_reference_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, r: &Reference) -> Result<()> {
    write_type_marker(writer, TypeMarker::Reference)?;
    writer.write_u16::<BigEndian>(r.0)?;
    Ok(())
}

fn write_number_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: f64) -> Result<()> {
    write_type_marker(writer, TypeMarker::Number)?;
    writer.write_f64::<BigEndian>(s)?;
    Ok(())
}

fn write_bool_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: bool) -> Result<()> {
    write_type_marker(writer, TypeMarker::Boolean)?;
    writer.write_u8(u8::from(s))?;
    Ok(())
}

fn write_long_string_content<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, s: &'b str) -> Result<()> {
    writer.write_u32::<BigEndian>(s.len() as u32)?;
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

fn write_object_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, o: &'b [Element]) -> Result<()> {
    write_type_marker(writer, TypeMarker::Object)?;
    for element in o {
        write_element(writer, element)?;
    }
    writer.write_u16::<BigEndian>(0)?;
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
    elements: &'b [Rc<Value>],
) -> Result<()> {
    write_type_marker(writer, TypeMarker::Array)?;
    writer.write_u32::<BigEndian>(elements.len() as u32)?;
    for element in elements {
        write_value(writer, element)?;
    }
    Ok(())
}

fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    date: f64,
    tz: Option<u16>,
) -> Result<()> {
    write_type_marker(writer, TypeMarker::Date)?;
    writer.write_f64::<BigEndian>(date)?;
    writer.write_u16::<BigEndian>(tz.unwrap_or(0))?;
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
    name: &'b str,
    elements: &'b [Element],
) -> Result<()> {
    write_type_marker(writer, TypeMarker::TypedObject)?;
    write_string(writer, name)?;
    for element in elements {
        write_element(writer, element)?;
    }
    writer.write_u16::<BigEndian>(0)?;
    write_type_marker(writer, TypeMarker::ObjectEnd)?;
    Ok(())
}

fn write_dense_element<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    index: usize,
    element: &'b Rc<Value>,
) -> Result<()> {
    let index_str = index.to_string();

    writer.write_u16::<BigEndian>(index_str.len() as u16)?;
    writer.write_all(index_str.as_bytes())?;
    write_value(writer, element)?;

    Ok(())
}

fn write_mixed_array<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    dense: &'b [Rc<Value>],
    elements: &'b [Element],
    length: u32,
) -> Result<()> {
    //TODO: what is the u16 padding
    //TODO: sometimes array length is ignored (u32) sometimes its: elements.len() as u32

    write_type_marker(writer, TypeMarker::MixedArrayStart)?;
    writer.write_u32::<BigEndian>(length)?;
    for (idx, value) in dense.iter().enumerate() {
        write_dense_element(writer, idx, value)?
    }
    for element in elements {
        write_element(writer, element)?
    }
    writer.write_u16::<BigEndian>(0)?;
    write_type_marker(writer, TypeMarker::ObjectEnd)?;
    Ok(())
}

pub(crate) fn write_value<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    element: &'b Rc<Value>,
) -> Result<()> {
    match element.deref() {
        Value::Number(n) => write_number_element(writer, *n),
        Value::Bool(b) => write_bool_element(writer, *b),
        Value::String(s) => {
            if s.len() > 65535 {
                write_long_string_element(writer, s)
            } else {
                write_string_element(writer, s)
            }
        }
        Value::Object(elements, class_def) => {
            if let Some(class_def) = class_def {
                write_typed_object_element(writer, &class_def.name, elements)
            } else {
                write_object_element(writer, elements)
            }
        }
        Value::Null => write_null_element(writer),
        Value::Undefined => write_undefined_element(writer),
        Value::StrictArray(a) => write_strict_array_element(writer, a.as_slice()),
        Value::Date(d, tz) => write_date_element(writer, *d, *tz),
        Value::Unsupported => write_unsupported_element(writer),
        Value::XML(x, _string) => write_xml_element(writer, x),
        Value::ECMAArray(dense, elems, elems_length) => {
            write_mixed_array(writer, dense, elems, *elems_length)
        }
        Value::AMF3(e) => {
            write_type_marker(writer, TypeMarker::AMF3)?;
            let encoder = AMF3Encoder::default();
            encoder.write_value_element(writer, e)
        }
        Value::Reference(r) => write_reference_element(writer, r),
        _ => {
            write_unsupported_element(writer) /* Not in amf0, TODO: use the amf3 embedding for every thing else */
        }
    }
}

fn write_element<'a, 'b: 'a, W: Write + 'a>(writer: &mut W, element: &'b Element) -> Result<()> {
    write_string(writer, &element.name)?;
    write_value(writer, &element.value)?;
    Ok(())
}

fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    element: &'b Element,
) -> Result<()> {
    write_element(writer, element)?;
    writer.write_all(&PADDING)?;
    Ok(())
}

pub(crate) fn write_body<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    elements: &'b [Element],
) -> Result<()> {
    for element in elements {
        write_element_and_padding(writer, element)?;
    }
    Ok(())
}
