use std::{collections::BTreeMap, path::Path};

use move_deps::{move_cli::{Move, base::{test::Test, disassemble::Disassemble}}, move_package::{BuildConfig, Architecture}};
use nova_compiler::compile as nova_compile;
use crate::{error::Error, ByteSliceView};

pub use nova_compiler::Command;

pub fn compile(
    move_args: Move,
    cmd: Command,
) -> Result<Vec<u8>, Error> {
    let action = cmd.to_string();

    match nova_compile(move_args, cmd) {
        Ok(_) => Ok(Vec::from("ok")),
        Err(e) => Err(Error::backend_failure(format!("failed to {}: {}", action, e))),
    }
}

/// cbindgen:prefix-with-name
#[allow(dead_code)]
#[derive(PartialEq)]
#[repr(u8)] // This makes it so the enum looks like a simple u32 to Go
pub enum CoverageOption{
    /// Display a coverage summary for all modules in this package
    Summary = 0, // no 0 for the purpose
    /// Display coverage information about the module against source code
    Source = 1,
    /// Display coverage information about the module against disassembled bytecode
    Bytecode = 2,
}

#[repr(C)]
pub struct NovaCompilerArgument {
    /// Path to a package which the command should be run with respect to.
    pub package_path: ByteSliceView,

    /// Print additional diagnostics if available.
    pub verbose: bool,

    /// Package build options
    pub build_config: NovaCompilerBuildConfig,
}

impl From<NovaCompilerArgument> for Move{
    fn from(val: NovaCompilerArgument) -> Self {
        let package_path = match val.package_path.read() {
            Some(s) => Some(Path::new(&String::from_utf8(s.to_vec()).unwrap()).to_path_buf()),
            None => None,
        };
        Self{ package_path, verbose: val.verbose, build_config: val.build_config.into() }
    }
}

#[repr(C)]
pub struct NovaCompilerBuildConfig {
    /// Compile in 'dev' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used if
    /// this flag is set. This flag is useful for development of packages that expose named
    /// addresses that are not set to a specific value.
    pub dev_mode: bool,

    /// Compile in 'test' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used
    /// along with any code in the 'tests' directory.
    pub test_mode: bool,

    /// Generate documentation for packages
    pub generate_docs: bool,

    /// Generate ABIs for packages
    pub generate_abis: bool,

    /// Installation directory for compiled artifacts. Defaults to current directory.
    pub install_dir: ByteSliceView,

    /// Force recompilation of all packages
    pub force_recompilation: bool,

    /* unused
    /// Additional named address mapping. Useful for tools in rust
    pub additional_named_addresses: BTreeMap<String, AccountAddress>,

    pub architecture: Option<Architecture>,
    */

    /// Only fetch dependency repos to MOVE_HOME
    pub fetch_deps_only: bool,
}

impl From<NovaCompilerBuildConfig> for BuildConfig {
    fn from(val: NovaCompilerBuildConfig) -> Self {
        let install_dir = match val.install_dir.read() {
            Some(s) => Some(Path::new(&String::from_utf8(s.to_vec()).unwrap()).to_path_buf()),
            None => None,
        };
        Self{
            dev_mode: val.dev_mode,
            test_mode: val.test_mode,
            generate_docs: val.generate_docs,
            generate_abis: val.generate_abis,
            install_dir,
            force_recompilation: val.force_recompilation,
            additional_named_addresses: BTreeMap::new(),
            architecture: Some(Architecture::Move),
            fetch_deps_only: val.fetch_deps_only,
        }
    }
}

#[repr(C)]
pub struct NovaCompilerTestOption {
    /// Bound the number of instructions that can be executed by any one test.
    /// set 0 to no-boundary
    pub instruction_execution_bound: u64,
    /// A filter string to determine which unit tests to run. A unit test will be run only if it
    /// contains this string in its fully qualified (<addr>::<module_name>::<fn_name>) name.
    pub filter: ByteSliceView,
    /// List all tests
    pub list: bool,
    /// Number of threads to use for running tests.
    pub num_threads: usize,
    /// Report test statistics at the end of testing
    pub report_statistics: bool,
    /// Show the storage state at the end of execution of a failing test
    pub report_storage_on_error: bool,

    /// Ignore compiler's warning, and continue run tests
    pub ignore_compile_warnings: bool,

    /// Use the stackless bytecode interpreter to run the tests and cross check its results with
    /// the execution result from Move VM.
    pub check_stackless_vm: bool,
    /// Verbose mode
    pub verbose_mode: bool,
    /// Collect coverage information for later use with the various `package coverage` subcommands
    pub compute_coverage: bool,
}

impl From<NovaCompilerTestOption> for Test {
    fn from(val: NovaCompilerTestOption) -> Self {
        let filter= match val.filter.read() {
            Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
            None => None,
        };
        Self {
            instruction_execution_bound: match val.instruction_execution_bound{
                0 => None,
                _ => Some(val.instruction_execution_bound)
            },
            filter,
            list: val.list,
            num_threads: val.num_threads,
            report_statistics: val.report_statistics,
            report_storage_on_error: val.report_storage_on_error,
            ignore_compile_warnings: val.ignore_compile_warnings,
            check_stackless_vm: val.check_stackless_vm,
            verbose_mode: val.verbose_mode,
            compute_coverage: val.compute_coverage
        }
    }
}

#[repr(C)]
pub struct NovaCompilerDisassembleOption {
    /// Start a disassembled bytecode-to-source explorer
    pub interactive: bool,
    /// The package name. If not provided defaults to current package modules only
    pub package_name: ByteSliceView,
    /// The name of the module or script in the package to disassemble
    pub module_or_script_name: ByteSliceView,
}

impl From<NovaCompilerDisassembleOption> for Disassemble {
    fn from(val: NovaCompilerDisassembleOption) -> Self {
        let package_name = match val.package_name.read() {
            Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
            None => None,
        };
        let module_or_script_name = String::from_utf8(val.module_or_script_name.read().unwrap().to_vec()).unwrap();
        Self { interactive: val.interactive, package_name, module_or_script_name }
    }
}