use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use nova_natives::table::NativeTableContext;
use nova_types::{
    size_change_set::SizeChangeSet, size_delta::SizeDelta, table::TableHandle,
    table_meta::TableMeta, table_meta_change_set::TableMetaChangeSet, write_set::WriteSet,
};

use nova_storage::size::size_resolver::SizeResolver;
use nova_storage::table_meta::table_meta_resolver::TableMetaResolver;
use nova_storage::{size::compute_size_changes, table_meta::compute_table_meta_changes};

use move_deps::{
    move_binary_format::errors::Location,
    move_core_types::{
        account_address::AccountAddress, effects::Event, resolver::MoveResolver,
        vm_status::VMStatus,
    },
    move_vm_runtime::session::Session,
};

pub type SessionOutput = (Vec<Event>, WriteSet, SizeChangeSet);
pub fn empty_session_output() -> SessionOutput {
    (vec![], WriteSet::default(), SizeChangeSet::default())
}

pub struct SessionExt<'r, 'l, S> {
    resolver: &'r S,
    inner: Session<'r, 'l, S>,
}

impl<'r, 'l, S> SessionExt<'r, 'l, S>
where
    S: MoveResolver + SizeResolver + TableMetaResolver,
{
    pub fn new(inner: Session<'r, 'l, S>, resolver: &'r S) -> Self {
        Self { inner, resolver }
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
            compute_size_changes(self.resolver, &change_set)?;

        // Compute storage size delta for all tables to charge storage fee
        let table_meta_changes: BTreeMap<TableHandle, TableMeta> =
            compute_table_meta_changes(self.resolver, &table_change_set, &mut size_changes)?;

        let table_meta_change_set = TableMetaChangeSet::new(
            table_meta_changes,
            &table_change_set.new_tables,
            &table_change_set.removed_tables,
        )?;

        // build output change set from the changes
        let size_change_set = SizeChangeSet::new(size_changes);
        let write_set = WriteSet::new(change_set, table_change_set, table_meta_change_set);

        Ok((events, write_set, size_change_set))
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
