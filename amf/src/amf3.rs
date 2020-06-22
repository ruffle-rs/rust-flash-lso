use crate::amf0::parse_padding;
use crate::types::*;
use nom::combinator::map;
use nom::multi::{many_m_n, separated_list};
use nom::number::complete::{be_f64, be_i32, be_u32, be_u8};
use nom::take_str;
use nom::IResult;

const TYPE_UNDEFINED: u8 = 0x00;
const TYPE_NULL: u8 = 0x01;
const TYPE_FALSE: u8 = 0x02;
const TYPE_TRUE: u8 = 0x03;
const TYPE_INTEGER: u8 = 0x04;
const TYPE_NUMBER: u8 = 0x05;
const TYPE_STRING: u8 = 0x06;
const TYPE_XML: u8 = 0x07;
const TYPE_DATE: u8 = 0x08;
const TYPE_ARRAY: u8 = 0x09;
const TYPE_OBJECT: u8 = 0x0A;
const TYPE_XML_STRING: u8 = 0x0B;
const TYPE_BYTE_ARRAY: u8 = 0x0C;
const TYPE_VECTOR_INT: u8 = 0x0D;
const TYPE_VECTOR_UINT: u8 = 0x0E;
const TYPE_VECTOR_DOUBLE: u8 = 0x0F;
const TYPE_VECTOR_OBJECT: u8 = 0x10;
const TYPE_DICT: u8 = 0x11;

const REFERENCE_FLAG: u32 = 0x01;

fn read_int_signed(i: &[u8]) -> IResult<&[u8], i32> {
    let mut vlu_len = 0;
    let mut result: i32 = 0;

    let (i, v) = be_u8(i)?;
    let mut i = i;
    let mut v = v;
    //TODO: magic numbers from where??
    while v & 0x80 != 0 && vlu_len < 3 {
        result <<= 7;
        result |= (v & 0x7f) as i32;
        vlu_len += 1;

        let (j, m) = be_u8(i)?;
        i = j;
        v = m;
    }

    if vlu_len < 3 {
        result <<= 7;
        result |= v as i32;
    } else {
        result <<= 8;
        result |= v as i32;

        if result & 0x10000000 != 0 {
            result -= 0x20000000;
        }
    }

    Ok((i, result))
}

fn read_int(i: &[u8]) -> IResult<&[u8], u32> {
    // log::debug!("Read uint");

    let mut n = 0;
    let mut result: u32 = 0;

    let (i, mut v) = be_u8(i)?;
    let mut i = i;
    //TODO: magic numbers from where??
    while v & 0x80 != 0 && n < 3 {
        result <<= 7;
        result |= (v & 0x7f) as u32;
        n += 1;

        let (j, v2) = be_u8(i)?;
        i = j;
        v = v2;
    }

    // log::warn!("n = {}", n);

    if n < 3 {
        // log::debug!("res < 3");
        result <<= 7;
        result |= v as u32;
    } else {
        // log::debug!("res > 3");
        result <<= 8;
        result |= v as u32;

        if result & 0x10000000 != 0 {
            result <<= 1;
            result += 1;
        }
    }

    Ok((i, result))
}

/// (value, reference)
fn read_length(i: &[u8]) -> IResult<&[u8], (u32, bool)> {
    // log::debug!("read_length");
    let (i, val) = read_int(i)?;

    Ok((i, (val >> 1, val & REFERENCE_FLAG == 0)))
}

use nom::take;
use std::sync::Mutex;

lazy_static! {
    static ref CACHE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
}

fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
    map(parse_byte_stream, |b| String::from_utf8(b).unwrap())(i)
}

