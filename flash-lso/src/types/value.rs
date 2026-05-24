use crate::amf3::length::Length;
use super::{ClassDefinition, Element, ObjectId, Reference};

/// The data contained within a Value of type Object
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectValue {
    /// The child elements of this Object
    pub elements: Vec<Element>,

    /// The class definition for this object, if it exists
    pub class_definition: Option<ClassDefinition>,
}

///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CustomObjectValue {
    ///
    pub elements: Vec<Element>,
    ///
    pub dynamic_elements: Vec<Element>,
    ///
    pub class_definition: ClassDefinition,
}

///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DictionaryObjectValue {
    ///
    pub elements: Vec<DictionaryEntry>,
    ///
    pub weak_keys: bool,
}

///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DictionaryEntry {
    ///
    key: Value,
    ///
    value: Value,
}

///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct VectorObjectValue<T> {
    ///
    values: Vec<T>,
    ///
    fixed_length: bool,
}

///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ECMAArrayObjectValue {
    pub dense: Vec<Value>,
    pub elements: Vec<Element>,

    /// The length of the array in AMF0, this can differ from the actual number of elements
    pub length: u32,
}



//TODO: should amf3 assoc arrays be their own type with a dense and assoc section
/// A single or compound value
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represent the type number (amf0) and double (amf3)
    Number(f64),

    /// Represents the type boolean (amf0) and both the true/false type (amf3)
    Bool(bool),

    /// Represent both the string (amf0/3) and long string type (amf0)
    String(String),

    /// Represents the object type in both amf0 and amf3, class definition are only available with amf3
    Object {
        /// The unique id for this object, referenced by `Amf3ObjectReference` instances
        id: ObjectId,

        /// The data within this value
        data: ObjectValue,
    },

    /// Represent the null type
    Null,

    /// Represent the undefined type
    Undefined,

    /// Represent ECMA-Arrays (amf0) and associative arrays (amf3, even if they contain a dense part)
    ECMAArray {
        id: ObjectId,
        data: ECMAArrayObjectValue,
    },

    /// Represent a strict array (amf0) or a dense array (amf3)
    StrictArray {
        ///
        id: ObjectId,
        ///
        values: Vec<Value>,
    },

    /// Represent a timezone in the format (seconds since epoch, timezone or UTC if missing (amf3) )
    Date {
        time: f64,
        timezone_or_utc: Option<u16>
    },

    /// Represent the unsupported type
    Unsupported,

    /// Represent the XML type, (value, is_string)
    XML {
        ///
        value: String,
        ///
        is_string: bool,
    },

    #[cfg(feature = "amf3")]
    /// Represent an amf3 element embedded in an AMF0 file
    AMF3(Box<Value>),

    // AMF3
    /// Represent the integer type (u29) (amf3)
    Integer(i32),

    /// Represent the bytearray type (amf3)
    ByteArray(Vec<u8>),

    /// Represent the int vector type (amf3)
    VectorInt(VectorObjectValue<i32>),

    /// Represent the unsigned int vector type (amf3)
    VectorUInt(VectorObjectValue<u32>),

    /// Represent the double vector type (amf3)
    VectorDouble(VectorObjectValue<f64>),

    /// Represent the object vector type (amf3)
    /// Format is (values, is_fixed_length)
    /// TODO
    VectorObject(ObjectId, Vec<Value>, String, bool),

    /// Represent the dictionary type (amf3)
    Dictionary {
        ///
        id: ObjectId,
        ///
        data: DictionaryObjectValue,
    },

    /// Represent an external object, such as from flex
    /// (custom_elements, regular elements, class def)
    Custom(CustomObjectValue),

    /// Represent an existing value, stored by reference, the value here should be considered opaque
    Reference(Reference),

    /// A reference to a previously parsed element
    ///
    /// While traversing the graph of `Value` instances you should maintain a mapping of `ObjectId` to your internal
    /// representation of a value and consider this a reference to the exact same value.
    ///
    /// As `Value` graphs can contain cycles which are best handled by garbage collected structures
    /// we leave the handling of this to the user, sorry
    Amf3ObjectReference(ObjectId),
}


