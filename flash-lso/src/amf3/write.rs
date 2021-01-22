///! Handles encoding AMF3
use crate::amf3::custom_encoder::CustomEncoder;
use crate::amf3::type_marker::TypeMarker;
use crate::amf3::element_cache::ElementCache;
use crate::amf3::length::Length;
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

/// Handles encoding AMF3
#[derive(Default)]
pub struct AMF3Encoder {
    /// The table used to cache repeated byte strings
    pub string_reference_table: ElementCache<Vec<u8>>,
    /// The table used to cache repeated trait definitions
    pub trait_reference_table: RefCell<Vec<ClassDefinition>>,
    /// The table used to cache repeated objects
    pub object_reference_table: ElementCache<Value>,
    /// Encoders used for handling externalized types
    pub external_encoders: HashMap<String, Box<dyn CustomEncoder>>,
}

impl AMF3Encoder {
    pub(crate) fn write_int<'a, 'b: 'a, W: Write + 'a>(&self, i: i32) -> impl SerializeFn<W> + 'a {
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

    fn write_number_element<'a, 'b: 'a, W: Write + 'a>(&self, i: f64) -> impl SerializeFn<W> + 'a {
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

    fn write_date_element<'a, 'b: 'a, W: Write + 'a>(&self, time: f64) -> impl SerializeFn<W> + 'a {
        let len = self
            .object_reference_table
            .to_length_store(Value::Date(time, None), 0);

        tuple((
            self.write_type_marker(TypeMarker::Date),
            len.write(&self),
            cond(len.is_size(), be_f64(time)),
        ))
    }

    fn write_integer_element<'a, 'b: 'a, W: Write + 'a>(&self, i: i32) -> impl SerializeFn<W> + 'a {
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
        let size = ((((((def.static_properties.len() as u32) << 2) | (encoding & 0xff) as u32)
            << 1)
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

    fn write_value<'a, 'b: 'a, W: Write + 'a>(&'b self, s: &'b Value) -> impl SerializeFn<W> + 'a {
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
            Value::Dictionary(kv, weak_keys) => self.write_dictionary_element(kv, *weak_keys)(out),

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

    pub(crate) fn write_body<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        elements: &'b [Element],
    ) -> impl SerializeFn<W> + 'a {
        all(elements
            .iter()
            .map(move |e| self.write_element_and_padding(e)))
    }
}