fn parse_byte_stream(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // log::warn!("Parse byte stream");
    let (i, (len, reference)) = read_length(i)?;
    // log::warn!("Byte stream len = {}", len);

    if reference {
        // log::debug!("Reading refernce stream at {}", len);
        //TODO: don't default to []
        Ok((
            i,
            CACHE
                .lock()
                .unwrap()
                .get(len as usize)
                .unwrap_or(&vec![])
                .clone(),
        ))
    } else if len == 0 {
        Ok((i, vec![]))
    } else {
        let (i, bytes) = take!(i, len)?;
        CACHE.lock().unwrap().push(bytes.to_vec());
        Ok((i, bytes.to_vec()))
    }
}

fn parse_element_string(i: &[u8]) -> IResult<&[u8], SolValue> {
    // log::debug!("parse_string");
    map(parse_string, SolValue::String)(i)
}

fn parse_element_int(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(read_int_signed, SolValue::Integer)(i)
}

fn parse_element_xml(i: &[u8]) -> IResult<&[u8], SolValue> {
    //TODO: helper for this maybe
    let (i, reference) = read_int(i)?;

    if reference & REFERENCE_FLAG == 0 {
        //TODO: make sure to reference>>1
        log::warn!("XML REF not impl");
        return Ok((i, SolValue::String("xml_ref_not_impl".to_string())));
    }

    let (i, data) = take_str!(i, reference >> 1)?;
    //TODO: custom type
    Ok((i, SolValue::String(data.into())))
}

fn parse_element_date(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, reference) = read_int(i)?;

    if reference & REFERENCE_FLAG == 0 {
        unimplemented!();
    }

    let (i, ms) = be_f64(i)?;
    //TODO: full logic, also for other date

    // map(read_int_signed, |s: i32| SolValue::Integer(s))(i)
    //TODO: maybe make timezone option, in amf3 always UTC
    Ok((i, SolValue::Date(ms, 0)))
}

fn parse_element_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    // log::info!("Parse array");
    let (i, mut length) = read_int(i)?;

    if length & REFERENCE_FLAG == 0 {
        log::warn!("Array reference not yet impl");
    }
    length >>= 1;

    let (i, mut key) = parse_byte_stream(i)?;

    if key == [] {
        let (i, elements) = many_m_n(length as usize, length as usize, parse_single_element)(i)?;
        return Ok((i, SolValue::ValueArray(elements)));
    }

    let mut elements = Vec::with_capacity(length as usize);

    let mut i = i;
    while key != [] {
        let (j, e) = parse_single_element(i)?;
        elements.push(SolElement {
            name: String::from_utf8(key).unwrap(),
            value: e,
        });
        let (j, k) = parse_byte_stream(j)?;
        i = j;
        key = k;
    }

    // Must parse `length` elements
    let (i, el) = many_m_n(length as usize, length as usize, parse_single_element)(i)?;
    let el_elemt: Vec<SolElement> = el
        .iter()
        .enumerate()
        .map(|(pos, val)| SolElement {
            name: format!("{}", pos),
            value: val.clone(),
        })
        .collect();
    elements.extend(el_elemt);

    Ok((i, SolValue::MixedArray(elements)))
}

const ENCODING_STATIC: u8 = 0;
const ENCODING_EXTERNAL: u8 = 1;
const ENCODING_DYNAMIC: u8 = 2;
const ENCODING_PROXY: u8 = 3;

lazy_static! {
    static ref CLASS_DEFINITION_CACHE: Mutex<Vec<ClassDefinition>> = Mutex::new(Vec::new());
}

