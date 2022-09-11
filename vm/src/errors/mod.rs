mod backend_error;
mod vm_error;

pub use backend_error::BackendError;
pub use vm_error::VmError;

pub type VmResult<T> = core::result::Result<T, VmError>;
