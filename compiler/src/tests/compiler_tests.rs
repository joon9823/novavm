
use std::path::Path;
use std::env;
use move_deps::{move_package::BuildConfig, move_cli::{Move, base::test::Test}};
use serial_test::serial;

use crate::compiler::{Command, compile, Clean};

#[test]
#[serial]
fn test_move_test() {
    // FIXME: move_cli seems to change current directory.. so we have to set current dir for now.
    let md= env::var("CARGO_MANIFEST_DIR").unwrap();
    let wd = Path::new(&md);
    let path = Path::new(&"../vm/move-test");
    let package_path = wd.join(path);
    
    let mut build_config = BuildConfig::default();
    build_config.test_mode = true;
    build_config.dev_mode = true;

    let move_args = Move{
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let test_arg = Test{ 
        instruction_execution_bound: None, 
        filter: None, 
        list: false, 
        num_threads: 8, // 8 is from clap trait of base/tests.rs
        report_statistics: true, 
        report_storage_on_error: true,
        ignore_compile_warnings: false, 
        check_stackless_vm: false, 
        verbose_mode: true, 
        compute_coverage: false,
    };
    let cmd = Command::Test(test_arg);

    let res = compile(move_args, cmd).expect("compiler err");
    assert!(res==Vec::from("ok"));

}

#[test]
#[serial]
fn test_move_compile() {
    // FIXME: move_cli seems to change current directory.. so we have to set current dir for now.
    let md= env::var("CARGO_MANIFEST_DIR").unwrap();
    let wd = Path::new(&md);
    let path = Path::new(&"../vm/move-test");
    let package_path = wd.join(path);

    let build_config = BuildConfig::default();
    let move_args = Move{
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let res = compile(move_args, Command::Build(move_deps::move_cli::base::build::Build)).expect("compiler err");
    assert!(res==Vec::from("ok"));
}

#[test]
#[serial]
fn test_move_clean() {
    // FIXME: move_cli seems to change current directory.. so we have to set current dir for now.
    let md= env::var("CARGO_MANIFEST_DIR").unwrap();
    let wd = Path::new(&md);
    let path = Path::new(&"../vm/move-test");
    let package_path = wd.join(path);
   
    let build_config = BuildConfig::default();
    let move_args = Move{
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let c = Clean{
        clean_cache: true,
    };

    let res = compile(move_args, Command::Clean(c)).expect("compiler err");
    assert!(res==Vec::from("ok"));
}

#[test]
#[serial]
fn test_move_info() {
    // FIXME: move_cli seems to change current directory.. so we have to set current dir for now.
    let md= env::var("CARGO_MANIFEST_DIR").unwrap();
    let wd = Path::new(&md);
    let path = Path::new(&"../vm/move-test");
    let package_path = wd.join(path);

    let build_config = BuildConfig::default();
    let move_args = Move{
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let res = compile(move_args, Command::Info(move_deps::move_cli::base::info::Info)).expect("compiler err");
    assert!(res==Vec::from("ok"));
}
