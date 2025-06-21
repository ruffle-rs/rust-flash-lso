use nom::error::{ErrorKind, FromExternalError, ParseError};
use thiserror::Error;

/// Enum for representing decoding errors
#[derive(Error, Debug, Clone, Eq, PartialEq)]
#[allow(variant_size_differences)] /* Allow the Nom variant to be large */
pub enum Error<'a> {
    /// Out of bounds decoding
    #[error("Out of bounds")]
    OutOfBounds,

    /// Invalid Amf0 reference tag
    #[error("Invalid reference")]
    InvalidReference(u16),

    /// Invalid type marker
    #[error("Unsupported tag")]
    UnsupportedType(u8),

    /// A nom internal error
    #[error("Nom internal error")]
    Nom(&'a [u8], ErrorKind),

    /// Packet is too large (too many headers or messages)
    #[error("Packet has too many headers or messages")]
    PacketTooLarge,

    /// Unable to find an object in the reference table
    #[error("Object not in reference table")]
    ObjectMissingFromReferenceTable(u64),

    /// An unknown IO error occured
    #[error("IO error: {0}")]
    IoError(String, std::io::ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Error::Nom(input, kind)
    }

    fn append(_: &[u8], _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a, E> FromExternalError<&'a [u8], E> for Error<'a> {
    fn from_external_error(input: &'a [u8], kind: ErrorKind, _e: E) -> Self {
        Error::Nom(input, kind)
    }
}
