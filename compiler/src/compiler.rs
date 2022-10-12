// from move-language/move/tools/move-cli/src/lib.rs
// SPDX-License-Identifier: Apache-2.0
use anyhow::bail;
use move_deps::move_cli::Move;
use move_deps::move_cli::base::{build::Build, test::Test};
use move_deps::move_core_types::{
    account_address::AccountAddress, identifier::Identifier,
};
use move_deps::move_vm_runtime::native_functions::NativeFunction;
use novavm::gas::NativeGasParameters;
use novavm::natives::nova_natives;

use std::fmt;

use crate::Clean;
use crate::New;

type NativeFunctionRecord = (AccountAddress, Identifier, Identifier, NativeFunction);

pub enum Command {
    Build(Build),
    New(New),
    Test(Test),
    Clean(Clean),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Build(_) => write!(f, "build"),
            Command::New(_) => write!(f, "new"),
            Command::Test(_) => write!(f, "test"),
            Command::Clean(_) => write!(f, "clean"),
        }
    }
}

fn run_compiler(
    natives: Vec<NativeFunctionRecord>,
    move_args: Move,
    cmd: Command,
) -> anyhow::Result<()> {
    match cmd {
        Command::Test(c) => c.execute(move_args.package_path, move_args.build_config, natives),
        Command::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::New(c) => c.execute_with_defaults(move_args.package_path),
        Command::Clean(c) => c.execute(move_args.package_path),
    }
}

// works as entrypoint
pub fn compile(
    move_args: Move,
    cmd: Command,
) -> anyhow::Result<Vec<u8>> {
    //let cost_table = &INITIAL_COST_SCHEDULE;
    //let error_descriptions: ErrorMapping = bcs::from_bytes(move_stdlib::error_descriptions()).unwrap();
    let natives = nova_natives(NativeGasParameters::zeros());

    let res = run_compiler(
        natives,
        //cost_table,
        //&error_descriptions,
        move_args,
        cmd
    );
    match res {
        Ok(_r) => Ok(Vec::from("ok")),  // FIXME: do we have to return some valuable contents?
        Err(e ) => bail!(e.to_string())
    }
}
