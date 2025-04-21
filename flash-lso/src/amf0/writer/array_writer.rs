use std::rc::Rc;

use crate::types::{Element, ObjectId, Reference, Value};

use super::{CacheKey, ObjWriter, ObjectWriter};

/// A writer for encoding `ECMAArray` contents
pub struct ArrayWriter<'a, 'b> {
    /// The elements in this array
    pub(crate) elements: Vec<Element>,

    /// The parent of this writer
    pub(crate) parent: &'a mut dyn ObjWriter<'b>,
}

impl<'a> ObjWriter<'a> for ArrayWriter<'a, '_> {
    fn add_element(&mut self, name: &str, s: Value, inc_ref: bool) {
        if inc_ref {
            self.make_reference();
        }

        self.elements
            .push(Element::new(name.to_string(), Rc::new(s)));
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
                    parent: self,
                }),
                r,
            )
        }
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

impl ArrayWriter<'_, '_> {
    /// Finalize this array, adding it to it's parent
    /// If this is not called, the array will not be added
    pub fn commit<T: AsRef<str>>(self, name: T, length: u32) {
        self.parent.add_element(
            name.as_ref(),
            Value::ECMAArray(ObjectId::INVALID, Vec::new(), self.elements, length),
            false,
        );
    }
}
