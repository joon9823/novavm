// Copyright (c) Kernel-Labs
// SPDX-License-Identifier: Apache-2.0

mod helpers;

pub mod code;
pub mod signature;
pub mod type_info;
pub mod util;

use move_deps::{
    move_core_types::language_storage::CORE_CODE_ADDRESS,
    move_core_types::{account_address::AccountAddress, identifier::Identifier},
    move_stdlib::natives::{
        all_natives as move_natives, nursery_natives as move_nursery_natives, NurseryGasParameters,
    },
    move_table_extension::{table_natives, GasParameters as TableGasParameter},
    move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable},
};

use crate::gas::NativeGasParameters;

pub mod status {
    // Failure in parsing a struct type tag
    pub const NFE_EXPECTED_STRUCT_TYPE_TAG: u64 = 0x1;
    // Failure in address parsing (likely no correct length)
    pub const NFE_UNABLE_TO_PARSE_ADDRESS: u64 = 0x2;
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub signature: signature::GasParameters,
    pub type_info: type_info::GasParameters,
    pub util: util::GasParameters,
    pub code: code::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            signature: signature::GasParameters {
                bls12381: signature::Bls12381GasParameters {
                    base: 0.into(),
                    per_pairing: 0.into(),
                    per_msg: 0.into(),
                    per_byte: 0.into(),
                    per_pubkey_deserialize: 0.into(),
                    per_pubkey_aggregate: 0.into(),
                    per_pubkey_subgroup_check: 0.into(),
                    per_sig_verify: 0.into(),
                    per_sig_deserialize: 0.into(),
                    per_sig_aggregate: 0.into(),
                    per_sig_subgroup_check: 0.into(),
                    per_proof_of_possession_verify: 0.into(),
                },
                ed25519: signature::Ed25519GasParameters {
                    base: 0.into(),
                    per_pubkey_deserialize: 0.into(),
                    per_pubkey_small_order_check: 0.into(),
                    per_sig_deserialize: 0.into(),
                    per_sig_strict_verify: 0.into(),
                    per_msg_hashing_base: 0.into(),
                    per_msg_byte_hashing: 0.into(),
                },
                secp256k1: signature::Secp256k1GasParameters {
                    base: 0.into(),
                    ecdsa_recover: 0.into(),
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

    add_natives_from_module!("signature", signature::make_all(gas_params.signature));
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

pub fn nova_natives(gas_params: NativeGasParameters) -> NativeFunctionTable {
    move_natives(CORE_CODE_ADDRESS, gas_params.move_stdlib)
        .into_iter()
        .chain(move_nursery_natives(
            CORE_CODE_ADDRESS,
            NurseryGasParameters::zeros(),
        ))
        .chain(all_natives(CORE_CODE_ADDRESS, gas_params.nova_stdlib))
        .chain(table_natives(CORE_CODE_ADDRESS, TableGasParameter::zeros()))
        .collect()
}
