use std::collections::{BTreeMap, BTreeSet};

use move_deps::move_core_types::effects::Op;
use nova_natives::table::{TableHandle, TableInfo};

use super::TableMeta;

#[derive(Debug)]
pub struct TableMetaChangeSet(BTreeMap<TableHandle, Op<TableMeta>>);

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
    ) -> TableMetaChangeSet {
        let mut change_set: BTreeMap<TableHandle, Op<TableMeta>> = BTreeMap::new();
        for (handle, _) in new_tables.iter() {
            // `new_tables` always create `table_meta_changes` with `payer_changes`
            let (handle, table_meta) = table_meta_changes.remove_entry(handle).unwrap();
            change_set.insert(handle, Op::New(table_meta));
        }

        for (handle, table_meta) in table_meta_changes.into_iter() {
            change_set.insert(handle, Op::Modify(table_meta));
        }

        // override all updates if the table removed
        for handle in remove_tables.iter() {
            change_set.insert(*handle, Op::Delete);
        }

        Self(change_set)
    }

    pub fn changes(&self) -> &BTreeMap<TableHandle, Op<TableMeta>> {
        &self.0
    }

    pub fn into_changes(self) -> BTreeMap<TableHandle, Op<TableMeta>> {
        self.0
    }
}
