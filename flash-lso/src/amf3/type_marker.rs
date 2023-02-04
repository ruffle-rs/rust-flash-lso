/// Type markers used in AMF3
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
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
    Xml = 0x07,
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

impl TryFrom<u8> for TypeMarker {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Undefined),
            0x01 => Ok(Self::Null),
            0x02 => Ok(Self::False),
            0x03 => Ok(Self::True),
            0x04 => Ok(Self::Integer),
            0x05 => Ok(Self::Number),
            0x06 => Ok(Self::String),
            0x07 => Ok(Self::Xml),
            0x08 => Ok(Self::Date),
            0x09 => Ok(Self::Array),
            0x0A => Ok(Self::Object),
            0x0B => Ok(Self::XmlString),
            0x0C => Ok(Self::ByteArray),
            0x0D => Ok(Self::VectorInt),
            0x0E => Ok(Self::VectorUInt),
            0x0F => Ok(Self::VectorDouble),
            0x10 => Ok(Self::VectorObject),
            0x11 => Ok(Self::Dictionary),
            _ => Err(())
        }
    }
}