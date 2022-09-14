use move_deps::move_core_types::account_address::AccountAddress;

use crate::{UnmanagedVector, vm, Db, ByteSliceView, api::GoApi, querier::GoQuerier, gas_meter::gas_meter_t, CosmosView};

// VM initializer
#[no_mangle]
pub extern "C" fn initialize(db: Db, api: GoApi, querier: GoQuerier, is_verbose: bool, errmsg: Option<&mut UnmanagedVector>, module_bundle: ByteSliceView) -> UnmanagedVector{
    let mb = module_bundle.read().unwrap().to_vec();

    vm::initialize_vm(mb, db);

    let retval = UnmanagedVector::new(None);
    retval
}

/// exported function to publish a module
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn publish_module(db: Db, api: GoApi, querier: GoQuerier, is_verbose: bool, gas_limit: u64, gas_used: Option<&mut u64>, errmsg: Option<&mut UnmanagedVector>, sender: ByteSliceView, module_bundle: ByteSliceView) -> UnmanagedVector{
    let payload = module_bundle.read().unwrap().to_vec();

    let s= sender.read().unwrap().to_vec();
    let addr = AccountAddress::from_bytes(s).unwrap();

    vm::publish_module(addr, payload, db, gas_limit);

    let retval = UnmanagedVector::new(None);
    retval
}

// exported function to execute (an entrypoint of) contract
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn execute_contract(db: Db, api: GoApi, querier: GoQuerier, is_verbose: bool, gas_limit: u64, gas_used: Option<&mut u64>, errmsg: Option<&mut UnmanagedVector>, sender: ByteSliceView, message: ByteSliceView) -> UnmanagedVector {
    let payload = message.read().unwrap().to_vec();

    let s= sender.read().unwrap().to_vec();
    let addr = AccountAddress::from_bytes(s).unwrap();

    vm::execute_contract(addr, payload, db, gas_limit);

    let retval = UnmanagedVector::new(None);

    retval
}