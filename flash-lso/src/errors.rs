use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Out of bounds")]
    OutOfBounds,
}
