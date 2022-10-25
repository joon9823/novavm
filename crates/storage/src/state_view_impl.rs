#![forbid(unsafe_code)]

use crate::size::size_resolver::SizeResolver;
use crate::table_meta::table_meta_resolver::TableMetaResolver;

use super::state_view::StateView;

use move_deps::move_binary_format::errors::{Location, PartialVMError, VMError, VMResult};
use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_core_types::language_storage::StructTag;
use move_deps::move_core_types::resolver::ResourceResolver;
use move_deps::move_core_types::{
    language_storage::ModuleId, resolver::ModuleResolver, vm_status::StatusCode,
};

use nova_types::access_path::AccessPath;
use nova_types::table::TableHandle;
use nova_types::table_meta::TableMeta;

pub struct StateViewImpl<'block, S> {
    state_view: &'block S,
}

impl<'block, S: StateView> StateViewImpl<'block, S> {
    pub fn new(state_view: &'block S) -> Self {
        Self { state_view }
    }
}

impl<'block, S: StateView> StateViewImpl<'block, S> {
    pub(crate) fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.state_view.get(access_path)
    }
}

impl<'block, S: StateView> ModuleResolver for StateViewImpl<'block, S> {
    type Error = VMError;

    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::from(module_id);

        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }
}

impl<'block, S: StateView> ResourceResolver for StateViewImpl<'block, S> {
    type Error = VMError;

    fn get_resource(&self, address: &AccountAddress, tag: &StructTag) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::resource_access_path(*address, tag.clone());

        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }
}

impl<'block, S: StateView> SizeResolver for StateViewImpl<'block, S> {
    fn get_size(&self, access_path: &AccessPath) -> anyhow::Result<usize> {
        match self.get(access_path)? {
            Some(v) => Ok(access_path.to_bytes()?.len() + v.len()),
            None => Ok(0),
        }
    }
}

impl<'block, S: StateView> TableMetaResolver for StateViewImpl<'block, S> {
    fn get_table_meta(&self, handle: &TableHandle) -> Result<Option<TableMeta>, anyhow::Error> {
        let ap = AccessPath::table_meta_access_path(handle.0);
        let table_meta = match self.get(&ap)? {
            Some(v) => Some(TableMeta::deserialize(&v)?),
            None => None,
        };
        Ok(table_meta)
    }
}
