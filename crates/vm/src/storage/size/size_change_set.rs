use move_deps::move_core_types::account_address::AccountAddress;

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use super::size_delta::SizeDelta;

#[derive(Debug)]
pub struct SizeChangeSet(BTreeMap<AccountAddress, SizeDelta>);

impl Default for SizeChangeSet {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

impl SizeChangeSet {
    pub fn new(map: BTreeMap<AccountAddress, SizeDelta>) -> SizeChangeSet {
        Self(map)
    }

    pub fn changes(&self) -> &BTreeMap<AccountAddress, SizeDelta> {
        &self.0
    }
    pub fn into_inner(self) -> BTreeMap<AccountAddress, SizeDelta> {
        self.0
    }

    pub fn merge(&mut self, another: SizeChangeSet) {
        for (key, size) in another.into_inner() {
            self.insert_size(key, size);
        }
    }

    pub fn insert_size(&mut self, key: AccountAddress, value: SizeDelta) {
        match self.0.entry(key) {
            Entry::Vacant(e) => {
                if value.amount != 0 {
                    e.insert(value);
                }
            }
            Entry::Occupied(mut e) => {
                e.get_mut().merge(value);
                if e.get().amount == 0 {
                    e.remove_entry();
                }
            }
        };
    }

    pub fn move_size(&mut self, from: AccountAddress, to: AccountAddress, size: usize) {
        self.0.insert(from, SizeDelta::decreasing(size));
        self.0.insert(to, SizeDelta::increasing(size));
    }
}
