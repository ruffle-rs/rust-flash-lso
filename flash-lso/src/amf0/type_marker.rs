/// Type markers used in AMF0
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum TypeMarker {
    /// Number
    Number = 0x00,

    /// Boolean
    Boolean = 0x01,

    /// String
    String = 0x02,

    /// Object start
    Object = 0x03,

    /// MovieClip (unused)
    MovieClip = 0x04,

    /// Null
    Null = 0x05,

    /// Undefined
    Undefined = 0x06,

    /// Reference
    Reference = 0x07,

    /// Start of an ECMA array
    ECMAArray = 0x08,

    /// Object end
    ObjectEnd = 0x09,

    /// Strict array start
    StrictArray = 0x0A,

    /// Date with timezone
    Date = 0x0B,

    /// Long string (length > 65535)
    LongString = 0x0C,

    /// Unsupported
    Unsupported = 0x0D,

    /// Recordset (unused)
    RecordSet = 0x0E,

    /// XML
    Xml = 0x0F,

    /// Typed object start
    TypedObject = 0x10,

    /// Embedded AMF3 element
    AMF3 = 0x11,
}

impl TryFrom<u8> for TypeMarker {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Number),
            1 => Ok(Self::Boolean),
            2 => Ok(Self::String),
            3 => Ok(Self::Object),
            4 => Ok(Self::MovieClip),
            5 => Ok(Self::Null),
            6 => Ok(Self::Undefined),
            7 => Ok(Self::Reference),
            8 => Ok(Self::ECMAArray),
            9 => Ok(Self::ObjectEnd),
            10 => Ok(Self::StrictArray),
            11 => Ok(Self::Date),
            12 => Ok(Self::LongString),
            13 => Ok(Self::Unsupported),
            14 => Ok(Self::RecordSet),
            15 => Ok(Self::Xml),
            16 => Ok(Self::TypedObject),
            17 => Ok(Self::AMF3),
            _ => Err(()),
        }
    }
}
