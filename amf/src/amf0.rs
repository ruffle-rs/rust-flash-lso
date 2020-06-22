use nom::bytes::complete::tag;
use nom::take_str;

const TYPE_NUMBER: u8 = 0;
const TYPE_BOOL: u8 = 1;
const TYPE_STRING: u8 = 2;
const TYPE_OBJECT: u8 = 3;
const TYPE_MOVIE_CLIP: u8 = 4;
const TYPE_NULL: u8 = 5;
const TYPE_UNDEFINED: u8 = 6;
const TYPE_REFERENCE: u8 = 7;
const TYPE_MIXED_ARRAY_START: u8 = 8;
const TYPE_OBJECT_END: u8 = 9;
const TYPE_ARRAY: u8 = 10;
const TYPE_DATE: u8 = 11;
const TYPE_LONG_STRING: u8 = 12;
const TYPE_UNSUPPORTED: u8 = 13;
const TYPE_RECORD_SET: u8 = 14;
const TYPE_XML: u8 = 15;
const TYPE_TYPED_OBJECT: u8 = 16;
const TYPE_AMF3: u8 = 17;

pub fn parse_version(i: &[u8]) -> IResult<&[u8], [u8; 2]> {
    map(tag(HEADER_VERSION), |v: &[u8]| v.try_into().unwrap())(i)
}

pub fn parse_length(i: &[u8]) -> IResult<&[u8], u32> {
    be_u32(i)
}

pub fn parse_signature(i: &[u8]) -> IResult<&[u8], [u8; 10]> {
    map(tag(HEADER_SIGNATURE), |sig: &[u8]| sig.try_into().unwrap())(i)
}

pub fn parse_string(i: &[u8]) -> IResult<&[u8], &str> {
    let (i, length) = be_u16(i)?;
    take_str!(i, length)
}

pub(crate) fn parse_padding(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(PADDING)(i)
}

pub fn parse_header(i: &[u8]) -> IResult<&[u8], SolHeader> {
    let (i, v) = parse_version(i)?;
    let (i, l) = parse_length(i)?;
    let (i, sig) = parse_signature(i)?;

    let (i, name) = parse_string(i)?;

    let (i, _) = parse_padding(i)?;
    let (i, _) = parse_padding(i)?;
    let (i, _) = parse_padding(i)?;

    let (i, format_version) = map(
        alt((tag(&[FORMAT_VERSION_AMF0]), tag(&[FORMAT_VERSION_AMF3]))),
        |v: &[u8]| v[0],
    )(i)?;

    Ok((
        i,
        SolHeader {
            version: v,
            length: l,
            signature: sig,
            name: name.to_string(),
            format_version,
        },
    ))
}

pub fn parse_element_number(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(be_f64, SolValue::Number)(i)
}

pub fn parse_element_bool(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(be_u8, |num: u8| SolValue::Bool(num > 0))(i)
}

pub fn parse_element_string(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(parse_string, |s: &str| SolValue::String(s.to_string()))(i)
}

fn parse_element_object(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(parse_array_element, |elms: Vec<SolElement>| {
        SolValue::Object(elms)
    })(i)
}

fn parse_element_movie_clip(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Found movie clip, but type is reserved and unused?");
    Ok((i, SolValue::Unsupported))
}

fn parse_element_mixed_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, _array_length) = be_u32(i)?;
    map(parse_array_element, |elms: Vec<SolElement>| {
        SolValue::Object(elms)
    })(i)
}

fn parse_element_object_end(i: &[u8]) -> IResult<&[u8], SolValue> {
    Ok((i, SolValue::ObjectEnd))
}

fn parse_element_null(i: &[u8]) -> IResult<&[u8], SolValue> {
    Ok((i, SolValue::Null))
}

fn parse_element_undefined(i: &[u8]) -> IResult<&[u8], SolValue> {
    Ok((i, SolValue::Undefined))
}

fn parse_element_reference(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Reference resolution is not currently supported");
    map(be_u16, SolValue::Reference)(i)
}

pub fn parse_element_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Reference resolution is not currently supported");
    let (i, length) = be_u32(i)?;

    let length_usize = length
        .try_into()
        .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

    // There must be at least `length_usize` bytes (u8) to read this, this prevents OOM errors with v.large arrays
    if i.len() < length_usize {
        return Err(Err::Error(make_error(i, ErrorKind::TooLarge)))
    }

    // This must parse length elements
    //TODO: should this be single elem
    // let (i, elements) = many_m_n(length as usize, length as usize, parse_element)(i)?;
    let (i, elements) = many_m_n(length_usize, length_usize, parse_single_element)(i)?;

    Ok((i, SolValue::Array(elements)))
}

