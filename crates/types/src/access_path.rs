// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Suppose we have the following data structure in a smart contract:
//!
//! struct B {
//!   Map<String, String> mymap;
//! }
//!
//! struct A {
//!   B b;
//!   int my_int;
//! }
//!
//! struct C {
//!   List<int> mylist;
//! }
//!
//! A a;
//! C c;
//!
//! and the data belongs to Alice. Then an access to `a.b.mymap` would be translated to an access
//! to an entry in key-value store whose key is `<Alice>/a/b/mymap`. In the same way, the access to
//! `c.mylist` would need to query `<Alice>/c/mylist`.
//!
//! So an account stores its data in a directory structure, for example:
//!   <Alice>/balance:   10
//!   <Alice>/a/b/mymap: {"Bob" => "abcd", "Carol" => "efgh"}
//!   <Alice>/a/myint:   20
//!   <Alice>/c/mylist:  [3, 5, 7, 9]
//!
//! If someone needs to query the map above and find out what value associated with "Bob" is,
//! `address` will be set to Alice and `path` will be set to "/a/b/mymap/Bob".
//!
//! On the other hand, if you want to query only <Alice>/a/*, `address` will be set to Alice and
//! `path` will be set to "/a" and use the `get_prefix()` method from statedb

use anyhow::{anyhow, bail, Result};
use move_deps::move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, ResourceKey, StructTag, TypeTag},
    parser::parse_type_tag,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

use std::fmt;
use std::str::FromStr;
use std::{fmt::Write, num::ParseIntError};

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: DataPath,
}

impl AccessPath {
    pub fn new(address: AccountAddress, path: DataPath) -> Self {
        AccessPath { address, path }
    }

    pub fn resource_access_path(address: AccountAddress, struct_tag: StructTag) -> Self {
        Self::new(address, Self::resource_data_path(struct_tag))
    }

    pub fn code_access_path(address: AccountAddress, module_name: Identifier) -> AccessPath {
        AccessPath::new(address, Self::code_data_path(module_name))
    }

    pub fn table_item_access_path(
        address /* Table Address */: AccountAddress,
        key: Vec<u8>,
    ) -> AccessPath {
        // table address created uniquely in move table extension
        AccessPath::new(address, Self::table_item_data_path(key))
    }

    pub fn table_meta_access_path(address /* Table Address */: AccountAddress) -> AccessPath {
        // table address created uniquely in move table extension
        AccessPath::new(address, Self::table_meta_data_path())
    }

    pub fn resource_data_path(tag: StructTag) -> DataPath {
        DataPath::Resource(tag)
    }

    pub fn code_data_path(module_name: ModuleName) -> DataPath {
        DataPath::Code(module_name)
    }

    pub fn table_item_data_path(key: Vec<u8>) -> DataPath {
        DataPath::TableItem(key)
    }

    pub fn table_meta_data_path() -> DataPath {
        DataPath::TableMeta
    }

    pub fn into_inner(self) -> (AccountAddress, DataPath) {
        let address = self.address;
        let path = self.path;
        (address, path)
    }

    pub fn as_module_id(&self) -> Option<ModuleId> {
        match &self.path {
            DataPath::Code(module_name) => Some(ModuleId::new(self.address, module_name.clone())),
            _ => None,
        }
    }

    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut out = self.address.to_vec();
        let path_bytes = self.path.encode()?;

        out.extend(path_bytes);
        Ok(out)
    }
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.path)
    }
}

impl From<&ModuleId> for AccessPath {
    fn from(id: &ModuleId) -> AccessPath {
        AccessPath::code_access_path(*id.address(), id.name().to_owned())
    }
}

impl From<&ResourceKey> for AccessPath {
    fn from(key: &ResourceKey) -> AccessPath {
        AccessPath::resource_access_path(key.address(), key.type_().clone())
    }
}

#[derive(
    IntoPrimitive, TryFromPrimitive, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Debug,
)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum DataType {
    CODE,
    RESOURCE,
    TABLE_ITEM,
    TABLE_META,
}

impl DataType {
    pub const LENGTH: usize = 2;

    pub fn is_code(self) -> bool {
        matches!(self, DataType::CODE)
    }
    pub fn is_resource(self) -> bool {
        matches!(self, DataType::RESOURCE)
    }

