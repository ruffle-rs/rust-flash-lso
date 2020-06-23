use crate::amf0::parse_element_number;
use crate::types::*;
use crate::types::{SolElement, SolValue};
use crate::PADDING;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{make_error, ErrorKind};
use nom::multi::{many_m_n, separated_list};
use nom::number::complete::{be_f64, be_i32, be_u32, be_u8};
use nom::sequence::tuple;
use nom::take;
use nom::take_str;
use nom::Err;
use nom::IResult;
use std::cell::RefCell;
use std::convert::TryInto;

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

const ENCODING_STATIC: u8 = 0;
const ENCODING_EXTERNAL: u8 = 1;
const ENCODING_DYNAMIC: u8 = 2;
const ENCODING_PROXY: u8 = 3;

pub fn read_int_signed(i: &[u8]) -> IResult<&[u8], i32> {
    let mut vlu_len = 0;
    let mut result: i32 = 0;

    let (mut i, mut v) = be_u8(i)?;
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

pub fn read_int(i: &[u8]) -> IResult<&[u8], u32> {
    let mut n = 0;
    let mut result: u32 = 0;

    let (mut i, mut v) = be_u8(i)?;
    //TODO: magic numbers from where??
    while v & 0x80 != 0 && n < 3 {
        result <<= 7;
        result |= (v & 0x7f) as u32;
        n += 1;

        let (j, v2) = be_u8(i)?;
        i = j;
        v = v2;
    }

    if n < 3 {
        result <<= 7;
        result |= v as u32;
    } else {
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
    let (i, val) = read_int(i)?;

    Ok((i, (val >> 1, val & REFERENCE_FLAG == 0)))
}

fn parse_element_int(i: &[u8]) -> IResult<&[u8], SolValue> {
    map(read_int_signed, SolValue::Integer)(i)
}

pub struct AMF3Decoder {
    pub string_reference_table: RefCell<Vec<Vec<u8>>>,
    pub trait_reference_table: RefCell<Vec<ClassDefinition>>,
    pub object_reference_table: RefCell<Vec<SolValue>>,
}

impl Default for AMF3Decoder {
    fn default() -> Self {
        Self {
            string_reference_table: RefCell::new(Vec::new()),
            trait_reference_table: RefCell::new(Vec::new()),
            object_reference_table: RefCell::new(Vec::new()),
        }
    }
}

impl AMF3Decoder {
    fn parse_element_string<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        map(|i| self.parse_string(i), SolValue::String)(i)
    }

    pub fn parse_string<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], String> {
        let (i, bytes) = self.parse_byte_stream(i)?;
        let bytes_str =
            String::from_utf8(bytes).map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;
        Ok((i, bytes_str))
    }

    fn parse_class_def<'a>(&self, length: u32, i: &'a [u8]) -> IResult<&'a [u8], ClassDefinition> {
        if length & REFERENCE_FLAG == 0 {
            let len_usize: usize = (length >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let class_def = self
                .trait_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, class_def));
        }
        let length = length >> 1;

        let (i, name) = self.parse_byte_stream(i)?;
        if name == [] {
            log::info!("Object has no name");
        }
        let name_str =
            String::from_utf8(name).map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;

        let encoding = (length & 0x03) as u8;
        let attributes_count = length >> 2;

        let attr_count_usize: usize = attributes_count
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // Read static attributes if they exist
        let (i, static_props) =
            many_m_n(attr_count_usize, attr_count_usize, |i| self.parse_string(i))(i)?;

        let class_def = ClassDefinition {
            name: name_str,
            encoding,
            attribute_count: attributes_count,
            static_properties: static_props,
            externalizable: false,
        };

        self.trait_reference_table
            .borrow_mut()
            .push(class_def.clone());
        Ok((i, class_def))
    }

    pub fn parse_byte_stream<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
        let (i, (len, reference)) = read_length(i)?;

        if reference {
            let length_usize: usize = len
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let ref_result = self
                .string_reference_table
                .borrow()
                .get(length_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            Ok((i, ref_result))
        } else if len == 0 {
            Ok((i, vec![]))
        } else {
            let (i, bytes) = take!(i, len)?;
            self.string_reference_table
                .borrow_mut()
                .push(bytes.to_vec());
            Ok((i, bytes.to_vec()))
        }
    }

    pub fn parse_object_static<'a>(
        &self,
        i: &'a [u8],
        class_def: &ClassDefinition,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let mut elements = Vec::new();
        let mut i = i;

        for name in class_def.static_properties.iter() {
            let (j, e) = self.parse_single_element(i)?;

            elements.push(SolElement {
                name: name.clone(),
                value: e,
            });

            i = j;
        }

        Ok((i, elements))
    }

    pub fn parse_element_object<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, mut length) = read_int(i)?;

        if length & REFERENCE_FLAG == 0 {
            let len_usize: usize = (length >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }
        length >>= 1;

        // Class def
        let (i, class_def) = self.parse_class_def(length, i)?;

        // TODO: rest of object loding
        if class_def.encoding == ENCODING_EXTERNAL || class_def.encoding == ENCODING_PROXY {
            log::warn!("Proxy objects not yet supported");
        }

        let mut elements = Vec::new();

        let mut i = i;
        if class_def.encoding == ENCODING_DYNAMIC {
            // log::info!("Dynamic encoding");
            let (j, x) = self.parse_object_static(i, &class_def)?;
            elements.extend(x);

            // Read dynamic
            let (mut j, mut attr) = self.parse_byte_stream(j)?;
            while attr != [] {
                let attr_str = String::from_utf8(attr)
                    .map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;
                let (k, val) = self.parse_single_element(j)?;
                elements.push(SolElement {
                    name: attr_str,
                    value: val,
                });

                let (k, attr2) = self.parse_byte_stream(k)?;
                j = k;
                attr = attr2;
            }
            i = j;
        }
        if class_def.encoding == ENCODING_STATIC {
            let (j, x) = self.parse_object_static(i, &class_def)?;
            elements.extend(x);

            i = j;
        }

        let obj = SolValue::Object(elements);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_byte_array<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;

        if reference {
            let len_usize: usize = len
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        } else {
            let (i, bytes) = take!(i, len)?;
            let obj = SolValue::ByteArray(bytes.to_vec());
            self.object_reference_table.borrow_mut().push(obj.clone());
            Ok((i, obj))
        }
    }

    fn parse_element_vector_int<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;

        let len_usize: usize = len
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize * 4` (i32 = 4 bytes) bytes to read this, this prevents OOM errors with v.large vecs
        if i.len() < len_usize * 4 {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        if reference {
            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }

        let (i, fixed_length) = be_u8(i)?;

        let (i, ints) = many_m_n(len_usize, len_usize, be_i32)(i)?;

        let obj = SolValue::VectorInt(ints, fixed_length == 1);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_vector_uint<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;

        let len_usize: usize = len
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize * 4` (u32 = 4 bytes) bytes to read this, this prevents OOM errors with v.large vecs
        if i.len() < len_usize * 4 {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        if reference {
            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }

        let (i, fixed_length) = be_u8(i)?;

        let (i, ints) = many_m_n(len_usize, len_usize, be_u32)(i)?;

        let obj = SolValue::VectorUInt(ints, fixed_length == 1);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_vector_double<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;
        let len_usize: usize = len
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize * 8` (f64 = 8 bytes) bytes to read this, this prevents OOM errors with v.large dicts
        if i.len() < len_usize * 8 {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        if reference {
            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }

        let (i, fixed_length) = be_u8(i)?;

        let (i, ints) = many_m_n(len_usize, len_usize, be_f64)(i)?;

        let obj = SolValue::VectorDouble(ints, fixed_length == 1);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_object_vector<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;

        let length_usize = len
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize` bytes to read this, this prevents OOM errors with v.large dicts
        if i.len() < length_usize {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        if reference {
            let obj = self
                .object_reference_table
                .borrow()
                .get(length_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }

        let (i, fixed_length) = be_u8(i)?;

        let (i, object_type_name) = self.parse_string(i)?;

        let (i, elems) = many_m_n(length_usize, length_usize, |i| self.parse_single_element(i))(i)?;

        let obj = SolValue::VectorObject(elems, object_type_name, fixed_length == 1);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_array<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, mut length) = read_int(i)?;

        if length & REFERENCE_FLAG == 0 {
            let len_usize: usize = (length >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }
        length >>= 1;

        let length_usize = length
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize` bytes to read this, this prevents OOM errors with v.large dicts
        if i.len() < length_usize {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        let (i, mut key) = self.parse_byte_stream(i)?;

        if key == [] {
            let (i, elements) =
                many_m_n(length_usize, length_usize, |i| self.parse_single_element(i))(i)?;
            return Ok((i, SolValue::StrictArray(elements)));
        }

        let mut elements = Vec::with_capacity(length_usize);

        let mut i = i;
        while key != [] {
            let (j, e) = self.parse_single_element(i)?;
            let key_str =
                String::from_utf8(key).map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;

            elements.push(SolElement {
                name: key_str,
                value: e,
            });
            let (j, k) = self.parse_byte_stream(j)?;
            i = j;
            key = k;
        }

        // Must parse `length` elements
        let (i, el) = many_m_n(length_usize, length_usize, |i| self.parse_single_element(i))(i)?;
        let el_elemt: Vec<SolElement> = el
            .iter()
            .enumerate()
            .map(|(pos, val)| SolElement {
                name: format!("{}", pos),
                value: val.clone(),
            })
            .collect();
        elements.extend(el_elemt);

        let obj = SolValue::ECMAArray(elements);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_dict<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;

        if reference {
            log::warn!("Not impl ref dict");
        }

        //TODO: implications of this
        let (i, weak_keys) = be_u8(i)?;

        let length_usize = len
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize * 2` bytes (due to (key,val) pairs) to read this, this prevents OOM errors with v.large dicts
        if i.len() < length_usize * 2 {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        let (i, pairs) = many_m_n(
            length_usize,
            length_usize,
            tuple((
                |i| self.parse_single_element(i),
                |i| self.parse_single_element(i),
            )),
        )(i)?;

        Ok((i, SolValue::Dictionary(pairs, weak_keys == 1)))
    }
    fn parse_element_date<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, reference) = read_int(i)?;

        if reference & REFERENCE_FLAG == 0 {
            let len_usize: usize = (reference >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }

        let (i, ms) = be_f64(i)?;

        let obj = SolValue::Date(ms, None);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_element_xml<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, reference) = read_int(i)?;

        if reference & REFERENCE_FLAG == 0 {
            let len_usize: usize = (reference >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj));
        }

        let (i, data) = take_str!(i, reference >> 1)?;
        let obj = SolValue::XML(data.into());
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
    }

    fn parse_single_element<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, type_) = be_u8(i)?;

        match type_ {
            TYPE_UNDEFINED => Ok((i, SolValue::Undefined)),
            TYPE_NULL => Ok((i, SolValue::Null)),
            TYPE_FALSE => Ok((i, SolValue::Bool(false))),
            TYPE_TRUE => Ok((i, SolValue::Bool(true))),
            TYPE_INTEGER => parse_element_int(i),
            TYPE_NUMBER => parse_element_number(i),
            TYPE_STRING => self.parse_element_string(i),
            TYPE_XML => self.parse_element_xml(i),
            TYPE_DATE => self.parse_element_date(i),
            TYPE_ARRAY => self.parse_element_array(i),
            TYPE_OBJECT => self.parse_element_object(i),
            TYPE_XML_STRING => self.parse_element_xml(i),
            TYPE_BYTE_ARRAY => self.parse_element_byte_array(i),
            TYPE_VECTOR_OBJECT => self.parse_element_object_vector(i),
            TYPE_VECTOR_INT => self.parse_element_vector_int(i),
            TYPE_VECTOR_UINT => self.parse_element_vector_uint(i),
            TYPE_VECTOR_DOUBLE => self.parse_element_vector_double(i),
            TYPE_DICT => self.parse_element_dict(i),
            _ => Err(Err::Error(make_error(i, ErrorKind::HexDigit))),
        }
    }

    pub fn parse_element<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolElement> {
        let (i, name) = self.parse_string(i)?;
        map(
            |i| self.parse_single_element(i),
            move |v: SolValue| SolElement {
                name: name.clone(),
                value: v,
            },
        )(i)
    }

    pub fn parse_body<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], Vec<SolElement>> {
        separated_list(tag(PADDING), |i| self.parse_element(i))(i)
    }
}

#[cfg(test)]
mod test {
    use crate::amf3::AMF3Decoder;

    #[test]
    pub fn test_amf3_error() {
        let _ = AMF3Decoder::default()
            .parse_body(&[1, 15, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
    }
}
