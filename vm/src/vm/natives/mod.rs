pub mod address;
mod helpers;

use move_deps::{
    move_core_types::{account_address::AccountAddress, identifier::Identifier},
    move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable},
    move_vm_types::values::Value,
    move_core_types::gas_algebra::GasQuantity
};

pub enum AbstractValueUnit {}
pub type AbstractValueSize = GasQuantity<AbstractValueUnit>;

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub address: address::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            address: address::GasParameters {
                canonicalize_address: address::CanonicalizeAddressGasParameters { base: 0.into() },
                humanize_address: address::HumanizeAddressGasParameters { base: 0.into() },
            },
        }
    }
}


pub fn all_natives(
    framework_addr: AccountAddress,
    gas_params: GasParameters,
) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives_from_module {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    add_natives_from_module!("address", address::make_all(gas_params.address));

    make_table_from_iter(framework_addr, natives)
}

/// A temporary hack to patch Table -> table module name as long as it is not upgraded
/// in the Move repo.
pub fn patch_table_module(table: NativeFunctionTable) -> NativeFunctionTable {
    table
        .into_iter()
        .map(|(m, _, f, i)| (m, Identifier::new("table").unwrap(), f, i))
        .collect()
}
