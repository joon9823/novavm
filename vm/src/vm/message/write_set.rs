use std::collections::{btree_map, BTreeMap};
use std::fmt;

use anyhow::bail;
use anyhow::Result;
use move_deps::move_core_types::effects::ChangeSet;
use move_deps::move_core_types::effects::Op;
use move_deps::move_core_types::language_storage::ModuleId;
use move_deps::move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::StructTag,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WriteOp {
    New(#[serde(with = "serde_bytes")] Vec<u8>),
    Modify(#[serde(with = "serde_bytes")] Vec<u8>),
    Delete,
}

impl fmt::Display for WriteOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteOp::New(d) => write!(f, "New({:?})", d),
            WriteOp::Modify(d) => write!(f, "Modify({:?})", d),
            WriteOp::Delete => write!(f, "Delete"),
        }
    }
}

impl WriteOp {
    pub fn from(op: Op<Vec<u8>>) -> Self {
        match op {
            Op::New(data) => Self::New(data),
            Op::Modify(data) => Self::Modify(data),
            Op::Delete => Self::Delete,
        }
    }

    pub fn ok(self) -> Option<Vec<u8>> {
        use WriteOp::*;

        match self {
            New(data) | Modify(data) => Some(data),
            Delete => None,
        }
    }
}

/// A collection of resource and module operations on a Move account.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountWriteSet {
    modules: BTreeMap<Identifier, WriteOp>,
    resources: BTreeMap<StructTag, WriteOp>,
}

impl AccountWriteSet {
    pub fn from_modules_resources(
        modules: BTreeMap<Identifier, WriteOp>,
        resources: BTreeMap<StructTag, WriteOp>,
    ) -> Self {
        Self { modules, resources }
    }

    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            resources: BTreeMap::new(),
        }
    }

    pub fn add_module_op(&mut self, name: Identifier, op: WriteOp) -> Result<()> {
        use btree_map::Entry::*;

        match self.modules.entry(name) {
            Occupied(entry) => bail!("Module {} already exists", entry.key()),
            Vacant(entry) => {
                entry.insert(op);
            }
        }

        Ok(())
    }

    pub fn add_resource_op(&mut self, struct_tag: StructTag, op: WriteOp) -> Result<()> {
        use btree_map::Entry::*;

        match self.resources.entry(struct_tag) {
            Occupied(entry) => bail!("Resource {} already exists", entry.key()),
            Vacant(entry) => {
                entry.insert(op);
            }
        }

        Ok(())
    }

    pub fn into_inner(self) -> (BTreeMap<Identifier, WriteOp>, BTreeMap<StructTag, WriteOp>) {
        (self.modules, self.resources)
    }

    pub fn into_resources(self) -> BTreeMap<StructTag, WriteOp> {
        self.resources
    }

    pub fn into_modules(self) -> BTreeMap<Identifier, WriteOp> {
        self.modules
    }

    pub fn modules(&self) -> &BTreeMap<Identifier, WriteOp> {
        &self.modules
    }

    pub fn resources(&self) -> &BTreeMap<StructTag, WriteOp> {
        &self.resources
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty() && self.resources.is_empty()
    }
}

/// A collection of changes to a Move state. Each AccountWriteSet in the domain of `accounts`
/// is guaranteed to be nonempty
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WriteSet {
    accounts: BTreeMap<AccountAddress, AccountWriteSet>,
}

impl WriteSet {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    pub fn from(change_set: ChangeSet) -> Result<Self> {
        let mut ws = Self::new();

        for (addr, account_changeset) in change_set.into_inner() {
            let mut account_writeset = AccountWriteSet::new();

            let (modules, resources) = account_changeset.into_inner();

            for (id, op) in modules {
                account_writeset.add_module_op(id, WriteOp::from(op))?;
            }

            for (tag, op) in resources {
                account_writeset.add_resource_op(tag, WriteOp::from(op))?;
            }

            ws.add_account_changeset(addr, account_writeset)?;
        }

        Ok(ws)
    }

    pub fn add_account_changeset(
        &mut self,
        addr: AccountAddress,
        account_changeset: AccountWriteSet,
    ) -> Result<()> {
        match self.accounts.entry(addr) {
            btree_map::Entry::Occupied(_) => bail!(
                "Failed to add account change set. Account {} already exists.",
                addr
            ),
            btree_map::Entry::Vacant(entry) => {
                entry.insert(account_changeset);
            }
        }

        Ok(())
    }

    pub fn accounts(&self) -> &BTreeMap<AccountAddress, AccountWriteSet> {
        &self.accounts
    }

    pub fn into_inner(self) -> BTreeMap<AccountAddress, AccountWriteSet> {
        self.accounts
    }

    fn get_or_insert_account_changeset(&mut self, addr: AccountAddress) -> &mut AccountWriteSet {
        match self.accounts.entry(addr) {
            btree_map::Entry::Occupied(entry) => entry.into_mut(),
            btree_map::Entry::Vacant(entry) => entry.insert(AccountWriteSet::new()),
        }
    }

    pub fn add_module_op(&mut self, module_id: ModuleId, op: WriteOp) -> Result<()> {
        let account = self.get_or_insert_account_changeset(*module_id.address());
        account.add_module_op(module_id.name().to_owned(), op)
    }

    pub fn add_resource_op(
        &mut self,
        addr: AccountAddress,
        struct_tag: StructTag,
        op: WriteOp,
    ) -> Result<()> {
        let account = self.get_or_insert_account_changeset(addr);
        account.add_resource_op(struct_tag, op)
    }

    pub fn into_modules(self) -> impl Iterator<Item = (ModuleId, WriteOp)> {
        self.accounts.into_iter().flat_map(|(addr, account)| {
            account
                .modules
                .into_iter()
                .map(move |(module_name, blob_opt)| (ModuleId::new(addr, module_name), blob_opt))
        })
    }
}
