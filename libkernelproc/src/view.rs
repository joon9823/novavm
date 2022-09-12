

// this crate handles communication between db.rs and vm.rs
use crate::Db;

use kernelvm::vm::{storage::state_view::StateView, access_path::AccessPath};

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
        anyhow::Result::Ok(None) // FIXME: just stub
    }
}