use enumset::EnumSetType;

/// Encodes the possible attributes that can be given to a trait
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(EnumSetType, Debug)]
pub enum Attribute {
    /// If a trait is dynamic then the object it constructs may have additional properties other than the ones specified in the trait
    Dynamic,
    
    /// If a trait is external then it requires custom serialization and deserialization support
    External,
}