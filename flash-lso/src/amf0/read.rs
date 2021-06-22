//! Support for decoding AMF0 data
use crate::amf0::type_marker::TypeMarker;

use crate::nom_utils::{take_str, AMFResult};
use crate::types::{ClassDefinition, Element, Value};
use crate::{amf3, PADDING};
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{make_error, ErrorKind};
use nom::multi::{many0, many_m_n};
use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
use nom::take_str;
use nom::Err;

use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

pub(crate) fn parse_string(i: &[u8]) -> AMFResult<'_, &str> {
    let (i, length) = be_u16(i)?;
    take_str(i, length)
}

fn parse_element_number(i: &[u8]) -> AMFResult<'_, Value> {
    map(be_f64, Value::Number)(i)
}

fn parse_element_bool(i: &[u8]) -> AMFResult<'_, Value> {
    map(be_u8, |num: u8| Value::Bool(num > 0))(i)
}

fn parse_element_string(i: &[u8]) -> AMFResult<'_, Value> {
    map(parse_string, |s: &str| Value::String(s.to_string()))(i)
}

fn parse_element_date(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, millis) = be_f64(i)?;
    let (i, time_zone) = be_u16(i)?;

    Ok((i, Value::Date(millis, Some(time_zone))))
}

fn parse_long_string_internal(i: &[u8]) -> AMFResult<'_, &str> {
    let (i, length) = be_u32(i)?;
    take_str!(i, length)
}

fn parse_element_long_string(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, str) = parse_long_string_internal(i)?;
    Ok((i, Value::String(str.to_string())))
}

fn parse_element_xml(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, content) = parse_long_string_internal(i)?;
    Ok((i, Value::XML(content.to_string(), true)))
}

fn parse_element_amf3(i: &[u8]) -> AMFResult<'_, Value> {
    // Hopefully amf3 objects wont have references
    let (i, x) = amf3::read::AMF3Decoder::default().parse_element_object(i)?;
    Ok((i, Value::AMF3(x)))
}

fn read_type_marker(i: &[u8]) -> AMFResult<'_, TypeMarker> {
    let (i, type_) = be_u8(i)?;
    Ok((
        i,
        TypeMarker::try_from(type_).unwrap_or(TypeMarker::Unsupported),
    ))
}

/// Handles decoding AMF0
#[derive(Debug, Default)]
pub struct AMF0Decoder {
    cache: Vec<Value>,
}

impl AMF0Decoder {
    fn parse_element_reference<'a>(&self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let (i, reference_index) = be_u16(i)?;

        let val = self.cache.get(reference_index as usize).ok_or(Err::Error(
            crate::errors::Error::InvalidReference(reference_index),
        ))?;

        Ok((i, val.clone()))
    }

    fn parse_element_mixed_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let (i, array_length) = be_u32(i)?;
        map(
            |i| self.parse_array_element(i),
            move |elms: Vec<Element>| Value::ECMAArray(Vec::new(), elms, array_length),
        )(i)
    }

    fn parse_element_typed_object<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let (i, name) = parse_string(i)?;

        map(
            |i| self.parse_array_element(i),
            move |elms: Vec<Element>| {
                Value::Object(
                    elms,
                    Some(ClassDefinition::default_with_name(name.to_string())),
                )
            },
        )(i)
    }

    fn parse_element_object<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        map(
            |i| self.parse_array_element(i),
            |elms: Vec<Element>| Value::Object(elms, None),
        )(i)
    }

    fn parse_element_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
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
            many_m_n(length_usize, length_usize, |i| self.parse_single_element(i))(i)?;

        Ok((
            i,
            Value::StrictArray(elements.into_iter().map(Rc::new).collect()),
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

    fn parse_single_element<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Value> {
        let (i, type_) = read_type_marker(i)?;

        match type_ {
            TypeMarker::Number => parse_element_number(i),
            TypeMarker::Boolean => parse_element_bool(i),
            TypeMarker::String => parse_element_string(i),
            TypeMarker::Object => {
                let (i, v) = self.parse_element_object(i)?;
                self.cache.push(v.clone());
                Ok((i, v))
            }
            TypeMarker::Null => Ok((i, Value::Null)),
            TypeMarker::Undefined => Ok((i, Value::Undefined)),
            TypeMarker::Reference => self.parse_element_reference(i),
            TypeMarker::MixedArrayStart => {
                let (i, v) = self.parse_element_mixed_array(i)?;
                self.cache.push(v.clone());
                Ok((i, v))
            }
            TypeMarker::Array => {
                let (i, v) = self.parse_element_array(i)?;
                self.cache.push(v.clone());
                Ok((i, v))
            }
            TypeMarker::Date => parse_element_date(i),
            TypeMarker::LongString => parse_element_long_string(i),
            TypeMarker::Unsupported => Ok((i, Value::Unsupported)),
            TypeMarker::Xml => parse_element_xml(i),
            TypeMarker::TypedObject => {
                let (i, v) = self.parse_element_typed_object(i)?;
                self.cache.push(v.clone());
                Ok((i, v))
            }
            TypeMarker::AMF3 => parse_element_amf3(i),
            TypeMarker::MovieClip | TypeMarker::RecordSet | TypeMarker::ObjectEnd => Err(
                Err::Error(crate::errors::Error::UnsupportedType(type_ as u8)),
            ),
        }
    }

    fn parse_element<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Element> {
        let (i, name) = parse_string(i)?;

        map(
            |i| self.parse_single_element(i),
            move |v| Element {
                name: name.to_string(),
                value: Rc::new(v),
            },
        )(i)
    }

    fn parse_element_and_padding<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Element> {
        let (i, e) = self.parse_element(i)?;
        let (i, _) = tag(PADDING)(i)?;

        Ok((i, e))
    }

    pub(crate) fn parse_body<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Vec<Element>> {
        many0(|i| self.parse_element_and_padding(i))(i)
    }
}
