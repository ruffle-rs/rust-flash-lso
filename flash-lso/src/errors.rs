use nom::error::{ErrorKind, FromExternalError, ParseError};
use thiserror::Error;

/// Enum for representing decoding errors
#[derive(Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error<'a> {
    /// Out of bounds decoding
    #[error("Out of bounds")]
    OutOfBounds,

    /// A nom internal error
    #[error("Nom internal error")]
    Nom(&'a [u8], ErrorKind),
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
