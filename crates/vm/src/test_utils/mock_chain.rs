use crate::storage::table_meta::table_meta_change_set::TableMetaChangeSet;
use crate::storage::table_meta::TableMeta;
use crate::{access_path::AccessPath, storage::state_view::StateView};
use std::collections::BTreeMap;

use move_deps::move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    language_storage::ModuleId,
    language_storage::StructTag,
    resolver::{ModuleResolver, ResourceResolver},
};
use nova_natives::{
    block::BlockInfoResolver,
    table::{TableChangeSet, TableHandle, TableResolver},
};

use anyhow::Error;

#[derive(Debug)]
pub struct MockChain {
    map: BTreeMap<AccessPath, Option<Vec<u8>>>,
}
impl MockChain {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    // not scalable because it simply clones current map
    pub fn create_state(&self) -> MockState {
        MockState {
            map: self.map.clone(),
        }
    }

    pub fn create_api(&self, height: u64, timestamp: u64) -> MockApi {
        MockApi { height, timestamp }
    }

    pub fn commit(&mut self, state: MockState) {
        self.map = state.map;
    }
}

pub struct MockState {
    map: BTreeMap<AccessPath, Option<Vec<u8>>>,
}

impl MockState {
    fn write_op(&mut self, ref ap: AccessPath, ref blob_opt: Op<Vec<u8>>) {
        match blob_opt {
            Op::New(blob) | Op::Modify(blob) => {
                self.map.insert(ap.clone(), Some(blob.clone()));
            }
            Op::Delete => {
                self.map.remove(ap);
                self.map.insert(ap.clone(), None);
            }
        }
    }

    fn write_table_meta_op(&mut self, ref ap: AccessPath, ref blob_opt: Op<TableMeta>) {
        match blob_opt {
            Op::New(blob) | Op::Modify(blob) => {
                self.map
                    .insert(ap.clone(), Some(TableMeta::serialize(blob).unwrap()));
            }
            Op::Delete => {
                self.map.remove(ap);
                self.map.insert(ap.clone(), None);
            }
        }
    }

    pub fn push_write_set(
        &mut self,
        changeset: ChangeSet,
        table_change_set: TableChangeSet,
        table_meta_change_set: TableMetaChangeSet,
    ) {
        for (addr, account_changeset) in changeset.into_inner() {
            let (modules, resources) = account_changeset.into_inner();
            for (struct_tag, blob_opt) in resources {
                let ap = AccessPath::resource_access_path(addr, struct_tag);
                self.write_op(ap, blob_opt)
            }

            for (name, blob_opt) in modules {
                let ap = AccessPath::from(&ModuleId::new(addr, name));
                self.write_op(ap, blob_opt)
            }
        }

        for (handle, change) in table_change_set.changes {
            for (key, blob_opt) in change.entries {
                let ap = AccessPath::table_item_access_path(handle.0, key.to_vec());
                self.write_op(ap, blob_opt)
            }
        }

        for (handle, blob_opt) in table_meta_change_set.into_changes() {
            let ap = AccessPath::table_meta_access_path(handle.0);
            self.write_table_meta_op(ap, blob_opt)
        }
    }
}

impl StateView for MockState {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.map.get(access_path) {
            Some(opt_data) => Ok(opt_data.clone()),
            None => Ok(None),
        }
    }
}

pub struct MockApi {
    pub height: u64,
    pub timestamp: u64,
}

impl BlockInfoResolver for MockApi {
    fn get_block_info(&self) -> anyhow::Result<(u64 /* height */, u64 /* timestamp */)> {
        Ok((self.height, self.timestamp))
    }
}

/// A dummy storage containing no modules or resources.
#[derive(Debug, Clone)]
pub struct BlankStorage;

impl BlankStorage {
    pub fn new() -> Self {
        Self
    }
}

impl ModuleResolver for BlankStorage {
    type Error = ();

    fn get_module(&self, _module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }
}

impl ResourceResolver for BlankStorage {
    type Error = ();

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }
}

impl TableResolver for BlankStorage {
    fn resolve_table_entry(
        &self,
        _handle: &TableHandle,
        _key: &[u8],
    ) -> Result<Option<Vec<u8>>, Error> {
        Ok(None)
    }
}
