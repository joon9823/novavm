//use std::collections::HashMap;
//use std::convert::TryInto;

use kernelvm::access_path::AccessPath;
//use kernelvm::BackendError;
use kernelvm::backend::{BackendResult, GasInfo};
use kernelvm::storage::state_view::StateView;

use crate::db::Db;
use crate::error::GoError;
//use crate::iterator::{GoIter, Order, Record};
use crate::memory::{U8SliceView, UnmanagedVector};

use anyhow::anyhow;

/// Access to the VM's backend storage, i.e. the chain
pub trait Storage {
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>>;

    /* we don't need iterators
    /// Allows iteration over a set of key/value pairs, either forwards or backwards.
    /// Returns an interator ID that is unique within the Storage instance.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    ///
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    ///
    /// This call must not change data in the storage, but creating and storing a new iterator can be a mutating operation on
    /// the Storage implementation.
    /// The implementation must ensure that iterator IDs are assigned in a deterministic manner as this is
    /// environment data that is injected into the contract.
    #[cfg(feature = "iterator")]
    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32>;

    /// Returns the next element of the iterator with the given ID.
    ///
    /// If the ID is not found, a BackendError::IteratorDoesNotExist is returned.
    ///
    /// This call must not change data in the storage, but incrementing an iterator can be a mutating operation on
    /// the Storage implementation.
    #[cfg(feature = "iterator")]
    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>>;
    */

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()>;

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> BackendResult<()>;
}

pub struct GoStorage {
    db: Db,
    //iterators: HashMap<u32, GoIter>,
}

impl GoStorage {
    pub fn new(db: Db) -> Self {
        GoStorage {
            db,
            //iterators: HashMap::new(),
        }
    }
}

impl StateView for GoStorage {
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

impl Storage for GoStorage {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            self.db.gas_meter,
            &mut used_gas as *mut u64,
            U8SliceView::new(Some(key)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to read a key in the db: {}",
                String::from_utf8_lossy(key)
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        (Ok(output), gas_info)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.db.vtable.write_db)(
            self.db.state,
            self.db.gas_meter,
            &mut used_gas as *mut u64,
            U8SliceView::new(Some(key)),
            U8SliceView::new(Some(value)),
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        let gas_info = GasInfo::with_externally_used(used_gas);
        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to set a key in the db: {}",
                String::from_utf8_lossy(key),
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }
        (Ok(()), gas_info)
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_error: GoError = (self.db.vtable.remove_db)(
            self.db.state,
            self.db.gas_meter,
            &mut used_gas as *mut u64,
            U8SliceView::new(Some(key)),
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        let gas_info = GasInfo::with_externally_used(used_gas);
        let default = || {
            format!(
                "Failed to delete a key in the db: {}",
                String::from_utf8_lossy(key),
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }
        (Ok(()), gas_info)
    }
}
