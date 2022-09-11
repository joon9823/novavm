use std::fmt::{Debug /*, Display*/};
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum VmError { 
    #[error("Ran out of gas during contract execution")]
    GasDepletion {
    },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr {
        msg: String,
    }
}

impl VmError {
    #[allow(dead_code)] // FIXME:: remove this later: have to  allow dead code during PoC only
    pub(crate) fn gas_depletion() -> Self {
        VmError::GasDepletion {
        }
    }

    #[allow(dead_code)] // FIXME:: remove this later: have to  allow dead code during PoC only
    pub(crate) fn generic_err(msg: impl Into<String>) -> Self {
        VmError::GenericErr {
            msg: msg.into(),
        }
    }

}

