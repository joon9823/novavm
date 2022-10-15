use std::borrow::Borrow;
use std::fmt::Formatter;
use std::str::FromStr;
use std::{collections::BTreeMap, fmt::Display};

use crate::natives::table::{TableChangeSet, TableHandle, TableInfo};
use crate::session::SessionExt;
use crate::size_change_set::{AccountSizeChangeSet, SizeChangeSet, SizeDelta};
use crate::storage::data_view_resolver::TableMetaResolver;
use anyhow::bail;

use move_deps::move_core_types::effects::ChangeSet;
use move_deps::move_core_types::language_storage::TypeTag;
use move_deps::move_core_types::resolver::MoveResolver;
use move_deps::move_core_types::value::MoveTypeLayout;
use move_deps::{
    move_binary_format::errors::{Location, PartialVMError, VMResult},
    move_core_types::{account_address::AccountAddress, effects::Op, vm_status::StatusCode},
    move_vm_types::{values::Value, views::ValueView},
};
use serde::{Deserialize, Serialize};

mod data_cache;
mod effects;
mod find_address_visitor;

pub use data_cache::TableMetaChangeSet;
use data_cache::TableMetaDataCache;

use find_address_visitor::FindingAddressVisitor;

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
            _ => bail!("unexpected table meta type"),
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

fn filter_table_handle<S: TableMetaResolver>(
    new_tables: &BTreeMap<TableHandle, TableInfo>,
    data_cache: &mut TableMetaDataCache<'_, S>,
    address_list: Vec<AccountAddress>,
) -> VMResult<Vec<TableHandle>> {
    let mut res: Vec<TableHandle> = Vec::default();
    for address in address_list {
        let handle = TableHandle(address);

        // address is new table's handle or
        // already stored table's handle
        if new_tables.contains_key(&handle) || data_cache.is_registerd_table(&handle)? {
            res.push(handle);
        }
    }
    Ok(res)
}

fn resolve_table_ownership<S: TableMetaResolver + MoveResolver>(
    session: SessionExt<'_, '_, S>,
    change_set: &ChangeSet,
    table_change_set: &TableChangeSet,
    data_cache: &mut TableMetaDataCache<'_, S>,
) -> VMResult<()> {
    let new_tables = &table_change_set.borrow().new_tables;

    for i in table_change_set.removed_tables.iter() {
        data_cache.del_owner(i);
    }

    for (addr, account_change_set) in change_set.borrow().accounts().iter() {
        for (i, op) in account_change_set.resources().iter() {
            let ty_tag = TypeTag::Struct(i.clone());
            let ty_layout = session.get_type_layout(&ty_tag)?;
            let addresses = find_all_address_occur(op, &ty_layout)?;
            let handles = filter_table_handle(new_tables, data_cache, addresses)?;

            for handle in handles {
                data_cache.set_owner(&handle, *addr);
            }
        }
    }

    for (outer_handle, change) in table_change_set.changes.iter() {
        for (_key, op) in &change.entries {
            let addresses = find_all_address_occur(op, &change.value_layout)?;
            let handles = filter_table_handle(new_tables, data_cache, addresses)?;

            for handle in handles {
                data_cache.set_owner(&handle, outer_handle.0);
            }
        }
    }
    Ok(())
}

fn resolve_table_size_change_by_account<S: TableMetaResolver>(
    table_change_set: &TableChangeSet,
    table_size_change: &SizeChangeSet<TableHandle>,
    data_cache: &mut TableMetaDataCache<'_, S>,
) -> VMResult<AccountSizeChangeSet> {
    let mut accounts_delta: AccountSizeChangeSet = AccountSizeChangeSet::default();

    // handle previous size
    for (handle, owner) in data_cache.get_owner_changes().into_iter() {
        let size = match data_cache.get_size(handle)? {
            Some(s) => s,
            None => {
                continue;
            } // skip new table
        };

        match owner {
            Some(new_owner) => {
                let old_owner = data_cache.get_old_root_owner(handle)?.unwrap();
                let new_root_owner = data_cache
                    .get_root_owner(&TableHandle(*new_owner))?
                    .unwrap();

                accounts_delta.move_size(old_owner, new_root_owner, size);
            }
            None => {
                let acc = data_cache.get_old_root_owner(handle)?.unwrap();
                let delta = SizeDelta::decreasing(size);
                accounts_delta.insert_size(acc, delta);
            }
        }
    }

    // handle size changes
    for i in table_change_set.removed_tables.iter() {
        data_cache.del_size(i);
    }

    for (handle, size_delta) in table_size_change.changes() {
        // should check deleted table??
        if table_change_set.removed_tables.contains(handle) {
            continue;
        }
        let owner = data_cache.get_root_owner(handle)?.unwrap();

        accounts_delta.insert_size(owner, size_delta.clone());

        let current_size = data_cache.get_size(&handle)?.unwrap_or(0);

        let mut delta = SizeDelta::increasing(current_size);
        delta.merge(size_delta.clone());
        assert!(!delta.is_decrease, "table size is less than zero");

        let new_size = delta.amount;

        data_cache.set_size(&handle, new_size);
    }

    Ok(accounts_delta)
}

pub fn resolve_table_size_change<S: TableMetaResolver + MoveResolver>(
    session: SessionExt<'_, '_, S>,
    change_set: &ChangeSet,
    table_change_set: &TableChangeSet,
    table_size_change: &SizeChangeSet<TableHandle>,
    remote: &S,
) -> VMResult<(AccountSizeChangeSet, TableMetaChangeSet)> {
    let mut data_cache = TableMetaDataCache::new(remote);

    resolve_table_ownership(session, change_set, table_change_set, &mut data_cache)?;

    let accounts_table_size_changes =
        resolve_table_size_change_by_account(table_change_set, table_size_change, &mut data_cache)?;

    let meta_change_set = data_cache
        .into_change_set()
        .map_err(|e| e.finish(Location::Undefined))?;

    return Ok((accounts_table_size_changes, meta_change_set));
}
