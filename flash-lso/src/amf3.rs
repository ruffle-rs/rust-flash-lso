#![allow(clippy::identity_op)]

mod type_marker;

use crate::amf3::encoder::AMF3Encoder;
use crate::amf3::type_marker::TypeMarker;
use crate::length::Length;
use crate::types::*;
use crate::types::{Element, Value};
use crate::PADDING;
use enumset::EnumSet;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{make_error, ErrorKind};
use nom::lib::std::collections::HashMap;
use nom::multi::{many_m_n, separated_list0};
use nom::number::complete::{be_f64, be_i32, be_u32, be_u8};
use nom::take;
use nom::take_str;
use nom::Err;
use nom::IResult;
use std::convert::{TryFrom, TryInto};
use std::ops::DerefMut;
use std::rc::Rc;

const REFERENCE_FLAG: u32 = 0x01;

fn read_int_signed(i: &[u8]) -> IResult<&[u8], i32> {
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

fn read_int(i: &[u8]) -> IResult<&[u8], u32> {
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

fn read_length(i: &[u8]) -> IResult<&[u8], Length> {
    let (i, val) = read_int(i)?;
    Ok((
        i,
        match val & REFERENCE_FLAG == 0 {
            true => Length::Reference(val as usize >> 1),
            false => Length::Size(val >> 1),
        },
    ))
}

fn parse_element_int(i: &[u8]) -> IResult<&[u8], Rc<Value>> {
    let (i, s) = map(read_int_signed, Value::Integer)(i)?;
    Ok((i, Rc::new(s)))
}

//TODO: could this be combined with the trait
type ExternalDecoderFn =
    Rc<Box<dyn for<'a> Fn(&'a [u8], &mut AMF3Decoder) -> IResult<&'a [u8], Vec<Element>>>>;

pub trait CustomEncoder {
    fn encode<'a>(
        &self,
        elements: &'a [Element],
        class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8>;
}

pub struct AMF3Decoder {
    pub string_reference_table: Vec<Vec<u8>>,
    pub trait_reference_table: Vec<ClassDefinition>,
    pub object_reference_table: Vec<Rc<Value>>,
    pub external_decoders: HashMap<String, ExternalDecoderFn>,
}

impl Default for AMF3Decoder {
    fn default() -> Self {
        Self {
            string_reference_table: Vec::new(),
            trait_reference_table: Vec::new(),
            object_reference_table: Vec::new(),
            external_decoders: HashMap::new(),
        }
    }
}

fn parse_element_number(i: &[u8]) -> IResult<&[u8], Rc<Value>> {
    let (i, v) = map(be_f64, Value::Number)(i)?;
    Ok((i, Rc::new(v)))
}

impl AMF3Decoder {
    fn parse_element_string<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        let (i, s) = map(|i| self.parse_string(i), Value::String)(i)?;
        Ok((i, Rc::new(s)))
    }

    fn parse_string<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], String> {
        let (i, bytes) = self.parse_byte_stream(i)?;
        let bytes_str =
            String::from_utf8(bytes).map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;
        Ok((i, bytes_str))
    }

    fn parse_class_def<'a>(
        &mut self,
        length: u32,
        i: &'a [u8],
    ) -> IResult<&'a [u8], ClassDefinition> {
        if length & REFERENCE_FLAG == 0 {
            let len_usize: usize = (length >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let class_def = self
                .trait_reference_table
                .get(len_usize)
                .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                .clone();

            return Ok((i, class_def));
        }
        let length = length >> 1;

        //TODO: should name be Option<String>
        let (i, name) = self.parse_byte_stream(i)?;
        let name_str = if name.is_empty() {
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
            attributes |= Attribute::EXTERNAL;
        }
        if is_dynamic {
            attributes |= Attribute::DYNAMIC;
        }

        let class_def = ClassDefinition {
            name: name_str,
            attributes,
            static_properties: static_props,
        };

        self.trait_reference_table.push(class_def.clone());
        Ok((i, class_def))
    }

    fn parse_reference_or_val<'a>(
        &mut self,
        i: &'a [u8],
        parser: impl FnOnce(&mut Self, &'a [u8], usize) -> IResult<&'a [u8], Value>,
    ) -> IResult<&'a [u8], Rc<Value>> {
        let (i, len) = read_length(i)?;

        match len {
            Length::Reference(index) => {
                let ref_result = Rc::clone(
                    self.object_reference_table
                        .get(index)
                        .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?,
                );

                Ok((i, ref_result))
            }
            Length::Size(len) => {
                let len_usize: usize = len
                    .try_into()
                    .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

                let initial = Rc::new(Value::Null);
                let index = self.object_reference_table.len();
                self.object_reference_table.push(initial);

                let (i, res) = parser(self, i, len_usize)?;

                //TODO: this should be an error case and also never happen
                let mut initial_inner = Rc::get_mut(
                    self.object_reference_table
                        .get_mut(index)
                        .expect("Index not in reference table"),
                )
                .expect("Reference still held to rc");
                *initial_inner.deref_mut() = res;

                Ok((
                    i,
                    Rc::clone(
                        self.object_reference_table
                            .get(index)
                            .expect("Index not in reference table"),
                    ),
                ))
            }
        }
    }

    fn parse_byte_stream<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
        let (i, len) = read_length(i)?;

        match len {
            Length::Size(len) => {
                if len == 0 {
                    Ok((i, vec![]))
                } else {
                    let (i, bytes) = take!(i, len)?;
                    self.string_reference_table.push(bytes.to_vec());
                    Ok((i, bytes.to_vec()))
                }
            }
            Length::Reference(index) => {
                let ref_result = self
                    .string_reference_table
                    .get(index)
                    .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?
                    .clone();

                Ok((i, ref_result))
            }
        }
    }

    fn parse_object_static<'a>(
        &mut self,
        i: &'a [u8],
        class_def: &ClassDefinition,
    ) -> IResult<&'a [u8], Vec<Element>> {
        let mut elements = Vec::new();
        let mut i = i;

        for name in class_def.static_properties.iter() {
            let (j, e) = self.parse_single_element(i)?;

            elements.push(Element {
                name: name.clone(),
                value: e,
            });

            i = j;
        }

        Ok((i, elements))
    }

    pub(crate) fn parse_element_object<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        let (i, mut length) = read_int(i)?;

        if length & REFERENCE_FLAG == 0 {
            let len_usize: usize = (length >> 1)
                .try_into()
                .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

            let obj = Rc::clone(
                self.object_reference_table
                    .get(len_usize)
                    .ok_or_else(|| Err::Error(make_error(i, ErrorKind::Digit)))?,
            );

            return Ok((i, obj));
        }
        length >>= 1;

        let obj = Rc::new(Value::Object(Vec::new(), None));
        let index = self.object_reference_table.len();
        self.object_reference_table.push(obj);

        // Class def
        let (i, class_def) = self.parse_class_def(length, i)?;

        {
            let mut_obj = Rc::get_mut(
                self.object_reference_table
                    .get_mut(index)
                    .expect("Index invalid"),
            )
            .expect("Unable to get Object");
            if let Value::Object(_, ref mut def) = mut_obj {
                *def = Some(class_def.clone());
            }
        }

        let mut elements = Vec::new();
        let external_elements;

        let mut i = i;
        if class_def.attributes.contains(Attribute::EXTERNAL) {
            return if self.external_decoders.contains_key(&class_def.name) {
                let decoder = Rc::clone(&self.external_decoders[&class_def.name]);
                let (j, v) = decoder(i, self)?;
                external_elements = v;
                i = j;
                //TODO: should it be possible to have both dynamic and external together
                Ok((
                    i,
                    Rc::new(Value::Custom(
                        external_elements,
                        vec![],
                        Some(class_def.clone()),
                    )),
                ))
            } else {
                Err(Err::Error(make_error(i, ErrorKind::Tag)))
            };
        }

        let mut i = i;
        if class_def.attributes.contains(Attribute::DYNAMIC) {
            let (j, x) = self.parse_object_static(i, &class_def)?;
            elements.extend(x);

            // Read dynamic
            let (mut j, mut attr) = self.parse_byte_stream(j)?;
            while !attr.is_empty() {
                let attr_str = String::from_utf8(attr)
                    .map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;
                let (k, val) = self.parse_single_element(j)?;
                elements.push(Element {
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

        {
            let mut_obj = Rc::get_mut(
                self.object_reference_table
                    .get_mut(index)
                    .expect("Index invalid"),
            )
            .expect("Unable to get Object");
            if let Value::Object(ref mut elements_inner, _) = mut_obj {
                *elements_inner = elements;
            }
        }

        Ok((
            i,
            Rc::clone(
                self.object_reference_table
                    .get(index)
                    .expect("Index invalid"),
            ),
        ))
    }

    fn parse_element_byte_array<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |_this, i, len| {
            let (i, bytes) = take!(i, len)?;
            Ok((i, Value::ByteArray(bytes.to_vec())))
        })
    }

    fn parse_element_vector_int<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |_this, i, len| {
            // There must be at least `len * 4` (i32 = 4 bytes) bytes to read this, this prevents OOM errors with v.large vecs
            if i.len() < len * 4 {
                return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
            }

            let (i, fixed_length) = be_u8(i)?;

            let (i, ints) = many_m_n(len, len, be_i32)(i)?;

            Ok((i, Value::VectorInt(ints, fixed_length == 1)))
        })
    }

    fn parse_element_vector_uint<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |_this, i, len| {
            // There must be at least `len * 4` (u32 = 4 bytes) bytes to read this, this prevents OOM errors with v.large vecs
            if i.len() < len * 4 {
                return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
            }
            let (i, fixed_length) = be_u8(i)?;

            let (i, ints) = many_m_n(len, len, be_u32)(i)?;

            Ok((i, Value::VectorUInt(ints, fixed_length == 1)))
        })
    }

    fn parse_element_vector_double<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |_this, i, len| {
            // There must be at least `len * 8` (f64 = 8 bytes) bytes to read this, this prevents OOM errors with v.large dicts
            if i.len() < len * 8 {
                return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
            }
            let (i, fixed_length) = be_u8(i)?;

            let (i, numbers) = many_m_n(len, len, be_f64)(i)?;

            Ok((i, Value::VectorDouble(numbers, fixed_length == 1)))
        })
    }

    fn parse_element_object_vector<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |this, i, len| {
            let (i, fixed_length) = be_u8(i)?;

            let (i, object_type_name) = this.parse_string(i)?;

            let (i, elems) = many_m_n(len, len, |i| this.parse_single_element(i))(i)?;

            Ok((
                i,
                Value::VectorObject(elems, object_type_name, fixed_length == 1),
            ))
        })
    }

    fn parse_element_array<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |this, i, length_usize| {
            // There must be at least `length_usize` bytes to read this, this prevents OOM errors with v.large dicts
            if i.len() < length_usize {
                return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
            }

            let (i, mut key) = this.parse_byte_stream(i)?;

            if key.is_empty() {
                let (i, elements) =
                    many_m_n(length_usize, length_usize, |i| this.parse_single_element(i))(i)?;

                return Ok((i, Value::StrictArray(elements)));
            }

            let mut elements = Vec::with_capacity(length_usize);

            let mut i = i;
            while !key.is_empty() {
                let (j, e) = this.parse_single_element(i)?;
                let key_str = String::from_utf8(key)
                    .map_err(|_| Err::Error(make_error(i, ErrorKind::Alpha)))?;

                elements.push(Element {
                    name: key_str,
                    value: e,
                });
                let (j, k) = this.parse_byte_stream(j)?;
                i = j;
                key = k;
            }

            // Must parse `length` elements
            let (i, el) =
                many_m_n(length_usize, length_usize, |i| this.parse_single_element(i))(i)?;

            let elements_len = elements.len() as u32;
            Ok((i, Value::ECMAArray(el, elements, elements_len)))
        })
    }

    fn parse_element_dict<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |this, i, len| {
            //TODO: implications of this
            let (i, weak_keys) = be_u8(i)?;

            // There must be at least `len * 2` bytes (due to (key,val) pairs) to read this, this prevents OOM errors with v.large dicts
            if i.len() < len * 2 {
                return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
            }

            let (i, pairs) = many_m_n(len * 2, len * 2, |i| this.parse_single_element(i))(i)?;

            let pairs = pairs
                .chunks_exact(2)
                .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
                .collect::<Vec<_>>();

            Ok((i, Value::Dictionary(pairs, weak_keys == 1)))
        })
    }

    fn parse_element_date<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |_this, i, _len| {
            let (i, ms) = be_f64(i)?;
            Ok((i, Value::Date(ms, None)))
        })
    }

    fn parse_element_xml<'a>(&mut self, i: &'a [u8], string: bool) -> IResult<&'a [u8], Rc<Value>> {
        self.parse_reference_or_val(i, |_this, i, len| {
            let (i, data) = take_str!(i, len as u32)?;
            Ok((i, Value::XML(data.into(), string)))
        })
    }

    fn read_type_marker<'a>(&self, i: &'a [u8]) -> IResult<&'a [u8], TypeMarker> {
        let (i, type_) = be_u8(i)?;
        if let Ok(type_) = TypeMarker::try_from(type_) {
            Ok((i, type_))
        } else {
            Err(Err::Error(make_error(i, ErrorKind::HexDigit)))
        }
    }

    /// Parse a single AMF3 element from the input
    #[inline]
    pub fn parse_single_element<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Rc<Value>> {
        let (i, type_) = self.read_type_marker(i)?;

        match type_ {
            TypeMarker::Undefined => Ok((i, Rc::new(Value::Undefined))),
            TypeMarker::Null => Ok((i, Rc::new(Value::Null))),
            TypeMarker::False => Ok((i, Rc::new(Value::Bool(false)))),
            TypeMarker::True => Ok((i, Rc::new(Value::Bool(true)))),
            TypeMarker::Integer => parse_element_int(i),
            TypeMarker::Number => parse_element_number(i),
            TypeMarker::String => self.parse_element_string(i),
            TypeMarker::XML => self.parse_element_xml(i, false),
            TypeMarker::Date => self.parse_element_date(i),
            TypeMarker::Array => self.parse_element_array(i),
            TypeMarker::Object => self.parse_element_object(i),
            TypeMarker::XmlString => self.parse_element_xml(i, true),
            TypeMarker::ByteArray => self.parse_element_byte_array(i),
            TypeMarker::VectorObject => self.parse_element_object_vector(i),
            TypeMarker::VectorInt => self.parse_element_vector_int(i),
            TypeMarker::VectorUInt => self.parse_element_vector_uint(i),
            TypeMarker::VectorDouble => self.parse_element_vector_double(i),
            TypeMarker::Dictionary => self.parse_element_dict(i),
        }
    }

    fn parse_element<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Element> {
        let (i, name) = self.parse_string(i)?;

        map(
            |i| self.parse_single_element(i),
            move |v| Element {
                name: name.clone(),
                value: v,
            },
        )(i)
    }

    pub fn parse_body<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Vec<Element>> {
        let (i, elements) = separated_list0(tag(PADDING), |i| self.parse_element(i))(i)?;
        let (i, _) = tag(PADDING)(i)?;
        Ok((i, elements))
    }
}

