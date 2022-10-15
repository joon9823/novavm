use crate::error::GoError;
use crate::memory::UnmanagedVector;

use anyhow::anyhow;
use nova_natives::block::BlockInfoResolver;

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
    pub get_block_info: extern "C" fn(
        *const api_t,
        *mut u64,             // height
        *mut u64,             // timestamp
        *mut UnmanagedVector, // error_msg
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

impl BlockInfoResolver for GoApi {
    // return latest block height and timestamp
    fn get_block_info(&self) -> anyhow::Result<(u64, u64)> {
        let mut height = 0_u64;
        let mut timestamp = 0_u64;
        let mut error_msg = UnmanagedVector::default();

        let go_error: GoError =
            (self.vtable.get_block_info)(self.state, &mut height, &mut timestamp, &mut error_msg)
                .into();

        // return complete error message (reading from buffer for GoError::Other)
        let default = || "Failed to get latest block info".to_string();
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return Err(anyhow!(err));
            }
        }

        Ok((height, timestamp))
    }
}
