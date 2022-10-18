use anyhow::Error;
use move_deps::move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
};
use nova_natives::{
    block::BlockInfoResolver,
    table::{TableHandle, TableResolver},
};

/// A dummy storage containing no modules or resources.
#[derive(Debug, Clone)]
pub struct BlankStorage;

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

pub struct MockApi {
    pub height: u64,
    pub timestamp: u64,
}

impl BlockInfoResolver for MockApi {
    fn get_block_info(&self) -> anyhow::Result<(u64 /* height */, u64 /* timestamp */)> {
        Ok((self.height, self.timestamp))
    }
}
