use move_deps::{
    move_binary_format::errors::PartialVMResult,
    move_core_types::{
        account_address::AccountAddress, gas_algebra::InternalGas,
        vm_status::sub_status::NFE_BCS_SERIALIZATION_FAILURE, value::{MoveTypeLayout, MoveStructLayout},
    },
    move_vm_runtime::native_functions::{NativeContext, NativeFunction},
    move_vm_types::{
        loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::{values_impl::Reference, Struct, Value},
    }, move_cli::Move,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::sync::Arc;
/***************************************************************************************************
 * native fun canonicalize_address
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
 #[derive(Debug, Clone)]
 pub struct CanonicalizeAddressGasParameters {
     pub base: InternalGas,
 }
 
 fn native_canonicalize_address(
    gas_params: &CanonicalizeAddressGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
 ) -> PartialVMResult<NativeResult> {

    // TODO: determine gas policy
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    // get string struct arguement
    let arg = pop_arg!(arguments, Struct );

    println!("canonicalize_address arg : {:?}", arg);
    let val = Value::u8(0);
    let serialized_value = match val.simple_serialize(&MoveTypeLayout::U8) {
        Some(serialized_value) => serialized_value,
        None => {
            return Ok(NativeResult::err(gas_params.base, NFE_BCS_SERIALIZATION_FAILURE));
        }
    };
    
    Ok(NativeResult::ok(
        gas_params.base,
        smallvec![Value::vector_u8(serialized_value)],
    ))
 }
 
 pub fn make_native_canonicalize_address(gas_params: CanonicalizeAddressGasParameters) -> NativeFunction {
     Arc::new(move |context, ty_args, args| {
         native_canonicalize_address(&gas_params, context, ty_args, args)
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
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {

    // TODO: determine gas policy
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    // pop type and value
    // let ref_to_val = pop_arg!(arguments, Reference);
    // let arg_type = ty_args.pop().unwrap();

    // // get type layout
    // let layout = match context.type_to_type_layout(&arg_type)? {
    //     Some(layout) => layout,
    //     None => {
    //         return Ok(NativeResult::err(gas_params.base, NFE_BCS_SERIALIZATION_FAILURE));
    //     }
    // };

    // deserialize value
    // let val = ref_to_val.read_ref()?;
    // let deserialized_value = match Value::simple_deserialize(&val, &layout) {
    //     Some(deserialized_value) => deserialized_value,
    //     None => {
    //         return Ok(NativeResult::err(gas_params.base, NFE_BCS_SERIALIZATION_FAILURE));
    //     }
    // };

    let type_name = String::from("test");
    
    Ok(NativeResult::ok(
        gas_params.base,
        smallvec![Value::struct_(Struct::pack(vec![Value::vector_u8(
            type_name.as_bytes().to_vec()
        )]))],
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
    pub canonicalize_address: CanonicalizeAddressGasParameters,
    pub humanize_address: HumanizeAddressGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "canonicalize_address",
            make_native_canonicalize_address(gas_params.canonicalize_address),
        ),   
        (
            "humanize_address",
            make_native_humanize_address(gas_params.humanize_address),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}
