use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Out of bounds")]
    OutOfBounds,
}
