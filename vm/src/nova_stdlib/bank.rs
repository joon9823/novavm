use move_deps::{
    move_binary_format::errors::PartialVMResult,
    move_core_types::gas_algebra::InternalGas,
    move_vm_runtime::native_functions::{NativeContext, NativeFunction},
    move_vm_types::{
        loaded_data::runtime_types::Type,
        natives::function::NativeResult,
        values::Value,
    },
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::sync::Arc;

/***************************************************************************************************
 * native fun balance
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
 #[derive(Debug, Clone)]
 pub struct BalanceGasParameters {
     pub base: InternalGas,
 }
 
 fn native_balance(
    gas_params: &BalanceGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
 ) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let balance = 100_000;
    Ok(NativeResult::ok( 
        gas_params.base,
        smallvec![Value::u64(balance)] 
    ))
 }
 
 pub fn make_native_balance(gas_params: BalanceGasParameters) -> NativeFunction {
     Arc::new(move |context, ty_args, args| {
         native_balance(&gas_params, context, ty_args, args)
     })
 }
/***************************************************************************************************
 * native fun transfer
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct TransferGasParameters {
    pub base: InternalGas,
}

fn native_transfer(
    gas_params: &TransferGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {

    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 3);
    
    Ok(NativeResult::ok(
        gas_params.base,
        smallvec![],
    ))
    
}

pub fn make_native_transfer(gas_params: TransferGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_transfer(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub transfer: TransferGasParameters,
    pub balance: BalanceGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "transfer",
            make_native_transfer(gas_params.transfer),
        ),   
        (
            "balance",
            make_native_balance(gas_params.balance),
        ),
    ];

    crate::nova_stdlib::helpers::make_module_natives(natives)
}
