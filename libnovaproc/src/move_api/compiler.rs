// from move-language/move/tools/move-cli/src/lib.rs
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result, bail};
use clap::Parser;
use move_deps::move_cli::{sandbox, experimental, Move};
use move_deps::move_cli::base::{
    build::Build, coverage::Coverage, disassemble::Disassemble, docgen::Docgen, errmap::Errmap,
    info::Info, movey_login::MoveyLogin, movey_upload::MoveyUpload, new::New, prove::Prove,
    test::Test,
};
use move_deps::move_core_types::{
    account_address::AccountAddress, identifier::Identifier,
};
use move_deps::move_vm_runtime::native_functions::NativeFunction;
use novavm::gas::NativeGasParameters;
use novavm::natives::nova_natives;
use std::fmt;
use std::path::{PathBuf, Path};
use move_deps::move_command_line_common::env::MOVE_HOME;

use crate::error::Error;


/// Default directory where saved Move resources live
pub const DEFAULT_STORAGE_DIR: &str = "storage";

type NativeFunctionRecord = (AccountAddress, Identifier, Identifier, NativeFunction);

/// MoveCLI is the CLI that will be executed by the `move-cli` command
/// The `cmd` argument is added here rather than in `Move` to make it
/// easier for other crates to extend `move-cli`
#[derive(Parser)]
pub struct MoveCLI {
    #[clap(flatten)]
    pub move_args: Move,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
#[clap(name = "clean")]
pub struct Clean {
    clean_cache: bool
}

impl Clean {
    pub fn execute(self, path: Option<PathBuf>) -> anyhow::Result<()> { 
        let path = match path{
            Some(p) => p,
            None => Path::new(".").to_path_buf(),
        }.join("build");

        let res = std::fs::remove_dir_all(path);
        match res {
            Err(e) => bail!("failed to clean the package: {}", e),
            Ok(_) => {
                let move_home = &*MOVE_HOME;
                if self.clean_cache {
                    match std::fs::remove_dir_all(PathBuf::from(move_home)) {
                        Err(e) => bail!("failed to clean cache: {}", e),
                        Ok(_) => Ok(()),
                    }
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Parser)]
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
    #[clap(name = "sandbox")]
    Sandbox {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[clap(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[clap(subcommand)]
        cmd: sandbox::cli::SandboxCommand,
    },
    /// (Experimental) Run static analyses on Move source or bytecode.
    #[clap(name = "experimental")]
    Experimental {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[clap(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[clap(subcommand)]
        cmd: experimental::cli::ExperimentalCommand,
    },
    #[clap(name = "movey-login")]
    MoveyLogin(MoveyLogin),
    #[clap(name = "clean")]
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

pub fn run_compiler(
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
        Command::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Test(c) => c.execute(move_args.package_path, move_args.build_config, natives),
        Command::Clean(c) => c.execute(move_args.package_path),
        c => Err(anyhow!("unimplemented function: {}", c)),
        /* TODO: unsupported yet
        Command::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Docgen(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::MoveyUpload(c) => c.execute(move_args.package_path),
        Command::New(c) => c.execute_with_defaults(move_args.package_path),
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
    }
}

// works as entrypoint for move-compiler
pub fn move_compiler(
    move_args: Move,
    cmd: Command,
) -> Result<Vec<u8>, Error> {
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
        Err(e ) => Err(Error::vm_err(e)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::env;
    use move_deps::{move_package::BuildConfig, move_cli::{Move, base::test::Test}};
    use serial_test::serial;

    use crate::move_api::compiler::Clean;

    use super::{move_compiler, Command};

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

        let res = move_compiler(move_args, cmd).expect("compiler err");
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

        let res = move_compiler(move_args, Command::Build(move_deps::move_cli::base::build::Build)).expect("compiler err");
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

        let res = move_compiler(move_args, Command::Clean(c)).expect("compiler err");
        assert!(res==Vec::from("ok"));
    }
}