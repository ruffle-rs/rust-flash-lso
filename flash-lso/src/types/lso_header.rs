use super::AMFVersion;

/// The header of a lso file
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    /// The length of the lso in bytes
    pub length: u32,

    /// The name of the lso file
    pub name: String,

    /// The version of AMF used to encode the data
    pub format_version: AMFVersion,
}

impl Header {
    /// Create a new header with the given name and version, will have a size of 0 by default
    #[inline]
    pub fn new(name: impl Into<String>, version: AMFVersion) -> Self {
        Self {
            length: 0,
            name: name.into(),
            format_version: version,
        }
    }
}