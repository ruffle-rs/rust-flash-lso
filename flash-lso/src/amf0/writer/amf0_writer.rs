use crate::amf0::writer::strict_array_writer::StrictArrayWriter;
use crate::types::{AMFVersion, Element, Lso, Reference, Value};
use std::{collections::BTreeMap, rc::Rc};

use super::{ArrayWriter, CacheKey, ObjWriter, ObjectWriter, TypedObjectWriter};

/// A writer for Amf0 encoded data
#[derive(Default)]
pub struct Amf0Writer {
    /// The elements present at the root level
    pub(crate) elements: Vec<Element>,

    /// The current reference number
    pub(crate) ref_num: u16,

    /// The reference cache, allows writing self-referential data
    pub(crate) cache: BTreeMap<CacheKey, Reference>,
}

impl<'a> ObjWriter<'a> for Amf0Writer {
    fn add_element(&mut self, name: &str, s: Value, inc_ref: bool) {
        if inc_ref {
            self.ref_num += 1;
        }

        self.elements.push(Element::new(name, Rc::new(s)))
    }

    fn object<'c: 'a, 'd>(
        &'d mut self,
        cache_key: CacheKey,
    ) -> (Option<ObjectWriter<'d, 'c>>, Reference)
    where
        'a: 'c,
        'a: 'd,
    {
        if let Some(existing_ref) = self.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new object reference
            let r = Reference(self.ref_num);
            self.ref_num += 1;

            // Cache this new object
            self.cache.insert(cache_key, r);

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
        if let Some(existing_ref) = self.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new array reference
            let r = Reference(self.ref_num);
            self.ref_num += 1;

            // Cache this new array
            self.cache.insert(cache_key, r);

            // Return the writer and the reference
            (
                Some(ArrayWriter {
                    elements: Vec::new(),
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
        if let Some(existing_ref) = self.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new typed object reference
            let r = Reference(self.ref_num);
            self.ref_num += 1;

            // Cache this new typed object
            self.cache.insert(cache_key, r);

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

    fn make_reference(&mut self) -> Reference {
        let ref_num = Reference(self.ref_num);
        self.ref_num += 1;
        ref_num
    }

    fn cache_get(&mut self, cache_key: &CacheKey) -> Option<Reference> {
        self.cache.get(cache_key).copied()
    }

    fn cache_add(&mut self, cache_key: CacheKey, reference: Reference) {
        self.cache.insert(cache_key, reference);
    }
}

impl Amf0Writer {
    /// Produce an `Lso` with the given name
    pub fn commit_lso(self, name: &str) -> Lso {
        Lso::new(self.elements, name, AMFVersion::AMF0)
    }
}
