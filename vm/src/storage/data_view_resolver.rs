#![forbid(unsafe_code)]

use std::{borrow::Borrow, cell::RefCell, collections::BTreeMap};

use crate::access_path::AccessPath;
use crate::table_owner::TableMetaType;

use super::state_view::StateView;
use crate::natives::table::{TableHandle, TableResolver};
use log::error;
use move_deps::move_binary_format::errors::{Location, PartialVMError, VMError, VMResult};
use move_deps::move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::{ModuleResolver, ResourceResolver},
    vm_status::StatusCode,
};

pub trait StoredSizeResolver {
    fn get_size(&self, access_path: &AccessPath) -> Option<usize>;
}

pub trait TableMetaResolver {
    fn get_table_meta(
        &self,
        handle: &TableHandle,
        meta: TableMetaType,
    ) -> VMResult<Option<Vec<u8>>>;
}

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
        match self.data_view.get(access_path) {
            Ok(remote_data) => {
                let mut cache = self.size_cache.borrow_mut();
                if !cache.contains_key(access_path) {
                    let size = match remote_data.borrow() {
                        Some(val) => {
                            let key_size = access_path.to_string().as_bytes().len();
                            key_size + val.len()
                        }
                        None => 0,
                    };
                    // let val_size = remote_data.borrow().as_ref().map_or(0, |f| f.len());
                    cache.insert(access_path.clone(), size);
                }
                Ok(remote_data)
            }
            Err(e) => {
                error!("[VM] Error getting data from storage for {:?}", access_path);
                Err(e)
            }
        }
    }
}

impl<'block, S: StateView> StoredSizeResolver for DataViewResolver<'block, S> {
    //TODO: should it return Result rather than Option?
    fn get_size(&self, access_path: &AccessPath) -> Option<usize> {
        self.size_cache.borrow().get(access_path).cloned()
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
    // type Error = VMError;
    //

    fn get_table_meta(
        &self,
        handle: &TableHandle,
        meta: TableMetaType,
    ) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::table_meta_access_path(handle.0, meta);
        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }
}
