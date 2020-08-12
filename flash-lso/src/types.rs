use enumset::EnumSet;
use enumset::EnumSetType;

/// A container for sol files
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct Sol {
    pub header: SolHeader,
    pub body: Vec<SolElement>,
}

/// The header of a sol file
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct SolHeader {
    pub version: [u8; 2],
    pub length: u32,
    pub signature: [u8; 10],
    pub name: String,
    //TODO: this could be an enum
    pub format_version: u8,
}

/// Represent a named element
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SolElement {
    pub name: String,
    pub value: SolValue,
}

//TODO: should amf3 assoc arrays be their own type with a dense and assoc section
/// A single or compound value
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum SolValue {
    /// Represent the type number (amf0) and double (amf3)
    Number(f64),
    /// Represents the type boolean (amf0) and both the true/false type (amf3)
    Bool(bool),
    /// Represent both the string (amf0/3) and long string type (amf0)
    String(String),
    Object(Vec<SolElement>, Option<ClassDefinition>),
    /// Represent the null type
    Null,
    /// Represent the undefined type
    Undefined,
    /// Represent ECMA-Arrays (amf0) and associative arrays (amf3, even if they contain a dense part)
    /// Final value represents the length of the array in amf0, this can differ from the actual number of elements
    ECMAArray(Vec<SolValue>, Vec<SolElement>, u32),
    /// Represent a strict array (amf0) or a dense array (amf3)
    StrictArray(Vec<SolValue>),
    /// Represent a timezone in the format (seconds since epoch, timezone or UTC if missing (amf3) )
    Date(f64, Option<u16>),
    /// Represent the unsupported type
    Unsupported,
    XML(String, bool),
    // TODo can this just be an object with the name in class def
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
    /// Format is (values, is_fixed_length)
    VectorUInt(Vec<u32>, bool),
    /// Represent the double vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorDouble(Vec<f64>, bool),
    /// Represent the object vector type (amf3)
    /// Format is (values, is_fixed_length)
    VectorObject(Vec<SolValue>, String, bool),
    /// Represent the dictionary type (amf3)
    /// Format is ((key, value), has_weak_keys)
    Dictionary(Vec<(SolValue, SolValue)>, bool),
    /// Represent a external object, such as from flex
    /// (custom_elements, regular elements, class def)
    Custom(Vec<SolElement>, Vec<SolElement>, Option<ClassDefinition>),
}

/// A class definition (trait) used in AMF3
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClassDefinition {
    pub name: String,
    pub attributes: EnumSet<Attribute>,
    pub attribute_count: u32,
    pub static_properties: Vec<String>,
}

/// Encodes the possible attributes that can be given to a trait
/// If a trait is dynamic then the object may have additional properties other than the ones specified in the trait
/// If a trait is external then it requires custom serialization and deserialization support
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(EnumSetType, Debug)]
pub enum Attribute {
    DYNAMIC,
    EXTERNAL,
}

pub mod amf0 {
    use derive_try_from_primitive::TryFromPrimitive;

    /// Type markers used in AMF0
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(TryFromPrimitive, Eq, PartialEq, Debug, Copy, Clone)]
    #[repr(u8)]
    pub enum TypeMarker {
        Number = 0,
        Boolean = 1,
        String = 2,
        Object = 3,
        MovieClip = 4,
        Null = 5,
        Undefined = 6,
        Reference = 7,
        MixedArrayStart = 8,
        ObjectEnd = 9,
        Array = 10,
        Date = 11,
        LongString = 12,
        Unsupported = 13,
        RecordSet = 14,
        XML = 15,
        TypedObject = 16,
        AMF3 = 17,
    }
}

pub mod amf3 {
    use derive_try_from_primitive::TryFromPrimitive;

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(TryFromPrimitive, Eq, PartialEq, Debug, Copy, Clone)]
    #[repr(u8)]
    pub enum TypeMarker {
        Undefined = 0x00,
        Null = 0x01,
        False = 0x02,
        True = 0x03,
        Integer = 0x04,
        Number = 0x05,
        String = 0x06,
        XML = 0x07,
        Date = 0x08,
        Array = 0x09,
        Object = 0x0A,
        XmlString = 0x0B,
        ByteArray = 0x0C,
        VectorInt = 0x0D,
        VectorUInt = 0x0E,
        VectorDouble = 0x0F,
        VectorObject = 0x10,
        Dictionary = 0x11,
    }
}
