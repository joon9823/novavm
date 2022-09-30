use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use crate::args::VM_ARG;
use crate::error::handle_c_error_default;
use crate::error::{handle_c_error_binary, Error};
use crate::move_api::handler as api_handler;
use crate::{api::GoApi, querier::GoQuerier, vm, ByteSliceView, Db, UnmanagedVector};

use move_deps::move_cli::Move;
use move_deps::move_cli::base::coverage::{Coverage, CoverageSummaryOptions};
use move_deps::move_cli::base::info::Info;
use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_cli::base::{
    build::Build,
    test::Test,
    // TODO: implement them
    // coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
    // info::Info, movey_login::MoveyLogin, movey_upload::MoveyUpload, new::New, prove::Prove,
};
use move_deps::move_package::{Architecture, BuildConfig};
use nova_compiler::New;
use crate::compiler::{compile, Command, self};
use novavm::NovaVM;


#[repr(C)]
pub struct vm_t {}

pub fn to_vm(ptr: *mut vm_t) -> Option<&'static mut NovaVM> {
    if ptr.is_null() {
        None
    } else {
        let c = unsafe { &mut *(ptr as *mut NovaVM) };
        Some(c)
    }
}

#[no_mangle]
pub extern "C" fn release_vm(vm: *mut vm_t) {
    if !vm.is_null() {
        // this will free cache when it goes out of scope
        let _ = unsafe { Box::from_raw(vm as *mut NovaVM) };
    }
}

#[no_mangle]
pub extern "C" fn allocate_vm() -> *mut vm_t {
    // let mut vm: Result<*mut NovaVM, RustError> = Ok(Box::into_raw(Box::new(NovaVM::new())));
    let vm = Box::into_raw(Box::new(NovaVM::new()));
    vm as *mut vm_t
}

// VM initializer
#[no_mangle]
pub extern "C" fn initialize(
    vm_ptr: *mut vm_t,
    db: Db,
    _verbose: bool,
    errmsg: Option<&mut UnmanagedVector>,
    module_bundle: ByteSliceView,
) -> () {
    let module_bundle = module_bundle.read().unwrap();
    let res = match to_vm(vm_ptr) {
        Some(vm) => catch_unwind(AssertUnwindSafe(move || {
            vm::initialize_vm(vm, db, module_bundle)
        }))
        .unwrap_or_else(|_| Err(Error::panic())),
        None => Err(Error::unset_arg(VM_ARG)),
    };

    handle_c_error_default(res, errmsg)
}

/// exported function to publish module bundle
#[no_mangle]
pub extern "C" fn publish_module_bundle(
    vm_ptr: *mut vm_t,
    db: Db,
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    session_id: ByteSliceView,
    sender: ByteSliceView,
    module_bundle: ByteSliceView,
) -> UnmanagedVector {
    let sid = session_id.read().unwrap();
    let module_bundle = module_bundle.read().unwrap();
    let addr = AccountAddress::from_bytes(sender.read().unwrap()).unwrap();
    
    let res = match to_vm(vm_ptr) {
        Some(vm) => catch_unwind(AssertUnwindSafe(move || {
            vm::publish_module_bundle(vm, sid.to_vec(), addr, module_bundle,  db, gas_limit)
        }))
        .unwrap_or_else(|_| Err(Error::panic())),
        None => Err(Error::unset_arg(VM_ARG)),
    };

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}


