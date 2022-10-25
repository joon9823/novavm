use std::collections::BTreeMap;

use move_deps::move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    language_storage::ModuleId,
    vm_status::{StatusCode, VMStatus},
};

use nova_types::access_path::AccessPath;
use nova_types::size_delta::SizeDelta;

use self::size_resolver::SizeResolver;

pub mod size_resolver;

pub fn compute_size_changes<S: SizeResolver>(
    remote: &S,
    change_set: &ChangeSet,
) -> Result<BTreeMap<AccountAddress, SizeDelta>, VMStatus> {
    change_set
        .accounts()
        .iter()
        .map(|f| {
            let mut account_delta = SizeDelta::zero();
            let addr = f.0;
            let account_change_set = f.1;
            for (i, op) in account_change_set.modules().iter() {
                let ap = AccessPath::from(&ModuleId::new(addr.clone(), i.clone()));
                let prev = remote.get_size(&ap)?;
                let new = get_kv_stored_size(&ap, op);
                let delta = SizeDelta::new(prev, new);
                account_delta.merge(delta);
            }

            for (i, op) in account_change_set.resources().iter() {
                let ap = AccessPath::resource_access_path(addr.clone(), i.clone());
                let prev = remote.get_size(&ap)?;
                let new = get_kv_stored_size(&ap, op);

                let delta = SizeDelta::new(prev, new);
                account_delta.merge(delta);
            }

            Ok((addr.clone(), account_delta))
        })
        .collect::<anyhow::Result<BTreeMap<AccountAddress, SizeDelta>>>()
        .map_err(|_| VMStatus::Error(StatusCode::LOOKUP_FAILED))
}

pub fn get_kv_stored_size(k: &AccessPath, v: &Op<Vec<u8>>) -> usize {
    match v.as_ref().ok() {
        Some(data) => {
            let ap_size = k.to_string().as_bytes().len();
            let op_size = data.len();
            ap_size + op_size
        }
        None => 0,
    }
}
