use crate::types::{Element, Value, Reference};

use super::{Amf0Writer, ObjWriter, CacheKey, ObjectWriter};

/// A writer for encoding `ECMAArray` contents
pub struct ArrayWriter<'a> {
    /// The elements in this array
    pub(crate) elements: Vec<Element>,

    /// The parent of this writer
    pub(crate) parent: &'a mut Amf0Writer,
}

impl<'a> ObjWriter<'a> for ArrayWriter<'a> {
    fn add_element(&mut self, name: &str, s: Value, inc_ref: bool) {
        if inc_ref {
            self.parent.ref_num += 1;
        }

        self.elements.push(Element::new(name.to_string(), s));
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

impl<'a> ArrayWriter<'a> {
    /// Finalize this array, adding it to it's parent
    /// If this is not called, the array will not be added
    pub fn commit<T: AsRef<str>>(self, name: T, length: u32) {
        self.parent.elements.push(Element::new(name.as_ref().to_string(), Value::ECMAArray(Vec::new(), self.elements, length)));
    }
}