

// this crate handles communication between db.rs and vm.rs
use crate::{Db, UnmanagedVector, U8SliceView, error::GoError};

use kernelvm::vm::{storage::state_view::StateView, access_path::AccessPath};
use anyhow::{anyhow, Error};

pub struct CosmosView {
    db: Db,
}

impl CosmosView {
    pub fn new(db: Db) -> Self{
        Self {db}
    }
}

impl StateView for CosmosView {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        let key = access_path.to_string(); // FIXME: replace to_string to to_cosmos_key
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            self.db.gas_meter,
            &mut used_gas as *mut u64,
            U8SliceView::new(Some(key.as_bytes())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        // FIXME: uncomment: let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to read a key in the db: {}",
                String::from_utf8_lossy(key.as_bytes())
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return Err(anyhow!(err))
            }
        }

        anyhow::Result::Ok(output/* , gas_info*/) //FIXME: add gas_info?
    }
}