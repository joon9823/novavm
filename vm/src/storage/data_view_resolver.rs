#![forbid(unsafe_code)]

use crate::access_path::AccessPath;

use super::state_view::StateView;
use log::error;
use move_deps::{
    move_binary_format::errors::{Location, PartialVMError, VMError, VMResult},
    move_table_extension::{TableHandle, TableResolver},
};
use move_deps::move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
    vm_status::StatusCode,
};

pub struct DataViewResolver<'a, S> {
    data_view: &'a S,
}

impl<'a, S: StateView> DataViewResolver<'a, S> {
    pub fn new(data_view: &'a S) -> Self {
        Self { data_view }
    }

    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.data_view.get(access_path) {
            Ok(remote_data) => Ok(remote_data),
            Err(e) => {
                error!("[VM] Error getting data from storage for {:?}", access_path);
                Err(e)
            }
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

impl<'block, S: TableResolver> TableResolver for DataViewResolver<'block, S> {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, anyhow::Error> {
        self.data_view.resolve_table_entry(handle, key)
    }
}