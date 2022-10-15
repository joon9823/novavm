#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::BTreeMap};

use crate::access_path::AccessPath;

use super::size::size_resolver::SizeResolver;
use super::state_view::StateView;
use super::table_meta::table_meta_resolver::TableMetaResolver;
use super::table_meta::TableMeta;

use move_deps::move_binary_format::errors::{Location, PartialVMError, VMError, VMResult};
use move_deps::move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
    vm_status::StatusCode,
};

use nova_natives::table::{TableHandle, TableResolver};

pub struct DataViewResolver<'a, S> {
    data_view: &'a S,
    pub size_cache: RefCell<BTreeMap<AccessPath, usize>>,
}

impl<'a, S: StateView> DataViewResolver<'a, S> {
    pub fn new(data_view: &'a S) -> Self {
        Self {
            data_view,
            size_cache: RefCell::new(BTreeMap::default()),
        }
    }

    pub(crate) fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.data_view.get(access_path)
    }
}

impl<'block, S: StateView> SizeResolver for DataViewResolver<'block, S> {
    fn get_size(&self, access_path: &AccessPath) -> anyhow::Result<usize> {
        match self.get(access_path)? {
            Some(v) => Ok(access_path.to_string().as_bytes().len() + v.len()),
            None => Ok(0),
        }
    }
}

impl<'block, S: StateView> ModuleResolver for DataViewResolver<'block, S> {
    type Error = VMError;

    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::from(module_id);

        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }
}

impl<'block, S: StateView> ResourceResolver for DataViewResolver<'block, S> {
    type Error = VMError;

    fn get_resource(&self, address: &AccountAddress, tag: &StructTag) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::resource_access_path(*address, tag.clone());

        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }
}

impl<'block, S: StateView> TableResolver for DataViewResolver<'block, S> {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, anyhow::Error> {
        let ap = AccessPath::table_item_access_path(handle.0, key.to_vec());
        self.get(&ap)
    }
}

impl<'block, S: StateView> TableMetaResolver for DataViewResolver<'block, S> {
    fn get_table_meta(&self, handle: &TableHandle) -> Result<Option<TableMeta>, anyhow::Error> {
        let ap = AccessPath::table_meta_access_path(handle.0);
        let table_meta = match self.get(&ap)? {
            Some(v) => Some(TableMeta::deserialize(&v)?),
            None => None,
        };
        Ok(table_meta)
    }
}
