use std::borrow::Borrow;
use std::fmt::Formatter;
use std::str::FromStr;
use std::{collections::BTreeMap, fmt, fmt::Display};

use crate::natives::table::{TableChangeSet, TableHandle};
use crate::session::SessionExt;
use crate::size_change_set::{AccountSizeChangeSet, SizeChangeSet, SizeDelta};
use crate::storage::data_view_resolver::TableMetaResolver;
use anyhow::bail;
use move_deps::move_core_types::effects::ChangeSet;
use move_deps::move_core_types::language_storage::TypeTag;
use move_deps::move_core_types::resolver::MoveResolver;
use move_deps::move_core_types::value::MoveTypeLayout;
use move_deps::{
    move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult},
    move_core_types::{account_address::AccountAddress, effects::Op, vm_status::StatusCode},
    move_vm_types::{
        values::Value,
        views::{ValueView, ValueVisitor},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub enum TableMetaType {
    #[serde(rename = "owner", alias = "Owner")]
    Owner,
    #[serde(rename = "size", alias = "Size")]
    Size,
}
impl Display for TableMetaType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TableMetaType::Owner => write!(f, "owner"),
            TableMetaType::Size => write!(f, "size"),
        }
    }
}

impl FromStr for TableMetaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::Owner),
            "size" => Ok(Self::Size),
            _ => bail!("error"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub enum TableMetaValue {
    Owner(AccountAddress),
    Size(usize),
}

fn find_all_address_occur(
    op: &Op<Vec<u8>>,
    ty_layout: &MoveTypeLayout,
) -> VMResult<Vec<AccountAddress>> {
    let v = match op.as_ref().ok() {
        Some(blob) => {
            let val = Value::simple_deserialize(&blob, ty_layout).ok_or(
                PartialVMError::new(StatusCode::FAILED_TO_SERIALIZE_WRITE_SET_CHANGES)
                    .finish(Location::Undefined),
            )?;

            let mut visitor = FindingAddressVisitor::new();
            val.visit(&mut visitor);
            let res = visitor.finish();
            res
        }
        None => Vec::default(),
    };

    Ok(v)
}

pub fn resolve_table_ownership<S: TableMetaResolver + MoveResolver>(
    session: SessionExt<'_, '_, S>,
    change_set: &ChangeSet,
    table_change_set: &TableChangeSet,
    table_owner_data_cache: &mut TableOwnerDataCache<'_, S>,
) -> VMResult<()> {
    let new_tables = &table_change_set.borrow().new_tables;

    println!("new table {:?}", table_change_set.new_tables);
    println!("remove table {:?}", table_change_set.removed_tables);

    for i in table_change_set.removed_tables.iter() {
        table_owner_data_cache.del_owner(i);
        table_owner_data_cache.del_size(i);
    }

    for (addr, account_change_set) in change_set.borrow().accounts().iter() {
        println!("checking changeset for {}", addr);
        for (i, op) in account_change_set.resources().iter() {
            let ty_tag = TypeTag::Struct(i.clone());
            let ty_layout = session.get_type_layout(&ty_tag)?;
            let res = find_all_address_occur(op, &ty_layout)?;

            for address_found in res.into_iter() {
                let found_handle = TableHandle(address_found);
                // address is new table's handle or
                // already stored table's handle

                if new_tables.contains_key(&found_handle)
                    || table_owner_data_cache.is_registerd_table(&found_handle)?
                {
                    table_owner_data_cache.set_owner(&found_handle, *addr);
                }
            }
        }
    }

    for (outer_handle, change) in table_change_set.changes.iter() {
        for (_key, op) in &change.entries {
            let res = find_all_address_occur(op, &change.value_layout)?;

            for address_found in res.iter() {
                let found_handle = TableHandle(*address_found);
                if new_tables.contains_key(&found_handle)
                    || table_owner_data_cache.is_registerd_table(&found_handle)?
                {
                    table_owner_data_cache.set_owner(&found_handle, outer_handle.0);
                }
            }
        }
    }
    Ok(())
}

pub fn resolve_table_size_change_by_account<S: TableMetaResolver>(
    table_change_set: &TableChangeSet,
    table_size_change: &SizeChangeSet<TableHandle>,
    table_owner_data_cache: &mut TableOwnerDataCache<'_, S>,
) -> VMResult<AccountSizeChangeSet> {
    let mut accounts_delta: AccountSizeChangeSet = AccountSizeChangeSet::default();

    println!("new table {:?}", table_change_set.new_tables);
    println!("remove table {:?}", table_change_set.removed_tables);

    // handle previous size
    for (handle, owner) in table_owner_data_cache.get_owner_changes().into_iter() {
        let size = match table_owner_data_cache.get_size(handle)? {
            Some(s) => s,
            None => {
                continue;
            } // skip new table
        };

        match owner {
            Some(new_owner) => {
                let old_owner = table_owner_data_cache.get_root_owner(handle)?.unwrap();
                accounts_delta.move_size(old_owner, new_owner.clone(), size);
            }
            None => {
                let acc = table_owner_data_cache.get_root_owner(handle)?.unwrap();
                let delta = SizeDelta::decreasing(size);
                accounts_delta.insert_size(acc, delta);
            }
        }
    }
    println!("table size owning???? {:?}", accounts_delta);

    // handle size changes
    for (handle, size_delta) in table_size_change.changes() {
        // todo: should check deleted table?? yes
        if table_change_set.removed_tables.contains(handle) {
            continue;
        }
        let owner = table_owner_data_cache.get_root_owner(handle)?.unwrap();

        accounts_delta.insert_size(owner, size_delta.clone());

        let current_size = table_owner_data_cache.get_size(&handle)?.unwrap_or(0);

        let mut delta = SizeDelta::increasing(current_size);
        delta.merge(size_delta.clone());
        if delta.is_decrease {
            panic!("dsdads");
        }
        let new_size = delta.amount;

        table_owner_data_cache.set_size(&handle, new_size);
    }

    println!("{:?}", accounts_delta);

    Ok(accounts_delta)
}

struct FindingAddressVisitor {
    addresses: Vec<AccountAddress>,
}

impl<'a> FindingAddressVisitor {
    fn new() -> Self {
        Self {
            addresses: Vec::default(),
        }
    }

    fn finish(self) -> Vec<AccountAddress> {
        self.addresses
    }
}

