use std::collections::{BTreeMap, BTreeSet};

use move_deps::move_core_types::{
    effects::Op,
    vm_status::{StatusCode, VMStatus},
};

use super::table::{TableHandle, TableInfo};
use super::table_meta::TableMeta;

#[derive(Debug)]
pub struct TableMetaChangeSet(BTreeMap<TableHandle, Op<Vec<u8>>>);

impl Default for TableMetaChangeSet {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

impl TableMetaChangeSet {
    pub fn new(
        mut table_meta_changes: BTreeMap<TableHandle, TableMeta>,
        new_tables: &BTreeMap<TableHandle, TableInfo>,
        remove_tables: &BTreeSet<TableHandle>,
    ) -> Result<TableMetaChangeSet, VMStatus> {
        let mut change_set: BTreeMap<TableHandle, Op<Vec<u8>>> = BTreeMap::new();
        for (handle, _) in new_tables.iter() {
            // `new_tables` always create `table_meta_changes` with `payer_changes`
            let (handle, table_meta) = table_meta_changes.remove_entry(handle).unwrap();
            change_set.insert(
                handle,
                Op::New(
                    TableMeta::serialize(&table_meta)
                        .map_err(|_| VMStatus::Error(StatusCode::VALUE_SERIALIZATION_ERROR))?,
                ),
            );
        }

        for (handle, table_meta) in table_meta_changes.into_iter() {
            change_set.insert(
                handle,
                Op::Modify(
                    TableMeta::serialize(&table_meta)
                        .map_err(|_| VMStatus::Error(StatusCode::VALUE_SERIALIZATION_ERROR))?,
                ),
            );
        }

        // override all updates if the table removed
        for handle in remove_tables.iter() {
            change_set.insert(*handle, Op::Delete);
        }

        Ok(Self(change_set))
    }

    pub fn changes(&self) -> &BTreeMap<TableHandle, Op<Vec<u8>>> {
        &self.0
    }

    pub fn into_changes(self) -> BTreeMap<TableHandle, Op<Vec<u8>>> {
        self.0
    }
}