pub mod encoder {
    use crate::amf3::type_marker::TypeMarker;
    use crate::amf3::CustomEncoder;
    use crate::element_cache::ElementCache;
    use crate::length::Length;
    use crate::nom_utils::either;
    use crate::types::{Attribute, ClassDefinition, Element, Value};
    use crate::PADDING;
    use cookie_factory::bytes::{be_f64, be_i32, be_u32, be_u8};
    use cookie_factory::combinator::{cond, slice};
    use cookie_factory::multi::all;
    use cookie_factory::sequence::tuple;
    use cookie_factory::{GenError, SerializeFn, WriteContext};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io::Write;
    use std::ops::Deref;
    use std::rc::Rc;

    #[derive(Default)]
    pub struct AMF3Encoder {
        pub string_reference_table: ElementCache<Vec<u8>>,
        pub trait_reference_table: RefCell<Vec<ClassDefinition>>,
        pub object_reference_table: ElementCache<Value>,
        pub external_encoders: HashMap<String, Box<dyn CustomEncoder>>,
    }

    impl AMF3Encoder {
        pub(crate) fn write_int<'a, 'b: 'a, W: Write + 'a>(
            &self,
            i: i32,
        ) -> impl SerializeFn<W> + 'a {
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

            move |out| all(bytes.iter().copied().map(be_u8))(out)
        }

