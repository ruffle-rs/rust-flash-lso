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

/// Represents the contents of a Custom object
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct CustomObjectValue {
    /// The external elements
    pub elements: Vec<Element>,
    /// The dynamic elements
    pub dynamic_elements: Vec<Element>,

    /// The ClassDefinition assigned to this Custom object
    pub class_definition: ClassDefinition,
}

/// The data contained within a Dictionary
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DictionaryObjectValue {
    /// The contents of this Dictionary
    pub elements: Vec<DictionaryEntry>,

    /// Are the keys of this Dictionary weakly referenced
    pub weak_keys: bool,
}

/// Represents a Key-Value pair in an AMF Dictionary
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DictionaryEntry {
    /// The key
    pub key: Value,

    /// The value
    pub value: Value,
}

/// An AMF vector of a some subtype
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct VectorPrimitiveValue<T> {
    /// The contents of this Vector
    pub values: Vec<T>,

    /// Is this vector of a fixed length
    pub fixed_length: bool,
}

/// An AMF vector of Objects
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct VectorObjectValue {
    /// The contents of this Vector
    pub values: Vec<Value>,

    /// The name of the class this is a vector of
    pub object_type_name: String,

    /// Is this vector of a fixed length
    pub fixed_length: bool,
}

/// The contents of a ECMAArray object
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ECMAArrayObjectValue {
    /// The dense component of this ECMAArray
    pub dense: Vec<Value>,

    /// The non-dense component of this ECMAArray
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
        /// The unique id that will be used to refer back to this Object
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
        /// A unique identifier that will be used to refer back to this ECMAArray
        id: ObjectId,

        /// The data within this ECMAArray
        data: ECMAArrayObjectValue,
    },

    /// Represent a strict array (amf0) or a dense array (amf3)
    StrictArray {
        /// A unique identifier that will be used to refer back to this StrictArray
        id: ObjectId,

        /// The data within this StrictArray
        values: Vec<Value>,
    },

    /// Represent a timestamp
    Date {
        /// Number of seconds since the epoch
        time: f64,

        /// The timezone identifier or UTC if missing
        timezone_or_utc: Option<u16>,
    },

    /// Represent the unsupported type
    Unsupported,

    /// Represent the XML type, (value, is_string)
    XML {
        /// The XML data
        value: String,
        /// Is this XML a string
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
    VectorInt(VectorPrimitiveValue<i32>),

    /// Represent the unsigned int vector type (amf3)
    VectorUInt(VectorPrimitiveValue<u32>),

    /// Represent the double vector type (amf3)
    VectorDouble(VectorPrimitiveValue<f64>),

    /// Represent the object vector type (amf3)
    VectorObject {
        /// A unique identifier that will be used to refer back to this VectorObject
        id: ObjectId,

        /// The data in this Vector
        data: VectorObjectValue,
    },

    /// Represent the dictionary type (amf3)
    Dictionary {
        /// A unique identifier that will be used to refer back to this Dictionary
        id: ObjectId,

        /// The data contained within this Dictionary
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
