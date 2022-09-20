use novavm::{VmError, BackendError};
use errno::{set_errno, Errno};
use thiserror::Error;

use crate::memory::UnmanagedVector;

#[derive(Error, Debug)]
pub enum RustError {
    #[error("Empty argument: {}", name)]
    EmptyArg {
        name: String,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[error("Cannot decode UTF8 bytes into string: {}", msg)]
    InvalidUtf8 {
        msg: String,
    },
    #[error("Ran out of gas")]
    OutOfGas {
    },
    #[error("Caught panic")]
    Panic {
    },
    #[error("Null/Nil argument: {}", name)]
    UnsetArg {
        name: String,
    },
    #[error("Error calling the VM: {}", msg)]
    VmErr {
        msg: String,
    },
    #[error("failure occured from backend: {}", msg)]
    BackendFailure {
        msg: String,
    }
}

impl RustError {
    pub fn empty_arg<T: Into<String>>(name: T) -> Self {
        RustError::EmptyArg {
            name: name.into(),
        }
    }

    pub fn invalid_utf8<S: ToString>(msg: S) -> Self {
        RustError::InvalidUtf8 {
            msg: msg.to_string(),
        }
    }

    pub fn panic() -> Self {
        RustError::Panic {
        }
    }

    pub fn unset_arg<T: Into<String>>(name: T) -> Self {
        RustError::UnsetArg {
            name: name.into(),
        }
    }

    pub fn vm_err<S: ToString>(msg: S) -> Self {
        RustError::VmErr {
            msg: msg.to_string(),
        }
    }

    pub fn out_of_gas() -> Self {
        RustError::OutOfGas {
        }
    }

    pub fn backend_failure<S: ToString>(msg: S) -> Self {
        RustError::BackendFailure {
            msg: msg.to_string(),
        }
    }
}

impl From<VmError> for RustError {
    fn from(source: VmError) -> Self {
        match source {
            VmError::GasDepletion { .. } => RustError::out_of_gas(),
            _ => RustError::vm_err(source),
        }
    }
}

impl From<BackendError> for RustError {
    fn from(source: BackendError) -> Self {
        RustError::backend_failure(source.to_string())
    }
}

impl From<std::str::Utf8Error> for RustError {
    fn from(source: std::str::Utf8Error) -> Self {
        RustError::invalid_utf8(source)
    }
}

impl From<std::string::FromUtf8Error> for RustError {
    fn from(source: std::string::FromUtf8Error) -> Self {
        RustError::invalid_utf8(source)
    }
}

/// cbindgen:prefix-with-name
#[repr(i32)]
enum ErrnoValue {
    Success = 0,
    Other = 1,
    OutOfGas = 2,
}

pub fn clear_error() {
    set_errno(Errno(ErrnoValue::Success as i32));
}

pub fn set_error(err: RustError, error_msg: Option<&mut UnmanagedVector>) {
    if let Some(error_msg) = error_msg {
        if error_msg.is_some() {
            panic!(
                "There is an old error message in the given pointer that has not been \
                cleaned up. Error message pointers should not be reused for multiple calls."
            )
        }

        let msg: Vec<u8> = err.to_string().into();
        *error_msg = UnmanagedVector::new(Some(msg));
    } else {
        // The caller provided a nil pointer for the error message.
        // That's not nice but we can live with it.
    }

    let errno = match err {
        RustError::OutOfGas { .. } => ErrnoValue::OutOfGas,
        _ => ErrnoValue::Other,
    } as i32;
    set_errno(Errno(errno));
}

/// If `result` is Ok, this returns the Ok value and clears [errno].
/// Otherwise it returns a null pointer, writes the error message to `error_msg` and sets [errno].
///
/// [errno]: https://utcc.utoronto.ca/~cks/space/blog/programming/GoCgoErrorReturns
pub fn handle_c_error_ptr<T>(
    result: Result<*mut T, RustError>,
    error_msg: Option<&mut UnmanagedVector>,
) -> *mut T {
    match result {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error, error_msg);
            std::ptr::null_mut()
        }
    }
}

/// If `result` is Ok, this returns the binary representation of the Ok value and clears [errno].
/// Otherwise it returns an empty vector, writes the error message to `error_msg` and sets [errno].
///
/// [errno]: https://utcc.utoronto.ca/~cks/space/blog/programming/GoCgoErrorReturns
pub fn handle_c_error_binary<T>(
    result: Result<T, RustError>,
    error_msg: Option<&mut UnmanagedVector>,
) -> Vec<u8>
where
    T: Into<Vec<u8>>,
{
    match result {
        Ok(value) => {
            clear_error();
            value.into()
        }
        Err(error) => {
            set_error(error, error_msg);
            Vec::new()
        }
    }
}

/// If `result` is Ok, this returns the Ok value and clears [errno].
/// Otherwise it returns the default value, writes the error message to `error_msg` and sets [errno].
///
/// [errno]: https://utcc.utoronto.ca/~cks/space/blog/programming/GoCgoErrorReturns
pub fn handle_c_error_default<T>(
    result: Result<T, RustError>,
    error_msg: Option<&mut UnmanagedVector>,
) -> T
where
    T: Default,
{
    match result {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error, error_msg);
            Default::default()
        }
    }
}