// exported function to execute (an entrypoint of) contract
#[no_mangle]
pub extern "C" fn execute_contract(
    vm_ptr: *mut vm_t,
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

    let res = match to_vm(vm_ptr) {
        Some(vm) => catch_unwind(AssertUnwindSafe(move || {
            vm::execute_contract(vm, sid.to_vec(), addr, payload.to_vec(), db, gas_limit)
        }))
        .unwrap_or_else(|_| Err(Error::panic())),
        None => Err(Error::unset_arg(VM_ARG)),
    };

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

// exported function to query contract (in smart way)
#[no_mangle]
pub extern "C" fn query_contract(
    vm_ptr: *mut vm_t,
    db: Db,
    _api: GoApi,
    _querier: GoQuerier,
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    message: ByteSliceView,
) -> UnmanagedVector {
    let payload = message.read().unwrap();

    let res = match to_vm(vm_ptr) {
        Some(vm) => catch_unwind(AssertUnwindSafe(move || {
            vm::query_contract(vm, payload.to_vec(), db, gas_limit)
        }))
        .unwrap_or_else(|_| Err(Error::panic())),
        None => Err(Error::unset_arg(VM_ARG)),
    };

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

// exported function to execute (an entrypoint of) script
#[no_mangle]
pub extern "C" fn execute_script(
    vm_ptr: *mut vm_t,
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

    let res = match to_vm(vm_ptr) {
        Some(vm) => catch_unwind(AssertUnwindSafe(move || {
            vm::execute_script(vm, sid.to_vec(), addr, payload.to_vec(), db, gas_limit)
        }))
        .unwrap_or_else(|_| Err(Error::panic())),
        None => Err(Error::unset_arg(VM_ARG)),
    };

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
pub extern "C" fn build_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
    verbose: bool,
    dev_mode: bool,
    test_mode: bool,
    generate_docs: bool,
    generate_abis: bool,
    install_dir: ByteSliceView,
    force_recompilation: bool,
    fetch_deps_only: bool,
) -> UnmanagedVector {
    let package_path_str = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let package_path_buf = Path::new(&package_path_str);

    let install_dir_str = String::from_utf8(install_dir.read().unwrap().to_vec()).unwrap();
    let install_dir_buf = if install_dir_str.len() > 0 {
        Some(Path::new(&install_dir_str).to_path_buf())
    } else {
        None
    };

    let build_config = BuildConfig {
        dev_mode,
        test_mode,
        generate_docs,
        generate_abis,
        install_dir: install_dir_buf,
        force_recompilation,
        additional_named_addresses: BTreeMap::new(),
        architecture: Some(Architecture::Move),
        fetch_deps_only,
    };

    let move_args = Move {
        package_path: Some(package_path_buf.to_path_buf()),
        verbose,
        build_config,
    };
    let cmd = Command::Build(Build);

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn test_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
    verbose: bool,
    dev_mode: bool,
    test_mode: bool,
    generate_docs: bool,
    generate_abis: bool,
    install_dir: ByteSliceView,
    force_recompilation: bool,
    fetch_deps_only: bool,
    /* for test config */
    instruction_execution_bound: u64,
    filter: ByteSliceView,
    list: bool,
    num_threads: usize,
    report_statistics: bool,
    report_storage_on_error: bool,
    ignore_compile_warnings: bool,
    check_stackless_vm: bool,
    verbose_mode: bool,
    compute_coverage: bool,
) -> UnmanagedVector {
    let package_path_str = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let package_path_buf = Path::new(&package_path_str);

    let install_dir_str = String::from_utf8(install_dir.read().unwrap().to_vec()).unwrap();
    let install_dir_buf = if install_dir_str.len() > 0 {
        Some(Path::new(&install_dir_str).to_path_buf())
    } else {
        None
    };

    let build_config = BuildConfig {
        dev_mode,
        test_mode,
        generate_docs,
        generate_abis,
        install_dir: install_dir_buf,
        force_recompilation,
        additional_named_addresses: BTreeMap::new(),
        architecture: Some(Architecture::Move),
        fetch_deps_only,
    };

    let move_args = Move {
        package_path: Some(package_path_buf.to_path_buf()),
        verbose,
        build_config,
    };

    let filter_opt = match filter.read() {
        Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
        None => None,
    };

    let test_arg = Test {
        instruction_execution_bound: Some(instruction_execution_bound),
        filter: filter_opt,
        list,
        num_threads,
        report_statistics,
        report_storage_on_error,
        ignore_compile_warnings,
        check_stackless_vm,
        verbose_mode,
        compute_coverage,
    };
    let cmd = Command::Test(test_arg);

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}



#[no_mangle]
pub extern "C" fn get_move_package_info(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
) -> UnmanagedVector {
    let package_path_str = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let package_path_buf = Path::new(&package_path_str);

    let move_args = Move {
        package_path: Some(package_path_buf.to_path_buf()),
        verbose: false,
        build_config: BuildConfig::default(),
    };
    let cmd = Command::Info(Info);

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn create_new_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
) -> UnmanagedVector {
    let package_path_str = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let package_path_buf = Path::new(&package_path_str);

    let move_args = Move {
        package_path: Some(package_path_buf.to_path_buf()),
        verbose: false,
        build_config: BuildConfig::default(),
    };

    let cmd = Command::New(New{name: package_path_str});

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn check_coverage_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
    summary_mode: compiler::CoverageOption,
    functions: bool, // Whether function coverage summaries should be displayed
    output_csv: bool, // Output CSV data of coverage
    module_name_view : ByteSliceView, // Display coverage information about the module against source code or dissambled bytecode
    /* for test coverage options */
) -> UnmanagedVector {
    let package_path_str = String::from_utf8(package_path.read().unwrap().to_vec()).unwrap();
    let package_path_buf = Path::new(&package_path_str);
    let module_name = String::from_utf8(module_name_view.read().unwrap().to_vec()).unwrap();

    let move_args = Move {
        package_path: Some(package_path_buf.to_path_buf()),
        verbose: false,
        build_config: BuildConfig::default(),
    };

    let options = match summary_mode {
        compiler::CoverageOption::Summary => CoverageSummaryOptions::Summary { functions, output_csv },
        compiler::CoverageOption::Source => CoverageSummaryOptions::Source { module_name },
        compiler::CoverageOption::Bytecode => CoverageSummaryOptions::Bytecode { module_name },
    };

    let cmd = Command::Coverage(Coverage{ options });

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}