impl<'a> ValueVisitor for FindingAddressVisitor {
    #[inline]
    fn visit_u8(&mut self, _depth: usize, _val: u8) {}

    #[inline]
    fn visit_u64(&mut self, _depth: usize, _val: u64) {}

    #[inline]
    fn visit_u128(&mut self, _depth: usize, _val: u128) {}

    #[inline]
    fn visit_bool(&mut self, _depth: usize, _val: bool) {}

    #[inline]
    fn visit_address(&mut self, _depth: usize, _val: AccountAddress) {
        self.addresses.push(_val);
    }

    #[inline]
    fn visit_struct(&mut self, _depth: usize, _len: usize) -> bool {
        true
    }

    #[inline]
    fn visit_vec(&mut self, _depth: usize, _len: usize) -> bool {
        true
    }

    #[inline]
    fn visit_vec_u8(&mut self, _depth: usize, _vals: &[u8]) {}

    #[inline]
    fn visit_vec_u64(&mut self, _depth: usize, _vals: &[u64]) {}

    #[inline]
    fn visit_vec_u128(&mut self, _depth: usize, _vals: &[u128]) {}

    #[inline]
    fn visit_vec_bool(&mut self, _depth: usize, _vals: &[bool]) {}

    #[inline]
    fn visit_vec_address(&mut self, _depth: usize, vals: &[AccountAddress]) {
        self.addresses.append(vals.to_vec().as_mut());
    }

    #[inline]
    fn visit_ref(&mut self, _depth: usize, _is_global: bool) -> bool {
        false
    }
}

#[derive(Default)]
pub struct TableMetaChangeSet {
    pub owner: BTreeMap<TableHandle, Op<Vec<u8>>>,
    pub size: BTreeMap<TableHandle, Op<Vec<u8>>>,
}

pub struct TableOwnerDataCache<'r, S> {
    remote: &'r S,
    table_owner: BTreeMap<TableHandle, WriteCache>,
    table_size: BTreeMap<TableHandle, WriteCache>,
}

impl<'r, S: TableMetaResolver> TableOwnerDataCache<'r, S> {
    pub fn new(remote: &'r S) -> Self {
        Self {
            remote,
            table_owner: BTreeMap::default(),
            table_size: BTreeMap::default(),
        }
    }

    pub fn get_owner_changes(&self) -> BTreeMap<&TableHandle, Option<&AccountAddress>> {
        self.table_owner
            .iter()
            .map(|(handle, cache)| (handle, cache.get_owner()))
            .collect()
    }

    pub fn into_change_set(self) -> PartialVMResult<TableMetaChangeSet> {
        let mut owner = BTreeMap::new();
        for (handle, val) in self.table_owner {
            let op = match val.into_effect() {
                Some(op) => op,
                None => continue,
            };

            owner.insert(handle, serialize_op(op)?);
        }

        let mut size = BTreeMap::new();
        for (handle, val) in self.table_size {
            let op = match val.into_effect() {
                Some(op) => op,
                None => continue,
            };

            size.insert(handle, serialize_op(op)?);
        }

        Ok(TableMetaChangeSet { owner, size })
    }

    pub fn get_size(&self, handle: &TableHandle) -> VMResult<Option<usize>> {
        let res = match self.table_size.get(handle) {
            Some(cached) => cached.get_size().cloned(),
            None => {
                let val = self.remote.get_table_meta(handle, TableMetaType::Size)?;
                match val {
                    Some(blob) => deserialize_size(&blob),
                    None => None,
                }
            }
        };
        Ok(res)
    }

