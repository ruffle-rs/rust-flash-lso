//! Support for decoding AMF0 data
use crate::amf0::type_marker::TypeMarker;

#[cfg(feature = "amf3")]
use crate::amf3;
use crate::nom_utils::{take_str, AMFResult};
use crate::types::{ClassDefinition, Element, ObjectId, Reference, Value};
use crate::PADDING;
use nom::bytes::complete::{tag, take};
use nom::combinator::{map, map_res};
use nom::error::{make_error, ErrorKind};
use nom::multi::{many0, many_m_n};
use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
use nom::Err;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

pub(crate) fn parse_string(i: &[u8]) -> AMFResult<'_, &str> {
    let (i, length) = be_u16(i)?;
    take_str(i, length)
}

fn parse_element_number(i: &[u8]) -> AMFResult<'_, Rc<Value>> {
    let (i, v) = be_f64(i)?;
    Ok((i, Rc::new(Value::Number(v))))
}

fn parse_element_bool(i: &[u8]) -> AMFResult<'_, Rc<Value>> {
    let (i, v) = be_u8(i)?;
    Ok((i, Rc::new(Value::Bool(v > 0))))
}

fn parse_element_string(i: &[u8]) -> AMFResult<'_, Rc<Value>> {
    let (i, v) = parse_string(i)?;
    Ok((i, Rc::new(Value::String(v.to_string()))))
}

fn parse_element_date(i: &[u8]) -> AMFResult<'_, Rc<Value>> {
    let (i, millis) = be_f64(i)?;
    let (i, time_zone) = be_u16(i)?;

    Ok((i, Rc::new(Value::Date(millis, Some(time_zone)))))
}

fn parse_long_string_internal(i: &[u8]) -> AMFResult<'_, &str> {
    let (i, length) = be_u32(i)?;
    map_res(take(length), std::str::from_utf8)(i)
}

fn parse_element_long_string(i: &[u8]) -> AMFResult<'_, Rc<Value>> {
    let (i, str) = parse_long_string_internal(i)?;
    Ok((i, Rc::new(Value::String(str.to_string()))))
}

fn parse_element_xml(i: &[u8]) -> AMFResult<'_, Rc<Value>> {
    let (i, content) = parse_long_string_internal(i)?;
    Ok((i, Rc::new(Value::XML(content.to_string(), true))))
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
    /// Cache of previously read values, that can be referenced later
    cache: Vec<Rc<Value>>,

    #[cfg(feature = "amf3")]
    amf3_decoder: amf3::read::AMF3Decoder,
}

impl AMF0Decoder {
    fn parse_element_reference<'a>(&self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        let (i, reference_index) = be_u16(i)?;

        Ok((i, Rc::new(Value::Reference(Reference(reference_index)))))
    }

    fn parse_element_mixed_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        let (i, array_length) = be_u32(i)?;
        map(
            |i| self.parse_array_element(i),
            move |elms: Vec<Element>| Rc::new(Value::ECMAArray(Vec::new(), elms, array_length)),
        )(i)
    }

    fn parse_element_typed_object<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        let (i, name) = parse_string(i)?;

        map(
            |i| self.parse_array_element(i),
            move |elms: Vec<Element>| {
                Rc::new(Value::Object(
                    ObjectId::INVALID,
                    elms,
                    Some(ClassDefinition::default_with_name(name.to_string())),
                ))
            },
        )(i)
    }

    fn parse_element_object<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        let (i, v) = self.parse_array_element(i)?;
        Ok((i, Rc::new(Value::Object(ObjectId::INVALID, v, None))))
    }

    #[cfg(fuzzing)]
    /// For fuzzing
    pub fn fuzz_parse_element_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        self.parse_element_array(i)
    }

    /// Parse an array of elements
    fn parse_element_array<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
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

        Ok((i, Rc::new(Value::StrictArray(elements))))
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

    fn parse_element_amf3<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        #[cfg(feature = "amf3")]
        {
            let (i, x) = self.amf3_decoder.parse_single_element(i)?;
            Ok((i, Rc::new(Value::AMF3(x))))
        }
        #[cfg(not(feature = "amf3"))]
        {
            Ok((i, Rc::new(Value::Unsupported)))
        }
    }

    /// Parse a single AMF0 element
    pub fn parse_single_element<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Rc<Value>> {
        // Get the type of the next element
        let (i, type_) = read_type_marker(i)?;

        let cache_idx = self.cache.len();
        self.cache.push(Rc::new(Value::Undefined));

        let (i, v) = match type_ {
            TypeMarker::Number => parse_element_number(i),
            TypeMarker::Boolean => parse_element_bool(i),
            TypeMarker::String => parse_element_string(i),
            TypeMarker::Object => {
                let (i, v) = self.parse_element_object(i)?;
                self.cache[cache_idx] = Rc::clone(&v);
                Ok((i, v))
            }
            TypeMarker::Null => Ok((i, Rc::new(Value::Null))),
            TypeMarker::Undefined => Ok((i, Rc::new(Value::Undefined))),
            TypeMarker::Reference => {
                let (i, v) = self.parse_element_reference(i)?;
                Ok((i, v))
            }
            TypeMarker::MixedArrayStart => {
                let (i, v) = self.parse_element_mixed_array(i)?;
                self.cache[cache_idx] = Rc::clone(&v);
                Ok((i, v))
            }
            TypeMarker::Array => {
                let (i, v) = self.parse_element_array(i)?;
                self.cache[cache_idx] = Rc::clone(&v);
                Ok((i, v))
            }
            TypeMarker::Date => parse_element_date(i),
            TypeMarker::LongString => parse_element_long_string(i),
            TypeMarker::Unsupported => Ok((i, Rc::new(Value::Unsupported))),
            TypeMarker::Xml => parse_element_xml(i),
            TypeMarker::TypedObject => {
                let (i, v) = self.parse_element_typed_object(i)?;
                self.cache[cache_idx] = Rc::clone(&v);
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
        )(i)
    }

    fn parse_element_and_padding<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Element> {
        let (i, e) = self.parse_element(i)?;
        let (i, _) = tag(PADDING)(i)?;

        Ok((i, e))
    }

    /// Parse a sequence of `PADDING` delimited `Values`
    pub fn parse_body<'a>(&mut self, i: &'a [u8]) -> AMFResult<'a, Vec<Element>> {
        many0(|i| self.parse_element_and_padding(i))(i)
    }

    /// Convert the given value into a reference, if possible
    /// This reference is only valid for values sourced from this decoder and will only reference values decoded by it
    pub fn as_reference(&self, v: &Value) -> Option<Reference> {
        self.cache
            .iter()
            .position(|cv| *cv == Rc::new(v.clone()))
            .map(|r| Reference(r as _))
    }
}
