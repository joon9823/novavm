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

use anyhow::Result;

/// Callbacks to system functions defined outside of the move modules.
/// This is a trait to allow Mocks in the test code.
pub trait BlockInfoResolver {
    fn get_block_info(&self) -> Result<(u64 /* height */, u64 /* timestamp */)>;
}

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
    pub api: &'a dyn BlockInfoResolver,

    #[cfg(feature = "testing")]
    height: u64,
    #[cfg(feature = "testing")]
    timestamp: u64,
}

impl<'a> NativeBlockContext<'a> {
    pub fn new(api: &'a dyn BlockInfoResolver) -> Self {
        Self {
            api,

            #[cfg(feature = "testing")]
            height: 0,
            #[cfg(feature = "testing")]
            timestamp: 0,
        }
    }

    #[cfg(feature = "testing")]
    pub fn set_block_info(&mut self, height: u64, timestamp: u64) {
        self.height = height;
        self.timestamp = timestamp;
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

    #[cfg(feature = "testing")]
    if block_context.height != 0 || block_context.timestamp != 0 {
        return Ok(NativeResult::ok(
            cost,
            smallvec![
                Value::u64(block_context.height),
                Value::u64(block_context.timestamp)
            ],
        ));
    }

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
    let natives = [(
        "get_block_info_internal",
        make_native_get_block_info(gas_params.get_block_info),
    )];

    crate::helpers::make_module_natives(natives)
}
