use derive_try_from_primitive::TryFromPrimitive;

/// Type markers used in AMF0
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(TryFromPrimitive, Eq, PartialEq, Debug, Copy, Clone)]
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
