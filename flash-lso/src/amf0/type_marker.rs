/// Type markers used in AMF0
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum TypeMarker {
    /// Number
    Number = 0,

    /// Boolean
    Boolean = 1,

    /// String
    String = 2,

    /// Object start
    Object = 3,

    /// MovieClip (unused)
    MovieClip = 4,

    /// Null
    Null = 5,

    /// Undefined
    Undefined = 6,

    /// Reference (unused)
    Reference = 7,

    /// Start of a mixed array
    MixedArrayStart = 8,

    /// Object end
    ObjectEnd = 9,

    /// Array start
    Array = 10,

    /// Date with timezone
    Date = 11,

    /// Long string (length > 65535)
    LongString = 12,

    /// Unsupported
    Unsupported = 13,

    /// Recordset
    RecordSet = 14,

    /// XML
    Xml = 15,

    /// Typed object start
    TypedObject = 16,
    
    /// Embedded AMF3 element
    AMF3 = 17,
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
            8 => Ok(Self::MixedArrayStart),
            9 => Ok(Self::ObjectEnd),
            10 => Ok(Self::Array),
            11 => Ok(Self::Date),
            12 => Ok(Self::LongString),
            13 => Ok(Self::Unsupported),
            14 => Ok(Self::RecordSet),
            15 => Ok(Self::Xml),
            16 => Ok(Self::TypedObject),
            17 => Ok(Self::AMF3),
            _ => Err(())
        }
    }
}