use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use nova_natives::table::{NativeTableContext, TableChangeSet, TableHandle, TableResolver};

use crate::storage::size::size_resolver::SizeResolver;
use crate::storage::table_meta::{
    table_meta_change_set::TableMetaChangeSet, table_meta_resolver::TableMetaResolver,
};
use crate::storage::{
    size::{compute_size_changes, size_change_set::SizeChangeSet, size_delta::SizeDelta},
    table_meta::{compute_table_meta_changes, TableMeta},
};

use move_deps::{
    move_binary_format::errors::Location,
    move_core_types::{
        account_address::AccountAddress,
        effects::{ChangeSet, Event},
        resolver::MoveResolver,
        vm_status::VMStatus,
    },
    move_vm_runtime::session::Session,
};

pub type SessionOutput = (
    ChangeSet,
    Vec<Event>,
    TableChangeSet,
    SizeChangeSet,
    TableMetaChangeSet,
);

pub struct SessionExt<'r, 'l, S> {
    remote: &'r S,
    inner: Session<'r, 'l, S>,
}

impl<'r, 'l, S> SessionExt<'r, 'l, S>
where
    S: MoveResolver + TableResolver + SizeResolver + TableMetaResolver,
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

        // Compute storage size delta for all accounts to charge storage fee
        let mut size_changes: BTreeMap<AccountAddress, SizeDelta> =
            compute_size_changes(self.remote, &change_set)?;

        // Compute storage size delta for all tables to charge storage fee
        let table_meta_changes: BTreeMap<TableHandle, TableMeta> =
            compute_table_meta_changes(self.remote, &table_change_set, &mut size_changes)?;

        let table_meta_change_set = TableMetaChangeSet::new(
            table_meta_changes,
            &table_change_set.new_tables,
            &table_change_set.removed_tables,
        );
        let size_change_set = SizeChangeSet::new(size_changes);

        Ok((
            change_set,
            events,
            table_change_set,
            size_change_set,
            table_meta_change_set,
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
