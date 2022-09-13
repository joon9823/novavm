use move_deps::{
    move_binary_format::errors::PartialVMResult,
    move_core_types::{account_address::AccountAddress, gas_algebra::InternalGas},
    move_vm_runtime::native_functions::{NativeContext, NativeFunction},
    move_vm_types::{
        loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
    },
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::sync::Arc;
/***************************************************************************************************
 * native fun canonize_address
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
 #[derive(Debug, Clone)]
 pub struct CanonizeAddressGasParameters {
     pub base: InternalGas,
 }
 
 fn native_canonize_address(
     gas_params: &CanonizeAddressGasParameters,
     _context: &mut NativeContext,
     ty_args: Vec<Type>,
     mut arguments: VecDeque<Value>,
 ) -> PartialVMResult<NativeResult> {

    // TODO: implement this function
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let address = pop_arg!(arguments, AccountAddress);
    Ok(NativeResult::ok(
        gas_params.base,
        smallvec![Value::signer(address)],
    ))
 }
 
 pub fn make_native_canonize_address(gas_params: CanonizeAddressGasParameters) -> NativeFunction {
     Arc::new(move |context, ty_args, args| {
         native_canonize_address(&gas_params, context, ty_args, args)
     })
 }
/***************************************************************************************************
 * native fun humanize_address
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct HumanizeAddressGasParameters {
    pub base: InternalGas,
}

fn native_humanize_address(
    gas_params: &HumanizeAddressGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {

    // TODO: implement this function
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let address = pop_arg!(arguments, AccountAddress);
    Ok(NativeResult::ok(
        gas_params.base,
        smallvec![Value::signer(address)],
    ))
}

pub fn make_native_humanize_address(gas_params: HumanizeAddressGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_humanize_address(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub canonize_address: CanonizeAddressGasParameters,
    pub humanize_address: HumanizeAddressGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "canonize_address",
            make_native_canonize_address(gas_params.canonize_address),
        ),   
        (
            "humanize_address",
            make_native_humanize_address(gas_params.humanize_address),
        ),
    ];

    crate::vm::natives::helpers::make_module_natives(natives)
}
