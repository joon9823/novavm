//use std::collections::HashMap;
//use std::convert::TryInto;

use novavm::access_path::AccessPath;
//use novavm::BackendError;
use novavm::backend::BackendResult;
use novavm::storage::state_view::StateView;

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

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()>;

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> BackendResult<()>;
}

pub struct GoStorage {
    db: Db,
}

impl GoStorage {
    pub fn new(db: Db) -> Self {
        GoStorage {
            db,
        }
    }
}

impl StateView for GoStorage {
        fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        let key = access_path.to_string();
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            U8SliceView::new(Some(key.as_bytes())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

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

        anyhow::Result::Ok(output)
    }
}

impl Storage for GoStorage {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.read_db)(
            self.db.state,
            U8SliceView::new(Some(key)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to read a key in the db: {}",
                String::from_utf8_lossy(key)
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return Err(err);
            }
        }

        Ok(output)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.write_db)(
            self.db.state,
            U8SliceView::new(Some(key)),
            U8SliceView::new(Some(value)),
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to set a key in the db: {}",
                String::from_utf8_lossy(key),
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return Err(err);
            }
        }
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        let mut error_msg = UnmanagedVector::default();
        let go_error: GoError = (self.db.vtable.remove_db)(
            self.db.state,
            U8SliceView::new(Some(key)),
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        let default = || {
            format!(
                "Failed to delete a key in the db: {}",
                String::from_utf8_lossy(key),
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return Err(err);
            }
        }
        Ok(())
    }
}
