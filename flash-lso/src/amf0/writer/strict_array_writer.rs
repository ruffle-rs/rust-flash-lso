use std::rc::Rc;

use crate::types::{ObjectId, Reference, Value};

use super::{ArrayWriter, CacheKey, ObjWriter, ObjectWriter, TypedObjectWriter};

/// A writer for encoding `StrictArray` contents
pub struct StrictArrayWriter<'a, 'b> {
    /// The values in this array
    pub(crate) values: Vec<Rc<Value>>,

    /// The parent of this writer
    pub(crate) parent: &'a mut dyn ObjWriter<'b>,
}

impl<'a> ObjWriter<'a> for StrictArrayWriter<'a, '_> {
    fn add_element(&mut self, _name: &str, s: Value, inc_ref: bool) {
        if inc_ref {
            self.make_reference();
        }

        self.values.push(Rc::new(s));
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

impl StrictArrayWriter<'_, '_> {
    /// Finalise this array, adding it to it's parent
    /// If this is not called, the array will not be added
    pub fn commit<T: AsRef<str>>(self, name: T) {
        self.parent.add_element(
            name.as_ref(),
            Value::StrictArray(ObjectId::INVALID, self.values),
            false,
        );
    }
}
