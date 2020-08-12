use crate::amf3::Length;
use std::cell::RefCell;
use std::fmt::Debug;

pub struct ElementCache<T> {
    cache: RefCell<Vec<T>>,
}

impl<T> Default for ElementCache<T> {
    fn default() -> Self {
        ElementCache {
            cache: RefCell::new(Vec::new()),
        }
    }
}

impl<T: PartialEq + Clone + Debug> ElementCache<T> {
    pub fn has(&self, val: &T) -> bool {
        self.cache.borrow().contains(val)
    }

    pub fn store(&self, val: T) {
        if !self.has(&val) {
            self.cache.borrow_mut().push(val);
        }
    }

    pub fn get_element(&self, index: usize) -> Option<T> {
        self.cache.borrow().get(index).cloned()
    }

    pub fn get_index(&self, val: T) -> Option<usize> {
        self.cache.borrow().iter().position(|i| *i == val)
    }

    pub fn to_length(&self, val: T, length: u32) -> Length {
        println!("Looking up {:?}", val);
        if let Some(i) = self.get_index(val) {
            println!("ref = {}", i);
            Length::Reference(i)
        } else {
            Length::Size(length)
        }
    }

    //TODO: no clone
    pub fn to_length_store(&self, val: T, length: u32) -> Length {
        let len = self.to_length(val.clone(), length);
        self.store(val);
        len
    }
}

impl<T: PartialEq + Clone + Debug> ElementCache<Vec<T>> {
    pub fn store_slice(&self, val: &[T]) {
        self.store(val.to_vec());
    }

    pub fn get_slice_index(&self, val: &[T]) -> Option<usize> {
        self.get_index(val.to_vec())
    }
}
