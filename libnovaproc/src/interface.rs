use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use crate::error::{handle_c_error_binary, Error};
use crate::move_api::compiler::{move_compiler, Command};
use crate::move_api::handler as api_handler;
use crate::{api::GoApi, querier::GoQuerier, vm, ByteSliceView, Db, UnmanagedVector};

use move_deps::move_cli::Move;
use move_deps::move_core_types::account_address::AccountAddress;

use move_deps::move_cli::base::{
    build::Build, test::Test,
    // TODO: implement them
    // coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
    // info::Info, movey_login::MoveyLogin, movey_upload::MoveyUpload, new::New, prove::Prove,
};
use move_deps::move_package::BuildConfig;

// VM initializer
#[no_mangle]
pub extern "C" fn initialize(
    db: Db,
    _verbose: bool,
    errmsg: Option<&mut UnmanagedVector>,
    module_bundle: ByteSliceView,
) -> UnmanagedVector {
    let module_bundle = module_bundle.read().unwrap();
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
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    sender: ByteSliceView,
    module_bytes: ByteSliceView,
) -> UnmanagedVector {
    let mb = module_bytes.read().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::publish_module(addr, mb.to_vec(), db, gas_limit)
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
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    session_id: ByteSliceView,
    sender: ByteSliceView,
    message: ByteSliceView,
) -> UnmanagedVector {
    let sid = session_id.read().unwrap();
    let payload = message.read().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::execute_contract(sid.to_vec(), addr, payload.to_vec(), db, gas_limit)
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
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    message: ByteSliceView,
) -> UnmanagedVector {
    let payload = message.read().unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::query_contract(payload.to_vec(), db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

// exported function to execute (an entrypoint of) script
#[no_mangle]
pub extern "C" fn execute_script(
    db: Db,
    _api: GoApi,
    _querier: GoQuerier,
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    session_id: ByteSliceView,
    sender: ByteSliceView,
    message: ByteSliceView,
) -> UnmanagedVector {
    let sid = session_id.read().unwrap();
    let payload = message.read().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        vm::execute_script(sid.to_vec(), addr, payload.to_vec(), db, gas_limit)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn decode_move_resource(
    db: Db,
    errmsg: Option<&mut UnmanagedVector>,
    struct_tag: ByteSliceView,
    resource_bytes: ByteSliceView,
) -> UnmanagedVector {
    let struct_tag = String::from_utf8(struct_tag.read().unwrap().to_vec()).unwrap();
    let payload = resource_bytes.read().unwrap();

    let res = catch_unwind(AssertUnwindSafe(move || {
        api_handler::decode_move_resource(db, struct_tag, payload)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn decode_module_bytes(
    errmsg: Option<&mut UnmanagedVector>,
    module_bytes: ByteSliceView,
) -> UnmanagedVector {
    let module_bytes = module_bytes.read().unwrap().to_vec();

    let res = catch_unwind(AssertUnwindSafe(move || {
        api_handler::decode_module_bytes(module_bytes)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn decode_script_bytes(
    errmsg: Option<&mut UnmanagedVector>,
    script_bytes: ByteSliceView,
) -> UnmanagedVector {
    let script_bytes = script_bytes.read().unwrap().to_vec();

    let res = catch_unwind(AssertUnwindSafe(move || {
        api_handler::decode_script_bytes(script_bytes)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn compile_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    package_path: ByteSliceView,
    verbose: bool
) -> UnmanagedVector {

    let path_vec = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let path = Path::new(&path_vec);

    let move_args = Move{
        package_path: Some(path.to_path_buf()),
        verbose,
        build_config: BuildConfig::default(),
    };
    let cmd = Command::Build(Build);

    let res = catch_unwind(AssertUnwindSafe(move|| {
        move_compiler(move_args, cmd)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn test_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    package_path: ByteSliceView,
    verbose: bool
) -> UnmanagedVector {

    let path_vec = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let path = Path::new(&path_vec);

    let move_args = Move{
        package_path: Some(path.to_path_buf()),
        verbose,
        build_config: BuildConfig::default(),
    };

    // TODO make adjustable
    let test_arg = Test{ 
        instruction_execution_bound: None, 
        filter: None, 
        list: true, 
        num_threads: 8, // 8 is from clap trait of base/tests.rs
        report_statistics: true, 
        report_storage_on_error: true,
        ignore_compile_warnings: false, 
        check_stackless_vm: false, 
        verbose_mode: true , 
        compute_coverage: true, };
    let cmd = Command::Test(test_arg);

    let res = catch_unwind(AssertUnwindSafe(move|| {
        move_compiler(move_args, cmd)
    }))
    .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}