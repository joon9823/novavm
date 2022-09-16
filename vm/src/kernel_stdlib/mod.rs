pub mod bank;
mod helpers;

use move_deps::{
    move_core_types::account_address::AccountAddress,
    move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable},
    move_core_types::gas_algebra::GasQuantity
};

pub enum AbstractValueUnit {}
pub type AbstractValueSize = GasQuantity<AbstractValueUnit>;

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub bank: bank::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            bank: bank::GasParameters {
                balance: bank::BalanceGasParameters { base: 0.into() },
                transfer: bank::TransferGasParameters { base: 0.into() },
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

    add_natives_from_module!("bank", bank::make_all(gas_params.bank));

    make_table_from_iter(framework_addr, natives)
}