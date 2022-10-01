use std::{collections::BTreeMap, fmt, fmt::Display};

use move_deps::{
    move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult},
    move_core_types::{
        account_address::AccountAddress, effects::Op, language_storage::TypeTag,
        resolver::MoveResolver, vm_status::StatusCode,
    },
    move_table_extension::{TableHandle, TableResolver},
    move_vm_types::{
        values::Value,
        views::{ValueView, ValueVisitor},
    },
};

use crate::{
    session::SessionExt,
    storage::data_view_resolver::{StoredSizeResolver, TableOwnerResolver},
};

pub fn find_all_address_occur<S: MoveResolver + TableResolver + StoredSizeResolver>(
    op: &Op<Vec<u8>>,
    session: &SessionExt<'_, '_, S>,
    ty_tag: &TypeTag,
) -> VMResult<Vec<AccountAddress>> {
    let v = match op.as_ref().ok() {
        Some(blob) => {
            let ty_layout = session.get_type_layout(&ty_tag)?;
            let val = Value::simple_deserialize(&blob, &ty_layout).ok_or(
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
        // let mut m = vals.clone().to_vec();
        self.addresses.append(vals.to_vec().as_mut());
    }

    #[inline]
    fn visit_ref(&mut self, _depth: usize, _is_global: bool) -> bool {
        false
    }
}

#[derive(Default)]
pub struct TableOwnerChangeSet {
    pub owner: BTreeMap<TableHandle, Op<Vec<u8>>>,
    pub value_type: BTreeMap<TableHandle, Op<Vec<u8>>>,
}

pub struct TableOwnerDataCache<'r, S> {
    remote: &'r S,
    table_owner: BTreeMap<TableHandle, WriteCache>,
    val_type_map: BTreeMap<TableHandle, WriteCache>,
}

impl<'r, S: TableOwnerResolver> TableOwnerDataCache<'r, S> {
    pub fn new(remote: &'r S) -> Self {
        Self {
            remote,
            table_owner: BTreeMap::default(),
            val_type_map: BTreeMap::default(),
        }
    }

    pub fn into_change_set(self) -> PartialVMResult<TableOwnerChangeSet> {
        let mut owner = BTreeMap::new();
        let mut value_type = BTreeMap::new();
        for (handle, val) in self.table_owner {
            let op = match val.into_effect() {
                Some(op) => op,
                None => continue,
            };

            owner.insert(handle, serialize_op(op)?);
        }

        for (handle, val) in self.val_type_map {
            let op = match val.into_effect() {
                Some(op) => op,
                None => continue,
            };

            value_type.insert(handle, serialize_op(op)?);
        }

        Ok(TableOwnerChangeSet { owner, value_type })
    }

    pub fn set_owner(&mut self, handle: &TableHandle, owner: &AccountAddress) {
        self.table_owner.insert(
            handle.clone(),
            WriteCache::Updated(WriteCacheValue::Owner(owner.clone())),
        );
    }

    pub fn del_owner(&mut self, handle: &TableHandle) {
        self.table_owner.insert(handle.clone(), WriteCache::Deleted);
    }

    pub fn set_table_value_type(&mut self, handle: &TableHandle, ty_tag: &TypeTag) {
        self.val_type_map.insert(
            handle.clone(),
            WriteCache::Updated(WriteCacheValue::TypeTag(ty_tag.clone())),
        );
    }

    pub fn del_table_value_type(&mut self, handle: &TableHandle) {
        self.val_type_map
            .insert(handle.clone(), WriteCache::Deleted);
    }

    pub fn is_registerd_table(&self, handle: &TableHandle) -> VMResult<bool> {
        match self.table_owner.contains_key(handle) {
            true => Ok(true),
            false => {
                let val = self.remote.get_owner(handle)?;

                match val {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
        }
    }

    pub fn get_owner(&self, handle: &TableHandle) -> VMResult<Option<AccountAddress>> {
        match self.table_owner.get(handle) {
            Some(cached) => Ok(cached.get_owner().cloned()),
            None => {
                let val = self.remote.get_owner(handle)?;

                match val {
                    Some(blob) => Ok(deserialize_owner(&blob)),
                    None => Ok(None),
                }
            }
        }
    }

    pub fn get_table_value_type(&self, handle: &TableHandle) -> VMResult<Option<TypeTag>> {
        match self.val_type_map.get(handle) {
            Some(cached) => Ok(cached.get_type().cloned()),
            None => {
                let val = self.remote.get_table_value_type(handle)?;
                match val {
                    Some(blob) => Ok(deserialize_value_type(&blob)),
                    None => Ok(None),
                }
            }
        }
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
    fn get_owner(&self) -> Option<&AccountAddress> {
        match self {
            WriteCache::Updated(val) => match val {
                WriteCacheValue::Owner(owner) => Some(owner),
                WriteCacheValue::TypeTag(_) => panic!(),
            },
            WriteCache::Deleted => None,
        }
    }

    fn get_type(&self) -> Option<&TypeTag> {
        match self {
            WriteCache::Updated(val) => match val {
                WriteCacheValue::Owner(_) => panic!(),
                WriteCacheValue::TypeTag(t) => Some(t),
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
    TypeTag(TypeTag),
}

impl WriteCacheValue {
    fn serialize(self) -> Option<Vec<u8>> {
        match self {
            WriteCacheValue::Owner(owner) => serialize_owner(&owner),
            WriteCacheValue::TypeTag(ty) => serialize_value_type(&ty),
        }
    }
}

impl Display for WriteCacheValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WriteCacheValue::Owner(val) => write!(f, "Owner({})", val),
            WriteCacheValue::TypeTag(val) => write!(f, "TypeTag({})", val),
        }
    }
}

fn serialize_op(op: Op<WriteCacheValue>) -> PartialVMResult<Op<Vec<u8>>> {
    fn serialize_or_error(v: WriteCacheValue) -> PartialVMResult<Vec<u8>> {
        match v.serialize() {
            Some(d) => Ok(d),
            None => Err(PartialVMError::new(
                StatusCode::FAILED_TO_SERIALIZE_WRITE_SET_CHANGES,
            )),
        }
    }

    match op {
        Op::New(v) => Ok(Op::New(serialize_or_error(v)?)),
        Op::Modify(v) => Ok(Op::Modify(serialize_or_error(v)?)),
        Op::Delete => Ok(Op::Delete),
    }
}

fn serialize_owner(owner: &AccountAddress) -> Option<Vec<u8>> {
    bcs::to_bytes(owner).ok()
}
fn serialize_value_type(ty_tag: &TypeTag) -> Option<Vec<u8>> {
    bcs::to_bytes(ty_tag).ok()
}

fn deserialize_owner(blob: &[u8]) -> Option<AccountAddress> {
    bcs::from_bytes(blob).ok()
}

fn deserialize_value_type(blob: &[u8]) -> Option<TypeTag> {
    bcs::from_bytes(blob).ok()
}
