use crate::types::{Element, Value, Reference};

use super::{Amf0Writer, ObjWriter, CacheKey, ArrayWriter};

/// A writer for encoding the contents of a child object
pub struct ObjectWriter<'a> {
    /// The elements of this object
    pub(crate) elements: Vec<Element>,

    /// The parent of this writer (contains cache and reference count)
    pub(crate) parent: &'a mut Amf0Writer,
}

impl<'a> ObjWriter<'a> for ObjectWriter<'a> {
    fn add_element(&mut self, name: &str, s: Value, inc_ref: bool) {
        if inc_ref {
            self.parent.ref_num += 1;
        }

        self.elements.push(Element::new(name, s));
    }

    fn object<'b, 'c: 'b>(&'c mut self, cache_key: CacheKey) -> (Option<ObjectWriter<'b>>, Reference) where 'a: 'b {
        if let Some(existing_ref) = self.parent.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new object reference
            let r = Reference(self.parent.ref_num);
            self.parent.ref_num += 1;

            // Cache this new object
            self.parent.cache.insert(cache_key, r);

            // Return the writer and the reference
            (Some(ObjectWriter {
                elements: Vec::new(),
                parent: self.parent
            }), r)
        }
    }

    fn array<'b, 'c: 'b>(&'c mut self, cache_key: CacheKey) -> (Option<ArrayWriter<'b>>, Reference) where 'a: 'b {
        if let Some(existing_ref) = self.parent.cache.get(&cache_key) {
            (None, *existing_ref)
        } else {
            // Create new array reference
            let r = Reference(self.parent.ref_num);
            self.parent.ref_num += 1;

            // Cache this new array
            self.parent.cache.insert(cache_key, r);

            // Return the writer and the reference
            (Some(ArrayWriter {
                elements: Vec::new(),
                parent: self.parent
            }), r)
        }
    }
}

impl<'a> ObjectWriter<'a> {
    /// Finalize this object, adding it to it's parent
    /// If this is not called, the object will not be added
    pub fn commit<T: AsRef<str>>(self, name: T) {
        self.parent.elements.push(Element::new(name.as_ref().to_string(), Value::Object(self.elements, None)))
    }
}