use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use crate::args::VM_ARG;
use crate::compiler::{CoverageOption, NovaCompilerArgument, NovaCompilerTestOption, NovaCompilerDisassembleOption, NovaCompilerProveOption, NovaCompilerDocgenOption};
use crate::error::handle_c_error_default;
use crate::error::{handle_c_error_binary, Error};
use crate::move_api::handler as api_handler;
use crate::{api::GoApi, vm, ByteSliceView, Db, UnmanagedVector};

use move_deps::move_cli::Move;
use move_deps::move_cli::base::coverage::{Coverage, CoverageSummaryOptions};
use move_deps::move_cli::base::errmap::Errmap;
use move_deps::move_cli::base::info::Info;
use move_deps::move_cli::base::movey_login::MoveyLogin;
use move_deps::move_cli::base::movey_upload::MoveyUpload;
use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_cli::base::{
    build::Build,
    // TODO: implement them
    // coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
    // info::Info, movey_login::MoveyLogin, movey_upload::MoveyUpload, new::New, prove::Prove,
};
use move_deps::move_package::BuildConfig;
use nova_compiler::New;
use crate::compiler::{compile, Command, self};
use novavm::NovaVM;


#[allow(non_camel_case_types)]
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
            vm::publish_module_bundle(vm, sid.to_vec(), addr, module_bundle, db, gas_limit)
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
    api: GoApi,
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
            vm::execute_contract(vm, sid.to_vec(), addr, payload.to_vec(), db, api, gas_limit)
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
    api: GoApi,
    _verbose: bool,
    gas_limit: u64,
    errmsg: Option<&mut UnmanagedVector>,
    message: ByteSliceView,
) -> UnmanagedVector {
    let payload = message.read().unwrap();

    let res = match to_vm(vm_ptr) {
        Some(vm) => catch_unwind(AssertUnwindSafe(move || {
            vm::query_contract(vm, payload.to_vec(), db, api, gas_limit)
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
    api: GoApi,
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
            vm::execute_script(vm, sid.to_vec(), addr, payload.to_vec(), db, api, gas_limit)
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
    nova_args: NovaCompilerArgument,
) -> UnmanagedVector {
    let cmd = Command::Build(Build);

    let res = catch_unwind(AssertUnwindSafe(move || compile(nova_args.into(), cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn test_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    nova_args: NovaCompilerArgument,
    test_opt: NovaCompilerTestOption,
) -> UnmanagedVector {
    let cmd = Command::Test(test_opt.into());

    let res = catch_unwind(AssertUnwindSafe(move || compile(nova_args.into(), cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn get_move_package_info(
    errmsg: Option<&mut UnmanagedVector>,
    package_path: ByteSliceView,
) -> UnmanagedVector {
    let move_args = generate_default_move_cli(Some(package_path), false);
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
    let move_args = generate_default_move_cli(Some(package_path), false);

    let cmd = Command::New(
        New{name: move_args.package_path.as_ref().expect("package path unset").to_str().expect("package path invalid").to_string()}
    );

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn check_coverage_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    package_path: ByteSliceView,
    summary_mode: compiler::CoverageOption,
    functions: bool, // Whether function coverage summaries should be displayed
    output_csv: bool, // Output CSV data of coverage
    module_name_view : ByteSliceView, // Display coverage information about the module against source code or dissambled bytecode
    /* for test coverage options */
) -> UnmanagedVector {
    let module_name = match module_name_view.read() {
        Some(s) => String::from_utf8(s.to_vec()).unwrap(),
        None => String::new(),
    };

    let move_args = generate_default_move_cli(Some(package_path), false);

    let options = match summary_mode {
        CoverageOption::Summary => CoverageSummaryOptions::Summary { functions, output_csv },
        CoverageOption::Source => CoverageSummaryOptions::Source { module_name },
        CoverageOption::Bytecode => CoverageSummaryOptions::Bytecode { module_name },
    };

    let cmd = Command::Coverage(Coverage{ options });

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn prove_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
    /* for prove config */
    prove_opt: NovaCompilerProveOption,
) -> UnmanagedVector {
    let move_args = generate_default_move_cli(Some(package_path), false);
    let cmd = Command::Prove(prove_opt.into());

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn disassemble_move_package(
    errmsg: Option<&mut UnmanagedVector>,
    /* for build config */
    package_path: ByteSliceView,
    disassemble_opt: NovaCompilerDisassembleOption,
) -> UnmanagedVector {
    let cmd = Command::Disassemble(disassemble_opt.into());

    let move_args = generate_default_move_cli(Some(package_path), false);

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn movey_login(
    errmsg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {

    let move_args = generate_default_move_cli(None, false);
    let cmd = Command::MoveyLogin(MoveyLogin);

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn movey_upload(
    errmsg: Option<&mut UnmanagedVector>,
    package_path: ByteSliceView,
) -> UnmanagedVector {
    let move_args = generate_default_move_cli(Some(package_path), false);
    let cmd = Command::MoveyUpload(MoveyUpload);

    let res = catch_unwind(AssertUnwindSafe(move || compile(move_args, cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn generate_error_map(
    errmsg: Option<&mut UnmanagedVector>,
    nova_args: NovaCompilerArgument,
    /* for generate errmap config */
    error_prefix_slice: ByteSliceView,
    output_file_slice: ByteSliceView
) -> UnmanagedVector {

    let error_prefix= match error_prefix_slice.read() {
        Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
        None => None,
    };

    let output_file = Path::new(&String::from_utf8(output_file_slice.read().unwrap().to_vec()).unwrap()).to_path_buf();

    let cmd = Command::Errmap(Errmap{ error_prefix, output_file });

    let res = catch_unwind(AssertUnwindSafe(move || compile(nova_args.into(), cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}

#[no_mangle]
pub extern "C" fn generate_docs(
    errmsg: Option<&mut UnmanagedVector>,
    nova_args: NovaCompilerArgument,
    docgen_opt: NovaCompilerDocgenOption,
) -> UnmanagedVector {

    let cmd = Command::Docgen(docgen_opt.into());

    let res = catch_unwind(AssertUnwindSafe(move || compile(nova_args.into(), cmd)))
        .unwrap_or_else(|_| Err(Error::panic()));

    let ret = handle_c_error_binary(res, errmsg);
    UnmanagedVector::new(Some(ret))
}


//
// internal functions
//

fn generate_default_move_cli(package_path_slice: Option<ByteSliceView>, verbose: bool) -> Move {
    let package_path = match package_path_slice {
        None => None,
        Some(slice) => match slice.read(){
            Some(s) => Some(Path::new(&String::from_utf8(s.to_vec()).unwrap()).to_path_buf()),
            None => None,
        }
    };
    Move{
        package_path,
        verbose,
        build_config: BuildConfig::default()
    }
}