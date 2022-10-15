use move_deps::move_core_types::{
    account_address::AccountAddress,
    resolver::MoveResolver,
    vm_status::{StatusCode, VMStatus},
};
use nova_natives::table::{TableChangeSet, TableHandle};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

use crate::access_path::AccessPath;

use self::table_meta_resolver::TableMetaResolver;

use super::size::{get_kv_stored_size, size_delta::SizeDelta, size_resolver::SizeResolver};

pub mod table_meta_change_set;
pub mod table_meta_resolver;

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

pub fn compute_table_meta_changes<S: MoveResolver + SizeResolver + TableMetaResolver>(
    remote: &S,
    table_change_set: &TableChangeSet,
    account_size_changes: &mut BTreeMap<AccountAddress, SizeDelta>,
) -> Result<BTreeMap<TableHandle, TableMeta>, VMStatus> {
    // compute table size delta from the table_change_set
    let mut table_meta_changes = table_change_set
        .changes
        .iter()
        .map(|(handle, change)| {
            let mut table_meta = remote
                .get_table_meta(&handle)?
                .unwrap_or_else(|| TableMeta::new());
            let mut table_delta = SizeDelta::zero();
            for (key, op) in &change.entries {
                let ap = AccessPath::table_item_access_path(handle.0, key.to_vec());
                let prev = remote.get_size(&ap)?;
                let new = get_kv_stored_size(&ap, op);
                let delta = SizeDelta::new(prev, new);

                table_delta.merge(delta);
            }

            table_meta.size = if table_delta.is_decrease {
                table_meta.size - table_delta.amount
            } else {
                table_meta.size + table_delta.amount
            };

            // apply table size delta to payer
            if table_meta.payer != AccountAddress::ZERO {
                if let Some(size_delta) = account_size_changes.get_mut(&table_meta.payer) {
                    size_delta.merge(table_delta);
                } else {
                    account_size_changes.insert(table_meta.payer, table_delta);
                }
            }

            Ok((*handle, table_meta))
        })
        .collect::<anyhow::Result<BTreeMap<TableHandle, TableMeta>>>()
        .map_err(|_| VMStatus::Error(StatusCode::LOOKUP_FAILED))?;

    // apply `payer_changes` to `table_meta_changes`
    // if the table has non-zero payer address => transfer storage fee to new address
    // else (the table ahs zero payer address) => just set payer to new table_meta
    for (handle, payer) in table_change_set.payer_changes.iter() {
        match table_meta_changes.get_mut(handle) {
            Some(mut table_meta) => {
                transfer_table_storage_fee(payer, &table_meta, account_size_changes);
                table_meta.payer = *payer;
            }
            None => {
                let table_meta = remote
                    .get_table_meta(handle)
                    .map_err(|_| VMStatus::Error(StatusCode::LOOKUP_FAILED))?
                    .unwrap_or_else(|| TableMeta::new());

                transfer_table_storage_fee(payer, &table_meta, account_size_changes);
                table_meta_changes.insert(*handle, table_meta);
            }
        }
    }

    Ok(table_meta_changes)
}

fn transfer_table_storage_fee(
    payer: &AccountAddress,
    table_meta: &TableMeta,
    account_size_changes: &mut BTreeMap<AccountAddress, SizeDelta>,
) {
    // decrease storage size of a previous payer
    if table_meta.payer != AccountAddress::ZERO {
        if let Some(size_delta) = account_size_changes.get_mut(&table_meta.payer) {
            size_delta.merge(SizeDelta::decreasing(table_meta.size));
        } else {
            account_size_changes.insert(table_meta.payer, SizeDelta::decreasing(table_meta.size));
        }
    }

    // increase storage size of a new payer
    if let Some(size_delta) = account_size_changes.get_mut(payer) {
        size_delta.merge(SizeDelta::increasing(table_meta.size));
    } else {
        account_size_changes.insert(payer.clone(), SizeDelta::increasing(table_meta.size));
    }
}
