// from move-language/move/tools/move-cli/src/lib.rs
// SPDX-License-Identifier: Apache-2.0
use anyhow::{anyhow, bail};
use move_deps::move_cli::{sandbox, experimental, Move};
use move_deps::move_cli::base::{
    build::Build, coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
    info::Info, movey_login::MoveyLogin, movey_upload::MoveyUpload, prove::Prove,
    test::Test,
};
use move_deps::move_core_types::{
    account_address::AccountAddress, identifier::Identifier,
};
use move_deps::move_vm_runtime::native_functions::NativeFunction;
use novavm::gas::NativeGasParameters;
use novavm::natives::nova_natives;
use std::fmt;
use std::path::{PathBuf};

use crate::Clean;
use crate::New;

/// Default directory where saved Move resources live
pub const DEFAULT_STORAGE_DIR: &str = "storage";

type NativeFunctionRecord = (AccountAddress, Identifier, Identifier, NativeFunction);

pub enum Command {
    Build(Build),
    Coverage(Coverage),
    Disassemble(Disassemble),
    Docgen(Docgen),
    Errmap(Errmap),
    Info(Info),
    MoveyUpload(MoveyUpload),
    New(New),
    Prove(Prove),
    Test(Test),
    /// Execute a sandbox command.
    Sandbox {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        storage_dir: PathBuf,
        cmd: sandbox::cli::SandboxCommand,
    },
    /// (Experimental) Run static analyses on Move source or bytecode.
    Experimental {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        storage_dir: PathBuf,
        cmd: experimental::cli::ExperimentalCommand,
    },
    MoveyLogin(MoveyLogin),
    Clean(Clean),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Build(_) => write!(f, "build"),
            Command::Coverage(_) => write!(f, "coverage"),
            Command::Disassemble(_) => write!(f, "disassemble"),
            Command::Docgen(_) => write!(f, "docgen"),
            Command::Errmap(_) => write!(f, "errmap"),
            Command::Info(_) => write!(f, "info"),
            Command::MoveyUpload(_) => write!(f, "movey upload"),
            Command::New(_) => write!(f, "new"),
            Command::Prove(_) => write!(f, "prove"),
            Command::Test(_) => write!(f, "test"),
            Command::MoveyLogin(_) => write!(f, "movey login"),
            Command::Sandbox { storage_dir: _, cmd: _ } => write!(f, "sandbox"),
            Command::Experimental { storage_dir: _, cmd: _ } => write!(f, "experimental"),
            Command::Clean(_) => write!(f, "clean"),
        }
    }
}

fn run_compiler(
    natives: Vec<NativeFunctionRecord>,
    //_cost_table: &CostTable,
    //_error_descriptions: &ErrorMapping,
    move_args: Move,
    cmd: Command,
) -> anyhow::Result<()> {
    // TODO: right now, the gas metering story for move-cli (as a library) is a bit of a mess.
    //         1. It's still using the old CostTable.
    //         2. The CostTable only affects sandbox runs, but not unit tests, which use a unit cost table.
    match cmd {
        // supported by move-cli
        Command::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Test(c) => c.execute(move_args.package_path, move_args.build_config, natives),
        Command::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::New(c) => c.execute_with_defaults(move_args.package_path),
        /* TODO: unsupported yet
        Command::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Docgen(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::MoveyUpload(c) => c.execute(move_args.package_path),
        Command::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Sandbox { storage_dir, cmd } => cmd.handle_command(
            natives,
            cost_table,
            error_descriptions,
            &move_args,
            &storage_dir,
        ),
        Command::Experimental { storage_dir, cmd } => cmd.handle_command(&move_args, &storage_dir),
        Command::MoveyLogin(c) => c.execute(),
        */
        // custom actions
		Command::Clean(c) => c.execute(move_args.package_path),
		c => Err(anyhow!("unimplemented function: {}", c)),

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

