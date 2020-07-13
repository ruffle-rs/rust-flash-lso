use nom::bytes::complete::tag;
use nom::take_str;
use crate::types::{SolElement, SolValue};
use crate::{amf3, PADDING};
use nom::combinator::map;
use nom::multi::{many0, many_m_n};
use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
use nom::IResult;
use std::convert::{TryInto, TryFrom};

use nom::error::{make_error, ErrorKind};
use nom::Err;
use derive_try_from_primitive::TryFromPrimitive;

//TODO: camel case
#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum TypeMarker {
 Number = 0,
 Boolean = 1,
 String = 2,
 Object = 3,
 MovieClip = 4,
 Null = 5,
 Undefined = 6,
 Reference = 7,
 MixedArrayStart = 8,
 ObjectEnd = 9,
 Array = 10,
 Date = 11,
 LongString = 12,
 Unsupported = 13,
 RecordSet = 14,
 XML = 15,
 TypedObject = 16,
 AMF3 = 17,
}

#[derive(Default)]
struct AMF0Decoder {}

impl AMF0Decoder {

}


pub fn parse_string(i: &[u8]) -> IResult<&[u8], &str> {
    let (i, length) = be_u16(i)?;
    take_str!(i, length)
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
        SolValue::ECMAArray(elms)
    })(i)
}

fn parse_element_reference(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Reference resolution is not currently supported");
    let (i, _ref) = be_u16(i)?;

    Ok((i, SolValue::Unsupported))
}

pub fn parse_element_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Reference resolution is not currently supported");
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

    Ok((i, SolValue::StrictArray(elements)))
}

fn parse_element_date(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, millis) = be_f64(i)?;
    let (i, time_zone) = be_u16(i)?;

    Ok((i, SolValue::Date(millis, Some(time_zone))))
}

pub fn parse_element_long_string(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, length) = be_u32(i)?;
    let (i, str) = take_str!(i, length)?;

    Ok((i, SolValue::String(str.to_string())))
}

fn parse_element_record_set(i: &[u8]) -> IResult<&[u8], SolValue> {
    log::warn!("Found record set, but type is reserved and unused?");
    Ok((i, SolValue::Unsupported))
}

fn parse_element_xml(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, content) = parse_element_long_string(i)?;
    if let SolValue::String(content_string) = content {
        Ok((i, SolValue::XML(content_string)))
    } else {
        // Will never happen
        Err(Err::Error(make_error(i, ErrorKind::Digit)))
    }
}

fn parse_element_typed_object(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, s) = parse_string(i)?;
    let (i, obj) = parse_element_object(i)?;
    if let SolValue::Object(obj_content) = obj {
        Ok((i, SolValue::TypedObject(s.to_string(), obj_content)))
    } else {
        // Will never happen
        Err(Err::Error(make_error(i, ErrorKind::Digit)))
    }
}

fn parse_element_amf3(i: &[u8]) -> IResult<&[u8], SolValue> {
    // Hopefully amf3 objects wont have references
    amf3::AMF3Decoder::default().parse_element_object(i)
}

