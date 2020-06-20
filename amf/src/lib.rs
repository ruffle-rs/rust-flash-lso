
const HEADER_VERSION: [u8; 2] = [0x00, 0xbf];
const HEADER_SIGNATURE: [u8; 10] = [0x54, 0x43, 0x53, 0x4f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
const PADDING: [u8; 1] = [0x00];

const FORMAT_VERSION_AMF0: u8 = 0x0;
const FORMAT_VERSION_AMF3: u8 = 0x3;

#[derive(Debug)]
pub struct Sol {
    pub header: SolHeader,
    pub body: Vec<SolElement>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SolHeader {
    pub version: [u8; 2],
    pub length: u32,
    pub signature: [u8; 10],
    pub name: String,
    //TODO: this could be an enum
    pub format_version: u8,
}

use nom::bytes::complete::{tag, take, take_while};
use nom::combinator::map;
use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
use nom::take_str;
use nom::{ multi::many0};
use nom::{IResult};
use nom::character::complete::anychar;
use std::convert::TryInto;
use nom::switch;
use nom::call;
use nom::branch::alt;
//TODO: look at do_parse!

fn parse_version(i: &[u8]) -> IResult<&[u8], [u8; 2]> {
    map(tag(HEADER_VERSION), |v: &[u8]| v.try_into().unwrap())(i)
}

fn parse_length(i: &[u8]) -> IResult<&[u8], u32> {
    be_u32(i)
}

fn parse_signature(i: &[u8]) -> IResult<&[u8], [u8; 10]> {
    map(tag(HEADER_SIGNATURE), |sig: &[u8]| sig.try_into().unwrap())(i)
}

fn parse_string(i: &[u8]) -> IResult<&[u8], &str> {
    let (i, length) = be_u16(i)?;
    take_str!(i, length)

}

fn parse_padding(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(PADDING)(i)
}

fn parse_header(i: &[u8]) -> IResult<&[u8], SolHeader> {
    let (i, v) = parse_version(i)?;
    let (i, l) = parse_length(i)?;
    let (i, sig) = parse_signature(i)?;

    let (i, name) = parse_string(i)?;

    let (i, _) = parse_padding(i)?;
    let (i, _) = parse_padding(i)?;
    let (i, _) = parse_padding(i)?;

    
    let (i, format_version) =
        map(alt((tag(&[FORMAT_VERSION_AMF0]), tag(&[FORMAT_VERSION_AMF3]))),
            |v: &[u8] | v[0])(i)?;

    Ok((
        i,
        SolHeader {
            version: v,
            length: l.try_into().unwrap(),
            signature: sig,
            name: name.to_string(),
            format_version
        },
    ))
}

#[derive(Clone, Debug)]
pub struct SolElement {
    name: String,
    value: SolValue,
}

#[derive(Debug, Clone)]
pub enum SolValue {
    Number(f64),
    Bool(bool),
    String(String),
    Object(Vec<SolElement>),
    ObjectEnd,
    MixedArray(Vec<SolElement>),
    Null,
    Undefined,
    Reference(u16),
    Array(Vec<SolElement>),
    Date(f64, u16),
    LongString(String), // TODO: should this just be a string
    Unsupported,
    // AMF3
    Integer(i32)
}

//TODO: if these are [u8; 1] could match with tag hmm
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

fn parse_element_number(i: &[u8]) -> IResult<&[u8], SolValue> {
   map(be_f64, | num: f64 | SolValue::Number(num))(i)
}

fn parse_element_bool(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(be_u8, | num: u8 | SolValue::Bool(num > 0))(i)
}

fn parse_element_string(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(parse_string, | s: &str | SolValue::String(s.to_string()))(i)
}

fn parse_element_object(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(parse_array_element, |elms: Vec<SolElement> | SolValue::Object(elms))(i)
}

fn parse_element_mixed_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, array_length) = be_u32(i)?;
    println!("Array of size: {}", array_length);
    map(parse_array_element, |elms: Vec<SolElement> | SolValue::Object(elms))(i)
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
    println!("Reference resolution is not currently supported");
    map(be_u16, | num: u16 | SolValue::Reference(num))(i)
}

