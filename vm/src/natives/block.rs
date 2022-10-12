use better_any::{Tid, TidAble};
use move_deps::{
    move_binary_format::errors::{PartialVMError, PartialVMResult},
    move_core_types::{gas_algebra::InternalGas, vm_status::StatusCode},
    move_vm_runtime::native_functions::{NativeContext, NativeFunction},
    move_vm_types::{
        loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
    },
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::api::ChainApi;

/***************************************************************************************************
 * native fun create_address
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GetBlockInfoGasParameters {
    pub base_cost: InternalGas,
}

/// The native code context.
#[derive(Tid)]
pub struct NativeBlockContext<'a> {
    pub api: &'a dyn ChainApi,
}

impl<'a> NativeBlockContext<'a> {
    pub fn new(api: &'a dyn ChainApi) -> Self {
        Self { api }
    }
}

fn native_get_block_info(
    gas_params: &GetBlockInfoGasParameters,
    context: &NativeContext,
    _ty_args: Vec<Type>,
    _args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let cost = gas_params.base_cost;

    let block_context = context.extensions().get::<NativeBlockContext>();
    let (height, timestamp) = block_context
        .api
        .get_block_info()
        .map_err(|_| PartialVMError::new(StatusCode::LOOKUP_FAILED))?;

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::u64(height), Value::u64(timestamp)],
    ))
}

pub fn make_native_get_block_info(gas_params: GetBlockInfoGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_get_block_info(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
 #[derive(Debug, Clone)]
 pub struct GasParameters {
     pub get_block_info: GetBlockInfoGasParameters,
 }

 
pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "get_block_info_internal",
            make_native_get_block_info(gas_params.get_block_info),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}
