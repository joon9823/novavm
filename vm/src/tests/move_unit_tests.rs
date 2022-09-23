// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_deps::{
    move_cli::base::test::{run_move_unit_tests, UnitTestResult},
    move_table_extension::{table_natives, GasParameters},
    move_unit_test::UnitTestingConfig,
    move_stdlib, move_unit_test, move_package,
    move_core_types::language_storage::CORE_CODE_ADDRESS,
    move_vm_runtime::{
        native_extensions::NativeContextExtensions,
        native_functions::NativeFunctionTable,
    },
};

use crate::{nova_natives::{self, code::NativeCodeContext}};
use crate::gas::NativeGasParameters;
use std::{path::PathBuf};
use tempfile::tempdir;

pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    exts.add(NativeCodeContext::default());
}

pub fn nova_natives(gas_params: NativeGasParameters) -> NativeFunctionTable {
    move_stdlib::natives::all_natives(CORE_CODE_ADDRESS, gas_params.move_stdlib)
        .into_iter()
        .chain(
            nova_natives::all_natives(
            CORE_CODE_ADDRESS,
            gas_params.nova_stdlib))
        .chain(
            table_natives(
                CORE_CODE_ADDRESS,
                GasParameters::zeros()))
        .chain(
            move_stdlib::natives::nursery_natives(
                CORE_CODE_ADDRESS,
                move_stdlib::natives::NurseryGasParameters::zeros(),
            )
            .into_iter()
            .filter(|(addr, module_name, _, _)| {
                !(*addr == CORE_CODE_ADDRESS && module_name.as_str() == "event")
            }),
        )
        .collect()
}

fn nova_test_natives() -> NativeFunctionTable {
    configure_for_unit_test();
    nova_natives(NativeGasParameters::zeros())
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

#[test]
fn move_unit_tests() {
    run_tests_for_pkg("move-test");
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}
