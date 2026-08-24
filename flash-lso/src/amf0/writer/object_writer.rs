use crate::amf0::writer::strict_array_writer::StrictArrayWriter;
use crate::types::{Element, ObjectId, ObjectValue, Reference, Value};

use super::{ArrayWriter, CacheKey, ObjWriter, TypedObjectWriter};

/// A writer for encoding the contents of a child object
pub struct ObjectWriter<'a, 'b> {
    /// The elements of this object
    pub(crate) elements: Vec<Element>,

    /// The parent of this writer
    pub(crate) parent: &'a mut dyn ObjWriter<'b>,
}

impl<'a> ObjWriter<'a> for ObjectWriter<'a, '_> {
    fn add_element(&mut self, name: &str, s: Value) {
        self.elements.push(Element::new(name, s));
    }

    fn object<'c: 'a, 'd>(
        &'d mut self,
        cache_key: CacheKey,
    ) -> (Option<ObjectWriter<'d, 'c>>, Reference)
    where
        'a: 'c,
        'a: 'd,
    {
        if let Some(existing_ref) = self.cache_get(&cache_key) {
            (None, existing_ref)
        } else {
            let r = self.make_reference();

            // Cache this new object
            self.cache_add(cache_key, r);

            // Return the writer and the reference
            (
                Some(ObjectWriter {
                    elements: Vec::new(),
                    parent: self,
                }),
                r,
            )
        }
    }

    fn array<'c: 'a, 'd>(
        &'d mut self,
        cache_key: CacheKey,
    ) -> (Option<ArrayWriter<'d, 'c>>, Reference)
    where
        'a: 'c,
        'a: 'd,
    {
        if let Some(existing_ref) = self.cache_get(&cache_key) {
            (None, existing_ref)
        } else {
            let r = self.make_reference();

            // Cache this new array
            self.cache_add(cache_key, r);

            // Return the writer and the reference
            (
                Some(ArrayWriter {
                    elements: Vec::new(),
                    length: 0,
                    parent: self,
                }),
                r,
            )
        }
    }

    fn strict_array<'c: 'a, 'd>(
        &'d mut self,
        cache_key: CacheKey,
    ) -> (Option<StrictArrayWriter<'d, 'c>>, Reference)
    where
        'a: 'c,
        'a: 'd,
    {
        if let Some(existing_ref) = self.cache_get(&cache_key) {
            (None, existing_ref)
        } else {
            let r = self.make_reference();

            // Cache this new array
            self.cache_add(cache_key, r);

            // Return the writer and the reference
            (
                Some(StrictArrayWriter {
                    values: Vec::new(),
                    parent: self,
                }),
                r,
            )
        }
    }

    fn typed_object<'c: 'a, 'd>(
        &'d mut self,
        class_name: &str,
        cache_key: CacheKey,
    ) -> (Option<TypedObjectWriter<'d, 'c>>, Reference)
    where
        'a: 'c,
        'a: 'd,
    {
        if let Some(existing_ref) = self.cache_get(&cache_key) {
            (None, existing_ref)
        } else {
            let r = self.make_reference();

            // Cache this new typed object
            self.cache_add(cache_key, r);

            // Return the writer and the reference
            (
                Some(TypedObjectWriter {
                    class_name: class_name.to_string(),
                    elements: Vec::new(),
                    parent: self,
                }),
                r,
            )
        }
    }

    fn commit(self, name: &str) {
        //TODO: this doesn't work for multi level nesting
        self.parent.add_element(
            name,
            Value::Object {
                id: ObjectId::INVALID,
                data: ObjectValue {
                    elements: self.elements,
                    class_definition: None,
                },
            },
        );
    }

    fn make_reference(&mut self) -> Reference {
        self.parent.make_reference()
    }

    fn cache_get(&mut self, cache_key: &CacheKey) -> Option<Reference> {
        self.parent.cache_get(cache_key)
    }

    fn cache_add(&mut self, cache_key: CacheKey, reference: Reference) {
        self.parent.cache_add(cache_key, reference);
    }
}

#[cfg(test)]
mod tests {
    use crate::amf0::writer::{Amf0Writer, ObjWriter};
    use crate::types::Value;

    #[test]
    fn test_dates_are_referenced() {
        let mut writer = Amf0Writer::default();
        let (aw, _) = writer.array(1.into());
        if let Some(mut aw) = aw {
            aw.string("foo", "bar");
            aw.string("bar", "baz");
            aw.date(2.into(), "date1", 0.0f64, None);
            aw.date(2.into(), "date1", 0.0f64, None);
            aw.commit("arr");
        }
        let lso = writer.commit_lso("Lso");

        assert!(matches!(
            lso.body.get(0).unwrap().value,
            Value::ECMAArray { .. }
        ));
        if let Value::ECMAArray { id: _, data } = &lso.body.get(0).unwrap().value {
            assert_eq!(
                data.elements.first().unwrap().value,
                Value::String("bar".to_string())
            );
            assert_eq!(
                data.elements.iter().nth(1).unwrap().value,
                Value::String("baz".to_string())
            );
            assert!(matches!(
                data.elements.iter().nth(2).unwrap().value,
                Value::Date { .. }
            ));
            assert!(matches!(
                data.elements.iter().nth(3).unwrap().value,
                Value::Reference(crate::types::Reference(1))
            ));
        }
    }

    #[test]
    fn test_xml_is_referenced() {
        let mut writer = Amf0Writer::default();
        let (aw, _) = writer.array(1.into());
        if let Some(mut aw) = aw {
            aw.xml(2.into(), "xml1", "<1></1>", true);
            aw.xml(2.into(), "xml2", "<1></1>", true);
            aw.xml(3.into(), "xml3", "<2></2>", true);
            aw.xml(3.into(), "xml4", "<2></2>", true);
            aw.commit("arr");
        }
        let lso = writer.commit_lso("Lso");

        assert!(matches!(
            lso.body.get(0).unwrap().value,
            Value::ECMAArray { .. }
        ));
        if let Value::ECMAArray { id: _, data } = &lso.body.get(0).unwrap().value {
            assert!(matches!(
                data.elements.iter().nth(0).unwrap().value,
                Value::XML { .. }
            ));
            assert!(matches!(
                data.elements.iter().nth(1).unwrap().value,
                Value::Reference(crate::types::Reference(1))
            ));
            assert!(matches!(
                data.elements.iter().nth(2).unwrap().value,
                Value::XML { .. }
            ));
            assert!(matches!(
                data.elements.iter().nth(3).unwrap().value,
                Value::Reference(crate::types::Reference(2))
            ));
        }
    }
}
