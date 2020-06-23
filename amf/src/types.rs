use std::fmt;
use nom::lib::std::fmt::Formatter;

/// A container for sol files
#[derive(Debug)]
pub struct Sol {
    pub header: SolHeader,
    pub body: Vec<SolElement>,
}

/// The header of a sol file
#[derive(Debug, Eq, PartialEq)]
pub struct SolHeader {
    pub version: [u8; 2],
    pub length: u32,
    pub signature: [u8; 10],
    pub name: String,
    //TODO: this could be an enum
    pub format_version: u8,
}

/// Represent a named element
#[derive(Clone, Debug)]
pub struct SolElement {
    pub name: String,
    pub value: SolValue,
}

//TODO: should amf3 assoc arrays be their own type with a dense and assoc section
/// A single or compound value
#[derive(Debug, Clone)]
pub enum SolValue {
    /// Represent the type number (amf0) and double (amf3)
    Number(f64),
    /// Represents the type boolean (amf0) and both the true/false type (amf3)
    Bool(bool),
    /// Represent both the string (amf0/3) and long string type (amf0)
    String(String),
    Object(Vec<SolElement>),
    /// Represent the null type
    Null,
    /// Represent the undefined type
    Undefined,
    /// Represent ECMA-Arrays (amf0) and associative arrays (amf3, even if they contain a dense part)
    ECMAArray(Vec<SolElement>),
    /// Represent the end of a list of object elements (amf0)
    ObjectEnd,
    /// Represent a strict array (amf0) or a dense array (amf3)
    StrictArray(Vec<SolValue>),
    /// Represent a timezone in the format (seconds since epoch, timezone or UTC if missing (amf3) )
    Date(f64, Option<u16>),
    /// Represent the unsupported type
    Unsupported,
    XML(String),
    TypedObject(String, Vec<SolElement>),
    // AMF3
    /// Represent the integer type (u29) (amf3)
    Integer(i32),
    /// Represent the bytearray type (amf3)
    ByteArray(Vec<u8>),
    /// Represent the int vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorInt(Vec<i32>, bool),
    /// Represent the unsigned int vector type (amf3)
    /// Format is (values, is_fixed_length)    VectorUInt(Vec<u32>, bool),
    VectorDouble(Vec<f64>, bool),
    /// Represent the object vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorObject(Vec<SolValue>, String, bool),
    /// Represent the dictionary type (amf3)
    /// Format is ((key, value), has_weak_keys)
    Dictionary(Vec<(SolValue, SolValue)>, bool),
}

/// A class definition (trait) used in AMF3
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClassDefinition {
    pub name: String,
    pub encoding: u8,
    pub attribute_count: u32,
    pub static_properties: Vec<String>,
    pub externalizable: bool,
}
