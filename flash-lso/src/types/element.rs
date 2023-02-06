use super::Value;
use core::ops::Deref;
use std::rc::Rc;

/// Represent a named element
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// The name of the element
    pub name: String,

    /// The value of the element
    pub value: Rc<Value>,
}

impl Element {
    /// Create a new Element
    #[inline]
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: Rc::new(value.into()),
        }
    }

    /// Get the Value of this element
    pub fn value(&self) -> &Value {
        self.value.deref()
    }

    /// Get the name of this element
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
