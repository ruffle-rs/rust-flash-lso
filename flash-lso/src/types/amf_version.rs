use core::fmt;

/// The version of AMF being used
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum AMFVersion {
    /// AMF0
    AMF0 = 0,

    /// AMF3
    AMF3 = 3,
}

impl TryFrom<u8> for AMFVersion {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::AMF0),
            3 => Ok(Self::AMF3),
            _ => Err(()),
        }
    }
}

impl fmt::Display for AMFVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AMFVersion::AMF0 => f.write_str("AMF0"),
            AMFVersion::AMF3 => f.write_str("AMF3"),
        }
    }
}
