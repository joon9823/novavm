// from move-language/move/tools/move-cli/src/lib.rs
// SPDX-License-Identifier: Apache-2.0
use crate::mock::{BlankStorage, MockApi};
use anyhow::bail;
use clap::__macro_refs::once_cell::sync::Lazy;
use move_deps::move_cli::base::{build::Build, test::Test};
use move_deps::move_cli::Move;
use move_deps::move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_deps::move_unit_test;
use move_deps::move_vm_runtime::native_extensions::NativeContextExtensions;
use move_deps::move_vm_runtime::native_functions::NativeFunction;
use novavm::gas::NativeGasParameters;
use novavm::natives::block::NativeBlockContext;
use novavm::natives::code::NativeCodeContext;
use novavm::natives::nova_natives;
use novavm::natives::table::NativeTableContext;

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
pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

static DUMMY_RESOLVER: Lazy<BlankStorage> = Lazy::new(|| BlankStorage);

fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    exts.add(NativeCodeContext::default());
    exts.add(NativeTableContext::new([0; 32], &*DUMMY_RESOLVER));
    exts.add(NativeBlockContext::new(&MockApi {
        height: 100,
        timestamp: 100,
    }));
}

// works as entrypoint
pub fn compile(move_args: Move, cmd: Command) -> anyhow::Result<Vec<u8>> {
    //let cost_table = &INITIAL_COST_SCHEDULE;
    //let error_descriptions: ErrorMapping = bcs::from_bytes(move_stdlib::error_descriptions()).unwrap();
    let natives = nova_natives(NativeGasParameters::zeros());
    configure_for_unit_test();

    let res = run_compiler(
        natives,
        //cost_table,
        //&error_descriptions,
        move_args,
        cmd,
    );
    match res {
        Ok(_r) => Ok(Vec::from("ok")), // FIXME: do we have to return some valuable contents?
        Err(e) => bail!(e.to_string()),
    }
}
