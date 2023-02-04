use enumset::EnumSet;
use super::Attribute;

/// A class definition (trait) used in AMF3
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClassDefinition {
    /// The name of the class definition
    pub name: String,

    /// The attributes on this trait
    pub attributes: EnumSet<Attribute>,

    /// The name of the static properties defined in this definition
    pub static_properties: Vec<String>,
}

impl Default for ClassDefinition {
    fn default() -> Self {
        Self {
            name: "Object".to_string(),
            attributes: EnumSet::empty(),
            static_properties: Vec::new(),
        }
    }
}

impl ClassDefinition {
    /// Creates a new ClassDefinition with the given name, and no attributes or properties
    pub fn default_with_name(name: String) -> Self {
        Self {
            name,
            attributes: EnumSet::empty(),
            static_properties: Vec::new(),
        }
    }
}