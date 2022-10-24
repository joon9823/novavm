use move_deps::move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SizeDelta {
    /// The account address of the storage size delta
    address: AccountAddress,
    /// The number of delta bytes size
    amount: u64,
    /// The sign flag
    is_decreasing: bool,
}

#[allow(dead_code)]
impl SizeDelta {
    pub fn new(address: AccountAddress, amount: u64, is_decreasing: bool) -> Self {
        Self {
            address,
            amount,
            is_decreasing,
        }
    }
}

impl std::fmt::Debug for SizeDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SizeDelta {{ address: {:?}, amount: {:?}, is_decreasing: {:?} }}",
            self.address, self.amount, self.is_decreasing,
        )
    }
}

impl std::fmt::Display for SizeDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
