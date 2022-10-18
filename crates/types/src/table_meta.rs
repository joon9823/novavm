use move_deps::move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TableMeta {
    pub payer: AccountAddress,
    pub size: usize,
}

impl TableMeta {
    pub fn new() -> Self {
        TableMeta {
            payer: AccountAddress::ZERO,
            size: 0,
        }
    }

    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        bcs::to_bytes(self).map_err(|_| anyhow::anyhow!("failed to serialize TableMeta"))
    }

    pub fn deserialize(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs::from_bytes(bytes).map_err(|_| anyhow::anyhow!("failed to deserialize TableMeta"))
    }
}

impl fmt::Display for TableMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TableMeta {{\"payer\": {}, \"size\": {}}}",
            self.payer, self.size
        )
    }
}