fn parse_element_date(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, millis) = be_f64(i)?;
    let (i, time_zone) = be_u16(i)?;

    Ok((i, SolValue::Date(millis, time_zone)))
}

pub fn parse_element_long_string(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, length) = be_u32(i)?;
    let (i, str) = take_str!(i, length)?;

    Ok((i, SolValue::LongString(str.to_string())))
}

fn parse_element_unsupported(i: &[u8]) -> IResult<&[u8], SolValue> {
    Ok((i, SolValue::Unsupported))
}

fn parse_element_record_set(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Found record set, but type is reserved and unused?");
    Ok((i, SolValue::Unsupported))
}

fn parse_element_xml(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, content) = parse_element_long_string(i)?;
    let content_string = match content {
        SolValue::LongString(s) => s,
        _ => unimplemented!(), //TODO: better handling of this
    };
    Ok((i, SolValue::XML(content_string)))
}

fn parse_element_typed_object(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, s) = parse_string(i)?;
    let (i, obj) = parse_element_object(i)?;
    let obj_content = match obj {
        SolValue::Object(x) => x,
        _ => unimplemented!(),
    };

    Ok((i, SolValue::TypedObject(s.to_string(), obj_content)))
}

fn parse_element_amf3(i: &[u8]) -> IResult<&[u8], SolValue> {
    // amf3::parse_element_object(i)
    Ok((i, SolValue::Unsupported))
}

use crate::types::{SolElement, SolHeader, SolValue};
use crate::{
    amf3, FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3, HEADER_SIGNATURE, HEADER_VERSION, PADDING,
};
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::{many0, many_m_n};
use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
use nom::IResult;
use std::convert::TryInto;

use nom::error::{make_error, ErrorKind};
use nom::Err;
fn parse_single_element(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, type_) = be_u8(i)?;

    match type_ {
        TYPE_NUMBER => parse_element_number(i),
        TYPE_BOOL => parse_element_bool(i),
        TYPE_STRING => parse_element_string(i),
        TYPE_OBJECT => parse_element_object(i),
        TYPE_MOVIE_CLIP => parse_element_movie_clip(i),
        TYPE_NULL => parse_element_null(i),
        TYPE_UNDEFINED => parse_element_undefined(i),
        TYPE_REFERENCE => parse_element_reference(i),
        TYPE_MIXED_ARRAY_START => parse_element_mixed_array(i),
        TYPE_OBJECT_END => parse_element_object_end(i),
        TYPE_ARRAY => parse_element_array(i),
        TYPE_DATE => parse_element_date(i),
        TYPE_LONG_STRING => parse_element_long_string(i),
        TYPE_UNSUPPORTED => parse_element_unsupported(i),
        TYPE_RECORD_SET => parse_element_record_set(i),
        TYPE_XML => parse_element_xml(i),
        TYPE_TYPED_OBJECT => parse_element_typed_object(i),
        TYPE_AMF3 => parse_element_amf3(i),
        _ => Err(Err::Error(make_error(i, ErrorKind::HexDigit))),
    }
}

fn parse_element(i: &[u8]) -> IResult<&[u8], SolElement> {
    let (i, name) = parse_string(i)?;

    map(parse_single_element, move |v: SolValue| SolElement {
        name: name.to_string(),
        value: v,
    })(i)
}

fn parse_element_and_padding(i: &[u8]) -> IResult<&[u8], SolElement> {
    let (i, e) = parse_element(i)?;
    let (i, _) = parse_padding(i)?;

    Ok((i, e))
}

//TODO: can this be done better somehow??
fn parse_array_element(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
    let mut out = Vec::new();

    let mut i = i;
    loop {
        let (j, e) = parse_element(i)?;
        i = j;

        if let SolValue::ObjectEnd = e.value {
            break;
        }

        out.push(e.clone());
    }

    Ok((i, out))
}

pub fn parse_body(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
    many0(parse_element_and_padding)(i)
}

#[cfg(test)]
mod test {
    use crate::amf0::{parse_array_element, parse_element_array};

    #[test]
    fn test_array_element_out_of_memory() {
        parse_element_array(&[93, 0, 0, 0]);
    }

    #[test]
    fn test_array_element() {
        parse_element_array(&[0, 0, 0, 1, 3, 0, 0, 17, 47, 4, 0, 0, 255, 255, 255]);
    }
}