fn parse_class_def(length: u32, i: &[u8]) -> IResult<&[u8], ClassDefinition> {
    // log::warn!("Parsing class def");
    if length & REFERENCE_FLAG == 0 {
        // log::warn!("Reference = true");
        //TODO: handle this error (ref that dosent exist) (how?)
        let class_def = CLASS_DEFINITION_CACHE
            .lock()
            .expect("Can't lock mutex")
            .get((length >> 1) as usize)
            .unwrap_or(&ClassDefinition {
                externalizable: false,
                static_properties: vec![],
                name: "".to_string(),
                attribute_count: 0,
                encoding: 0,
            })
            .clone();
        return Ok((i, class_def));
    }
    let length = length >> 1;

    let (i, name) = parse_byte_stream(i)?;
    if name == [] {
        log::warn!("No object default available");
    }
    let name_str = String::from_utf8(name).map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;

    //TODO: handle empty name
    //TODO: handle alias
    let encoding = (length & 0x03) as u8;
    let attributes_count = length >> 2;

    // Read static attributes if they exist
    let (i, static_props) = many_m_n(
        attributes_count as usize,
        attributes_count as usize,
        parse_string,
    )(i)?;

    let class_def = ClassDefinition {
        name: name_str,
        encoding,
        attribute_count: attributes_count,
        static_properties: static_props,
        externalizable: false,
    };

    CLASS_DEFINITION_CACHE
        .lock()
        .unwrap()
        .push(class_def.clone());
    Ok((i, class_def))
}

fn parse_object_static<'a>(
    i: &'a [u8],
    class_def: &ClassDefinition,
) -> IResult<&'a [u8], Vec<SolElement>> {
    // log::warn!("Parse static props");
    let mut elements = Vec::new();
    let mut i = i;

    for name in class_def.static_properties.iter() {
        // log::warn!("Got static name {}", name);

        let (j, e) = parse_single_element(i)?;
        // log::warn!("Got static value {} = {:?}", name, e);

        elements.push(SolElement {
            name: name.clone(),
            value: e,
        });

        i = j;
    }

    Ok((i, elements))
}

pub fn parse_element_object(i: &[u8]) -> IResult<&[u8], SolValue> {
    // log::warn!("parse obj");
    // log::warn!("First obj byte = {}", i[0]);
    let (i, mut length) = read_int(i)?;

    log::warn!("Obj len = {}", length);

    if length & REFERENCE_FLAG == 0 {
        log::warn!("Element references not yet impl");
        return Ok((i, SolValue::Null));
    }
    length >>= 1;

    // Class def
    let (i, class_def) = parse_class_def(length, i)?;

    log::warn!("Got class def: {:?}", class_def);

    // if class_def.externalizable {
    //     return Ok((i, SolValue::Null))
    // }

    // TOD: rest of object loding
    if class_def.encoding == ENCODING_EXTERNAL || class_def.encoding == ENCODING_PROXY {
        log::warn!("Proxy objects not yet supported");
    }

    let mut elements = Vec::new();

    let mut i = i;
    if class_def.encoding == ENCODING_DYNAMIC {
        // log::info!("Dynamic encoding");
        let (j, x) = parse_object_static(i, &class_def)?;
        elements.extend(x);

        // Read dynamic
        let (mut j, mut attr) = parse_byte_stream(j)?;
        // log::warn!("Got first dyn name {:?}", attr);
        while attr != [] {
            let attr_str = String::from_utf8(attr).unwrap();
            // log::warn!("Parsing child element {}", attr_str);
            let (k, val) = parse_single_element(j)?;
            elements.push(SolElement {
                name: attr_str,
                value: val,
            });

            let (k, attr2) = parse_byte_stream(k)?;
            j = k;
            attr = attr2;
        }
        i = j;
    }
    if class_def.encoding == ENCODING_STATIC {
        // log::warn!("Static encoding");
        let (j, x) = parse_object_static(i, &class_def)?;
        elements.extend(x);

        i = j;
    }

    Ok((i, SolValue::Object(elements)))
}

fn parse_element_byte_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, (len, reference)) = read_length(i)?;

    if reference {
        log::warn!("Byte array reference not impl");
        Ok((i, SolValue::Null))
    } else {
        let (i, bytes) = take!(i, len)?;
        Ok((i, SolValue::ByteArray(bytes.to_vec())))
    }
}

