use nom::bytes::complete::tag;
use nom::take_str;
use crate::types::{SolElement, SolValue};
use crate::{amf3, PADDING};
use nom::combinator::map;
use nom::multi::{many0, many_m_n};
use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
use nom::IResult;
use std::convert::TryInto;

use nom::error::{make_error, ErrorKind};
use nom::Err;

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

    match type_ {
        TYPE_NUMBER => parse_element_number(i),
        TYPE_BOOL => parse_element_bool(i),
        TYPE_STRING => parse_element_string(i),
        TYPE_OBJECT => parse_element_object(i),
        TYPE_MOVIE_CLIP => parse_element_movie_clip(i),
        TYPE_NULL => Ok((i, SolValue::Null)),
        TYPE_UNDEFINED => Ok((i, SolValue::Undefined)),
        TYPE_REFERENCE => parse_element_reference(i),
        TYPE_MIXED_ARRAY_START => parse_element_mixed_array(i),
        TYPE_OBJECT_END => Ok((i, SolValue::ObjectEnd)),
        TYPE_ARRAY => parse_element_array(i),
        TYPE_DATE => parse_element_date(i),
        TYPE_LONG_STRING => parse_element_long_string(i),
        TYPE_UNSUPPORTED => Ok((i, SolValue::Unsupported)),
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
    use crate::types::{SolHeader, Sol};
    use cookie_factory::bytes::{be_u32, be_u8, be_u16, be_f64};
    use crate::{PADDING, FORMAT_VERSION_AMF0, FORMAT_VERSION_AMF3};
    use nom::lib::std::alloc::handle_alloc_error;
    use cookie_factory::{SerializeFn, WriteContext};

    use cookie_factory::combinator::slice;
    use cookie_factory::multi::all;
    use cookie_factory::combinator::string;
    use cookie_factory::combinator::cond;
    use cookie_factory::sequence::tuple;
    use nom::error::ErrorKind::SeparatedList;
    use crate::amf0::{TYPE_STRING, TYPE_NULL, TYPE_NUMBER, TYPE_BOOL, TYPE_OBJECT_END, TYPE_OBJECT, TYPE_UNDEFINED, TYPE_UNSUPPORTED, TYPE_XML, TYPE_ARRAY, TYPE_DATE, TYPE_TYPED_OBJECT, TYPE_LONG_STRING, TYPE_MIXED_ARRAY_START};
    use crate::encoder::write_string;
    
    pub fn write_number_element<'a, 'b: 'a, W: Write + 'a>(s: f64) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_NUMBER), be_f64(s)))
    }

    pub fn write_bool_element<'a, 'b: 'a, W: Write + 'a>(s: bool) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_BOOL), be_u8(if s {1u8} else {0u8} )))
    }

    pub fn write_long_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_LONG_STRING), be_u32(s.len() as u32), string(s)))
    }

    pub fn write_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_STRING), write_string(s)))
    }

    pub fn write_object_element<'a, 'b: 'a, W: Write + 'a>(o: &'b[SolElement]) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_OBJECT), all(o.iter().map(write_element)), be_u8(TYPE_OBJECT_END)))
    }

    pub fn write_null_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        be_u8(TYPE_NULL)
    }

    pub fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        be_u8(TYPE_UNDEFINED)
    }

    pub fn write_object_end_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        be_u8(TYPE_OBJECT_END)
    }

    pub fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(elements: &'b [SolValue]) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_ARRAY), be_u32(elements.len() as u32), all(elements.iter().map(write_value))))
    }

    pub fn write_date_element<'a, 'b: 'a, W: Write + 'a>(date: f64, tz: Option<u16>) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_DATE), be_f64(date), be_u16(tz.unwrap_or(0))))
    }

    pub fn write_unsupported_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        be_u8(TYPE_UNSUPPORTED)
    }

    pub fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(content: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_XML), write_string(content)))
    }

    pub fn write_typed_object_element<'a, 'b: 'a, W: Write + 'a>(name: &'b str, elements: &'b[SolElement]) -> impl SerializeFn<W> + 'a {
        tuple((be_u8(TYPE_TYPED_OBJECT), write_string(name), all(elements.iter().map(write_element)), be_u8(TYPE_OBJECT_END)))
    }

    pub fn write_mixed_array<'a, 'b: 'a, W: Write + 'a>(elements: &'b[SolElement]) -> impl SerializeFn<W> + 'a {
        //TODO: what is the u16 padding
        //TODO: sometimes array length is ignored (u32) sometimes its: elements.len() as u32

        let length = if elements.len() == 2 {
            0
        } else {
            elements.len() as u32
        };

        tuple((be_u8(TYPE_MIXED_ARRAY_START), be_u32(length), all(elements.iter().map(write_element)), be_u16(0), be_u8(TYPE_OBJECT_END)))
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
