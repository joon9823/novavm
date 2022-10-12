use std::collections::BTreeMap;

use move_deps::{
    move_binary_format::errors::{PartialVMResult, VMResult},
    move_core_types::{account_address::AccountAddress, effects::Op},
};

use crate::{natives::table::TableHandle, storage::data_view_resolver::TableMetaResolver};

use super::{
    effects::{deserialize_owner, deserialize_size, serialize_op, WriteCache, WriteCacheValue},
    TableMetaType,
};

#[derive(Default)]
pub struct TableMetaChangeSet {
    pub owner: BTreeMap<TableHandle, Op<Vec<u8>>>,
    pub size: BTreeMap<TableHandle, Op<Vec<u8>>>,
}

pub struct TableMetaDataCache<'r, S> {
    remote: &'r S,
    table_owner: BTreeMap<TableHandle, WriteCache>,
    table_size: BTreeMap<TableHandle, WriteCache>,
}

impl<'r, S: TableMetaResolver> TableMetaDataCache<'r, S> {
    pub fn new(remote: &'r S) -> Self {
        Self {
            remote,
            table_owner: BTreeMap::default(),
            table_size: BTreeMap::default(),
        }
    }

    pub fn get_owner_changes(&self) -> BTreeMap<&TableHandle, Option<&AccountAddress>> {
        self.table_owner
            .iter()
            .map(|(handle, cache)| (handle, cache.get_owner()))
            .collect()
    }

    pub fn into_change_set(self) -> PartialVMResult<TableMetaChangeSet> {
        let mut owner = BTreeMap::new();
        for (handle, val) in self.table_owner {
            let op = match val.into_effect() {
                Some(op) => op,
                None => continue,
            };

            owner.insert(handle, serialize_op(op)?);
        }

        let mut size = BTreeMap::new();
        for (handle, val) in self.table_size {
            let op = match val.into_effect() {
                Some(op) => op,
                None => continue,
            };

            size.insert(handle, serialize_op(op)?);
        }

        Ok(TableMetaChangeSet { owner, size })
    }

    pub fn get_size(&self, handle: &TableHandle) -> VMResult<Option<usize>> {
        let res = match self.table_size.get(handle) {
            Some(cached) => cached.get_size().cloned(),
            None => {
                let val = self.remote.get_table_meta(handle, TableMetaType::Size)?;
                match val {
                    Some(blob) => deserialize_size(&blob),
                    None => None,
                }
            }
        };
        Ok(res)
    }

    pub fn set_size(&mut self, handle: &TableHandle, size: usize) {
        self.table_size.insert(
            handle.clone(),
            WriteCache::Updated(WriteCacheValue::Size(size)),
        );
    }

    pub fn del_size(&mut self, handle: &TableHandle) {
        self.table_size.insert(handle.clone(), WriteCache::Deleted);
    }

    pub fn set_owner(&mut self, handle: &TableHandle, owner: AccountAddress) {
        self.table_owner.insert(
            handle.clone(),
            WriteCache::Updated(WriteCacheValue::Owner(owner)),
        );
    }

    pub fn del_owner(&mut self, handle: &TableHandle) {
        self.table_owner.insert(handle.clone(), WriteCache::Deleted);
    }

    pub fn is_registerd_table(&self, handle: &TableHandle) -> VMResult<bool> {
        match self.table_owner.contains_key(handle) {
            true => Ok(true),
            false => {
                let val = self.remote.get_table_meta(handle, TableMetaType::Owner)?;

                match val {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
        }
    }

    // get table owner recursively
    pub fn get_root_owner(&self, handle: &TableHandle) -> VMResult<Option<AccountAddress>> {
        let owner = match self.table_owner.get(handle) {
            Some(cached) => cached.get_owner().cloned(),
            None => {
                let val = self.remote.get_table_meta(handle, TableMetaType::Owner)?;

                match val {
                    Some(blob) => deserialize_owner(&blob),
                    None => None,
                }
            }
        };

        let child_or_me = match owner {
            Some(o) => Some(self.get_root_owner(&TableHandle(o))?.unwrap_or(o)),
            None => None,
        };

        Ok(child_or_me)
    }

    // get previous uncached owner
    pub fn get_old_root_owner(&self, handle: &TableHandle) -> VMResult<Option<AccountAddress>> {
        let val = self.remote.get_table_meta(handle, TableMetaType::Owner)?;
        let owner = match val {
            Some(blob) => deserialize_owner(&blob),
            None => None,
        };

        let child_or_me = match owner {
            Some(o) => Some(self.get_old_root_owner(&TableHandle(o))?.unwrap_or(o)),
            None => None,
        };

        Ok(child_or_me)
    }
}
