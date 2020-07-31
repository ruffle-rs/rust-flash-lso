#![allow(clippy::identity_op)]

use crate::amf0::decoder::parse_element_number;
use crate::types::*;
use crate::types::{SolElement, SolValue};
use crate::PADDING;
use cookie_factory::SerializeFn;
use enumset::EnumSet;
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
use std::fmt::Debug;
use std::io::Write;

pub fn either<Fa, Fb, W: Write>(b: bool, t: Fa, f: Fb) -> impl SerializeFn<W>
where
    Fa: SerializeFn<W>,
    Fb: SerializeFn<W>,
{
    move |out| {
        if b {
            t(out)
        } else {
            f(out)
        }
    }
}

pub struct ElementCache<T> {
    cache: RefCell<Vec<T>>,
}

impl<T> Default for ElementCache<T> {
    fn default() -> Self {
        ElementCache {
            cache: RefCell::new(Vec::new()),
        }
    }
}

impl<T: PartialEq + Clone + Debug> ElementCache<T> {
    pub fn has(&self, val: &T) -> bool {
        self.cache.borrow().contains(val)
    }

    pub fn store(&self, val: T) {
        if !self.has(&val) {
            self.cache.borrow_mut().push(val);
        }
    }

    pub fn get_element(&self, index: usize) -> Option<T> {
        self.cache.borrow().get(index).cloned()
    }

    pub fn get_index(&self, val: T) -> Option<usize> {
        self.cache.borrow().iter().position(|i| *i == val)
    }

    pub fn to_length(&self, val: T, length: u32) -> Length {
        if let Some(i) = self.get_index(val) {
            Length::Reference(i)
        } else {
            Length::Size(length)
        }
    }

    //TODO: no clone
    pub fn to_length_store(&self, val: T, length: u32) -> Length {
        let len = self.to_length(val.clone(), length);
        self.store(val);
        len
    }
}

//tOdo: remove
impl<T: PartialEq + Clone + Debug> ElementCache<Vec<T>> {
    pub fn store_slice(&self, val: &[T]) {
        self.store(val.to_vec());
    }

    pub fn get_slice_index(&self, val: &[T]) -> Option<usize> {
        self.get_index(val.to_vec())
    }
}

#[repr(u8)]
pub enum TypeMarker {
    Undefined = 0x00,
    Null = 0x01,
    False = 0x02,
    True = 0x03,
    Integer = 0x04,
    Number = 0x05,
    String = 0x06,
    XML = 0x07,
    Date = 0x08,
    Array = 0x09,
    Object = 0x0A,
    XmlString = 0x0B,
    ByteArray = 0x0C,
    VectorInt = 0x0D,
    VectorUInt = 0x0E,
    VectorDouble = 0x0F,
    VectorObject = 0x10,
    Dictionary = 0x11,
}

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

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum Length {
    Size(u32),
    Reference(usize),
}

impl Length {
    fn is_reference(&self) -> bool {
        matches!(self, Length::Reference(_))
    }

    fn is_size(&self) -> bool {
        matches!(self, Length::Size(_))
    }

