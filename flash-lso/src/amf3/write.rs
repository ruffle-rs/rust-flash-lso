//! Handles encoding AMF3
use crate::amf3::custom_encoder::CustomEncoder;
use crate::amf3::element_cache::ElementCache;
use crate::amf3::length::Length;
use crate::amf3::type_marker::TypeMarker;
use crate::types::{Attribute, ClassDefinition, Element, Value};
use crate::PADDING;
use byteorder::{BigEndian, WriteBytesExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Result;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;

/// Handles encoding AMF3
#[derive(Default)]
pub struct AMF3Encoder {
    /// The table used to cache repeated byte strings
    string_reference_table: ElementCache<Vec<u8>>,

    /// The table used to cache repeated trait definitions
    trait_reference_table: RefCell<Vec<ClassDefinition>>,

    /// The table used to cache repeated objects
    object_reference_table: ElementCache<Value>,

    /// Encoders used for handling externalized types
    pub external_encoders: HashMap<String, Box<dyn CustomEncoder>>,
}

impl AMF3Encoder {
    #[allow(clippy::unusual_byte_groupings)]
    pub(crate) fn write_int<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        i: i32,
    ) -> Result<()> {
        let n = if i < 0 {
            i + 0b001_0000000_0000000_0000000_00000000
        } else {
            i
        };

        if n > 0x1fffff {
            writer.write_u8(((n >> (7 * 3 + 1)) | 0b10000000) as u8)?;
            writer.write_u8(((n >> (7 * 2 + 1)) | 0b10000000) as u8)?;
            writer.write_u8(((n >> (7 + 1)) | 0b10000000) as u8)?;
            writer.write_u8((n & 0b11111111) as u8)?;
        } else if n > 0x3fff {
            writer.write_u8(((n >> (7 * 2)) | 0b10000000) as u8)?;
            writer.write_u8(((n >> 7) | 0b10000000) as u8)?;
            writer.write_u8((n & 0b01111111) as u8)?;
        } else if n > 0x7f {
            writer.write_u8(((n >> 7) | 0b10000000) as u8)?;
            writer.write_u8((n & 0b01111111) as u8)?;
        } else {
            writer.write_u8((n & 0b01111111) as u8)?;
        }

        Ok(())
    }

    fn write_byte_string<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        s: &'b [u8],
    ) -> Result<()> {
        let len = if !s.is_empty() {
            self.string_reference_table
                .to_length(s.to_vec(), s.len() as u32)
        } else {
            Length::Size(0)
        };

        let only_length = len.is_reference() && !s.is_empty();
        let s_vec = s.to_vec();

        if !s_vec.is_empty() {
            self.string_reference_table.store(s_vec.clone());
        }

        len.write(writer, self)?;
        if !only_length {
            writer.write_all(s)?;
        }

        Ok(())
    }

    fn write_string<'a, 'b: 'a, W: Write + 'a>(&'a self, writer: &mut W, s: &'b str) -> Result<()> {
        self.write_byte_string(writer, s.as_bytes())
    }

    fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        s: TypeMarker,
    ) -> Result<()> {
        writer.write_u8(s as u8)
    }

    fn write_number_element<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        i: f64,
    ) -> Result<()> {
        self.write_type_marker(writer, TypeMarker::Number)?;
        writer.write_f64::<BigEndian>(i)?;
        Ok(())
    }

    fn write_boolean_element<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        b: bool,
    ) -> Result<()> {
        if b {
            self.write_type_marker(writer, TypeMarker::True)
        } else {
            self.write_type_marker(writer, TypeMarker::False)
        }
    }

    fn write_string_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        s: &'b str,
    ) -> Result<()> {
        self.write_type_marker(writer, TypeMarker::String)?;
        self.write_byte_string(writer, s.as_bytes())?;
        Ok(())
    }

    fn write_null_element<'a, 'b: 'a, W: Write + 'a>(&self, writer: &mut W) -> Result<()> {
        self.write_type_marker(writer, TypeMarker::Null)
    }

    fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>(&self, writer: &mut W) -> Result<()> {
        self.write_type_marker(writer, TypeMarker::Undefined)
    }

    fn write_int_vector<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [i32],
        fixed_length: bool,
    ) -> Result<()> {
        let len = self.object_reference_table.to_length(
            Value::VectorInt(items.to_vec(), fixed_length),
            items.len() as u32,
        );

        self.write_type_marker(writer, TypeMarker::VectorInt)?;
        if len.is_reference() {
            len.write(writer, self)?;
        } else {
            Length::Size(items.len() as u32).write(writer, self)?;
            writer.write_u8(fixed_length as u8)?;
            for item in items {
                writer.write_i32::<BigEndian>(*item)?;
            }
        }
        Ok(())
    }

    fn write_uint_vector<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [u32],
        fixed_length: bool,
    ) -> Result<()> {
        let len = self.object_reference_table.to_length(
            Value::VectorUInt(items.to_vec(), fixed_length),
            items.len() as u32,
        );

        self.write_type_marker(writer, TypeMarker::VectorUInt)?;
        if len.is_reference() {
            len.write(writer, self)?;
        } else {
            Length::Size(items.len() as u32).write(writer, self)?;
            writer.write_u8(fixed_length as u8)?;
            for item in items {
                writer.write_u32::<BigEndian>(*item)?;
            }
        }
        Ok(())
    }

    fn write_number_vector<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [f64],
        fixed_length: bool,
    ) -> Result<()> {
        let len = self.object_reference_table.to_length(
            Value::VectorDouble(items.to_vec(), fixed_length),
            items.len() as u32,
        );

        self.write_type_marker(writer, TypeMarker::VectorDouble)?;
        if len.is_reference() {
            len.write(writer, self)?;
        } else {
            Length::Size(items.len() as u32).write(writer, self)?;
            writer.write_u8(fixed_length as u8)?;
            for item in items {
                writer.write_f64::<BigEndian>(*item)?;
            }
        }
        Ok(())
    }

    fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        time: f64,
    ) -> Result<()> {
        let len = self
            .object_reference_table
            .to_length(Value::Date(time, None), 0);

        self.write_type_marker(writer, TypeMarker::Date)?;
        len.write(writer, self)?;
        if len.is_size() {
            writer.write_f64::<BigEndian>(time)?;
        }
        Ok(())
    }

    fn write_integer_element<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        i: i32,
    ) -> Result<()> {
        self.write_type_marker(writer, TypeMarker::Integer)?;
        self.write_int(writer, i)?;
        Ok(())
    }

    fn write_byte_array_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        bytes: &'b [u8],
    ) -> Result<()> {
        let len = self
            .object_reference_table
            .to_length(Value::ByteArray(bytes.to_vec()), bytes.len() as u32);

        self.write_type_marker(writer, TypeMarker::ByteArray)?;
        len.write(writer, self)?;
        if len.is_size() {
            writer.write_all(bytes)?;
        }
        Ok(())
    }

    fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        bytes: &'b str,
        string: bool,
    ) -> Result<()> {
        let len = Length::Size(bytes.len() as u32);

        if string {
            self.write_type_marker(writer, TypeMarker::XmlString)?;
        } else {
            self.write_type_marker(writer, TypeMarker::Xml)?;
        }

        len.write(writer, self)?;
        if len.is_size() {
            writer.write_all(bytes.as_bytes())?;
        }
        Ok(())
    }

    fn write_class_definition<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        class_def: &'b ClassDefinition,
    ) -> Result<()> {
        self.write_byte_string(writer, class_def.name.as_bytes())?;
        for p in &class_def.static_properties {
            self.write_string(writer, p)?;
        }
        Ok(())
    }

    //TODO: conds should be common somehwere
    fn write_trait_reference<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        index: u32,
        children: &'b [Element],
        custom_props: Option<&'b [Element]>,
        def: &'b ClassDefinition,
    ) -> Result<()> {
        #[allow(clippy::identity_op)]
        let size = (((index << 1) | 0u32) << 1) | 1u32;

        self.write_int(writer, size as i32)?;
        if def.attributes.contains(Attribute::External) {
            if let Some(encoder) = self.external_encoders.get(&def.name) {
                writer.write_all(&encoder.encode(
                    custom_props.unwrap(),
                    &Some(def.clone()),
                    self,
                ))?;
            } else {
                unimplemented!();
            }
        }

        if !def.attributes.contains(Attribute::External) {
            if def.attributes.is_empty() {
                for c in children {
                    if def.static_properties.contains(&c.name) {
                        self.write_value_element(writer, &c.value)?;
                    }
                }
            }

            if def.attributes.contains(Attribute::Dynamic) {
                for c in children {
                    if def.static_properties.contains(&c.name) {
                        self.write_value_element(writer, &c.value)?;
                    }
                }

                for c in children {
                    if !def.static_properties.contains(&c.name) {
                        self.write_byte_string(writer, c.name.as_bytes())?;
                        self.write_value_element(writer, &c.value)?;
                    }
                }

                self.write_byte_string(writer, &[])?;
            }
        }
        Ok(())
    }

    fn write_object_reference<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        index: u32,
    ) -> Result<()> {
        #[allow(clippy::identity_op)]
        let size = (index << 1) | 0u32;
        self.write_int(writer, size as i32)
    }

    fn write_object_full<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        custom_props: Option<&'b [Element]>,
        children: &'b [Element],
        def: &'b ClassDefinition,
    ) -> Result<()> {
        let is_external = def.attributes.contains(Attribute::External);
        let is_dynamic = def.attributes.contains(Attribute::Dynamic);

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

        self.trait_reference_table.borrow_mut().push(def.clone());
        self.write_int(writer, size as i32)?;
        self.write_class_definition(writer, def)?;
        if def.attributes.contains(Attribute::External) {
            if let Some(encoder) = self.external_encoders.get(&def.name) {
                writer.write_all(&encoder.encode(
                    custom_props.unwrap(),
                    &Some(def.clone()),
                    self,
                ))?;
            } else {
                unimplemented!();
            }
        }
        if !def.attributes.contains(Attribute::External) {
            if def.attributes.is_empty() {
                for c in children {
                    if def.static_properties.contains(&c.name) {
                        self.write_value_element(writer, &c.value)?;
                    }
                }
            }

            if def.attributes.contains(Attribute::Dynamic) {
                for c in children {
                    if def.static_properties.contains(&c.name) {
                        self.write_value_element(writer, &c.value)?;
                    }
                }
                for c in children {
                    if !def.static_properties.contains(&c.name) {
                        self.write_byte_string(writer, c.name.as_bytes())?;
                        self.write_value_element(writer, &c.value)?;
                    }
                }
                self.write_byte_string(writer, &[])?;
            }
        }
        Ok(())
    }

    fn write_object_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        children: &'b [Element],
        custom_props: Option<&'b [Element]>,
        class_def: &'b Option<ClassDefinition>,
    ) -> Result<()> {
        let had_object = Length::Size(0);

        self.object_reference_table
            .store(Value::Object(children.to_vec(), class_def.clone()));

        let def = class_def.clone().unwrap_or_default();
        let def2 = def.clone();

        let has_trait = self
            .trait_reference_table
            .borrow()
            .iter()
            .position(|cd| *cd == def);

        self.write_type_marker(writer, TypeMarker::Object)?;
        if had_object.is_reference() {
            self.write_object_reference(writer, had_object.as_position().unwrap() as u32)?;
        }
        if !had_object.is_reference() {
            if has_trait.is_some() {
                self.write_trait_reference(
                    writer,
                    has_trait.unwrap() as u32,
                    children,
                    custom_props,
                    &def2,
                )?;
            }
            if has_trait.is_none() {
                self.write_object_full(writer, custom_props, children, &def)?;
            }
        }

        Ok(())
    }

    fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        children: &'b [Rc<Value>],
    ) -> Result<()> {
        //TODO: why is this not a reference
        let len = Length::Size(children.len() as u32);

        //TODO: why does this not offset the cache if StrictArray([]) is saved but always written as Size(0) instead of Ref(n)
        if children.is_empty() {
            self.write_type_marker(writer, TypeMarker::Array)?;
            Length::Size(0).write(writer, self)?;
            self.write_byte_string(writer, &[])?; // Empty key
        } else {
            self.write_type_marker(writer, TypeMarker::Array)?;
            len.write(writer, self)?;
            if len.is_size() {
                self.write_byte_string(writer, &[])?; // Empty key
                for v in children {
                    self.write_value_element(writer, v)?;
                }
            }
        }

        Ok(())
    }

    fn write_ecma_array_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        dense: &'b [Rc<Value>],
        assoc: &'b [Element],
    ) -> Result<()> {
        let len = Length::Size(dense.len() as u32);

        //TODO: would this also work for strict arrays if they have [] for assoc part?

        self.write_type_marker(writer, TypeMarker::Array)?;
        len.write(writer, self)?;
        if len.is_size() {
            for out in assoc {
                self.write_element(writer, out)?;
            }
            self.write_byte_string(writer, &[])?;
            for out in dense {
                self.write_value_element(writer, out)?;
            }
        }
        Ok(())
    }

    fn write_object_vector_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [Rc<Value>],
        type_name: &'b str,
        fixed_length: bool,
    ) -> Result<()> {
        let len = self.object_reference_table.to_length(
            Value::VectorObject(items.to_vec(), type_name.to_string(), fixed_length),
            items.len() as u32,
        );

        self.write_type_marker(writer, TypeMarker::VectorObject)?;
        len.write(writer, self)?;
        if len.is_size() {
            writer.write_u8(fixed_length as u8)?;
            self.write_string(writer, type_name)?;
            for i in items {
                self.write_value_element(writer, i)?;
            }
        }
        Ok(())
    }

    fn write_dictionary_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [(Rc<Value>, Rc<Value>)],
        weak_keys: bool,
    ) -> Result<()> {
        let len = self.object_reference_table.to_length(
            Value::Dictionary(items.to_vec(), weak_keys),
            items.len() as u32,
        );
        self.object_reference_table
            .store(Value::Dictionary(items.to_vec(), weak_keys));

        self.write_type_marker(writer, TypeMarker::Dictionary)?;
        len.write(writer, self)?;
        if len.is_size() {
            writer.write_u8(weak_keys as u8)?;
            for i in items {
                self.write_value_element(writer, &i.0)?;
                self.write_value_element(writer, &i.1)?;
            }
        }
        Ok(())
    }

    pub(crate) fn write_value_element<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        writer: &mut W,
        s: &'b Rc<Value>,
    ) -> Result<()> {
        self.write_value(writer, s.deref())
    }

    fn write_value<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        writer: &mut W,
        s: &'b Value,
    ) -> Result<()> {
        match s {
            Value::Number(x) => self.write_number_element(writer, *x),
            Value::Bool(b) => self.write_boolean_element(writer, *b),
            Value::String(s) => self.write_string_element(writer, s),
            Value::Object(children, class_def) => {
                self.write_object_element(writer, children, None, class_def)
            }
            Value::Null => self.write_null_element(writer),
            Value::Undefined => self.write_undefined_element(writer),
            Value::ECMAArray(dense, elements, _) => {
                self.write_ecma_array_element(writer, dense, elements)
            }
            Value::StrictArray(children) => self.write_strict_array_element(writer, children),
            Value::Date(time, _tz) => self.write_date_element(writer, *time),
            Value::XML(content, string) => self.write_xml_element(writer, content, *string),
            Value::Integer(i) => self.write_integer_element(writer, *i),
            Value::ByteArray(bytes) => self.write_byte_array_element(writer, bytes),
            Value::VectorInt(items, fixed_length) => {
                self.write_int_vector(writer, items, *fixed_length)
            }
            Value::VectorUInt(items, fixed_length) => {
                self.write_uint_vector(writer, items, *fixed_length)
            }
            Value::VectorDouble(items, fixed_length) => {
                self.write_number_vector(writer, items, *fixed_length)
            }
            Value::VectorObject(items, type_name, fixed_length) => {
                self.write_object_vector_element(writer, items, type_name, *fixed_length)
            }
            Value::Dictionary(kv, weak_keys) => {
                self.write_dictionary_element(writer, kv, *weak_keys)
            }

            Value::Custom(elements, dynamic_elements, def) => {
                self.write_object_element(writer, dynamic_elements, Some(elements), def)
            }
            Value::AMF3(e) => self.write_value_element(writer, e),
            Value::Unsupported => self.write_undefined_element(writer),
            Value::Reference(_) => unimplemented!(),
        }
    }

    fn write_element<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        writer: &mut W,
        element: &'b Element,
    ) -> Result<()> {
        self.write_string(writer, &element.name)?;
        self.write_value_element(writer, &element.value)?;
        Ok(())
    }

    fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        writer: &mut W,
        element: &'b Element,
    ) -> Result<()> {
        self.write_element(writer, element)?;
        writer.write_all(&PADDING)?;
        Ok(())
    }

    pub(crate) fn write_body<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        writer: &mut W,
        elements: &'b [Element],
    ) -> Result<()> {
        for e in elements {
            self.write_element_and_padding(writer, e)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod write_number_tests {
    use crate::amf3::write::AMF3Encoder;

    #[test]
    fn test_write_1byte_number() {
        let e = AMF3Encoder::default();
        let mut v = vec![];
        e.write_int(&mut v, 0b00101011).unwrap();
        assert_eq!(v, &[0b00101011]);
    }

    #[test]
    fn test_write_4byte_number() {
        let e = AMF3Encoder::default();
        let mut v = vec![];
        e.write_int(&mut v, 2097280).unwrap();
        assert_eq!(v, &[0b10000000, 0b11000000, 0b10000000, 0b10000000]);
    }

    #[test]
    fn write_neg_number() {
        let e = AMF3Encoder::default();
        let mut v = vec![];
        e.write_int(&mut v, -268435455).unwrap();
        assert_eq!(v, &[192, 128, 128, 1]);
    }
}
