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

fn parse_element_object(i: &[u8]) -> AMFResult<'_, Value> {
    map(parse_array_element, |elms: Vec<Element>| {
        Value::Object(elms, None)
    })(i)
}

fn parse_element_movie_clip(i: &[u8]) -> AMFResult<'_, Value> {
    // Reserved but unsupported
    Err(Err::Error(make_error(i, ErrorKind::Tag)))
}

#[allow(clippy::let_and_return)]
fn parse_element_mixed_array(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, array_length) = be_u32(i)?;
    // this `let x = ...` fixes a borrow error on array_length
    let x = map(parse_array_element, |elms: Vec<Element>| {
        Value::ECMAArray(Vec::new(), elms, array_length)
    })(i);

    x
}

fn parse_element_reference(i: &[u8]) -> AMFResult<'_, Value> {
    // References arent supported
    Err(Err::Error(make_error(i, ErrorKind::Tag)))
}

fn parse_element_array(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, length) = be_u32(i)?;

    let length_usize = length
        .try_into()
        .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

    // There must be at least `length_usize` bytes (u8) to read this, this prevents OOM errors with v.large arrays
    if i.len() < length_usize {
        return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
    }

    // This must parse length elements
    let (i, elements) = many_m_n(length_usize, length_usize, parse_single_element)(i)?;

    Ok((
        i,
        Value::StrictArray(elements.into_iter().map(Rc::new).collect()),
    ))
}

fn parse_element_date(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, millis) = be_f64(i)?;
    let (i, time_zone) = be_u16(i)?;

    Ok((i, Value::Date(millis, Some(time_zone))))
}

fn parse_element_long_string(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, length) = be_u32(i)?;
    let (i, str) = take_str!(i, length)?;

    Ok((i, Value::String(str.to_string())))
}

fn parse_element_record_set(i: &[u8]) -> AMFResult<'_, Value> {
    // Unsupported
    Err(Err::Error(make_error(i, ErrorKind::Tag)))
}

fn parse_element_xml(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, content) = parse_element_long_string(i)?;
    if let Value::String(content_string) = content {
        Ok((i, Value::XML(content_string, true)))
    } else {
        // Will never happen
        Err(Err::Error(make_error(i, ErrorKind::Digit)))
    }
}

#[allow(clippy::let_and_return)]
fn parse_element_typed_object(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, name) = parse_string(i)?;

    let x = map(parse_array_element, |elms: Vec<Element>| {
        Value::Object(
            elms,
            Some(ClassDefinition::default_with_name(name.to_string())),
        )
    })(i);
    x
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

pub fn parse_single_element(i: &[u8]) -> AMFResult<'_, Value> {
    let (i, type_) = read_type_marker(i)?;

    match type_ {
        TypeMarker::Number => parse_element_number(i),
        TypeMarker::Boolean => parse_element_bool(i),
        TypeMarker::String => parse_element_string(i),
        TypeMarker::Object => parse_element_object(i),
        TypeMarker::MovieClip => parse_element_movie_clip(i),
        TypeMarker::Null => Ok((i, Value::Null)),
        TypeMarker::Undefined => Ok((i, Value::Undefined)),
        TypeMarker::Reference => parse_element_reference(i),
        TypeMarker::MixedArrayStart => parse_element_mixed_array(i),
        TypeMarker::Array => parse_element_array(i),
        TypeMarker::Date => parse_element_date(i),
        TypeMarker::LongString => parse_element_long_string(i),
        TypeMarker::Unsupported => Ok((i, Value::Unsupported)),
        TypeMarker::RecordSet => parse_element_record_set(i),
        TypeMarker::XML => parse_element_xml(i),
        TypeMarker::TypedObject => parse_element_typed_object(i),
        TypeMarker::AMF3 => parse_element_amf3(i),
        TypeMarker::ObjectEnd => Err(Err::Error(make_error(i, ErrorKind::Digit))),
    }
}

fn parse_element(i: &[u8]) -> AMFResult<'_, Element> {
    let (i, name) = parse_string(i)?;

    map(parse_single_element, move |v| Element {
        name: name.to_string(),
        value: Rc::new(v),
    })(i)
}

fn parse_element_and_padding(i: &[u8]) -> AMFResult<'_, Element> {
    let (i, e) = parse_element(i)?;
    let (i, _) = tag(PADDING)(i)?;

    Ok((i, e))
}

//TODO: can this be done better somehow??
fn parse_array_element(i: &[u8]) -> AMFResult<'_, Vec<Element>> {
    let mut out = Vec::new();

    let mut i = i;
    loop {
        let (k, _) = parse_string(i)?;
        let (k, next_type) = read_type_marker(k)?;
        if next_type == TypeMarker::ObjectEnd {
            i = k;
            break;
        }

        let (j, e) = parse_element(i)?;
        i = j;

        out.push(e.clone());
    }

    Ok((i, out))
}

pub(crate) fn parse_body(i: &[u8]) -> AMFResult<'_, Vec<Element>> {
    many0(parse_element_and_padding)(i)
}
