use crate::{
    access_path::AccessPath, table::TableChangeSet, table_meta_change_set::TableMetaChangeSet,
};
use move_deps::move_core_types::{
    effects::{ChangeSet, Op},
    language_storage::ModuleId,
};
use std::collections::{btree_map, BTreeMap};

pub type WriteOp = Op<Vec<u8>>;

pub struct WriteSet(BTreeMap<AccessPath, WriteOp>);

impl WriteSet {
    pub fn default() -> Self {
        Self(BTreeMap::new())
    }

    pub fn new(
        change_set: ChangeSet,
        table_change_set: TableChangeSet,
        table_meta_change_set: TableMetaChangeSet,
    ) -> Self {
        let mut write_set: BTreeMap<AccessPath, WriteOp> = BTreeMap::new();
        for (addr, account_changeset) in change_set.into_inner() {
            let (modules, resources) = account_changeset.into_inner();
            for (struct_tag, blob_opt) in resources {
                let ap = AccessPath::resource_access_path(addr, struct_tag);
                write_set.insert(ap, blob_opt);
            }

            for (name, blob_opt) in modules {
                let ap = AccessPath::from(&ModuleId::new(addr, name));
                write_set.insert(ap, blob_opt);
            }
        }

        for (handle, changes) in table_change_set.changes.into_iter() {
            for (key, blob_opt) in changes.entries {
                let ap = AccessPath::table_item_access_path(handle.0, key.to_vec());
                write_set.insert(ap, blob_opt);
            }
        }

        for (handle, blob_opt) in table_meta_change_set.into_changes() {
            let ap = AccessPath::table_meta_access_path(handle.0);
            write_set.insert(ap, blob_opt);
        }

        Self(write_set)
    }
}

impl ::std::iter::FromIterator<(AccessPath, WriteOp)> for WriteSet {
    fn from_iter<I: IntoIterator<Item = (AccessPath, WriteOp)>>(iter: I) -> Self {
        let mut ws = WriteSet::default();
        for write in iter {
            ws.0.insert(write.0, write.1);
        }
        ws
    }
}

impl<'a> IntoIterator for &'a WriteSet {
    type Item = (&'a AccessPath, &'a WriteOp);
    type IntoIter = btree_map::Iter<'a, AccessPath, WriteOp>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl ::std::iter::IntoIterator for WriteSet {
    type Item = (AccessPath, WriteOp);
    type IntoIter = btree_map::IntoIter<AccessPath, WriteOp>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
