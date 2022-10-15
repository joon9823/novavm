// Copyright (c) Kernel-Labs
// SPDX-License-Identifier: Apache-2.0

mod helpers;

pub mod account;
pub mod block;
pub mod code;
pub mod table;
pub mod type_info;
pub mod util;

use move_deps::{
    move_core_types::language_storage::CORE_CODE_ADDRESS,
    move_core_types::{account_address::AccountAddress, identifier::Identifier},
    move_stdlib::natives::nursery_natives,
    move_stdlib::natives::{self as move_natives},
    move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable},
};
use table as table_natives;

pub mod status {
    // Failure in parsing a struct type tag
    pub const NFE_EXPECTED_STRUCT_TYPE_TAG: u64 = 0x1;
    // Failure in address parsing (likely no correct length)
    pub const NFE_UNABLE_TO_PARSE_ADDRESS: u64 = 0x2;
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub account: account::GasParameters,
    pub block: block::GasParameters,
    pub type_info: type_info::GasParameters,
    pub util: util::GasParameters,
    pub code: code::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            account: account::GasParameters {
                create_address: account::CreateAddressGasParameters {
                    base_cost: 0.into(),
                },
                create_signer: account::CreateSignerGasParameters {
                    base_cost: 0.into(),
                },
            },
            block: block::GasParameters {
                get_block_info: block::GetBlockInfoGasParameters {
                    base_cost: 0.into(),
                },
            },
            type_info: type_info::GasParameters {
                type_of: type_info::TypeOfGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
                type_name: type_info::TypeNameGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
            },
            util: util::GasParameters {
                from_bytes: util::FromBytesGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
            },
            code: code::GasParameters {
                request_publish: code::RequestPublishGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
            },
        }
    }
}

pub fn nova_natives(
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

    add_natives_from_module!("account", account::make_all(gas_params.account));
    add_natives_from_module!("block", block::make_all(gas_params.block));
    add_natives_from_module!("type_info", type_info::make_all(gas_params.type_info));
    add_natives_from_module!("util", util::make_all(gas_params.util.clone()));
    add_natives_from_module!("code", code::make_all(gas_params.code));

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

pub fn all_natives(
    move_natives_gas_params: move_natives::GasParameters,
    nova_natives_gas_params: GasParameters,
    table_natives_gas_params: table_natives::GasParameters,
) -> NativeFunctionTable {
    move_natives::all_natives(CORE_CODE_ADDRESS, move_natives_gas_params)
        .into_iter()
        .chain(nursery_natives(
            CORE_CODE_ADDRESS,
            // TODO - change this as arguments
            move_natives::NurseryGasParameters::zeros(),
        ))
        .chain(nova_natives(CORE_CODE_ADDRESS, nova_natives_gas_params))
        .chain(table_natives::all_natives(
            CORE_CODE_ADDRESS,
            table_natives_gas_params,
        ))
        .collect()
}