        fn write_byte_string<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: &'b [u8],
        ) -> impl SerializeFn<W> + 'a {
            let len = if !s.is_empty() {
                self.string_reference_table
                    .to_length(s.to_vec(), s.len() as u32)
            } else {
                Length::Size(0)
            };

            let only_length = len.is_reference() && !s.is_empty();

            if !s.is_empty() {
                self.string_reference_table.store_slice(s);
            }

            either(
                only_length,
                len.write(&self),
                tuple((len.write(&self), slice(s))),
            )
        }

        fn write_string<'a, 'b: 'a, W: Write + 'a>(&self, s: &'b str) -> impl SerializeFn<W> + 'a {
            self.write_byte_string(s.as_bytes())
        }

        fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: TypeMarker,
        ) -> impl SerializeFn<W> + 'a {
            be_u8(s as u8)
        }

        fn write_number_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            i: f64,
        ) -> impl SerializeFn<W> + 'a {
            tuple((self.write_type_marker(TypeMarker::Number), be_f64(i)))
        }

        fn write_boolean_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            b: bool,
        ) -> impl SerializeFn<W> + 'a {
            either(
                b,
                self.write_type_marker(TypeMarker::True),
                self.write_type_marker(TypeMarker::False),
            )
        }

        fn write_string_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            s: &'b str,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_type_marker(TypeMarker::String),
                self.write_byte_string(s.as_bytes()),
            ))
        }

        fn write_null_element<'a, 'b: 'a, W: Write + 'a>(&self) -> impl SerializeFn<W> + 'a {
            self.write_type_marker(TypeMarker::Null)
        }

        fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>(&self) -> impl SerializeFn<W> + 'a {
            self.write_type_marker(TypeMarker::Undefined)
        }

        fn write_int_vector<'a, 'b: 'a, W: Write + 'a>(
            &self,
            items: &'b [i32],
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                Value::VectorInt(items.to_vec(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorInt),
                either(
                    len.is_reference(),
                    len.write(&self),
                    tuple((
                        Length::Size(items.len() as u32).write(&self),
                        be_u8(fixed_length as u8),
                        all(items.iter().copied().map(be_i32)),
                    )),
                ),
            ))
        }

        fn write_uint_vector<'a, 'b: 'a, W: Write + 'a>(
            &self,
            items: &'b [u32],
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                Value::VectorUInt(items.to_vec(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorUInt),
                either(
                    len.is_reference(),
                    len.write(&self),
                    tuple((
                        Length::Size(items.len() as u32).write(&self),
                        be_u8(fixed_length as u8),
                        all(items.iter().copied().map(be_u32)),
                    )),
                ),
            ))
        }

        fn write_number_vector<'a, 'b: 'a, W: Write + 'a>(
            &self,
            items: &'b [f64],
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                Value::VectorDouble(items.to_vec(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorDouble),
                either(
                    len.is_reference(),
                    len.write(&self),
                    tuple((
                        Length::Size(items.len() as u32).write(&self),
                        be_u8(fixed_length as u8),
                        all(items.iter().copied().map(be_f64)),
                    )),
                ),
            ))
        }

        fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            time: f64,
        ) -> impl SerializeFn<W> + 'a {
            let len = self
                .object_reference_table
                .to_length_store(Value::Date(time, None), 0);

            tuple((
                self.write_type_marker(TypeMarker::Date),
                len.write(&self),
                cond(len.is_size(), be_f64(time)),
            ))
        }

        fn write_integer_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            i: i32,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_type_marker(TypeMarker::Integer),
                self.write_int(i),
            ))
        }

        fn write_byte_array_element<'a, 'b: 'a, W: Write + 'a>(
            &self,
            bytes: &'b [u8],
        ) -> impl SerializeFn<W> + 'a {
            let len = self
                .object_reference_table
                .to_length_store(Value::ByteArray(bytes.to_vec()), bytes.len() as u32);

            tuple((
                self.write_type_marker(TypeMarker::ByteArray),
                len.write(&self),
                cond(len.is_size(), slice(bytes)),
            ))
        }

        fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(
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
                len.write(&self),
                cond(len.is_size(), slice(bytes.as_bytes())),
            ))
        }

        fn write_class_definition<'a, 'b: 'a, W: Write + 'a>(
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
        fn write_trait_reference<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            index: u32,
            children: &'b [Element],
            custom_props: Option<&'b [Element]>,
            def: &'b ClassDefinition,
        ) -> impl SerializeFn<W> + 'a {
            let size = (((index << 1) | 0u32) << 1) | 1u32;

            tuple((
                self.write_int(size as i32),
                cond(def.attributes.contains(Attribute::EXTERNAL), move |out| {
                    if let Some(encoder) = self.external_encoders.get(&def.name) {
                        slice(encoder.encode(custom_props.unwrap(), &Some(def.clone()), self))(out)
                    } else {
                        Err(GenError::NotYetImplemented)
                    }
                }),
                cond(
                    !def.attributes.contains(Attribute::EXTERNAL),
                    tuple((
                        cond(
                            def.attributes.is_empty(),
                            all(children
                                .iter()
                                .filter(move |c| def.static_properties.contains(&c.name))
                                .map(move |e| &e.value)
                                .map(move |e| self.write_value_element(e))),
                        ),
                        cond(
                            def.attributes.contains(Attribute::DYNAMIC),
                            tuple((
                                all(children
                                    .iter()
                                    .filter(move |c| def.static_properties.contains(&c.name))
                                    .map(move |e| &e.value)
                                    .map(move |e| self.write_value_element(e))),
                                all(children
                                    .iter()
                                    .filter(move |c| !def.static_properties.contains(&c.name))
                                    // .map(move |e| &e.value)
                                    .map(move |e| {
                                        tuple((
                                            self.write_byte_string(e.name.as_bytes()),
                                            self.write_value_element(&e.value),
                                        ))
                                    })),
                                self.write_byte_string(&[]),
                            )),
                        ),
                    )),
                ),
            ))
        }

        fn write_object_reference<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            index: u32,
        ) -> impl SerializeFn<W> + 'a {
            let size = (index << 1) | 0u32;
            tuple((self.write_int(size as i32),))
        }

        fn write_object_full<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            custom_props: Option<&'b [Element]>,
            children: &'b [Element],
            def: &'b ClassDefinition,
        ) -> impl SerializeFn<W> + 'a {
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
            let size =
                ((((((def.static_properties.len() as u32) << 2) | (encoding & 0xff) as u32) << 1)
                    | 1u32)
                    << 1)
                    | 1u32;

            tuple((
                self.write_int(size as i32),
                self.write_class_definition(def),
                cond(def.attributes.contains(Attribute::EXTERNAL), move |out| {
                    if let Some(encoder) = self.external_encoders.get(&def.name) {
                        slice(encoder.encode(custom_props.unwrap(), &Some(def.clone()), self))(out)
                    } else {
                        Err(GenError::NotYetImplemented)
                    }
                }),
                cond(
                    !def.attributes.contains(Attribute::EXTERNAL),
                    tuple((
                        cond(
                            def.attributes.is_empty(),
                            all(children
                                .iter()
                                .filter(move |c| def.static_properties.contains(&c.name))
                                .map(move |e| &e.value)
                                .map(move |e| self.write_value_element(e))),
                        ),
                        cond(
                            def.attributes.contains(Attribute::DYNAMIC),
                            tuple((
                                all(children
                                    .iter()
                                    .filter(move |c| def.static_properties.contains(&c.name))
                                    .map(move |e| &e.value)
                                    .map(move |e| self.write_value_element(e))),
                                all(children
                                    .iter()
                                    .filter(move |c| !def.static_properties.contains(&c.name))
                                    // .map(move |e| &e.value)
                                    .map(move |e| {
                                        tuple((
                                            self.write_byte_string(e.name.as_bytes()),
                                            self.write_value_element(&e.value),
                                        ))
                                    })),
                                self.write_byte_string(&[]),
                            )),
                        ),
                    )),
                ),
            ))
        }

        fn write_object_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            children: &'b [Element],
            custom_props: Option<&'b [Element]>,
            class_def: &'b Option<ClassDefinition>,
        ) -> impl SerializeFn<W> + 'a {
            // let mut had_object = self
            //     .object_reference_table
            //     .to_length(SolValue::Object(children.to_vec(), class_def.clone()), 0);
            let had_object = Length::Size(0);

            self.object_reference_table
                .store(Value::Object(children.to_vec(), class_def.clone()));

            move |out| {
                let def = class_def.clone().unwrap_or_default();
                let def2 = def.clone();

                let has_trait = self
                    .trait_reference_table
                    .borrow()
                    .iter()
                    .position(|cd| *cd == def);

                let x = tuple((
                    self.write_type_marker(TypeMarker::Object),
                    cond(had_object.is_reference(), move |out| {
                        self.write_object_reference(had_object.to_position().unwrap() as u32)(out)
                    }),
                    cond(
                        !had_object.is_reference(),
                        tuple((
                            cond(has_trait.is_some(), move |out| {
                                self.write_trait_reference(
                                    has_trait.unwrap() as u32,
                                    children,
                                    custom_props,
                                    &def2,
                                )(out)
                            }),
                            cond(
                                has_trait.is_none(),
                                self.write_object_full(custom_props, children, &def),
                            ),
                        )),
                    ),
                ))(out);

                x
            }
        }

        fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            children: &'b [Rc<Value>],
        ) -> impl SerializeFn<W> + 'a {
            // let mut len = self.object_reference_table.to_length_store(
            //     SolValue::StrictArray(children.to_vec()),
            //     children.len() as u32,
            // );

            //TODO: why is this not a reference
            let len = Length::Size(children.len() as u32);

            //TODO: why does this not offset the cache if StrictArray([]) is saved but always written as Size(0) instead of Ref(n)
            either(
                children.is_empty(),
                tuple((
                    self.write_type_marker(TypeMarker::Array),
                    Length::Size(0).write(&self),
                    self.write_byte_string(&[]), // Empty key
                )),
                tuple((
                    self.write_type_marker(TypeMarker::Array),
                    len.write(&self),
                    cond(
                        len.is_size(),
                        tuple((
                            self.write_byte_string(&[]), // Empty key
                            all(children.iter().map(move |v| self.write_value_element(v))),
                        )),
                    ),
                )),
            )
        }

        fn write_ecma_array_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            dense: &'b [Rc<Value>],
            assoc: &'b [Element],
        ) -> impl SerializeFn<W> + 'a {
            // let mut len = self.object_reference_table.to_length_store(
            //     SolValue::ECMAArray(dense.to_vec(), assoc.clone().to_vec(), assoc.len() as u32),
            //     dense.len() as u32,
            // );

            let len = Length::Size(dense.len() as u32);

            //TODO: would this also work for strict arrays if they have [] for assoc part?
            tuple((
                self.write_type_marker(TypeMarker::Array),
                len.write(&self),
                cond(
                    len.is_size(),
                    tuple((
                        all(assoc.iter().map(move |out| self.write_element(out))),
                        self.write_byte_string(&[]),
                        all(dense.iter().map(move |out| self.write_value_element(out))),
                    )),
                ),
            ))
        }

        fn write_object_vector_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            items: &'b [Rc<Value>],
            type_name: &'b str,
            fixed_length: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length_store(
                Value::VectorObject(items.to_vec(), type_name.to_string(), fixed_length),
                items.len() as u32,
            );

            tuple((
                self.write_type_marker(TypeMarker::VectorObject),
                len.write(&self),
                cond(
                    len.is_size(),
                    tuple((
                        be_u8(fixed_length as u8),
                        self.write_string(type_name),
                        all(items.iter().map(move |i| self.write_value_element(i))),
                    )),
                ),
            ))
        }

        fn write_dictionary_element<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            items: &'b [(Rc<Value>, Rc<Value>)],
            weak_keys: bool,
        ) -> impl SerializeFn<W> + 'a {
            let len = self.object_reference_table.to_length(
                Value::Dictionary(items.to_vec(), weak_keys),
                items.len() as u32,
            );
            self.object_reference_table
                .store(Value::Dictionary(items.to_vec(), weak_keys));

            tuple((
                self.write_type_marker(TypeMarker::Dictionary),
                len.write(&self),
                cond(
                    len.is_size(),
                    tuple((
                        be_u8(weak_keys as u8),
                        all(items.iter().map(move |i| {
                            tuple((
                                self.write_value_element(&i.0),
                                self.write_value_element(&i.1),
                            ))
                        })),
                    )),
                ),
            ))
        }

        pub(crate) fn write_value_element<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            s: &'b Rc<Value>,
        ) -> impl SerializeFn<W> + 'a {
            move |out| self.write_value(s.deref())(out)
        }

        fn write_value<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            s: &'b Value,
        ) -> impl SerializeFn<W> + 'a {
            move |out: WriteContext<W>| match s {
                Value::Number(x) => self.write_number_element(*x)(out),
                Value::Bool(b) => self.write_boolean_element(*b)(out),
                Value::String(s) => self.write_string_element(s)(out),
                Value::Object(children, class_def) => {
                    self.write_object_element(children, None, class_def)(out)
                }
                Value::Null => self.write_null_element()(out),
                Value::Undefined => self.write_undefined_element()(out),
                Value::ECMAArray(dense, elements, _) => {
                    self.write_ecma_array_element(dense, elements)(out)
                }
                Value::StrictArray(children) => self.write_strict_array_element(children)(out),
                Value::Date(time, _tz) => self.write_date_element(*time)(out),
                Value::XML(content, string) => self.write_xml_element(content, *string)(out),
                Value::Integer(i) => self.write_integer_element(*i)(out),
                Value::ByteArray(bytes) => self.write_byte_array_element(bytes)(out),
                Value::VectorInt(items, fixed_length) => {
                    self.write_int_vector(items, *fixed_length)(out)
                }
                Value::VectorUInt(items, fixed_length) => {
                    self.write_uint_vector(items, *fixed_length)(out)
                }
                Value::VectorDouble(items, fixed_length) => {
                    self.write_number_vector(items, *fixed_length)(out)
                }
                Value::VectorObject(items, type_name, fixed_length) => {
                    self.write_object_vector_element(items, type_name, *fixed_length)(out)
                }
                Value::Dictionary(kv, weak_keys) => {
                    self.write_dictionary_element(kv, *weak_keys)(out)
                }

                Value::Custom(elements, dynamic_elements, def) => {
                    self.write_object_element(dynamic_elements, Some(elements), def)(out)
                }
                Value::AMF3(e) => self.write_value_element(e)(out),
                Value::Unsupported => self.write_undefined_element()(out),
            }
        }

        fn write_element<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            element: &'b Element,
        ) -> impl SerializeFn<W> + 'a {
            tuple((
                self.write_string(&element.name),
                self.write_value_element(&element.value),
            ))
        }

        fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            element: &'b Element,
        ) -> impl SerializeFn<W> + 'a {
            tuple((self.write_element(element), slice(PADDING)))
        }

        pub fn write_body<'a, 'b: 'a, W: Write + 'a>(
            &'b self,
            elements: &'b [Element],
        ) -> impl SerializeFn<W> + 'a {
            all(elements
                .iter()
                .map(move |e| self.write_element_and_padding(e)))
        }
    }
}
