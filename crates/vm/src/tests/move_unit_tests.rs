// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_utils::mock_chain::{BlankStorage, MockApi};
use move_deps::{
    move_cli::base::test::{run_move_unit_tests, UnitTestResult},
    move_package, move_unit_test,
    move_unit_test::UnitTestingConfig,
    move_vm_runtime::{
        native_extensions::NativeContextExtensions, native_functions::NativeFunctionTable,
    },
};
use nova_gas::NativeGasParameters;
use nova_natives::all_natives;
use nova_natives::{block::NativeBlockContext, code::NativeCodeContext, table::NativeTableContext};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use tempfile::tempdir;

static DUMMY_RESOLVER: Lazy<BlankStorage> = Lazy::new(|| BlankStorage);

pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    exts.add(NativeCodeContext::default());
    exts.add(NativeTableContext::new([0; 32], &*DUMMY_RESOLVER));
    exts.add(NativeBlockContext::new(&MockApi {
        height: 0,
        timestamp: 0,
    }));
}

fn nova_test_natives() -> NativeFunctionTable {
    configure_for_unit_test();
    let gas_params = NativeGasParameters::zeros();
    let abs_val_size_gas_params = nova_gas::AbstractValueSizeGasParameters::zeros();
    all_natives(
        gas_params.move_stdlib,
        gas_params.nova_stdlib,
        gas_params.table,
        abs_val_size_gas_params,
    )
}

fn run_tests_for_pkg(path_to_pkg: impl Into<String>) {
    let pkg_path = path_in_crate(path_to_pkg);

    let res = run_move_unit_tests(
        &pkg_path,
        move_package::BuildConfig {
            test_mode: true,
            install_dir: Some(tempdir().unwrap().path().to_path_buf()),
            ..Default::default()
        },
        UnitTestingConfig::default_with_bound(Some(100_000)),
        nova_test_natives(),
        /* compute_coverage */ false,
        &mut std::io::stdout(),
    )
    .unwrap();

    if res != UnitTestResult::Success {
        panic!("aborting because of Move unit test failures");
    }
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}

#[test]
fn move_unit_tests() {
    run_tests_for_pkg("../move-test");
}

#[test]
fn stdlib_move_unit_tests() {
    run_tests_for_pkg("../stdlib/src/nova_stdlib");
}
