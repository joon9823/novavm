use std::fmt::{Debug /*, Display*/};
use move_deps::move_core_types::vm_status::{VMStatus, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum NovaVMError { 
    #[error("Ran out of gas during contract execution")]
    GasDepletion {
    },
    #[error("errors from the move vm")]
    MoveError {
        status: VMStatus
    },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr {
        msg: String,
    }
}

impl NovaVMError {
    pub(crate) fn gas_depletion() -> Self {
        NovaVMError::GasDepletion {
        }
    }

    pub(crate) fn move_error(status: VMStatus) -> Self {
        NovaVMError::MoveError{
            status
        }
    }

    pub(crate) fn generic_err(msg: impl Into<String>) -> Self {
        NovaVMError::GenericErr {
            msg: msg.into(),
        }
    }

}

impl From<VMStatus> for NovaVMError {
    fn from(source: VMStatus) -> Self {
         match &source{
            VMStatus::ExecutionFailure { status_code, location, function, code_offset } => {
                match status_code {
                    StatusCode::OUT_OF_GAS => {
                        NovaVMError::gas_depletion()
                    },
                    _ => NovaVMError::move_error(source)
                }
            },
            _ => NovaVMError::move_error(source)
        }
    }
}