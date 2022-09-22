use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::error::{handle_c_error_binary, Error};
use crate::{api::GoApi, querier::GoQuerier, vm, ByteSliceView, Db, UnmanagedVector};

use move_deps::move_core_types::account_address::AccountAddress;

// VM initializer
#[no_mangle]
pub extern "C" fn initialize(
    db: Db,
    _is_verbose: bool,
    errmsg: Option<&mut UnmanagedVector>,
    module_bundle: ByteSliceView,
) -> UnmanagedVector {
    let module_bundle = module_bundle.to_owned().unwrap();
    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::initialize_vm(db, module_bundle)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

/// exported function to publish a module
#[no_mangle]
pub extern "C" fn publish_module(
    db: Db,
    _is_verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    sender: ByteSliceView,
    module_bytes: ByteSliceView,
) -> UnmanagedVector {
    let mb = module_bytes.to_owned().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::publish_module(addr, mb, db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

// exported function to execute (an entrypoint of) contract
#[no_mangle]
pub extern "C" fn execute_contract(
    db: Db,
    _api: GoApi,
    _querier: GoQuerier,
    _is_verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    session_id: ByteSliceView,
    sender: ByteSliceView,
    message: ByteSliceView,
) -> UnmanagedVector {
    let sid = session_id.to_owned().unwrap();
    let payload = message.to_owned().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::execute_contract(sid, addr, payload, db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

// exported function to query contract (in smart way)
#[no_mangle]
pub extern "C" fn query_contract(
    db: Db,
    _api: GoApi,
    _querier: GoQuerier,
    _is_verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    message: ByteSliceView,
) -> UnmanagedVector {
    let payload = message.to_owned().unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::query_contract(payload, db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}
