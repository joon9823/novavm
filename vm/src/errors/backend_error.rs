use std::fmt::Debug;
use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendError {
    #[error("Panic in FFI call")]
    ForeignPanic {},
    #[error("Bad argument")]
    BadArgument {},
    #[error("VM received invalid UTF-8 data from backend")]
    InvalidUtf8 {},
    #[error("Iterator with ID {id} does not exist")]
    IteratorDoesNotExist { id: u32 },
    #[error("Ran out of gas during call into backend")]
    OutOfGas {},
    #[error("Unknown error during call into backend: {msg}")]
    Unknown { msg: String },
    // This is the only error case of BackendError that is reported back to the contract.
    #[error("User error during call into backend: {msg}")]
    UserErr { msg: String },
}

impl BackendError {
    pub fn foreign_panic() -> Self {
        BackendError::ForeignPanic {}
    }

    pub fn bad_argument() -> Self {
        BackendError::BadArgument {}
    }

    pub fn iterator_does_not_exist(iterator_id: u32) -> Self {
        BackendError::IteratorDoesNotExist { id: iterator_id }
    }

    pub fn out_of_gas() -> Self {
        BackendError::OutOfGas {}
    }

    pub fn unknown(msg: impl Into<String>) -> Self {
        BackendError::Unknown { msg: msg.into() }
    }

    pub fn user_err(msg: impl Into<String>) -> Self {
        BackendError::UserErr { msg: msg.into() }
    }
}

impl From<FromUtf8Error> for BackendError {
    fn from(_original: FromUtf8Error) -> Self {
        BackendError::InvalidUtf8 {}
    }
}