    #[inline]
    pub fn type_index(self) -> u8 {
        self.into()
    }

    /// Every DataType has a storage root in AccountState
    #[inline]
    pub fn storage_index(self) -> usize {
        self.type_index() as usize
    }

    pub fn from_index(idx: u8) -> Result<Self> {
        Ok(Self::try_from_primitive(idx)?)
    }
}

pub type ModuleName = Identifier;

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub enum DataPath {
    Code(ModuleName),
    Resource(StructTag),
    TableItem(Vec<u8>),
    TableMeta,
}

impl DataPath {
    pub fn is_code(&self) -> bool {
        matches!(self, DataPath::Code(_))
    }

    pub fn is_resource(&self) -> bool {
        matches!(self, DataPath::Resource(_))
    }

    pub fn is_table_item(&self) -> bool {
        matches!(self, DataPath::TableItem(_))
    }

    pub fn as_struct_tag(&self) -> Option<&StructTag> {
        match self {
            DataPath::Resource(struct_tag) => Some(struct_tag),
            _ => None,
        }
    }

    pub fn data_type(&self) -> DataType {
        match self {
            DataPath::Code(_) => DataType::CODE,
            DataPath::Resource(_) => DataType::RESOURCE,
            DataPath::TableItem(_) => DataType::TABLE_ITEM,
            DataPath::TableMeta => DataType::TABLE_META,
        }
    }

    /// Serializes to bytes for physical storage.
    pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
        let mut out = vec![];

        let prefix = self.data_type().storage_index();
        let raw_key = match self {
            DataPath::Code(module_name) => bcs::to_bytes(module_name)?,
            DataPath::Resource(struct_tag) => bcs::to_bytes(struct_tag)?,
            DataPath::TableItem(key) => key.to_vec(),
            DataPath::TableMeta => {
                vec![]
            }
        };

        out.push(prefix as u8);
        out.extend(raw_key);
        Ok(out)
    }

    /// Recovers from serialized bytes in physical storage.
    pub fn decode(val: &[u8]) -> anyhow::Result<Self> {
        if val.is_empty() {
            return Err(anyhow!("empty input bytes"));
        }

        let data_type = val[0];
        let data_type = DataType::from_index(data_type).map_err(|e| anyhow!(e))?;
        match data_type {
            DataType::CODE => Ok(DataPath::Code(bcs::from_bytes(&val[1..])?)),
            DataType::RESOURCE => Ok(DataPath::Resource(bcs::from_bytes(&val[1..])?)),
            DataType::TABLE_ITEM => Ok(DataPath::TableItem(val[1..].to_vec())),
            DataType::TABLE_META => Ok(DataPath::TableMeta),
        }
    }
}

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let storage_index = self.data_type().storage_index();
        match self {
            DataPath::Resource(struct_tag) => {
                write!(f, "{}/{}", storage_index, struct_tag)
            }
            DataPath::Code(module_name) => {
                write!(f, "{}/{}", storage_index, module_name)
            }
            DataPath::TableItem(key) => {
                write!(f, "{}/{}", storage_index, encode_hex(key))
            }
            DataPath::TableMeta => {
                // put zero for consistency
                write!(f, "{}/0", storage_index)
            }
        }
    }
}

impl FromStr for AccessPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<_>>();
        if parts.len() != 3 {
            bail!("Invalid access_path string: {}", s);
        }

        let address = AccountAddress::from_str(parts[0])?;
        let data_type = DataType::from_index(parts[1].parse()?)?;

        let data_path = match data_type {
            DataType::CODE => AccessPath::code_data_path(Identifier::new(parts[2])?),
            DataType::RESOURCE => AccessPath::resource_data_path(parse_struct_tag(parts[2])?),
            DataType::TABLE_ITEM => AccessPath::table_item_data_path(decode_hex(parts[2])?),
            DataType::TABLE_META => AccessPath::table_meta_data_path(),
        };

        Ok(AccessPath::new(address, data_path))
    }
}

fn parse_struct_tag(s: &str) -> Result<StructTag> {
    let type_tag = parse_type_tag(s)?;
    match type_tag {
        TypeTag::Struct(st) => Ok(st),
        t => bail!("expect a struct tag, found: {:?}", t),
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}
