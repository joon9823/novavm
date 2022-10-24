// Copyright (c) Kernel-Labs
// SPDX-License-Identifier: Apache-2.0

mod helpers;

pub mod account;
pub mod block;
pub mod code;
pub mod event;
pub mod table;
pub mod type_info;
pub mod util;

#[cfg(feature = "testing")]
pub mod unit_test;

use move_deps::{
    move_core_types::account_address::AccountAddress,
    move_core_types::language_storage::CORE_CODE_ADDRESS,
    move_stdlib::natives::nursery_natives,
    move_stdlib::natives::{self as move_natives},
    move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable},
    move_vm_types::values::Value,
};
use nova_gas::AbstractValueSize;
use nova_gas::{
    nova::GasParameters as NovaGasParameters, table::GasParameters as TableGasParameters,
    AbstractValueSizeGasParameters,
};
use table as table_natives;

pub mod status {
    // Failure in parsing a struct type tag
    pub const NFE_EXPECTED_STRUCT_TYPE_TAG: u64 = 0x1;
    // Failure in address parsing (likely no correct length)
    pub const NFE_UNABLE_TO_PARSE_ADDRESS: u64 = 0x2;
}

pub fn nova_natives(
    nova_std_addr: AccountAddress,
    gas_params: NovaGasParameters,
    calc_abstract_val_size: impl Fn(&Value) -> AbstractValueSize + Send + Sync + 'static,
) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives_from_module {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    add_natives_from_module!("account", account::make_all(gas_params.account));
    add_natives_from_module!("block", block::make_all(gas_params.block));
    add_natives_from_module!("type_info", type_info::make_all(gas_params.type_info));
    add_natives_from_module!("util", util::make_all(gas_params.util));
    add_natives_from_module!("code", code::make_all(gas_params.code));
    add_natives_from_module!(
        "event",
        event::make_all(gas_params.event, calc_abstract_val_size)
    );

    #[cfg(feature = "testing")]
    add_natives_from_module!("unit_test", unit_test::make_all(gas_params.unit_test));

    make_table_from_iter(nova_std_addr, natives)
}

pub fn all_natives(
    move_natives_gas_params: move_natives::GasParameters,
    nova_natives_gas_params: NovaGasParameters,
    table_natives_gas_params: TableGasParameters,
    abs_val_size_gas_params: AbstractValueSizeGasParameters,
) -> NativeFunctionTable {
    move_natives::all_natives(CORE_CODE_ADDRESS, move_natives_gas_params)
        .into_iter()
        .filter(|(_, name, _, _)| name.as_str() != "unit_test")
        .chain(
            nursery_natives(
                CORE_CODE_ADDRESS,
                // TODO - change this as arguments
                move_natives::NurseryGasParameters::zeros(),
            )
            .into_iter()
            .filter(|(addr, module_name, _, _)| {
                !(*addr == CORE_CODE_ADDRESS && module_name.as_str() == "event")
            }),
        )
        .chain(nova_natives(
            CORE_CODE_ADDRESS,
            nova_natives_gas_params,
            move |val| abs_val_size_gas_params.abstract_value_size(val),
        ))
        .chain(table_natives::all_natives(
            CORE_CODE_ADDRESS,
            table_natives_gas_params,
        ))
        .collect()
}