fn parse_element_object_vector(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, (len, reference)) = read_length(i)?;

    if reference {
        log::warn!("Object vector ref not impl");
    }

    let (i, fixed_length) = be_u8(i)?;

    let (i, object_type_name) = parse_string(i)?;

    let (i, elems) = many_m_n(len as usize, len as usize, parse_single_element)(i)?;

    Ok((
        i,
        SolValue::VectorObject(elems, object_type_name, fixed_length == 1),
    ))
}
use nom::Err;
fn parse_element_vector_int(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, (len, reference)) = read_length(i)?;
    let len_usize = len
        .try_into()
        .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

    if reference {
        log::warn!("Not impl ref intvec");
    }

    let (i, fixed_length) = be_u8(i)?;

    let (i, ints) = many_m_n(len_usize, len_usize, be_i32)(i)?;

    Ok((i, SolValue::VectorInt(ints, fixed_length == 1)))
}

fn parse_element_vector_uint(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, (len, reference)) = read_length(i)?;
    let len_usize = len
        .try_into()
        .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

    if reference {
        log::warn!("Not impl ref intvec");
    }

    let (i, fixed_length) = be_u8(i)?;

    let (i, ints) = many_m_n(len_usize, len_usize, be_u32)(i)?;

    Ok((i, SolValue::VectorUInt(ints, fixed_length == 1)))
}

fn parse_element_vector_double(i: &[u8]) -> IResult<&[u8], SolValue> {
    // log::warn!("Reading vec<dub>");
    let (i, (len, reference)) = read_length(i)?;
    let len_usize = len
        .try_into()
        .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

    if reference {
        log::warn!("Not impl ref dubvec");
    }

    let (i, fixed_length) = be_u8(i)?;

    let (i, ints) = many_m_n(len_usize, len_usize, be_f64)(i)?;

    Ok((i, SolValue::VectorDouble(ints, fixed_length == 1)))
}

fn parse_element_dict(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, (len, reference)) = read_length(i)?;

    if reference {
        log::warn!("Not impl ref dict");
    }

    //TODO: implications of this
    let (i, weak_keys) = be_u8(i)?;

    let (i, pairs) = many_m_n(
        len as usize,
        len as usize,
        tuple((parse_single_element, parse_single_element)),
    )(i)?;

    Ok((i, SolValue::Dictionary(pairs, weak_keys == 1)))
}

use crate::amf0::parse_element_number;
use crate::types::{SolElement, SolValue};
use nom::error::{make_error, ErrorKind};
use nom::sequence::tuple;
use nom::{call, named, switch, value};
use std::convert::TryInto;
named!(parse_single_element<&[u8], SolValue>,
   switch!(be_u8,
    TYPE_UNDEFINED => value!(SolValue::Undefined) |
    TYPE_NULL => value!(SolValue::Null) |
    TYPE_FALSE => value!(SolValue::Bool(false)) |
    TYPE_TRUE => value!(SolValue::Bool(true)) |
    TYPE_INTEGER => call!(parse_element_int) |
    TYPE_NUMBER => call!(parse_element_number) |
    TYPE_STRING => call!(parse_element_string) |
    TYPE_XML => call!(parse_element_xml) |
    TYPE_DATE => call!(parse_element_date) |
    TYPE_ARRAY => call!(parse_element_array) |
    TYPE_OBJECT => call!(parse_element_object) |
    TYPE_XML_STRING => call!(parse_element_xml) |
    TYPE_BYTE_ARRAY => call!(parse_element_byte_array) |
    TYPE_VECTOR_OBJECT => call!(parse_element_object_vector) |
    TYPE_VECTOR_INT => call!(parse_element_vector_int) |
    TYPE_VECTOR_UINT => call!(parse_element_vector_uint) |
    TYPE_VECTOR_DOUBLE => call!(parse_element_vector_double) |
    TYPE_DICT => call!(parse_element_dict)
   )
);
fn parse_element(i: &[u8]) -> IResult<&[u8], SolElement> {
    let (i, name) = parse_string(i)?;
    log::debug!("Got name {:?}", name);

    map(parse_single_element, move |v: SolValue| SolElement {
        name: name.clone(),
        value: v,
    })(i)
}

pub fn parse_body(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
    separated_list(parse_padding, parse_element)(i)
}
