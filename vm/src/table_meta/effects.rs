use std::{fmt, fmt::Display};

use move_deps::{
    move_binary_format::errors::{PartialVMError, PartialVMResult},
    move_core_types::{account_address::AccountAddress, effects::Op, vm_status::StatusCode},
};

#[derive(Debug)]
pub(crate) enum WriteCache {
    /// A resource resides in this slot and will be write into storage.
    Updated(WriteCacheValue),
    /// A resource used to exist in storage but has been deleted by the current transaction.
    Deleted,
}

impl WriteCache {
    pub(crate) fn get_size(&self) -> Option<&usize> {
        match self {
            WriteCache::Updated(val) => match val {
                WriteCacheValue::Owner(_) => panic!("not found"),
                WriteCacheValue::Size(size) => Some(size),
            },
            WriteCache::Deleted => None,
        }
    }

    pub(crate) fn get_owner(&self) -> Option<&AccountAddress> {
        match self {
            WriteCache::Updated(val) => match val {
                WriteCacheValue::Owner(owner) => Some(owner),
                WriteCacheValue::Size(_) => panic!("not"),
            },
            WriteCache::Deleted => None,
        }
    }

    pub(crate) fn into_effect(self) -> Option<Op<WriteCacheValue>> {
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
pub(crate) enum WriteCacheValue {
    Owner(AccountAddress),
    Size(usize),
}

impl WriteCacheValue {
    pub(crate) fn serialize(self) -> Option<Vec<u8>> {
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

pub(crate) fn serialize_op(op: Op<WriteCacheValue>) -> PartialVMResult<Op<Vec<u8>>> {
    match op {
        Op::New(v) => Ok(Op::New(serialize_or_error(v)?)),
        Op::Modify(v) => Ok(Op::Modify(serialize_or_error(v)?)),
        Op::Delete => Ok(Op::Delete),
    }
}

pub(crate) fn serialize_or_error(v: WriteCacheValue) -> PartialVMResult<Vec<u8>> {
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

pub(crate) fn deserialize_owner(blob: &[u8]) -> Option<AccountAddress> {
    bcs::from_bytes(blob).ok()
}

fn serialize_size(size: &usize) -> Option<Vec<u8>> {
    bcs::to_bytes(size).ok()
}

pub(crate) fn deserialize_size(blob: &[u8]) -> Option<usize> {
    bcs::from_bytes(blob).ok()
}
