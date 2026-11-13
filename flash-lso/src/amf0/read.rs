//! Support for decoding AMF0 data
use crate::amf0::type_marker::TypeMarker;
use nom::Parser;

use crate::PADDING;
#[cfg(feature = "amf3")]
use crate::amf3;
use crate::nom_utils::{AMFResult, take_str};
use crate::types::{
    ClassDefinition, DateValue, ECMAArrayObjectValue, Element, ObjectId, ObjectValue, Reference,
    Value,
};
use nom::Err;
use nom::bytes::complete::{tag, take};
use nom::combinator::{map, map_res};
use nom::error::{ErrorKind, make_error};
use nom::multi::{many_m_n, many0};
use nom::number::complete::{be_f64, be_u8, be_u16, be_u32};
use std::convert::{TryFrom, TryInto};

pub(crate) fn parse_string(i: &[u8]) -> AMFResult<'_, &str> {
    let (i, length) = be_u16(i)?;
    take_str(i, length)
}

fn parse_element_number(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, v) = be_f64(i)?;
    Ok((i, Value::Number(v)))
}

fn parse_element_bool(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, v) = be_u8(i)?;
    Ok((i, Value::Bool(v > 0)))
}

fn parse_element_string(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, v) = parse_string(i)?;
    Ok((i, Value::String(v.to_string())))
}

fn parse_long_string_internal(i: &[u8]) -> AMFResult<'_, &str> {
    let (i, length) = be_u32(i)?;
    map_res(take(length), std::str::from_utf8).parse(i)
}

fn parse_element_long_string(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, str) = parse_long_string_internal(i)?;
    Ok((i, Value::String(str.to_string())))
}

fn parse_element_xml(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, content) = parse_long_string_internal(i)?;
    Ok((
        i,
        Value::XML {
            value: content.to_string(),
            is_string: true,
        },
    ))
}

fn read_type_marker(i: &[u8]) -> AMFResult<'_, TypeMarker> {
    let (i, type_) = be_u8(i)?;
    Ok((
        i,
        TypeMarker::try_from(type_).unwrap_or(TypeMarker::Unsupported),
    ))
}

/// Handles decoding AMF0
#[derive(Default)]
pub struct AMF0Decoder {
    /// Complex values per AMF0 spec §2.11 (Object, TypedObject, ECMAArray, StrictArray, Date)
    /// used to resolve reference markers back to `ObjectId`.
    reference_table: Vec<ObjectId>,

    #[cfg(feature = "amf3")]
    amf3_decoder: amf3::read::AMF3Decoder,

    #[cfg(not(feature = "amf3"))]
    object_id: i64,
}

impl AMF0Decoder {
    fn register_complex(&mut self) -> ObjectId {
        let id = self.next_object_id();
        self.reference_table.push(id);
        id
    }

    #[cfg(feature = "amf3")]
    const fn next_object_id(&mut self) -> ObjectId {
        self.amf3_decoder.next_object_id()
    }

    #[cfg(not(feature = "amf3"))]
    const fn next_object_id(&mut self) -> ObjectId {
        let id = ObjectId(self.object_id);
        self.object_id += 1;
        id
    }

    fn parse_element_reference<'a>(&self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let (i, reference_index) = be_u16(i)?;

        let value = match self.reference_table.get(reference_index as usize).copied() {
            Some(id) => Value::Amf3ObjectReference(id),
            None => Value::Reference(Reference(reference_index)),
        };

