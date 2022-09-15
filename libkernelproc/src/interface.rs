use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::error::{handle_c_error_binary, Error};
use crate::{api::GoApi, querier::GoQuerier, vm, ByteSliceView, Db, UnmanagedVector};

use move_deps::move_core_types::account_address::AccountAddress;

// VM initializer
#[no_mangle]
pub extern "C" fn initialize(
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    is_verbose: bool,
    errmsg: Option<&mut UnmanagedVector>,
    module_bundle: ByteSliceView,
) -> UnmanagedVector {
    let mb = module_bundle.to_owned().unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || vm::initialize_vm(mb, db)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    return UnmanagedVector::new(Some(ret));
}

/// exported function to publish a module
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn publish_module(
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    is_verbose: bool,
    gas_limit: u64,
    gas_used: Option<&mut u64>,
    errmsg: Option<&mut UnmanagedVector>,
    sender: ByteSliceView,
    module: ByteSliceView,
) -> UnmanagedVector {
    let m = module.to_owned().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::publish_module(addr, m, db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    return UnmanagedVector::new(Some(ret));
}

// exported function to execute (an entrypoint of) contract
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn execute_contract(
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    is_verbose: bool,
    gas_limit: u64,
    gas_used: Option<&mut u64>,
    errmsg: Option<&mut UnmanagedVector>,
    sender: ByteSliceView,
    message: ByteSliceView,
) -> UnmanagedVector {
    let payload = message.to_owned().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::publish_module(addr, payload, db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    return UnmanagedVector::new(Some(ret))
}

// exported function to query contract (in smart way)
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn query_contract(db: Db, api: GoApi, querier: GoQuerier, is_verbose: bool, gas_limit: u64, gas_used: Option<&mut u64>, errmsg: Option<&mut UnmanagedVector>, sender: ByteSliceView, message: ByteSliceView) -> UnmanagedVector {
    let payload = message.to_owned().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe( move || {
        vm::query_contract(addr, payload, db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    return UnmanagedVector::new(Some(ret));
}
