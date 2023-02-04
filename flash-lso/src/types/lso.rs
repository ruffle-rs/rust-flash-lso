use super::{Header, Element, AMFVersion};

/// A container for lso files
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Lso {
    /// The header of this lso
    pub header: Header,

    /// The elements at the root level of this lso
    pub body: Vec<Element>,
}

impl Lso {
    /// Create a new Lso with a header with the given name and version and an empty body
    #[inline]
    pub fn new_empty(name: impl Into<String>, version: AMFVersion) -> Self {
        Self::new(Vec::new(), name, version)
    }

    /// Crate a new Lso with a header with the given name, version and body
    #[inline]
    pub fn new(body: Vec<Element>, name: impl Into<String>, version: AMFVersion) -> Self {
        Self {
            header: Header::new(name, version),
            body,
        }
    }
}

impl IntoIterator for Lso {
    type Item = Element;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.body.into_iter()
    }
}