    pub fn set_size(&mut self, handle: &TableHandle, size: usize) {
        self.table_size.insert(
            handle.clone(),
            WriteCache::Updated(WriteCacheValue::Size(size)),
        );
    }

    pub fn del_size(&mut self, handle: &TableHandle) {
        self.table_size.insert(handle.clone(), WriteCache::Deleted);
    }

    pub fn set_owner(&mut self, handle: &TableHandle, owner: AccountAddress) {
        self.table_owner.insert(
            handle.clone(),
            WriteCache::Updated(WriteCacheValue::Owner(owner)),
        );
    }

    pub fn del_owner(&mut self, handle: &TableHandle) {
        self.table_owner.insert(handle.clone(), WriteCache::Deleted);
    }

    pub fn is_registerd_table(&self, handle: &TableHandle) -> VMResult<bool> {
        match self.table_owner.contains_key(handle) {
            true => Ok(true),
            false => {
                let val = self.remote.get_table_meta(handle, TableMetaType::Owner)?;

                match val {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
        }
    }

    // get table owner recursively
    pub fn get_root_owner(&self, handle: &TableHandle) -> VMResult<Option<AccountAddress>> {
        let owner = match self.table_owner.get(handle) {
            Some(cached) => cached.get_owner().cloned(),
            None => {
                let val = self.remote.get_table_meta(handle, TableMetaType::Owner)?;

                match val {
                    Some(blob) => deserialize_owner(&blob),
                    None => None,
                }
            }
        };

        let child_or_me = match owner {
            Some(o) => Some(self.get_root_owner(&TableHandle(o))?.unwrap_or(o)),
            None => None,
        };

        Ok(child_or_me)
    }
}

#[derive(Debug)]
enum WriteCache {
    /// A resource resides in this slot and will be write into storage.
    Updated(WriteCacheValue),
    /// A resource used to exist in storage but has been deleted by the current transaction.
    Deleted,
}

impl WriteCache {
    fn get_size(&self) -> Option<&usize> {
        match self {
            WriteCache::Updated(val) => match val {
                WriteCacheValue::Owner(_) => panic!("not found"),
                WriteCacheValue::Size(size) => Some(size),
            },
            WriteCache::Deleted => None,
        }
    }

    fn get_owner(&self) -> Option<&AccountAddress> {
        match self {
            WriteCache::Updated(val) => match val {
                WriteCacheValue::Owner(owner) => Some(owner),
                WriteCacheValue::Size(_) => panic!("not"),
            },
            WriteCache::Deleted => None,
        }
    }

    fn into_effect(self) -> Option<Op<WriteCacheValue>> {
        match self {
            Self::Deleted => Some(Op::Delete),
            Self::Updated(val) => Some(Op::Modify(val)),
        }
    }
}

impl Display for WriteCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WriteCache::Updated(val) => write!(f, "Updated({})", val),
            WriteCache::Deleted => write!(f, "Deleted"),
        }
    }
}

#[derive(Debug)]
enum WriteCacheValue {
    Owner(AccountAddress),
    Size(usize),
}

impl WriteCacheValue {
    fn serialize(self) -> Option<Vec<u8>> {
        match self {
            WriteCacheValue::Owner(owner) => serialize_owner(&owner),
            WriteCacheValue::Size(size) => serialize_size(&size),
        }
    }
}

impl Display for WriteCacheValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WriteCacheValue::Owner(val) => write!(f, "Owner({})", val),
            WriteCacheValue::Size(val) => write!(f, "Size({})", val),
        }
    }
}

fn serialize_op(op: Op<WriteCacheValue>) -> PartialVMResult<Op<Vec<u8>>> {
    match op {
        Op::New(v) => Ok(Op::New(serialize_or_error(v)?)),
        Op::Modify(v) => Ok(Op::Modify(serialize_or_error(v)?)),
        Op::Delete => Ok(Op::Delete),
    }
}

fn serialize_or_error(v: WriteCacheValue) -> PartialVMResult<Vec<u8>> {
    match v.serialize() {
        Some(d) => Ok(d),
        None => Err(PartialVMError::new(
            StatusCode::FAILED_TO_SERIALIZE_WRITE_SET_CHANGES,
        )),
    }
}

fn serialize_owner(owner: &AccountAddress) -> Option<Vec<u8>> {
    bcs::to_bytes(owner).ok()
}

fn deserialize_owner(blob: &[u8]) -> Option<AccountAddress> {
    bcs::from_bytes(blob).ok()
}

fn serialize_size(size: &usize) -> Option<Vec<u8>> {
    bcs::to_bytes(size).ok()
}

fn deserialize_size(blob: &[u8]) -> Option<usize> {
    bcs::from_bytes(blob).ok()
}
