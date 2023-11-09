use crate::amf3::length::Length;
use std::cell::RefCell;
use std::fmt::Debug;

/// Abstraction over the Amf3 caching mechanism
#[derive(Clone, Debug)]
pub struct ElementCache<T> {
    cache: RefCell<Vec<T>>,
}

impl<T> Default for ElementCache<T> {
    /// Create a new ElementCache
    fn default() -> Self {
        ElementCache {
            cache: RefCell::new(Vec::new()),
        }
    }
}

impl<T: PartialEq + Clone + Debug> ElementCache<T> {
    /// Check if the cache contains a given element
    #[inline]
    pub(crate) fn has(&self, val: &T) -> bool {
        self.cache.borrow().contains(val)
    }

    /// Add the given item to the cache, if the item already exists will do nothing
    #[inline]
    pub(crate) fn store(&self, val: T) {
        if !self.has(&val) {
            self.cache.borrow_mut().push(val);
        }
    }

    /// Retrieve the item at the given index from the cache
    #[inline]
    pub fn get_element(&self, index: usize) -> Option<T> {
        self.cache.borrow().get(index).cloned()
    }

    /// Retrieve the index for the given value
    #[inline]
    pub(crate) fn get_index(&self, val: T) -> Option<usize> {
        self.cache.borrow().iter().position(|i| *i == val)
    }

    /// Get a Length reference to an item in the cache
    /// If the item exists, will return a `Length::Reference` to the item
    /// If the item does not exist, will return the given size as `Length::Size`
    pub(crate) fn to_length(&self, val: T, length: u32) -> Length {
        if let Some(i) = self.get_index(val) {
            Length::Reference(i)
        } else {
            Length::Size(length)
        }
    }

    /// See #to_length, except will store the given value via #add after retrieving the index (if it does not already exist)
    pub(crate) fn to_length_store(&self, val: T, length: u32) -> Length {
        let len = self.to_length(val.clone(), length);
        self.store(val);
        len
    }
}

impl<T: PartialEq + Clone + Debug> ElementCache<Vec<T>> {
    /// See #store, will convert slices of &\[T\] into Vec<T> before storing
    #[inline]
    #[allow(unused)]
    pub(crate) fn store_slice(&self, val: &[T]) {
        self.store(val.to_vec());
    }

    /// See #get_index, will convert slices of &\[T\] into Vec<T> before retrieving
    #[inline]
    pub fn get_slice_index(&self, val: &[T]) -> Option<usize> {
        self.get_index(val.to_vec())
    }
}
