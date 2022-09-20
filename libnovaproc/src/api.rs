use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
use novavm::backend::{BackendApi, BackendResult};

// this represents something passed in from the caller side of FFI
// in this case a struct with go function pointers
#[repr(C)]
pub struct api_t {
    _private: [u8; 0],
}

// These functions should return GoError but because we don't trust them here, we treat the return value as i32
// and then check it when converting to GoError manually
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoApi_vtable {
    pub bank_transfer: extern "C" fn(
        *const api_t,
        U8SliceView,
        U8SliceView,
        U8SliceView,
        *mut UnmanagedVector, // error message output
    ) -> i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoApi {
    pub state: *const api_t,
    pub vtable: GoApi_vtable,
}

// We must declare that these are safe to Send, to use in wasm.
// The known go caller passes in immutable function pointers, but this is indeed
// unsafe for possible other callers.
//
// see: https://stackoverflow.com/questions/50258359/can-a-struct-containing-a-raw-pointer-implement-send-and-be-ffi-safe
unsafe impl Send for GoApi {}

impl BackendApi for GoApi {
    fn bank_transfer(&self, recipient: &[u8], denom: &str, amount: &str) -> BackendResult<()> {
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.vtable.bank_transfer)(
            self.state,
            U8SliceView::new(Some(recipient)),
            U8SliceView::new(Some(denom.as_bytes())),
            U8SliceView::new(Some(amount.as_bytes())),
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();

        // return complete error message (reading from buffer for GoError::Other)
        let default = || format!("Failed to transfer coin to the address: {:?}", recipient);
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return Err(err);
            }
        }

        Ok(())
    }
}