        Ok((i, value))
    }

    fn parse_element_date<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let id = self.register_complex();

        let (i, millis) = be_f64(i)?;
        let (i, time_zone) = be_u16(i)?;

        Ok((
            i,
            Value::Date {
                id,
                data: DateValue {
                    time: millis,
                    timezone_or_utc: Some(time_zone),
                },
            },
        ))
    }

    fn parse_element_ecma_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let id = self.register_complex();
        let (i, array_length) = be_u32(i)?;

        map(
            |i| self.parse_array_element(i),
            move |elms: Vec<Element>| Value::ECMAArray {
                id,
                data: ECMAArrayObjectValue {
                    dense: Vec::new(),
                    elements: elms,
                    length: array_length,
                },
            },
        )
        .parse(i)
    }

    fn parse_element_typed_object<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let id = self.register_complex();
        let (i, name) = parse_string(i)?;

        map(
            |i| self.parse_array_element(i),
            move |elms: Vec<Element>| Value::Object {
                id,
                data: ObjectValue {
                    elements: elms,
                    class_definition: Some(ClassDefinition::default_with_name(name.to_string())),
                },
            },
        )
        .parse(i)
    }

    fn parse_element_object<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let id = self.register_complex();
        let (i, v) = self.parse_array_element(i)?;

        Ok((
            i,
            Value::Object {
                id,
                data: ObjectValue {
                    elements: v,
                    class_definition: None,
                },
            },
        ))
    }

    #[cfg(fuzzing)]
    /// For fuzzing
    pub fn fuzz_parse_element_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        self.parse_element_strict_array(i)
    }

    /// Parse an array of elements
    fn parse_element_strict_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let id = self.register_complex();
        let (i, length) = be_u32(i)?;

        let length_usize = length
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize` bytes (u8) to read this, this prevents OOM errors with v.large arrays
        if i.len() < length_usize {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        // This must parse length elements
        let (i, elements) =
            many_m_n(length_usize, length_usize, |i| self.parse_single_element(i)).parse(i)?;

        Ok((
            i,
            Value::StrictArray {
                id,
                values: elements,
            },
        ))
    }

    fn parse_array_element<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Vec<Element>> {
        let mut out = Vec::new();

        let mut i = i;
        loop {
            let (k, _) = parse_string(i)?;
            let (k, next_type) = read_type_marker(k)?;
            if next_type == TypeMarker::ObjectEnd {
                i = k;
                break;
            }

            let (j, e) = self.parse_element(i)?;
            i = j;

            out.push(e.clone());
        }

        Ok((i, out))
    }

    fn parse_element_amf3<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        #[cfg(feature = "amf3")]
        {
            let (i, x) = self.amf3_decoder.parse_single_element(i)?;
            Ok((i, Value::AMF3(Box::new(x))))
        }
        #[cfg(not(feature = "amf3"))]
        {
            Ok((i, Value::Unsupported))
        }
    }

    /// Parse a single AMF0 element
    pub fn parse_single_element<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        // Get the type of the next element
        let (i, type_) = read_type_marker(i)?;

        let (i, v) = match type_ {
            TypeMarker::Number => parse_element_number(i),
            TypeMarker::Boolean => parse_element_bool(i),
            TypeMarker::String => parse_element_string(i),
            TypeMarker::Object => {
                let (i, v) = self.parse_element_object(i)?;
                Ok((i, v))
            }
            TypeMarker::Null => Ok((i, (Value::Null))),
            TypeMarker::Undefined => Ok((i, (Value::Undefined))),
            TypeMarker::Reference => {
                let (i, v) = self.parse_element_reference(i)?;
                Ok((i, v))
            }
            TypeMarker::ECMAArray => {
                let (i, v) = self.parse_element_ecma_array(i)?;
                Ok((i, v))
            }
            TypeMarker::StrictArray => {
                let (i, v) = self.parse_element_strict_array(i)?;
                Ok((i, v))
            }
            TypeMarker::Date => self.parse_element_date(i),
            TypeMarker::LongString => parse_element_long_string(i),
            TypeMarker::Unsupported => Ok((i, (Value::Unsupported))),
            TypeMarker::Xml => parse_element_xml(i),
            TypeMarker::TypedObject => {
                let (i, v) = self.parse_element_typed_object(i)?;
                Ok((i, v))
            }
            TypeMarker::AMF3 => self.parse_element_amf3(i),
            TypeMarker::MovieClip | TypeMarker::RecordSet | TypeMarker::ObjectEnd => Err(
                Err::Error(crate::errors::Error::UnsupportedType(type_ as u8)),
            ),
        }?;

        Ok((i, v))
    }

    fn parse_element<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Element> {
        let (i, name) = parse_string(i)?;

        map(
            |i| self.parse_single_element(i),
            move |v| Element {
                name: name.to_string(),
                value: v,
            },
        )
        .parse(i)
    }

    fn parse_element_and_padding<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Element> {
        let (i, e) = self.parse_element(i)?;
        let (i, _) = tag(PADDING.as_slice())(i)?;

        Ok((i, e))
    }

    /// Parse a sequence of `PADDING` delimited `Values`
    pub fn parse_body<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Vec<Element>> {
        many0(|i| self.parse_element_and_padding(i)).parse(i)
    }
}
