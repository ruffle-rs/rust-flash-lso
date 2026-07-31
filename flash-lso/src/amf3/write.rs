//! Handles encoding AMF3

use crate::PADDING;
use crate::amf3::custom_encoder::CustomEncoder;
use crate::amf3::element_cache::ElementCache;
use crate::amf3::length::Length;
use crate::amf3::type_marker::TypeMarker;
use crate::types::{
    Attribute, ClassDefinition, DictionaryEntry, DictionaryObjectValue, Element, ObjectId, Value,
    VectorObjectValue, VectorPrimitiveValue,
};
use crate::write::WriteExt;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io::Result;
use std::io::Write;

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

    object_id_to_reference: RefCell<BTreeMap<ObjectId, (TypeMarker, usize)>>,
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
        writer.write_f64(i)?;
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
        self.object_reference_table
            .occupy(Value::VectorInt(VectorPrimitiveValue {
                values: items.to_vec(),
                fixed_length,
            }));

        self.write_type_marker(writer, TypeMarker::VectorInt)?;
        Length::Size(items.len() as u32).write(writer, self)?;
        writer.write_u8(fixed_length as u8)?;
        for item in items {
            writer.write_i32(*item)?;
        }
        Ok(())
    }

    fn write_uint_vector<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [u32],
        fixed_length: bool,
    ) -> Result<()> {
        self.object_reference_table
            .occupy(Value::VectorUInt(VectorPrimitiveValue {
                values: items.to_vec(),
                fixed_length,
            }));

        self.write_type_marker(writer, TypeMarker::VectorUInt)?;
        Length::Size(items.len() as u32).write(writer, self)?;
        writer.write_u8(fixed_length as u8)?;
        for item in items {
            writer.write_u32(*item)?;
        }
        Ok(())
    }

    fn write_number_vector<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        items: &'b [f64],
        fixed_length: bool,
    ) -> Result<()> {
        self.object_reference_table
            .occupy(Value::VectorDouble(VectorPrimitiveValue {
                values: items.to_vec(),
                fixed_length,
            }));

        self.write_type_marker(writer, TypeMarker::VectorDouble)?;
        Length::Size(items.len() as u32).write(writer, self)?;
        writer.write_u8(fixed_length as u8)?;
        for item in items {
            writer.write_f64(*item)?;
        }
        Ok(())
    }

    fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        time: f64,
    ) -> Result<()> {
        self.object_reference_table.occupy(Value::Date {
            time,
            timezone_or_utc: None,
        });

        self.write_type_marker(writer, TypeMarker::Date)?;
        Length::Size(0).write(writer, self)?;
        writer.write_f64(time)?;
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
        self.object_reference_table
            .occupy(Value::ByteArray(bytes.to_vec()));

        self.write_type_marker(writer, TypeMarker::ByteArray)?;
        Length::Size(bytes.len() as u32).write(writer, self)?;
        writer.write_all(bytes)?;
        Ok(())
    }

    fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(
        &self,
        writer: &mut W,
        bytes: &'b str,
        string: bool,
    ) -> Result<()> {
        self.object_reference_table.occupy(Value::XML {
            value: bytes.to_string(),
            is_string: string,
        });

        if string {
            self.write_type_marker(writer, TypeMarker::XmlString)?;
        } else {
            self.write_type_marker(writer, TypeMarker::Xml)?;
        }

        Length::Size(bytes.len() as u32).write(writer, self)?;
        writer.write_all(bytes.as_bytes())?;
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

    /// The external representation of an externalizable object, via its registered encoder.
    fn write_external_body<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        custom_props: Option<&'b [Element]>,
        def: &'b ClassDefinition,
    ) -> Result<()> {
        let encoder = self.external_encoders.get(&def.name).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("no encoder registered for external class \"{}\"", def.name),
            )
        })?;
        let custom_props = custom_props.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "externalizable value of class \"{}\" has no external elements",
                    def.name
                ),
            )
        })?;
        writer.write_all(&encoder.encode(custom_props, &Some(def.clone()), self))
    }

    /// The sealed property values of an object, in the order its traits declare them, followed
    /// by the dynamic section if the traits are dynamic.
    fn write_object_properties<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        children: &'b [Element],
        def: &'b ClassDefinition,
    ) -> Result<()> {
        for name in &def.static_properties {
            let child = children.iter().find(|c| &c.name == name).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "object of class \"{}\" has no value for static property \"{}\"",
                        def.name, name
                    ),
                )
            })?;
            self.write_value_element(writer, &child.value)?;
        }

        if def.attributes.contains(Attribute::Dynamic) {
            for c in children
                .iter()
                .filter(|c| !def.static_properties.contains(&c.name))
            {
                self.write_byte_string(writer, c.name.as_bytes())?;
                self.write_value_element(writer, &c.value)?;
            }
            self.write_byte_string(writer, &[])?;
        }

        Ok(())
    }

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
            self.write_external_body(writer, custom_props, def)?;
        }

        if !def.attributes.contains(Attribute::External) {
            self.write_object_properties(writer, children, def)?;
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
            self.write_external_body(writer, custom_props, def)?;
        }
        if !def.attributes.contains(Attribute::External) {
            self.write_object_properties(writer, children, def)?;
        }
        Ok(())
    }

    fn write_object_element<'a, 'b: 'a, W: Write + 'a>(
        &'a self,
        writer: &mut W,
        id: ObjectId,
        children: &'b [Element],
        custom_props: Option<&'b [Element]>,
        class_def: &'b Option<ClassDefinition>,
    ) -> Result<()> {
        let had_object = Length::Size(0);
        use crate::types::ObjectValue;

        let obj = Value::Object {
            id,
            data: ObjectValue {
                elements: children.to_vec(),
                class_definition: class_def.clone(),
            },
        };
        let slot = self.object_reference_table.occupy(obj);
        if id != ObjectId::INVALID {
            self.object_id_to_reference
                .borrow_mut()
                .insert(id, (TypeMarker::Object, slot));
        }

        let def = class_def.clone().unwrap_or_default();
        let def2 = def.clone();

        let has_trait = self
            .trait_reference_table
            .borrow()
            .iter()
            .position(|cd| *cd == def);

        self.write_type_marker(writer, TypeMarker::Object)?;
        if let Length::Reference(r) = had_object {
            self.write_object_reference(writer, r as u32)?;
        } else {
            if let Some(has_trait) = has_trait {
                self.write_trait_reference(
                    writer,
                    has_trait as u32,
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
        children: &'b [Value],
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
        dense: &'b [Value],
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
        id: ObjectId,
        items: &'b [Value],
        type_name: &'b str,
        fixed_length: bool,
    ) -> Result<()> {
        let vo = Value::VectorObject {
            id,
            data: VectorObjectValue {
                values: items.to_vec(),
                object_type_name: type_name.to_string(),
                fixed_length,
            },
        };

        let len = self
            .object_reference_table
            .to_length(vo.clone(), items.len() as u32);
        let slot = self.object_reference_table.occupy(vo);
        if id != ObjectId::INVALID {
            self.object_id_to_reference
                .borrow_mut()
                .insert(id, (TypeMarker::VectorObject, slot));
        }

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
        id: ObjectId,
        items: &'b [DictionaryEntry],
        weak_keys: bool,
    ) -> Result<()> {
        let dict = Value::Dictionary {
            id,
            data: DictionaryObjectValue {
                elements: items.to_vec(),
                weak_keys,
            },
        };

        let len = self
            .object_reference_table
            .to_length(dict.clone(), items.len() as u32);
        let slot = self.object_reference_table.occupy(dict);
        if id != ObjectId::INVALID {
            self.object_id_to_reference
                .borrow_mut()
                .insert(id, (TypeMarker::Dictionary, slot));
        }

        self.write_type_marker(writer, TypeMarker::Dictionary)?;
        len.write(writer, self)?;
        if len.is_size() {
            writer.write_u8(weak_keys as u8)?;
            for i in items {
                self.write_value_element(writer, &i.key)?;
                self.write_value_element(writer, &i.value)?;
            }
        }
        Ok(())
    }

    pub(crate) fn write_value_element<'a, 'b: 'a, W: Write + 'a>(
        &'b self,
        writer: &mut W,
        s: &'b Value,
    ) -> Result<()> {
        self.write_value(writer, s)
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
            Value::Object { id, data } => {
                self.write_object_element(writer, *id, &data.elements, None, &data.class_definition)
            }
            Value::Null => self.write_null_element(writer),
            Value::Undefined => self.write_undefined_element(writer),
            Value::ECMAArray { id, data } => {
                let slot = self.object_reference_table.occupy(s.clone());
                if *id != ObjectId::INVALID {
                    self.object_id_to_reference
                        .borrow_mut()
                        .insert(*id, (TypeMarker::Array, slot));
                }

                self.write_ecma_array_element(writer, &data.dense, &data.elements)
            }
            Value::StrictArray { id, values } => {
                let slot = self.object_reference_table.occupy(s.clone());
                if *id != ObjectId::INVALID {
                    self.object_id_to_reference
                        .borrow_mut()
                        .insert(*id, (TypeMarker::Array, slot));
                }

                self.write_strict_array_element(writer, values)
            }
            Value::Date {
                time,
                timezone_or_utc: _,
            } => self.write_date_element(writer, *time),
            Value::XML { value, is_string } => self.write_xml_element(writer, value, *is_string),
            Value::Integer(i) => self.write_integer_element(writer, *i),
            Value::ByteArray(bytes) => self.write_byte_array_element(writer, bytes),
            Value::VectorInt(v) => self.write_int_vector(writer, &v.values, v.fixed_length),
            Value::VectorUInt(v) => self.write_uint_vector(writer, &v.values, v.fixed_length),
            Value::VectorDouble(v) => self.write_number_vector(writer, &v.values, v.fixed_length),
            Value::VectorObject { id, data } => self.write_object_vector_element(
                writer,
                *id,
                &data.values,
                &data.object_type_name,
                data.fixed_length,
            ),
            Value::Dictionary { id, data } => {
                self.write_dictionary_element(writer, *id, &data.elements, data.weak_keys)
            }

            Value::Custom(c) => self.write_object_element(
                writer,
                ObjectId::INVALID,
                &c.dynamic_elements,
                Some(&c.elements),
                &Some(c.class_definition.clone()),
            ),
            Value::AMF3(e) => self.write_value_element(writer, e),
            Value::Unsupported => self.write_undefined_element(writer),
            Value::Reference(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "AMF0 reference in an AMF3 stream",
            )),
            Value::Amf3ObjectReference(id) => {
                let entry = self.object_id_to_reference.borrow().get(id).copied();
                let (ty, r) = entry.ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("unresolvable AMF3 object reference {id:?}"),
                    )
                })?;
                self.write_type_marker(writer, ty)?;
                self.write_object_reference(writer, r as u32)
            }
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
        let mut v = Vec::new();
        e.write_int(&mut v, 0b00101011).expect("Test fail");
        assert_eq!(v, &[0b00101011]);
    }

    #[test]
    fn test_write_4byte_number() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        e.write_int(&mut v, 2097280).expect("Test fail");
        assert_eq!(v, &[0b10000000, 0b11000000, 0b10000000, 0b10000000]);
    }

    #[test]
    fn write_neg_number() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        e.write_int(&mut v, -268435455).expect("Test fail");
        assert_eq!(v, &[192, 128, 128, 1]);
    }
}

