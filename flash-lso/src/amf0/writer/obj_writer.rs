use crate::types::{Value, Reference};

use super::{CacheKey, ObjectWriter, ArrayWriter};

/// A trait of common functions between writers
pub trait ObjWriter<'a> {
    /// Add an element to this object
    fn add_element(&mut self, name: &str, s: Value, inc_ref: bool);

    /// Create a writer that can serialize an object
    ///
    /// If an object with the same `cache_key` has already been written, then this will return `None` for the Writer and the existing reference
    /// If this key is unique, then both a Writer and a Reference will be returned
    fn object<'c: 'a, 'd>(&'d mut self, cache_key: CacheKey) -> (Option<ObjectWriter<'d, 'c>>, Reference) where 'a: 'c, 'a: 'd;

    /// Create a writer that can serialize an array
    ///
    /// If an object with the same `cache_key` has already been written, then this will return `None` for the Writer and the existing reference
    /// If this key is unique, then both a Writer and a Reference will be returned
    fn array<'c: 'a, 'd>(&'d mut self, cache_key: CacheKey) -> (Option<ArrayWriter<'d, 'c>>, Reference) where 'a: 'c, 'a: 'd;
    //Typed objects can also be sent cached

    /// Write a string
    fn string(&mut self, name: &str, s: &str) {
        self.add_element(name, Value::String(s.to_string()), true);
    }

    /// Write a number
    fn number(&mut self, name: &str, s: f64) {
        self.add_element(name, Value::Number(s), true);
    }

    /// Write a reference
    fn reference(&mut self, name: &str, v: Reference) {
        self.add_element(name, Value::Reference(v), false);
    }

    /// Write an undefined
    fn undefined(&mut self, name: &str) {
        self.add_element(name, Value::Undefined, true);
    }

    /// Write a null
    fn null(&mut self, name: &str) {
        self.add_element(name, Value::Null, true);
    }

    /// Write a bool
    fn bool(&mut self, name: &str, v: bool) {
        self.add_element(name, Value::Bool(v), true);
    }

    /// Write a date
    fn date(&mut self, name: &str, ms: f64, tz: Option<u16>) {
        self.add_element(name, Value::Date(ms, tz), true)
    }

    /// Write a XML
    fn xml(&mut self, name: &str, v: &str, s: bool) {
        self.add_element(name, Value::XML(v.to_string(), s), true);
    }

    /// Create a reference in the root
    fn make_reference(&mut self) -> Reference;
    fn cache_get(&mut self, cache_key: &CacheKey) -> Option<Reference>;
    fn cache_add(&mut self, cache_key: CacheKey, reference: Reference);
}