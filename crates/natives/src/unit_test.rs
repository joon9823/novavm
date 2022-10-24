// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_deps::{
    move_binary_format::errors::PartialVMResult,
    move_core_types::{account_address::AccountAddress, gas_algebra::NumArgs},
    move_vm_runtime::native_functions::{NativeContext, NativeFunction},
    move_vm_types::{
        loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
    },
};

use nova_gas::gas_params::unit_test::*;

use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

use crate::block::NativeBlockContext;

/***************************************************************************************************
 * native fun create_signers_for_testing
 *
 *   gas cost: base_cost + unit_cost * num_of_signers
 *
 **************************************************************************************************/
fn to_le_bytes(i: u64) -> [u8; AccountAddress::LENGTH] {
    let bytes = i.to_le_bytes();
    let mut result = [0u8; AccountAddress::LENGTH];
    result[..bytes.len()].clone_from_slice(bytes.as_ref());
    result
}

fn native_create_signers_for_testing(
    gas_params: &CreateSignersForTestingGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let num_signers = pop_arg!(args, u64);
    let signers = Value::vector_for_testing_only(
        (0..num_signers).map(|i| Value::signer(AccountAddress::new(to_le_bytes(i)))),
    );

    let cost = gas_params.base_cost + gas_params.unit_cost * NumArgs::new(num_signers);

    Ok(NativeResult::ok(cost, smallvec![signers]))
}

pub fn make_native_create_signers_for_testing(
    gas_params: CreateSignersForTestingGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_create_signers_for_testing(&gas_params, context, ty_args, args)
        },
    )
}

fn native_set_block_info_for_testing(
    gas_params: &SetBlockInfoForTestingGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let timestamp = pop_arg!(args, u64);
    let height = pop_arg!(args, u64);

    let mut block_context = context.extensions_mut().get_mut::<NativeBlockContext>();
    NativeBlockContext::set_block_info(&mut block_context, height, timestamp);

    let cost = gas_params.base_cost;

    Ok(NativeResult::ok(cost, smallvec![]))
}

pub fn make_native_set_block_info_for_testing(
    gas_params: SetBlockInfoForTestingGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_set_block_info_for_testing(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/
pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "create_signers_for_testing",
            make_native_create_signers_for_testing(gas_params.create_signers_for_testing),
        ),
        (
            "set_block_info_for_testing",
            make_native_set_block_info_for_testing(gas_params.set_block_info_for_testing),
        ),
    ];

    crate::helpers::make_module_natives(natives)
}