#[cfg(test)]
mod write_property_tests {
    use crate::amf3::write::AMF3Encoder;
    use crate::types::{Attribute, ClassDefinition, Element, ObjectId, ObjectValue, Value};

    fn class(
        name: &str,
        props: &[&str],
        attributes: enumset::EnumSet<Attribute>,
    ) -> ClassDefinition {
        ClassDefinition {
            name: name.to_string(),
            attributes,
            static_properties: props.iter().map(|p| p.to_string()).collect(),
        }
    }

    fn object(def: ClassDefinition, children: Vec<Element>) -> Value {
        Value::Object {
            id: ObjectId(1),
            data: ObjectValue {
                elements: children,
                class_definition: Some(def),
            },
        }
    }

    fn int(name: &str, v: i32) -> Element {
        Element {
            name: name.to_string(),
            value: Value::Integer(v),
        }
    }

    /// Sealed values follow the order the traits declare, not the order the children happen to
    /// be in, or a reader assigns them to the wrong properties.
    #[test]
    fn sealed_values_follow_trait_order() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        e.write_value_element(
            &mut v,
            &object(
                class("C", &["a", "b"], Default::default()),
                vec![int("b", 2), int("a", 1)],
            ),
        )
        .expect("Test fail");

        // ...traits, "C", "a", "b", then the values for a and b in that order.
        assert_eq!(&v[v.len() - 4..], &[0x04, 0x01, 0x04, 0x02]);
    }

    /// A duplicate sealed name must not emit two values for one declared property.
    #[test]
    fn duplicate_sealed_names_emit_one_value_each() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        e.write_value_element(
            &mut v,
            &object(
                class("C", &["a"], Default::default()),
                vec![int("a", 1), int("a", 9)],
            ),
        )
        .expect("Test fail");

        assert_eq!(&v[v.len() - 2..], &[0x04, 0x01]);
    }

    /// A declared property with no value cannot be silently skipped: that truncates the object.
    #[test]
    fn missing_sealed_value_errors() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        assert!(
            e.write_value_element(
                &mut v,
                &object(
                    class("C", &["a", "b"], Default::default()),
                    vec![int("a", 1)]
                ),
            )
            .is_err()
        );
    }

    /// Children whose names are declared sealed must not be repeated in the dynamic section.
    #[test]
    fn dynamic_section_excludes_sealed_names() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        e.write_value_element(
            &mut v,
            &object(
                class("C", &["a"], Attribute::Dynamic.into()),
                vec![int("a", 1), int("d", 2)],
            ),
        )
        .expect("Test fail");

        // value of a, then "d" (utf8-vr len 1) + its value, then the empty terminator.
        assert_eq!(
            &v[v.len() - 7..],
            &[0x04, 0x01, 0x03, b'd', 0x04, 0x02, 0x01]
        );
    }
}

