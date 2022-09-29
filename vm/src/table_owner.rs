use move_deps::{
    move_binary_format::errors::{Location, PartialVMError, VMError, VMResult},
    move_core_types::{
        account_address::AccountAddress,
        effects::{ChangeSet, Event, Op},
        language_storage::TypeTag,
        resolver::MoveResolver,
        vm_status::StatusCode,
    },
    move_table_extension::TableResolver,
    move_vm_types::{
        values::{Struct, VMValueCast, Value, Vector},
        views::{TypeView, ValueView, ValueVisitor},
    },
};

use crate::{session::SessionExt, storage::data_view_resolver::StoredSizeResolver};

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
            println!("visiting result : {:?}", res);

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
