use std::{
    borrow::Borrow,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use crate::size_change_set::SizeDelta;
use crate::{access_path::AccessPath, storage::data_view_resolver::StoredSizeResolver};
use crate::{
    natives::table::{NativeTableContext, TableChangeSet, TableHandle, TableResolver},
    size_change_set::SizeChangeSet,
};
use move_deps::{
    move_binary_format::errors::Location,
    move_core_types::{
        account_address::AccountAddress,
        effects::{ChangeSet, Event, Op},
        language_storage::ModuleId,
        resolver::MoveResolver,
        vm_status::VMStatus,
    },
    move_vm_runtime::session::Session,
};

pub type SessionOutput = (
    ChangeSet,
    Vec<Event>,
    TableChangeSet,
    SizeChangeSet<AccountAddress>,
    SizeChangeSet<TableHandle>,
);

pub struct SessionExt<'r, 'l, S> {
    remote: &'r S,
    inner: Session<'r, 'l, S>,
}

impl<'r, 'l, S> SessionExt<'r, 'l, S>
where
    S: MoveResolver + TableResolver + StoredSizeResolver,
{
    pub fn new(inner: Session<'r, 'l, S>, remote: &'r S) -> Self {
        Self { inner, remote }
    }

    pub fn finish(self) -> Result<SessionOutput, VMStatus> {
        let (change_set, events, mut extensions) = self
            .inner
            .finish_with_extensions()
            .map_err(|e| e.into_vm_status())?;
        let table_context: NativeTableContext = extensions.remove::<NativeTableContext>();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;

        let account_size_change: BTreeMap<AccountAddress, SizeDelta> = change_set
            .borrow()
            .accounts()
            .iter()
            .map(|f| {
                let mut account_delta = SizeDelta::zero();
                let addr = f.0;
                let account_change_set = f.1;
                for (i, op) in account_change_set.modules().iter() {
                    let ap = AccessPath::from(&ModuleId::new(addr.clone(), i.clone()));
                    let prev = self.remote.get_size(&ap);
                    let new = get_kv_stored_size(&ap, op);
                    let delta = SizeDelta::new(prev, new);
                    println!("module size {} : {} => {} : {}", ap, prev, new, delta);
                    account_delta.merge(delta);
                }

                for (i, op) in account_change_set.resources().iter() {
                    let ap = AccessPath::resource_access_path(addr.clone(), i.clone());
                    let prev = self.remote.get_size(&ap);
                    let new = get_kv_stored_size(&ap, op);

                    let delta = SizeDelta::new(prev, new);
                    println!("resource size {} : {} => {} : {}", ap, prev, new, delta);
                    account_delta.merge(delta);
                }

                println!("account delta : {}", account_delta);
                return (addr.clone(), account_delta);
            })
            .collect();

        let table_size_change: BTreeMap<TableHandle, SizeDelta> = table_change_set
            .changes
            .iter()
            .map(|(handle, change)| {
                let mut table_delta = SizeDelta::zero();
                for (key, op) in &change.entries {
                    let ap = AccessPath::table_item_access_path(handle.0, key.to_vec());
                    let prev = self.remote.get_size(&ap);
                    let new = get_kv_stored_size(&ap, op);
                    let delta = SizeDelta::new(prev, new);

                    println!("table size {} : {} => {} : {}", ap, prev, new, delta);
                    table_delta.merge(delta);
                }
                return (handle.clone(), table_delta);
            })
            .collect();

        Ok((
            change_set,
            events,
            table_change_set,
            SizeChangeSet::new(account_size_change),
            SizeChangeSet::new(table_size_change),
        ))
    }
}

impl<'r, 'l, S> Deref for SessionExt<'r, 'l, S> {
    type Target = Session<'r, 'l, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'r, 'l, S> DerefMut for SessionExt<'r, 'l, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn get_kv_stored_size(k: &AccessPath, v: &Op<Vec<u8>>) -> usize {
    match v.as_ref().ok() {
        Some(data) => {
            let ap_size = k.to_string().as_bytes().len();
            let op_size = data.len();
            ap_size + op_size
        }
        None => 0,
    }
}
