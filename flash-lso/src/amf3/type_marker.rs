use derive_try_from_primitive::TryFromPrimitive;

/// Type markers used in AMF3
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(TryFromPrimitive, Eq, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum TypeMarker {
    /// Undefined
    Undefined = 0x00,
    /// Null
    Null = 0x01,
    /// Boolean false
    False = 0x02,
    /// Boolean true
    True = 0x03,
    /// Variable length integer
    Integer = 0x04,
    /// Floating point number
    Number = 0x05,
    /// String
    String = 0x06,
    /// XML
    XML = 0x07,
    /// Date (always UTC)
    Date = 0x08,
    /// Array
    Array = 0x09,
    /// Object
    Object = 0x0A,
    /// XML string
    XmlString = 0x0B,
    /// Byte array
    ByteArray = 0x0C,
    /// Vector<Int>
    VectorInt = 0x0D,
    /// Vector<UInt>
    VectorUInt = 0x0E,
    /// Vector<Double>
    VectorDouble = 0x0F,
    /// Vector<Object>
    VectorObject = 0x10,
    /// Dictionary
    Dictionary = 0x11,
}
