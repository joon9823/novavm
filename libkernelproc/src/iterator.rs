use crate::error::GoError;
use crate::error::Error;
use crate::gas_meter::gas_meter_t;
use crate::memory::UnmanagedVector;

use kernelvm::{BackendError, vm::backend::GasInfo, vm::backend::BackendResult};

/// A record of a key-value storage that is created through an iterator API.
/// The first element (key) is always raw binary data. The second element
/// (value) is binary by default but can be changed to a custom type. This
/// allows contracts to reuse the type when deserializing database records.
pub type Record<V = Vec<u8>> = (Vec<u8>, V);

#[derive(Copy, Clone)]
// We assign these to integers to provide a stable API for passing over FFI (to wasm and Go)
pub enum Order {
    Ascending = 1,
    Descending = 2,
}

impl TryFrom<i32> for Order {
    type Error = BackendError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Order::Ascending),
            2 => Ok(Order::Descending),
            _ => Err(BackendError::bad_argument()) // FIXME add value for ::bad_agrument like ::generic_err("Order must be 1 or 2")),
        }
    }
}

impl From<Order> for i32 {
    fn from(original: Order) -> i32 {
        original as _
    }
}

// Iterator maintains integer references to some tables on the Go side
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct iterator_t {
    /// An ID assigned to this contract call
    pub call_id: u64,
    pub iterator_index: u64,
}

// These functions should return GoError but because we don't trust them here, we treat the return value as i32
// and then check it when converting to GoError manually
#[repr(C)]
#[derive(Default)]
pub struct Iterator_vtable {
    pub next_db: Option<
        extern "C" fn(
            iterator_t,
            *mut gas_meter_t,
            *mut u64,
            *mut UnmanagedVector, // key output
            *mut UnmanagedVector, // value output
            *mut UnmanagedVector, // error message output
        ) -> i32,
    >,
}

#[repr(C)]
pub struct GoIter {
    pub gas_meter: *mut gas_meter_t,
    pub state: iterator_t,
    pub vtable: Iterator_vtable,
}

impl GoIter {
    pub fn new(gas_meter: *mut gas_meter_t) -> Self {
        GoIter {
            gas_meter,
            state: iterator_t::default(),
            vtable: Iterator_vtable::default(),
        }
    }

    pub fn next(&mut self) -> BackendResult<Option<Record>> {
        let next_db = match self.vtable.next_db {
            Some(f) => f,
            None => {
                let result = Err(BackendError::unknown("iterator vtable not set"));
                return (result, GasInfo::free());
            }
        };

        let mut output_key = UnmanagedVector::default();
        let mut output_value = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_result: GoError = (next_db)(
            self.state,
            self.gas_meter,
            &mut used_gas as *mut u64,
            &mut output_key as *mut UnmanagedVector,
            &mut output_value as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the `UnmanagedVector`s here, no matter if we need the data.
        let output_key = output_key.consume();
        let output_value = output_value.consume();

        let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || "Failed to fetch next item from iterator".to_string();
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = match output_key {
            Some(key) => {
                if let Some(value) = output_value {
                    Ok(Some((key, value)))
                } else {
                    Err(BackendError::unknown(
                        "Failed to read value while reading the next key in the db",
                    ))
                }
            }
            None => Ok(None),
        };
        (result, gas_info)
    }
}