fn parse_element_array(i: &[u8]) -> IResult<&[u8], SolValue> {
    println!("Reference resolution is not currently supported");
    let (i, length) = be_u32(i)?;

    let mut elements = Vec::with_capacity(length as usize);
    let mut i = i;
    for _x in 0..length {
        let (j, e) = parse_element(i)?;
        i = j;
        elements.push(e);
    }

    Ok((i, SolValue::Array(elements)))
}

fn parse_element_date(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, millis) = be_f64(i)?;
    let (i, time_zone) = be_u16(i)?;

    Ok((i, SolValue::Date(millis, time_zone)))
}

fn parse_element_long_string(i: &[u8]) -> IResult<&[u8], SolValue> {
    let (i, length) = be_u32(i)?;
    let (i, bytes) = take(length)(i)?;

    //TODO: unwrap
    Ok((i, SolValue::LongString(String::from_utf8(bytes.to_vec()).unwrap())))
}

fn parse_element_unsupported(i: &[u8]) -> IResult<&[u8], SolValue> {
    Ok((i, SolValue::Unsupported))
}

fn parse_element_xml(i: &[u8]) -> IResult<&[u8], SolValue> {
    //TODO: xml obj
    parse_element_long_string(i)
}

fn parse_element_typed_object(i: &[u8]) -> IResult<&[u8], SolValue> {
    //TODO: xml obj
    let (i, s) = parse_string(i)?;

    parse_element_object(i)
}

fn parse_element_amf3(i: &[u8]) -> IResult<&[u8], SolValue> {
    //TODO: xml obj
    let (i, s) = parse_string(i)?;

    parse_element_object(i)
}

use nom::{named, take, tag};
named!(parse_single_element<&[u8], SolValue>,
   switch!(be_u8,
    TYPE_NUMBER => call!(parse_element_number) | 
    TYPE_BOOL => call!(parse_element_bool) |
    TYPE_STRING => call!(parse_element_string) |
    TYPE_OBJECT => call!(parse_element_object) |
    TYPE_NULL => call!(parse_element_null) |
    TYPE_UNDEFINED => call!(parse_element_undefined) |
    TYPE_REFERENCE => call!(parse_element_reference) |
    TYPE_MIXED_ARRAY_START => call!(parse_element_mixed_array) |
    TYPE_OBJECT_END => call!(parse_element_object_end) |
    TYPE_ARRAY => call!(parse_element_array) |
    TYPE_DATE => call!(parse_element_date) |
    TYPE_LONG_STRING => call!(parse_element_long_string) |
    TYPE_UNSUPPORTED => call!(parse_element_unsupported) |
    TYPE_XML => call!(parse_element_xml) |
    TYPE_TYPED_OBJECT => call!(parse_element_typed_object) |
    TYPE_AMF3 => call!(parse_element_amf3)
   )
);