fn parse_single_element(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, type_) = be_u8(i)?;


    match TypeMarker::try_from(type_).unwrap_or(TypeMarker::Unsupported) {
        TypeMarker::Number => parse_element_number(i),
        TypeMarker::Boolean => parse_element_bool(i),
        TypeMarker::String => parse_element_string(i),
        TypeMarker::Object => parse_element_object(i),
        TypeMarker::MovieClip => parse_element_movie_clip(i),
        TypeMarker::Null => Ok((i, SolValue::Null)),
        TypeMarker::Undefined => Ok((i, SolValue::Undefined)),
        TypeMarker::Reference => parse_element_reference(i),
        TypeMarker::MixedArrayStart => parse_element_mixed_array(i),
        TypeMarker::ObjectEnd => Ok((i, SolValue::ObjectEnd)),
        TypeMarker::Array => parse_element_array(i),
        TypeMarker::Date => parse_element_date(i),
        TypeMarker::LongString => parse_element_long_string(i),
        TypeMarker::Unsupported => Ok((i, SolValue::Unsupported)),
        TypeMarker::RecordSet => parse_element_record_set(i),
        TypeMarker::XML => parse_element_xml(i),
        TypeMarker::TypedObject => parse_element_typed_object(i),
        TypeMarker::AMF3 => parse_element_amf3(i),
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
    let (i, _) = tag(PADDING)(i)?;

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

pub mod encoder {
    use crate::types::{SolElement, SolValue};
    use std::io::Write;
    use cookie_factory::bytes::{be_u32, be_u8, be_u16, be_f64};
    use crate::{PADDING};
    use cookie_factory::{SerializeFn, WriteContext};

    use cookie_factory::combinator::slice;
    use cookie_factory::multi::all;
    use cookie_factory::combinator::string;
    use cookie_factory::sequence::tuple;
    use crate::encoder::write_string;
    use crate::amf0::TypeMarker;

    pub fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(type_: TypeMarker) -> impl SerializeFn<W> + 'a {
        be_u8(type_ as u8)
    }

    pub fn write_number_element<'a, 'b: 'a, W: Write + 'a>(s: f64) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::Number), be_f64(s)))
    }

    pub fn write_bool_element<'a, 'b: 'a, W: Write + 'a>(s: bool) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::Boolean), be_u8(if s {1u8} else {0u8} )))
    }

    pub fn write_long_string_content<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((be_u32(s.len() as u32), string(s)))
    }

    pub fn write_long_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::LongString), write_long_string_content(s)))
    }

    pub fn write_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::String), write_string(s)))
    }

    pub fn write_object_element<'a, 'b: 'a, W: Write + 'a>(o: &'b[SolElement]) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::Object), all(o.iter().map(write_element)), be_u16(0), write_type_marker(TypeMarker::ObjectEnd)))
    }

    pub fn write_null_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::Null)
    }

    pub fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::Undefined)
    }

    pub fn write_object_end_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::ObjectEnd)
    }

    pub fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(elements: &'b [SolValue]) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::Array), be_u32(elements.len() as u32), all(elements.iter().map(write_value))))
    }

    pub fn write_date_element<'a, 'b: 'a, W: Write + 'a>(date: f64, tz: Option<u16>) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::Date), be_f64(date), be_u16(tz.unwrap_or(0))))
    }

    pub fn write_unsupported_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::Unsupported)
    }

    pub fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(content: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::XML), write_long_string_content(content)))
    }

    pub fn write_typed_object_element<'a, 'b: 'a, W: Write + 'a>(name: &'b str, elements: &'b[SolElement]) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::TypedObject), write_string(name), all(elements.iter().map(write_element)), be_u16(0), write_type_marker(TypeMarker::ObjectEnd)))
    }

    pub fn write_mixed_array<'a, 'b: 'a, W: Write + 'a>(elements: &'b[SolElement]) -> impl SerializeFn<W> + 'a {
        //TODO: what is the u16 padding
        //TODO: sometimes array length is ignored (u32) sometimes its: elements.len() as u32

        let length = if elements.len() == 2 {
            0
        } else {
            elements.len() as u32
        };

        tuple((write_type_marker(TypeMarker::MixedArrayStart), be_u32(length), all(elements.iter().map(write_element)), be_u16(0), write_type_marker(TypeMarker::ObjectEnd)))
    }

    pub fn write_value<'a, 'b: 'a, W: Write + 'a>(element: &'b SolValue) -> impl SerializeFn<W> + 'a {
        println!("Writing element: {:?}", element);
        move |out: WriteContext<W>| match element {
            SolValue::Number(n) => write_number_element(*n)(out),
            SolValue::Bool(b) => write_bool_element(*b)(out),
            SolValue::String(s) => {
                if s.len() > 65535 {
                    write_long_string_element(s)(out)
                } else {
                    write_string_element(s)(out)
                }
            },
            SolValue::Object(o) => write_object_element(o)(out),
            SolValue::Null => write_null_element()(out),
            SolValue::Undefined => write_undefined_element()(out),
            SolValue::ObjectEnd => write_object_end_element()(out),
            SolValue::StrictArray(a) => write_strict_array_element(a)(out),
            SolValue::Date(d, tz) => write_date_element(*d, *tz)(out),
            SolValue::Unsupported => write_unsupported_element()(out),
            SolValue::XML(x) => write_xml_element(x)(out),
            SolValue::TypedObject(name, elements) => write_typed_object_element(name, elements)(out),
            SolValue::ECMAArray(elems) => write_mixed_array(elems)(out),
            _ => { write_unsupported_element()(out) /* Not in amf0, TODO: use the amf3 embedding for every thing else */ }
        }
    }

    pub fn write_element<'a, 'b: 'a, W: Write + 'a>(element: &'b SolElement) -> impl SerializeFn<W> + 'a {
      tuple((write_string(&element.name), write_value(&element.value)))
    }

    pub fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(element: &'b SolElement) -> impl SerializeFn<W> + 'a {
        tuple((write_element(element), slice(PADDING)))
    }

    pub fn write_body<'a, 'b: 'a, W: Write + 'a>(elements: &'b [SolElement]) -> impl SerializeFn<W> + 'a {
       all(elements.iter().map(write_element_and_padding))
    }
}
