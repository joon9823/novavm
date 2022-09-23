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
                bls12381_validate_pubkey: signature::Bls12381ValidatePubkeyGasParameters {
                    base_cost: 0.into(),
                },
                ed25519_validate_pubkey: signature::Ed25519ValidatePubkeyGasParameters {
                    base_cost: 0.into(),
                },
                ed25519_verify: signature::Ed25519VerifyGasParameters {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
                },
                secp256k1_ecdsa_recover: signature::Secp256k1ECDSARecoverGasParameters {
                    base_cost: 0.into(),
                },
                bls12381_verify_signature: signature::Bls12381VerifySignatureGasParams {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
                },
                bls12381_aggregate_pop_verified_pubkeys:
                    signature::Bls12381AggregatePopVerifiedPubkeysGasParameters {
                        base_cost: 0.into(),
                        per_pubkey_cost: 0.into(),
                    },
                bls12381_verify_proof_of_possession:
                    signature::Bls12381VerifyProofOfPosessionGasParameters {
                        base_cost: 0.into(),
                    },
            },
            type_info: type_info::GasParameters {
                type_of: type_info::TypeOfGasParameters {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
                },
                type_name: type_info::TypeNameGasParameters {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
                },
            },
            util: util::GasParameters {
                from_bytes: util::FromBytesGasParameters {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
                },
            },
            code: code::GasParameters {
                request_publish: code::RequestPublishGasParameters {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
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
