use std::collections::BTreeMap;

use crate::types::{Element, Reference, Value, Lso, AMFVersion};

use super::{CacheKey, ObjWriter, ObjectWriter, ArrayWriter};

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

        self.elements.push(Element::new(name, s))
    }

    fn object<'b, 'c: 'b>(&'c mut self, cache_key: CacheKey) -> (Option<ObjectWriter<'b>>, Reference) where 'a: 'b {
        if let Some(existing_ref) = self.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new object reference
            let r = Reference(self.ref_num);
            self.ref_num += 1;

            // Cache this new object
            self.cache.insert(cache_key, r);

            // Return the writer and the reference
            (Some(ObjectWriter {
                elements: Vec::new(),
                parent: self
            }), r)
        }
    }

    fn array<'b, 'c: 'b>(&'c mut self, cache_key: CacheKey) -> (Option<ArrayWriter<'b>>, Reference) where 'a: 'b {
        if let Some(existing_ref) = self.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new array reference
            let r = Reference(self.ref_num);
            self.ref_num += 1;

            // Cache this new array
            self.cache.insert(cache_key, r);

            // Return the writer and the reference
            (Some(ArrayWriter {
                elements: Vec::new(),
                parent: self
            }), r)
        }
    }
}

impl Amf0Writer {
    /// Produce an `Lso` with the given name
    pub fn commit_lso(self, name: &str) -> Lso {
        Lso::new(self.elements, name, AMFVersion::AMF0)
    }
}