#[cfg(test)]
mod write_reference_tests {
    use crate::amf3::write::AMF3Encoder;
    use crate::types::{ObjectId, ObjectValue, Value};

    fn empty_object(id: i64) -> Value {
        Value::Object {
            id: ObjectId(id),
            data: ObjectValue {
                elements: Vec::new(),
                class_definition: None,
            },
        }
    }

    /// A conformant reader allocates a reference slot for every Date, so an object written after
    /// one sits at slot 1 and a back-reference to it must encode as index 1.
    #[test]
    fn payload_values_occupy_a_reference_slot() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();

        e.write_value_element(
            &mut v,
            &Value::Date {
                time: 0.0,
                timezone_or_utc: None,
            },
        )
        .expect("Test fail");
        e.write_value_element(&mut v, &empty_object(7))
            .expect("Test fail");

        let mut r = Vec::new();
        e.write_value_element(&mut r, &Value::Amf3ObjectReference(ObjectId(7)))
            .expect("Test fail");

        assert_eq!(r, &[0x0a, 0x02]);
    }

    /// Equal payloads must not collapse into one slot: Value carries no identity for them, so
    /// each occurrence takes its own.
    #[test]
    fn equal_payload_values_take_separate_slots() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();

        for _ in 0..3 {
            e.write_value_element(&mut v, &Value::ByteArray(vec![1, 2, 3]))
                .expect("Test fail");
        }
        e.write_value_element(&mut v, &empty_object(9))
            .expect("Test fail");

        let mut r = Vec::new();
        e.write_value_element(&mut r, &Value::Amf3ObjectReference(ObjectId(9)))
            .expect("Test fail");

        assert_eq!(r, &[0x0a, 0x06]);
    }

    /// Objects that compare equal must still take separate slots. Every `Value::Custom` is
    /// written with `ObjectId::INVALID`, so two externalizable values of one class are equal;
    /// deduplicating them would skew every later reference index.
    #[test]
    fn equal_objects_take_separate_slots() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();

        for _ in 0..2 {
            e.write_value_element(&mut v, &empty_object(ObjectId::INVALID.0))
                .expect("Test fail");
        }
        e.write_value_element(&mut v, &empty_object(5))
            .expect("Test fail");

        let mut r = Vec::new();
        e.write_value_element(&mut r, &Value::Amf3ObjectReference(ObjectId(5)))
            .expect("Test fail");

        assert_eq!(r, &[0x0a, 0x04]);
    }

    #[test]
    fn unresolvable_object_reference_errors() {
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        assert!(
            e.write_value_element(&mut v, &Value::Amf3ObjectReference(ObjectId(1)))
                .is_err()
        );
    }

    #[test]
    fn amf0_reference_in_an_amf3_stream_errors() {
        use crate::types::Reference;
        let e = AMF3Encoder::default();
        let mut v = Vec::new();
        assert!(
            e.write_value_element(&mut v, &Value::Reference(Reference(0)))
                .is_err()
        );
    }
}