    fn to_position(&self) -> Option<usize> {
        match self {
            Length::Reference(x) => Some(*x),
            _ => None,
        }
    }
}

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

        //TODO: should name be Option<String>
        let (i, name) = self.parse_byte_stream(i)?;
        let name_str = if name == [] {
            "".to_string()
        } else {
            String::from_utf8(name).map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?
        };

        let encoding = (length & 0x03) as u8;

        let attributes_count = length >> 2;

        let attr_count_usize: usize = attributes_count
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // Read static attributes if they exist
        let (i, static_props) =
            many_m_n(attr_count_usize, attr_count_usize, |i| self.parse_string(i))(i)?;

        let is_external = encoding & 0b1 == 1;
        let is_dynamic = encoding & 0b10 == 0b10;

        let mut attributes = EnumSet::empty();

        if is_external {
            attributes = attributes | Attribute::EXTERNAL;
        }
        if is_dynamic {
            attributes = attributes | Attribute::DYNAMIC;
        }

        let class_def = ClassDefinition {
            name: name_str,
            //TODO: encodings should be an enumset
            attributes,
            attribute_count: attributes_count,
            static_properties: static_props,
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

            if obj == SolValue::Null {
                //TODO: more descriptive error something about `cyclic data`
                return Err(Err::Error(make_error(i, ErrorKind::Alpha)));
            }

            return Ok((i, obj));
        }
        length >>= 1;

        let old_len = self.object_reference_table.borrow().len();
        self.object_reference_table
            .borrow_mut()
            .push(SolValue::Null);

        // Class def
        let (i, class_def) = self.parse_class_def(length, i)?;

        // TODO: rest of object loding
        if class_def.attributes.contains(Attribute::EXTERNAL) {
            // println!("Proxy objects not yet supported");
            return Err(Err::Error(make_error(i, ErrorKind::Tag)));
        }

        let mut elements = Vec::new();

        let mut i = i;
        if class_def.attributes.contains(Attribute::DYNAMIC) {
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
        if class_def.attributes.is_empty() {
            let (j, x) = self.parse_object_static(i, &class_def)?;
            elements.extend(x);

            i = j;
        }

        let obj = SolValue::Object(elements, Some(class_def));
        // self.object_reference_table.borrow_mut().push(obj.clone());
        self.object_reference_table.borrow_mut()[old_len] = obj.clone();
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

            Ok((i, obj))
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

            if obj == SolValue::Null {
                //TODO: again cyclic err
                return Err(Err::Error(make_error(i, ErrorKind::Alpha)));
            }

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

        let old_len = self.object_reference_table.borrow().len();
        self.object_reference_table
            .borrow_mut()
            .push(SolValue::Null);

        let (i, mut key) = self.parse_byte_stream(i)?;

        if key == [] {
            let (i, elements) =
                many_m_n(length_usize, length_usize, |i| self.parse_single_element(i))(i)?;
            let obj = SolValue::StrictArray(elements);
            self.object_reference_table.borrow_mut()[old_len] = obj.clone();
            return Ok((i, obj));
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

        let elements_len = elements.len() as u32;
        let obj = SolValue::ECMAArray(el, elements, elements_len);
        self.object_reference_table.borrow_mut()[old_len] = obj.clone();
        Ok((i, obj))
    }

    fn parse_element_dict<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], SolValue> {
        let (i, (len, reference)) = read_length(i)?;

        if reference {
            let len_usize: usize = len
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj_ref = self
                .object_reference_table
                .borrow()
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, obj_ref));
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

        let obj = SolValue::Dictionary(pairs, weak_keys == 1);
        self.object_reference_table.borrow_mut().push(obj.clone());
        Ok((i, obj))
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

    fn parse_element_xml<'a>(&self, i: &'a [u8], string: bool) -> IResult<&'a [u8], SolValue> {
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
        let obj = SolValue::XML(data.into(), string);
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
            TYPE_XML => self.parse_element_xml(i, false),
            TYPE_DATE => self.parse_element_date(i),
            TYPE_ARRAY => self.parse_element_array(i),
            TYPE_OBJECT => self.parse_element_object(i),
            TYPE_XML_STRING => self.parse_element_xml(i, true),
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
        let (i, elements) = separated_list(tag(PADDING), |i| self.parse_element(i))(i)?;
        let (i, _) = tag(PADDING)(i)?;
        Ok((i, elements))
    }
}

pub mod encoder {
    use crate::amf3::{either, ElementCache, Length, TypeMarker};
    use crate::types::{Attribute, ClassDefinition, SolElement, SolValue};
    use crate::PADDING;
    use cookie_factory::bytes::{be_f64, be_i32, be_u32, be_u8};
    use cookie_factory::combinator::{cond, slice};
    use cookie_factory::multi::all;
    use cookie_factory::sequence::tuple;
    use cookie_factory::{GenError, SerializeFn, WriteContext};
    use std::cell::RefCell;
    use std::io::Write;

    #[derive(Default)]
    pub struct AMF3Encoder {
        pub string_reference_table: ElementCache<Vec<u8>>,
        pub trait_reference_table: RefCell<Vec<ClassDefinition>>,
        pub object_reference_table: ElementCache<SolValue>,
    }

