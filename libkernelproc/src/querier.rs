use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};

use kernelvm::backend::{BackendResult, Querier};

// this represents something passed in from the caller side of FFI
#[repr(C)]
#[derive(Clone)]
pub struct querier_t {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone)]
pub struct Querier_vtable {
    // We return errors through the return buffer, but may return non-zero error codes on panic
    pub query_external: extern "C" fn(
        *const querier_t,
        U8SliceView,
        *mut UnmanagedVector, // result output
        *mut UnmanagedVector, // error message output
    ) -> i32,
}

#[repr(C)]
#[derive(Clone)]
pub struct GoQuerier {
    pub state: *const querier_t,
    pub vtable: Querier_vtable,
}

// TODO: check if we can do this safer...
unsafe impl Send for GoQuerier {}

impl Querier for GoQuerier {
    fn query_raw(
        &self,
        request: &[u8],
    ) -> BackendResult<Vec<u8>> { /* FIXME: put Vec<u8> as stub */
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let go_result: GoError = (self.vtable.query_external)(
            self.state,
            U8SliceView::new(Some(request)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to query another contract with this request: {}",
                String::from_utf8_lossy(request)
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return Err(err);
            }
        }

        let bin_result: Vec<u8> = output.unwrap_or_default();
        let result = serde_json::from_slice(&bin_result).or_else(|e| {
            todo!() 
		  // FIXME: put it as stub
			/* original.. remove it after porting  
			Ok(SystemResult::Err(SystemError::InvalidResponse {
				error: format!("Parsing Go response: {}", e),
				response: bin_result.into(),
			}))
		});
		*/
         });
        result
    }
}
