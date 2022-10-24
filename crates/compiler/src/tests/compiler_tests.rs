use move_deps::{
    move_cli::{base::test::Test, Move},
    move_package::BuildConfig,
};
use serial_test::serial;
use std::env;
use std::{env::temp_dir, path::PathBuf};

use crate::{compile, Clean, Command, New};

const MOVE_TEST_PATH: &str = "../move-test";

#[test]
#[serial]
fn test_move_test() {
    let package_path = path_in_crate(MOVE_TEST_PATH);

    let mut build_config = BuildConfig::default();
    build_config.test_mode = true;
    build_config.dev_mode = true;
    build_config.install_dir = Some(package_path.join("build-test"));

    let move_args = Move {
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let test_arg = Test {
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
    assert!(res == Vec::from("ok"));
}

#[test]
#[serial] // NOTE: should be run after test_move_test()
fn test_move_clean() {
    let package_path = path_in_crate(MOVE_TEST_PATH);
    let mut build_config = BuildConfig::default();
    build_config.install_dir = Some(package_path.join("build-test"));
    let move_args = Move {
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let c = Clean { clean_cache: false , clean_byproduct: false, force: true};

    let res = compile(move_args, Command::Clean(c)).expect("compiler err");
    assert!(res == Vec::from("ok"));
}

#[test]
#[serial]
fn test_move_compile() {
    let package_path = path_in_crate(MOVE_TEST_PATH);
    let build_config = BuildConfig::default();
    let move_args = Move {
        package_path: Some(package_path.canonicalize().unwrap()),
        verbose: true,
        build_config,
    };

    let res = compile(
        move_args,
        Command::Build(move_deps::move_cli::base::build::Build),
    )
    .expect("compiler err");
    assert!(res == Vec::from("ok"));
}

#[test]
#[serial]
fn test_move_new() {
    let build_config = BuildConfig::default();
    let temp_package_path = temp_dir().join("test_move_package");
    eprint!(
        "TEMPORARY MOVE PACKAGE PATH: {}",
        temp_package_path.display()
    );
    let move_args = Move {
        package_path: Some(temp_package_path.clone()),
        verbose: true,
        build_config,
    };

    let res = compile(
        move_args,
        Command::New(New {
            name: String::from("test_move_package"),
        }),
    )
    .expect("compiler err");
    assert!(res == Vec::from("ok"));

    // remove temporary package
    assert!(std::fs::remove_dir_all(temp_package_path).is_ok());
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}
