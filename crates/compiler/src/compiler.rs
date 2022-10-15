// from move-language/move/tools/move-cli/src/lib.rs
// SPDX-License-Identifier: Apache-2.0
use anyhow::bail;
use move_deps::move_cli::Move;
use move_deps::move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_deps::move_vm_runtime::native_functions::NativeFunction;
use nova_gas::NativeGasParameters;
use nova_natives::all_natives;

use crate::extensions::configure_for_unit_test;
use crate::Command;

// works as entrypoint
pub fn compile(move_args: Move, cmd: Command) -> anyhow::Result<Vec<u8>> {
    //let cost_table = &INITIAL_COST_SCHEDULE;
    //let error_descriptions: ErrorMapping = bcs::from_bytes(move_stdlib::error_descriptions()).unwrap();
    let gas_params = NativeGasParameters::zeros();
    let natives = all_natives(
        gas_params.move_stdlib,
        gas_params.nova_stdlib,
        gas_params.table,
    );
    configure_for_unit_test();

    let res = run_compiler(
        natives, //cost_table,
        //&error_descriptions,
        move_args, cmd,
    );
    match res {
        Ok(_r) => Ok(Vec::from("ok")), // FIXME: do we have to return some valuable contents?
        Err(e) => bail!(e.to_string()),
    }
}

fn run_compiler(
    natives: Vec<(AccountAddress, Identifier, Identifier, NativeFunction)>,
    move_args: Move,
    cmd: Command,
) -> anyhow::Result<()> {
    match cmd {
        Command::Test(c) => c.execute(move_args.package_path, move_args.build_config, natives),
        Command::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::New(c) => c.execute_with_defaults(move_args.package_path),
        Command::Clean(c) => c.execute(move_args.package_path, move_args.build_config),
    }
}
