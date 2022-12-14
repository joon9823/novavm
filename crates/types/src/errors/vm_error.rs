use move_deps::move_core_types::vm_status::{StatusCode, VMStatus};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum NovaVMError {
    #[error("Ran out of gas during contract execution")]
    GasDepletion {},
    #[error("errors from the move vm")]
    MoveError { status: VMStatus },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr { msg: String },
}

impl NovaVMError {
    pub fn gas_depletion() -> Self {
        NovaVMError::GasDepletion {}
    }

    pub fn move_error(status: VMStatus) -> Self {
        match &status.status_code() {
            StatusCode::OUT_OF_GAS => NovaVMError::gas_depletion(),
            _ => NovaVMError::MoveError { status },
        }
    }

    pub fn generic_err(msg: impl Into<String>) -> Self {
        NovaVMError::GenericErr { msg: msg.into() }
    }
}

impl From<VMStatus> for NovaVMError {
    fn from(source: VMStatus) -> Self {
        NovaVMError::move_error(source)
    }
}
