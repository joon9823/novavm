use crate::natives::table::TableHandle;
use move_deps::move_core_types::account_address::AccountAddress;

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;

pub type AccountSizeChangeSet = SizeChangeSet<AccountAddress>;
pub type TablsSizeChangeSet = SizeChangeSet<TableHandle>;

#[derive(Debug)]
pub struct SizeChangeSet<T>(BTreeMap<T, SizeDelta>);

impl<T: Clone + Ord> Default for SizeChangeSet<T> {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

impl<T: Clone + Ord> SizeChangeSet<T> {
    pub fn new(map: BTreeMap<T, SizeDelta>) -> SizeChangeSet<T> {
        Self(map)
    }

    pub fn changes(&self) -> &BTreeMap<T, SizeDelta> {
        &self.0
    }
    pub fn into_inner(self) -> BTreeMap<T, SizeDelta> {
        self.0
    }

    pub fn merge(&mut self, another: SizeChangeSet<T>) {
        for (key, size) in another.into_inner() {
            self.insert_size(key, size);
        }
    }

    pub fn insert_size(&mut self, key: T, value: SizeDelta) {
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

    pub fn move_size(&mut self, from: T, to: T, size: usize) {
        self.0.insert(from, SizeDelta::decreasing(size));
        self.0.insert(to, SizeDelta::increasing(size));
    }
}

#[derive(Debug, Clone)]
pub struct SizeDelta {
    pub amount: usize,
    pub is_decrease: bool,
}

impl SizeDelta {
    pub fn zero() -> Self {
        Self {
            amount: 0,
            is_decrease: false,
        }
    }

    pub fn new(old: usize, new: usize) -> Self {
        Self {
            amount: new.abs_diff(old),
            is_decrease: new < old,
        }
    }

    pub fn increasing(amount: usize) -> Self {
        Self {
            amount,
            is_decrease: false,
        }
    }

    pub fn decreasing(amount: usize) -> Self {
        Self {
            amount,
            is_decrease: true,
        }
    }

    // TODO: it panics if overflow. How do we handle?
    pub fn merge(&mut self, delta: SizeDelta) {
        if self.is_decrease == delta.is_decrease {
            self.amount += delta.amount;
        } else {
            if self.amount < delta.amount {
                self.is_decrease = !self.is_decrease;
            }
            self.amount = self.amount.abs_diff(delta.amount);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::size_change_set::SizeDelta;

    #[test]
    fn test_size_delta() {
        let mut z = SizeDelta::zero();
        z.merge(SizeDelta::new(10, 0));
        assert_eq!(z.amount, 10);
        assert_eq!(z.is_decrease, true);

        //a is 1
        let mut a = SizeDelta::new(2, 3);
        assert_eq!(a.amount, 1);
        assert_eq!(a.is_decrease, false);

        // b is -5
        let b = SizeDelta::new(10, 5);
        assert_eq!(b.amount, 5);
        assert_eq!(b.is_decrease, true);

        // now a is 3
        a.merge(SizeDelta {
            amount: 2,
            is_decrease: false,
        });
        assert_eq!(a.amount, 3);
        assert_eq!(a.is_decrease, false);

        // now a is -2
        a.merge(b);
        assert_eq!(a.amount, 2);
        assert_eq!(a.is_decrease, true);

        // now a is -7
        a.merge(SizeDelta {
            amount: 5,
            is_decrease: true,
        });
        assert_eq!(a.amount, 7);
        assert_eq!(a.is_decrease, true);
    }
}

impl fmt::Display for SizeDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            if self.is_decrease { "-" } else { "+" },
            self.amount
        )
    }
}