fn parse_element(i: &[u8]) -> IResult<&[u8], SolElement> {
    let (i, name) = parse_string(i)?;
    log::debug!("Got name {:?}", name);

    map(parse_single_element, move | v: SolValue | SolElement {
        name: name.to_string(),
        value: v
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

fn parse_body_amf0(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
    many0(parse_element_and_padding)(i)
}

pub fn parse_full(i: &[u8]) -> IResult<&[u8], Sol> {
    let (i, header) = parse_header(i)?;
    match header.format_version {
        FORMAT_VERSION_AMF0 => {
            let (i, body) = parse_body_amf0(i)?;
            Ok((i, Sol { header, body }))
        },
        FORMAT_VERSION_AMF3 => {
            let (i, body) = amf3::parse_body(i)?;
            Ok((i, Sol { header, body: body }))
        }
        _ => {unimplemented!()}
    }
}


mod amf3 {
    use nom::IResult;
    use crate::{SolElement, SolValue, parse_padding};
    use nom::multi::many0;
    use log;
    use nom::number::complete::{be_u8, be_f64};
    use nom::take_str;
    use nom::combinator::map;

    const TYPE_UNDEFINED: u8 = 0;
    const TYPE_NULL: u8 = 1;
    const TYPE_FALSE: u8 = 2;
    const TYPE_TRUE: u8 = 3;
    const TYPE_INTEGER: u8 = 4;
    const TYPE_NUMBER: u8 = 5;
    const TYPE_STRING: u8 = 6;
    const TYPE_XML: u8 = 7;
    const TYPE_DATE: u8 = 8;
    const TYPE_ARRAY: u8 = 9;
    const TYPE_OBJECT: u8 = 10;
    const TYPE_XML_STRING: u8 = 11;
    const TYPE_BYTE_ARRAY: u8 = 12;

    const REFERENCE_FLAG: u32 = 0x01;

    fn read_int_signed(i: &[u8]) -> IResult<&[u8], i32> {
        log::debug!("Read int");
        
        let mut n = 0;
        let mut result: i32 = 0;

        let (i, v) = be_u8(i)?;
        let mut i = i;
        let mut v = v;
        //TODO: magic numbers from where??
        while v & 0x80 != 0 && n < 3 {
            result <<= 7;
            result |= (v & 0x7f) as i32;
            n += 1;

            let (j, m) = be_u8(i)?;
            i = j;
            v = m;
        }

        if n < 3 {
            log::debug!("res < 3");
            result <<= 7;
            result |= v as i32;
        } else {
            log::debug!("res > 3");
            result <<= 8;
            result |= v as i32;
            

            //TODO: signed will have to be a seperate one because of u32/i32
            if result & 0x10000000 != 0 {
                let signed = true;
                if signed {
                    result -= 0x20000000;
                } else {
                    result <<= 1;
                    result += 1;
                }
            }
        }



        Ok((i, result))
    }

    fn read_int(i: &[u8]) -> IResult<&[u8], u32> {
        log::debug!("Read int");
        
        let mut n = 0;
        let mut result: u32 = 0;

        let (i, v) = be_u8(i)?;
        let mut i = i;
        //TODO: magic numbers from where??
        while v & 0x80 != 0 && n < 3 {
            result <<= 7;
            result |= (v & 0x7f) as u32;
            n += 1;

            let (j, v) = be_u8(i)?;
            i = j;
        }

        if n < 3 {
            log::debug!("res < 3");
            result <<= 7;
            result |= v as u32;
        } else {
            log::debug!("res > 3");
            result <<= 8;
            result |= v as u32;
            

            //TODO: signed will have to be a seperate one because of u32/i32
            if result & 0x10000000 != 0 {
                let signed = false;
                if signed {
                    result -= 0x20000000;
                } else {
                    result <<= 1;
                    result += 1;
                }
            }
        }



        Ok((i, result))
    }


    /// (value, reference)
    fn read_length(i: &[u8]) -> IResult<&[u8], (u32, bool)> {
        let (i, val) = read_int(i)?;

        Ok((i, (val >> 1, val & REFERENCE_FLAG == 0)))
    }


    //TODO: use parse_byte_stream
    fn parse_string(i: &[u8]) -> IResult<&[u8], &str> {

        let (i, (len, reference)) = read_length(i)?;
        if reference {
            log::debug!("String ref not yet impl");
            Ok((i, ""))
        } else {
            if len == 0 {
                Ok((i, ""))
            } else {
                take_str!(i, len)

            }
        }
       // let (i, length) = be_u16(i)?;
       // take_str!(i, length)
    }

    use nom::take;

    fn parse_byte_stream(i: &[u8]) -> IResult<&[u8], &[u8]> {
        let (i, (len, reference)) = read_length(i)?;

        if reference {
            unimplemented!();
        }

        if len == 0 {
            Ok((i, &[]))
        } else {
            take!(i, len)
        }
    }


    fn parse_element_string(i: &[u8]) -> IResult<&[u8], SolValue> {
        map(parse_string, | s: &str | SolValue::String(s.to_string()))(i)
    }

    use crate::parse_element_number;

    fn parse_element_int(i: &[u8]) -> IResult<&[u8], SolValue> {
        map(read_int_signed, | s: i32 | SolValue::Integer(s))(i)
    }


    fn parse_element_xml(i: &[u8]) -> IResult<&[u8], SolValue> {
        //TODO: helper for this maybe
        let (i, reference) = read_int(i)?;

        if reference & REFERENCE_FLAG == 0 {
            unimplemented!();
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

        map(read_int_signed, | s: i32 | SolValue::Integer(s))(i)
    }

    fn parse_element_array(i: &[u8]) -> IResult<&[u8], SolValue> {
        let (i, mut length) = read_int(i)?;


        if length & REFERENCE_FLAG == 0 {
            unimplemented!();
        }
        length >>= 1;

        let mut elements = Vec::with_capacity(length as usize);

        // key = readBytes()
        let (i, mut key) = parse_byte_stream(i)?;

        if key == &[] {
            //TODO: return and enum
            let mut i = i;
            for _x in 0..length {
                let (j, e) = parse_element(i)?;
                elements.push(e);
                i = j;
            }
        }


        let mut i = i;
        while key != &[] {
            let (j, e) = parse_element(i)?;
            elements.push(e);
            let (j, k) = parse_byte_stream(j)?;
            i = j;
            key = k;
        }
//TODO: could use multi for this
        let mut i = i;
        for _x in 0..length {
            let (j, e) = parse_element(i)?;
            elements.push(e);
            i = j;
        }

        Ok((i, SolValue::Array(elements)))
    }


    fn un(i: &[u8]) -> IResult<&[u8], SolValue> {
        log::warn!("Not impl yet");
        Ok((i, SolValue::Null))
    }

    use nom::{named, switch, call, value};
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
        _ => call!(un)
        //TYPE_OBJECT => call!(parse_element_object) |
        //TYPE_REFERENCE => call!(parse_element_reference) |
        //TYPE_MIXED_ARRAY_START => call!(parse_element_mixed_array) |
        //TYPE_OBJECT_END => call!(parse_element_object_end) |
   //     TYPE_TYPED_OBJECT => call!(parse_element_typed_object) |
       )
    );
    fn parse_element(i: &[u8]) -> IResult<&[u8], SolElement> {
        let (i, name) = parse_string(i)?;
        log::debug!("Got name {:?}", name);

        map(parse_single_element, move | v: SolValue | SolElement {
            name: name.to_string(),
            value: v
        })(i)
    }

    fn parse_element_and_padding(i: &[u8]) -> IResult<&[u8], SolElement> {
        let (i, e) = parse_element(i)?;
        log::debug!("Got element {:?}", e);
        let (i, _) = parse_padding(i)?;


        Ok((i, e))
    }
    
    pub fn parse_body(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
        many0(parse_element_and_padding)(i)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_body_amf0, parse_full, parse_header};
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn test_header_parsing() {
        let mut x = File::open(Path::new("ballSave.sol")).unwrap();
        let mut data = Vec::new();
        let _ = x.read_to_end(&mut data).unwrap();
        let (_, header) = parse_header(&data).unwrap();
        assert_eq!(header.name, "ballSave")
    }

    #[test]
    fn test_body_parsing() {
        let mut x = File::open(Path::new("ballSave.sol")).unwrap();
        let mut data = Vec::new();
        let _ = x.read_to_end(&mut data).unwrap();
        let (i, sol) = parse_full(&data).unwrap();
        println!("{:#?}", sol);
    }

    #[test]
    fn test_terrwar_parsing() {
        let mut x = File::open(Path::new("TWAR.sol")).unwrap();
        let mut data = Vec::new();
        let _ = x.read_to_end(&mut data).unwrap();
        let (_, sol) = parse_full(&data).unwrap();
        assert_eq!(sol.header.name, "TWAR");

        println!("{:#?}", sol);
    }

    #[test]
    fn test_ball_save_parsing() {
        let mut x = File::open(Path::new("ballSave.sol")).unwrap();
        let mut data = Vec::new();
        let _ = x.read_to_end(&mut data).unwrap();
        let (_, sol) = parse_full(&data).unwrap();
        assert_eq!(sol.header.name, "ballSave");

        println!("{:#?}", sol);
    }
}