    impl AMF3Encoder {
        fn write_int<'a, 'b: 'a, W: Write + 'a>(&self, i: i32) -> impl SerializeFn<W> + 'a {
            let mut n = i;
            if n < 0 {
                n += 0x20000000;
            }

            let mut real_value = None;
            let mut bytes: Vec<u8> = Vec::new();

            if n > 0x1fffff {
                real_value = Some(n);
                n >>= 1;
                bytes.push((0x80 | ((n >> 21) & 0xff)) as u8)
            }

            if n > 0x3fff {
                bytes.push((0x80 | ((n >> 14) & 0xff)) as u8)
            }

            if n > 0x7f {
                bytes.push((0x80 | ((n >> 7) & 0xff)) as u8)
            }

            if let Some(real_value) = real_value {
                n = real_value;
            }

            if n > 0x1fffff {
                bytes.push((n & 0xff) as u8);
            } else {
                bytes.push((n & 0x7f) as u8);
            }

            //TODO: cache

            //TODO: must be a better way
            // be_u8(*bytes.get(0).unwrap())

            // slice(&bytes.as_slice())

            move |out| all(bytes.iter().copied().map(be_u8))(out)
        }

        fn write_length<'a, 'b: 'a, W: Write + 'a>(&self, s: Length) -> impl SerializeFn<W> + 'a {
            match s {
                Length::Size(x) => {
                    // With the last bit set
                    self.write_int(((x << 1) | 0b1) as i32)
                }
                Length::Reference(x) => self.write_int((x << 1) as i32),
            }
        }

        fn write_byte_string<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: &'b [u8],
        ) -> impl SerializeFn<W> + 'a {
            let len = if s != [] {
                self.string_reference_table
                    .to_length(s.to_vec(), s.len() as u32)
            } else {
                Length::Size(0)
            };

            let only_length = len.is_reference() && s != [];

            if s != [] {
                self.string_reference_table.store_slice(s);
            }

            either(
                only_length,
                self.write_length(len),
                tuple((self.write_length(len), slice(s))),
            )
        }

        pub fn write_string<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: &'b str,
        ) -> impl SerializeFn<W> + 'a {
            //TODO: references (handle in byte str?)
            self.write_byte_string(s.as_bytes())
        }

        pub fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: TypeMarker,
        ) -> impl SerializeFn<W> + 'a {
            be_u8(s as u8)
        }

        pub fn write_number_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            i: f64,
        ) -> impl SerializeFn<W> + 'a {
            tuple((self.write_type_marker(TypeMarker::Number), be_f64(i)))
        }

        pub fn write_boolean_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            b: bool,
        ) -> impl SerializeFn<W> + 'a {
            either(
                b,
                self.write_type_marker(TypeMarker::True),
                self.write_type_marker(TypeMarker::False),
            )
        }

        pub fn write_string_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: &'b str,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_type_marker(TypeMarker::String),
                self.write_byte_string(s.as_bytes()),
            ))
        }

        pub fn write_null_element<'a, 'b: 'a, W: Write + 'a>(&self) -> impl SerializeFn<W> + 'a {
            self.write_type_marker(TypeMarker::Null)
        }

        pub fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
        ) -> impl SerializeFn<W> + 'a {
            self.write_type_marker(TypeMarker::Undefined)
        }

        pub fn write_int_vector<'a, 'b: 'a, W: Write + 'a>(
            &self,
            items: &'b [i32],
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                SolValue::VectorInt(items.to_vec(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorInt),
                either(
                    len.is_reference(),
                    self.write_length(len),
                    tuple((
                        self.write_length(Length::Size(items.len() as u32)),
                        be_u8(fixed_length as u8),
                        all(items.iter().copied().map(be_i32)),
                    )),
                ),
            ))
        }

        pub fn write_uint_vector<'a, 'b: 'a, W: Write + 'a>(
            &self,
            items: &'b [u32],
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                SolValue::VectorUInt(items.to_vec(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorUInt),
                either(
                    len.is_reference(),
                    self.write_length(len),
                    tuple((
                        self.write_length(Length::Size(items.len() as u32)),
                        be_u8(fixed_length as u8),
                        all(items.iter().copied().map(be_u32)),
                    )),
                ),
            ))
        }

        pub fn write_number_vector<'a, 'b: 'a, W: Write + 'a>(
            &self,
            items: &'b [f64],
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                SolValue::VectorDouble(items.to_vec(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorDouble),
                either(
                    len.is_reference(),
                    self.write_length(len),
                    tuple((
                        self.write_length(Length::Size(items.len() as u32)),
                        be_u8(fixed_length as u8),
                        all(items.iter().copied().map(be_f64)),
                    )),
                ),
            ))
        }

        pub fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            time: f64,
        ) -> impl SerializeFn<W> + 'a {
            let len = self
                .object_reference_table
                .to_length_store(SolValue::Date(time, None), 0);

            tuple((
                self.write_type_marker(TypeMarker::Date),
                self.write_length(len),
                cond(len.is_size(), be_f64(time)),
            ))
        }

        pub fn write_integer_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            i: i32,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_type_marker(TypeMarker::Integer),
                self.write_int(i),
            ))
        }

        pub fn write_byte_array_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            bytes: &'b [u8],
        ) -> impl SerializeFn<W> + 'a {
            let len = self
                .object_reference_table
                .to_length_store(SolValue::ByteArray(bytes.to_vec()), bytes.len() as u32);

            tuple((
                self.write_type_marker(TypeMarker::ByteArray),
                self.write_length(len),
                cond(len.is_size(), slice(bytes)),
            ))
        }

        pub fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            bytes: &'b str,
            string: bool,
        ) -> impl SerializeFn<W> + 'a {
            // let mut len = self
            //     .object_reference_table
            //     .to_length_store(SolValue::XML(bytes.to_string(), string), bytes.len() as u32);

            let len = Length::Size(bytes.len() as u32);

            tuple((
                either(
                    string,
                    self.write_type_marker(TypeMarker::XmlString),
                    self.write_type_marker(TypeMarker::XML),
                ),
                self.write_length(len),
                cond(len.is_size(), slice(bytes.as_bytes())),
            ))
        }

        pub fn write_class_definition<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            class_def: &'b ClassDefinition,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_byte_string(class_def.name.as_bytes()),
                all(class_def
                    .static_properties
                    .iter()
                    .map(move |p| self.write_string(p))),
            ))
        }

        //TODO: conds should be common somehwere
        pub fn write_trait_reference<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            index: u32,
            children: &'b [SolElement],
            def: &'b ClassDefinition,
        ) -> impl SerializeFn<W> + 'a {
            let size = (((index << 1) | 0u32) << 1) | 1u32;

            tuple((
                self.write_int(size as i32),
                cond(
                    def.attributes.is_empty(),
                    all(children
                        .iter()
                        .filter(move |c| def.static_properties.contains(&c.name))
                        .map(move |e| &e.value)
                        .map(move |e| self.write_value(e))),
                ),
                cond(
                    def.attributes.contains(Attribute::DYNAMIC),
                    tuple((
                        all(children
                            .iter()
                            .filter(move |c| def.static_properties.contains(&c.name))
                            .map(move |e| &e.value)
                            .map(move |e| self.write_value(e))),
                        all(children
                            .iter()
                            .filter(move |c| !def.static_properties.contains(&c.name))
                            .map(move |e| {
                                tuple((
                                    self.write_byte_string(e.name.as_bytes()),
                                    self.write_value(&e.value),
                                ))
                            })),
                        self.write_byte_string(&[]),
                    )),
                ),
            ))
        }

        pub fn write_object_reference<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            index: u32,
        ) -> impl SerializeFn<W> + 'a {
            let size = (index << 1) | 0u32;
            tuple((self.write_int(size as i32),))
        }

        pub fn write_object_full<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            children: &'b [SolElement],
            def: &'b ClassDefinition,
        ) -> impl SerializeFn<W> + 'a {
            //TODO: object references (not just traits)

            self.trait_reference_table.borrow_mut().push(def.clone());

            let is_external = def.attributes.contains(Attribute::EXTERNAL);
            let is_dynamic = def.attributes.contains(Attribute::DYNAMIC);

            let mut encoding = 0b00;
            if is_external {
                encoding |= 0b01;
            }
            if is_dynamic {
                encoding |= 0b10;
            }

            // Format attribute_count[:4] | encoding[4:2] | class_def_ref flag (1 bit) | class_ref flag (1 bit)
            let size = (((((def.attribute_count << 2) | (encoding & 0xff) as u32) << 1) | 1u32)
                << 1)
                | 1u32;

            tuple((
                self.write_int(size as i32),
                self.write_class_definition(def),
                cond(
                    def.attributes.is_empty(),
                    all(children
                        .iter()
                        .filter(move |c| def.static_properties.contains(&c.name))
                        .map(move |e| &e.value)
                        .map(move |e| self.write_value(e))),
                ),
                cond(
                    def.attributes.contains(Attribute::DYNAMIC),
                    tuple((
                        all(children
                            .iter()
                            .filter(move |c| def.static_properties.contains(&c.name))
                            .map(move |e| &e.value)
                            .map(move |e| self.write_value(e))),
                        all(children
                            .iter()
                            .filter(move |c| !def.static_properties.contains(&c.name))
                            // .map(move |e| &e.value)
                            .map(move |e| {
                                tuple((
                                    self.write_byte_string(e.name.as_bytes()),
                                    self.write_value(&e.value),
                                ))
                            })),
                        self.write_byte_string(&[]),
                    )),
                ),
            ))
        }

        pub fn write_object_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            children: &'b [SolElement],
            class_def: &'b Option<ClassDefinition>,
        ) -> impl SerializeFn<W> + 'a {
            // let mut had_object = self
            //     .object_reference_table
            //     .to_length(SolValue::Object(children.to_vec(), class_def.clone()), 0);
            let had_object = Length::Size(0);

            self.object_reference_table
                .store(SolValue::Object(children.to_vec(), class_def.clone()));

            move |out| {
                if let Some(def) = class_def {
                    let has_trait = self
                        .trait_reference_table
                        .borrow()
                        .iter()
                        .position(|cd| *cd == *def);

                    tuple((
                        self.write_type_marker(TypeMarker::Object),
                        cond(had_object.is_reference(), move |out| {
                            self.write_object_reference(had_object.to_position().unwrap() as u32)(
                                out,
                            )
                        }),
                        cond(
                            !had_object.is_reference(),
                            tuple((
                                cond(has_trait.is_some(), move |out| {
                                    self.write_trait_reference(
                                        has_trait.unwrap() as u32,
                                        children,
                                        def,
                                    )(out)
                                }),
                                cond(has_trait.is_none(), self.write_object_full(children, def)),
                            )),
                        ),
                    ))(out)
                } else {
                    //TODO: should have a default class def, this should only be possible if the input was parsed from an amf0 file
                    Err(GenError::NotYetImplemented)
                }
            }
        }

        pub fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            children: &'b [SolValue],
        ) -> impl SerializeFn<W> + 'a {
            // let mut len = self.object_reference_table.to_length_store(
            //     SolValue::StrictArray(children.to_vec()),
            //     children.len() as u32,
            // );

            //TODO: why is this not a reference
            let len = Length::Size(children.len() as u32);

            //TODO: why does this not offset the cache if StrictArray([]) is saved but always written as Size(0) instead of Ref(n)
            either(
                children == [],
                tuple((
                    self.write_type_marker(TypeMarker::Array),
                    self.write_length(Length::Size(0)),
                    self.write_byte_string(&[]), // Empty key
                )),
                tuple((
                    self.write_type_marker(TypeMarker::Array),
                    self.write_length(len),
                    cond(
                        len.is_size(),
                        tuple((
                            self.write_byte_string(&[]), // Empty key
                            all(children.iter().map(move |v| self.write_value(v))),
                        )),
                    ),
                )),
            )
        }

        pub fn write_ecma_array_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            dense: &'b [SolValue],
            assoc: &'b [SolElement],
        ) -> impl SerializeFn<W> + 'a {
            // let mut len = self.object_reference_table.to_length_store(
            //     SolValue::ECMAArray(dense.to_vec(), assoc.clone().to_vec(), assoc.len() as u32),
            //     dense.len() as u32,
            // );

            let len = Length::Size(dense.len() as u32);

            //TODO: would this also work for strict arrays if they have [] for assoc part?
            tuple((
                self.write_type_marker(TypeMarker::Array),
                self.write_length(len),
                cond(
                    len.is_size(),
                    tuple((
                        all(assoc.iter().map(move |out| self.write_element(out))),
                        self.write_byte_string(&[]),
                        all(dense.iter().map(move |out| self.write_value(out))),
                    )),
                ),
            ))
        }

        pub fn write_object_vector_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            items: &'b [SolValue],
            type_name: &'b str,
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                SolValue::VectorObject(items.to_vec(), type_name.to_string(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorObject),
                self.write_length(len),
                cond(
                    len.is_size(),
                    tuple((
                        be_u8(fixed_length as u8),
                        self.write_string(type_name),
                        all(items.iter().map(move |i| self.write_value(i))),
                    )),
                ),
            ))
        }

        pub fn write_dictionary_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            items: &'b [(SolValue, SolValue)],
            weak_keys: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length(
                SolValue::Dictionary(items.to_vec(), weak_keys),
                items.len() as u32,
            );
            self.object_reference_table
                .store(SolValue::Dictionary(items.to_vec(), weak_keys));

            tuple((
                self.write_type_marker(TypeMarker::Dictionary),
                self.write_length(len),
                cond(
                    len.is_size(),
                    tuple((
                        be_u8(weak_keys as u8),
                        all(items
                            .iter()
                            .map(move |i| tuple((self.write_value(&i.0), self.write_value(&i.1))))),
                    )),
                ),
            ))
        }

        //TODO: eventually remove
        #[allow(unreachable_code)]
        pub fn write_unsupported_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
        ) -> impl SerializeFn<W> + 'a {
            unimplemented!();
            self.write_type_marker(TypeMarker::Undefined)
        }

        pub fn write_value<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            s: &'b SolValue,
        ) -> impl SerializeFn<W> + 'a {
            move |out: WriteContext<W>| match s {
                SolValue::Number(x) => self.write_number_element(*x)(out),
                SolValue::Bool(b) => self.write_boolean_element(*b)(out),
                SolValue::String(s) => self.write_string_element(s)(out),
                SolValue::Object(children, class_def) => {
                    self.write_object_element(children, class_def)(out)
                }
                SolValue::Null => self.write_null_element()(out),
                SolValue::Undefined => self.write_undefined_element()(out),
                SolValue::ECMAArray(dense, elements, _) => {
                    self.write_ecma_array_element(dense, elements)(out)
                }
                SolValue::StrictArray(children) => self.write_strict_array_element(children)(out),
                SolValue::Date(time, _tz) => self.write_date_element(*time)(out),
                SolValue::XML(content, string) => self.write_xml_element(content, *string)(out),
                SolValue::Integer(i) => self.write_integer_element(*i)(out),
                SolValue::ByteArray(bytes) => self.write_byte_array_element(bytes)(out),
                SolValue::VectorInt(items, fixed_length) => {
                    self.write_int_vector(items, *fixed_length)(out)
                }
                SolValue::VectorUInt(items, fixed_length) => {
                    self.write_uint_vector(items, *fixed_length)(out)
                }
                SolValue::VectorDouble(items, fixed_length) => {
                    self.write_number_vector(items, *fixed_length)(out)
                }
                SolValue::VectorObject(items, type_name, fixed_length) => {
                    self.write_object_vector_element(items, type_name, *fixed_length)(out)
                }
                SolValue::Dictionary(kv, weak_keys) => {
                    self.write_dictionary_element(kv, *weak_keys)(out)
                }

                SolValue::TypedObject(_, _) => self.write_unsupported_element()(out),
                SolValue::Unsupported => self.write_unsupported_element()(out),
            }
        }

        pub fn write_element<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            element: &'b SolElement,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_string(&element.name),
                self.write_value(&element.value),
            ))
        }

        pub fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            element: &'b SolElement,
        ) -> impl SerializeFn<W> + 'a {
            tuple((self.write_element(element), slice(PADDING)))
        }

        pub fn write_body<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            elements: &'b [SolElement],
        ) -> impl SerializeFn<W> + 'a {
            all(elements
                .iter()
                .map(move |e| self.write_element_and_padding(e)))
        }
    }
}
