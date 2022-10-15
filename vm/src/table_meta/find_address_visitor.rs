use move_deps::{
    move_core_types::account_address::AccountAddress, move_vm_types::views::ValueVisitor,
};

pub(crate) struct FindingAddressVisitor {
    addresses: Vec<AccountAddress>,
}

impl<'a> FindingAddressVisitor {
    pub(crate) fn new() -> Self {
        Self {
            addresses: Vec::default(),
        }
    }

    pub(crate) fn finish(self) -> Vec<AccountAddress> {
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
