use std::{collections::BTreeMap, path::Path};

use crate::{error::Error, ByteSliceView/*, move_api::move_types::MoveType*/};
use move_deps::{
    move_cli::{
        base::{
            disassemble::Disassemble,
            docgen::Docgen,
            prove::{Prove, ProverOptions},
            test::Test,
        }, Move,
        //experimental::cli::{ConcretizeMode, ExperimentalCommand},
        //Move, sandbox::cli::{SandboxCommand, StructLayoutOptions},
    },
    //move_core_types::{parser::{parse_transaction_arguments, parse_type_tags}, transaction_argument::TransactionArgument, language_storage::TypeTag},
    move_package::{Architecture, BuildConfig},
};
use nova_compiler::compile as nova_compile;
//use nova_compiler::compiler::DEFAULT_STORAGE_DIR;

pub use nova_compiler::Command;

pub fn compile(move_args: Move, cmd: Command) -> Result<Vec<u8>, Error> {
    let action = cmd.to_string();

    match nova_compile(move_args, cmd) {
        Ok(_) => Ok(Vec::from("ok")),
        Err(e) => Err(Error::backend_failure(format!(
            "failed to {}: {}",
            action, e
        ))),
    }
}

/// cbindgen:prefix-with-name
#[allow(dead_code)]
#[derive(PartialEq)]
#[repr(u8)] // This makes it so the enum looks like a simple u32 to Go
pub enum CoverageOption {
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

impl From<NovaCompilerArgument> for Move {
    fn from(val: NovaCompilerArgument) -> Self {
        let package_path = match val.package_path.read() {
            Some(s) => Some(Path::new(&String::from_utf8(s.to_vec()).unwrap()).to_path_buf()),
            None => None,
        };
        Self {
            package_path,
            verbose: val.verbose,
            build_config: val.build_config.into(),
        }
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
        Self {
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
        let filter = match val.filter.read() {
            Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
            None => None,
        };
        Self {
            instruction_execution_bound: match val.instruction_execution_bound {
                0 => None,
                _ => Some(val.instruction_execution_bound),
            },
            filter,
            list: val.list,
            num_threads: val.num_threads,
            report_statistics: val.report_statistics,
            report_storage_on_error: val.report_storage_on_error,
            ignore_compile_warnings: val.ignore_compile_warnings,
            check_stackless_vm: val.check_stackless_vm,
            verbose_mode: val.verbose_mode,
            compute_coverage: val.compute_coverage,
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
        let module_or_script_name =
            String::from_utf8(val.module_or_script_name.read().unwrap().to_vec()).unwrap();
        Self {
            interactive: val.interactive,
            package_name,
            module_or_script_name,
        }
    }
}

#[repr(C)]
pub struct NovaCompilerProveOption {
    /// The target filter used to prune the modules to verify. Modules with a name that contains
    /// this string will be part of verification.
    pub target_filter: ByteSliceView,
    /// Internal field indicating that this prover run is for a test.
    pub for_test: bool,
    /// Any options passed to the prover.
    pub options: ByteSliceView,
}

impl From<NovaCompilerProveOption> for Prove {
    fn from(val: NovaCompilerProveOption) -> Self {
        let target_filter = match val.target_filter.read() {
            Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
            None => None,
        };
        let options = match val.options.read() {
            Some(s) => Some(ProverOptions::Options(
                String::from_utf8(s.to_vec())
                    .unwrap()
                    .split(' ')
                    .map(|o| o.to_string())
                    .collect::<Vec<String>>(),
            )),
            None => None,
        };
        Self {
            target_filter,
            for_test: val.for_test,
            options,
        }
    }
}

#[repr(C)]
pub struct NovaCompilerDocgenOption {
    /// The level where we start sectioning. Often markdown sections are rendered with
    /// unnecessary large section fonts, setting this value high reduces the size
    /// set 0 for default
    pub section_level_start: usize, /*Option<usize>*/
    /// Whether to exclude private functions in the generated docs
    pub exclude_private_fun: bool,
    /// Whether to exclude specifications in the generated docs
    pub exclude_specs: bool,
    /// Whether to put specifications in the same section as a declaration or put them all
    /// into an independent section
    pub independent_specs: bool,
    /// Whether to exclude Move implementations
    pub exclude_impl: bool,
    /// Max depth to which sections are displayed in table-of-contents
    /// /set 0 for default
    pub toc_depth: usize, /*Option<usize>*/
    /// Do not use collapsed sections (<details>) for impl and specs
    pub no_collapsed_sections: bool,
    /// In which directory to store output
    pub output_directory: ByteSliceView, /*Option<String>*/
    /// A template for documentation generation. Can be multiple
    /// delimiter: , (comma)
    pub template: ByteSliceView, /*Vec<String>*/
    /// An optional file containing reference definitions. The content of this file will
    /// be added to each generated markdown doc
    pub references_file: ByteSliceView, /*Option<String>*/
    /// Whether to include dependency diagrams in the generated docs
    pub include_dep_diagrams: bool,
    /// Whether to include call diagrams in the generated docs
    pub include_call_diagrams: bool,
    /// If this is being compiled relative to a different place where it will be stored (output directory)
    pub compile_relative_to_output_dir: bool,
}

impl From<NovaCompilerDocgenOption> for Docgen {
    fn from(val: NovaCompilerDocgenOption) -> Self {
        let output_directory = match val.output_directory.read() {
            Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
            None => None,
        };
        let template: Vec<String> = match val.template.read() {
            Some(s) => Vec::from_iter(
                String::from_utf8(s.to_vec())
                    .unwrap()
                    .split(',')
                    .map(String::from),
            ),
            None => vec![],
        };
        let references_file = match val.references_file.read() {
            Some(s) => Some(String::from_utf8(s.to_vec()).unwrap()),
            None => None,
        };
        Self {
            section_level_start: match val.section_level_start {
                0 => None,
                _ => Some(val.section_level_start),
            },
            exclude_private_fun: val.exclude_private_fun,
            exclude_specs: val.exclude_specs,
            independent_specs: val.independent_specs,
            exclude_impl: val.exclude_impl,
            toc_depth: match val.toc_depth {
                0 => None,
                _ => Some(val.toc_depth),
            },
            no_collapsed_sections: val.no_collapsed_sections,
            output_directory,
            template,
            references_file,
            include_dep_diagrams: val.include_dep_diagrams,
            include_call_diagrams: val.include_call_diagrams,
            compile_relative_to_output_dir: val.compile_relative_to_output_dir,
        }
    }
}

/* TODO: revive it when Sandbox problem(unimplementable because of private struct) is solved.
/// cbindgen:prefix-with-name
#[repr(u8)]
#[allow(dead_code)]
#[derive(PartialEq)]
pub enum NovaExperimentalSubcommandType {
    ReadWriteSet = 1,
}

#[repr(C)]
pub struct NovaCompilerExperimentalOption {
    /// Directory storing Move resources, events, and module bytecodes produced by module publishing
    /// and script execution. (default: `storage`)
    storage_dir: ByteSliceView,
    cmd_type: NovaExperimentalSubcommandType,
    rws: ReadWriteSet,
}

impl From<NovaCompilerExperimentalOption> for nova_compiler::Command {
    fn from(val: NovaCompilerExperimentalOption) -> Self {
        let storage_dir = match val.storage_dir.read() {
            Some(s) => Path::new(&String::from_utf8(s.to_vec()).unwrap()).to_path_buf(),
            None => Path::new(DEFAULT_STORAGE_DIR).to_path_buf(),
        };
        return match val.cmd_type {
            NovaExperimentalSubcommandType::ReadWriteSet => nova_compiler::Command::Experimental {
                storage_dir,
                cmd: val.rws.into(),
            },
        };
    }
}

/// cbindgen:prefix-with-name
#[allow(dead_code)]
#[repr(u8)]
#[derive(PartialEq)]
pub enum NovaConcretizeMode {
    /// Show the full concretized access paths read or written (e.g. 0xA/0x1::M::S/f/g)
    Paths = 1,
    /// Show only the concrete resource keys that are read (e.g. 0xA/0x1::M::S)
    Reads = 2,
    /// Show only the concrete resource keys that are written (e.g. 0xA/0x1::M::S)
    Writes = 3,
    /// Do not concretize; show the results from the static analysis
    Dont = 4,
}

#[repr(C)]
/// Perform a read/write set analysis and print the results for
/// `module_file`::`script_name`.
pub struct ReadWriteSet {
    /// Path to .mv file containing module bytecode.
    module_file: ByteSliceView,
    /// A function inside `module_file`.
    fun_name: ByteSliceView,
    /// delimiter: , (comma)
    signers: ByteSliceView,
    /// delimiter: , (comma)
    args: ByteSliceView,
    /// delimiter: , (comma)
    type_args: ByteSliceView,
    concretize: NovaConcretizeMode,
}

impl From<ReadWriteSet> for ExperimentalCommand {
    fn from(val: ReadWriteSet) -> Self {
        let module_file =
            Path::new(&String::from_utf8(val.module_file.read().unwrap().to_vec()).unwrap())
                .to_path_buf();
        let fun_name = String::from_utf8(val.fun_name.read().unwrap().to_vec()).unwrap();

        let signers: Vec<String> = match val.signers.read() {
            Some(s) => Vec::from_iter(
                String::from_utf8(s.to_vec())
                    .unwrap()
                    .split(',')
                    .map(String::from),
            ),
            None => vec![],
        };

        let args = match val.args.read() {
            Some(s) => parse_transaction_arguments(&std::str::from_utf8(s).unwrap()).unwrap(),
            None => vec![],
        };
        let type_args = match val.type_args.read() {
            Some(s) => parse_type_tags(&std::str::from_utf8(s).unwrap()).unwrap(),
            None => vec![],
        };

        let concretize = match val.concretize {
            NovaConcretizeMode::Paths => ConcretizeMode::Paths,
            NovaConcretizeMode::Reads => ConcretizeMode::Reads,
            NovaConcretizeMode::Writes => ConcretizeMode::Writes,
            NovaConcretizeMode::Dont => ConcretizeMode::Dont,
        };

        ExperimentalCommand::ReadWriteSet {
            module_file,
            fun_name,
            signers,
            args,
            type_args,
            concretize,
        }
    }
}

#[repr(C)]
pub struct NovaCompilerSandboxOption {
    /// Directory storing Move resources, events, and module bytecodes produced by module publishing
    /// and script execution. (default: `storage`)
    storage_dir: ByteSliceView,
    cmd_type: NovaSandboxSubcommandType,
    /* TODO: fill subcommands */
}

// TODO: impl From for Sandbox

/// cbindgen:prefix-with-name
#[repr(u8)]
#[allow(dead_code)]
#[derive(PartialEq)]
pub enum NovaSandboxSubcommandType {
    /// Compile the modules in this package and its dependencies and publish the resulting bytecodes in global storage.
    Publish = 1,
    /// Run a Move script that reads/writes resources stored on disk in `storage-dir`.
    /// The script must be defined in the package.
    Run = 2,
    /// Run expected value tests using the given batch file.
    Test = 3,
    /// View Move resources, events files, and modules stored on disk.
    View = 4,
    /// Delete all resources, events, and modules stored on disk under `storage-dir`.
    /// Does *not* delete anything in `src`.
    Clean = 5,
    /// Run well-formedness checks on the `storage-dir` and `install-dir` directories.
    Doctor = 6,
    /// Generate struct layout bindings for the modules stored on disk under `storage-dir`
    // TODO: expand this to generate script bindings, etc.?.
    Generate = 7,
}

#[repr(C)]
pub struct SandboxPublish {
    /// If set, fail when attempting to publish a module that already
    /// exists in global storage.
    no_republish: bool,
    /// By default, code that might cause breaking changes for bytecode
    /// linking or data layout compatibility checks will not be published.
    /// Set this flag to ignore breaking changes checks and publish anyway.
    ignore_breaking_changes: bool,
    /// If set, publish not only the modules in this package but also
    /// modules in all its dependencies.
    with_deps: bool,
    /// If set, all modules at once as a bundle. The default is to publish
    /// modules sequentially.
    bundle: bool,
    /// Manually specify the publishing order of modules.
    override_ordering: ByteSliceView,
}

impl From<SandboxPublish> for SandboxCommand{
    fn from(val: SandboxPublish) -> Self {
        let override_ordering: Option<Vec<String>> = match val.override_ordering.read() {
            Some(s) => Some(Vec::from_iter(
                String::from_utf8(s.to_vec())
                    .unwrap()
                    .split(',')
                    .map(String::from),
            )),
            None => None,
        };
        SandboxCommand::Publish {
            no_republish: val.no_republish,
            ignore_breaking_changes: val.ignore_breaking_changes,
            with_deps: val.with_deps,
            bundle: val.bundle,
            override_ordering: override_ordering
        }
    }
}

#[repr(C)]
pub struct SandboxRun {
    /// Path to .mv file containing either script or module bytecodes. If the file is a module, the
    /// `script_name` parameter must be set.
    script_file: ByteSliceView,
    /// Name of the script function inside `script_file` to call. Should only be set if `script_file`
    /// points to a module.
    script_name: ByteSliceView,
    /// Possibly-empty list of signers for the current transaction (e.g., `account` in
    /// `main(&account: signer)`). Must match the number of signers expected by `script_file`.
    signers: ByteSliceView,
    /// Possibly-empty list of arguments passed to the transaction (e.g., `i` in
    /// `main(i: u64)`). Must match the arguments types expected by `script_file`.
    /// Supported argument types are
    /// bool literals (true, false),
    /// u64 literals (e.g., 10, 58),
    /// address literals (e.g., 0x12, 0x0000000000000000000000000000000f),
    /// hexadecimal strings (e.g., x"0012" will parse as the vector<u8> value [00, 12]), and
    /// ASCII strings (e.g., 'b"hi" will parse as the vector<u8> value [68, 69]).
    args: ByteSliceView,
    /// Possibly-empty list of type arguments passed to the transaction (e.g., `T` in
    /// `main<T>()`). Must match the type arguments kinds expected by `script_file`.
    type_args: ByteSliceView,
    /// Maximum number of gas units to be consumed by execution.
    /// When the budget is exhaused, execution will abort.
    /// By default, no `gas-budget` is specified and gas metering is disabled.
    gas_budget: u64,
    /// If set, the effects of executing `script_file` (i.e., published, updated, and
    /// deleted resources) will NOT be committed to disk.
    dry_run: bool
}

impl From<SandboxRun> for SandboxCommand{
    fn from(val: SandboxRun) -> Self {
        let gas_budget = if val.gas_budget == 0 {None} else {Some(val.gas_budget)};
        let script_file: Option<PathBuf> = val.script_file.into();
        let script_name: Option<String> = val.script_name.into();
        let signers: Option<Vec<String>> = val.signers.into();
        let args: Option<Vec<TransactionArgument>> = val.args.into();
        let type_args: Option<Vec<TypeTag>> = val.type_args.into();
        SandboxCommand::Run{
            script_file: script_file.unwrap(),
            script_name,
            signers: signers.unwrap_or(vec![]),
            args: args.unwrap_or(vec![]),
            type_args: type_args.unwrap_or(vec![]),
            gas_budget, 
            dry_run: val.dry_run,
        }
    }
}

#[repr(C)]
pub struct SandboxTest {
    /// Use an ephemeral directory to serve as the testing workspace.
    /// By default, the directory containing the `args.txt` will be the workspace.
    use_temp_dir: bool,
    /// Show coverage information after tests are done.
    /// By default, coverage will not be tracked nor shown.
    track_cov: bool,
}

impl From<SandboxTest> for SandboxCommand{
    fn from(val: SandboxTest) -> Self {
        SandboxCommand::Test{
            use_temp_dir: val.use_temp_dir,
            track_cov: val.track_cov
        }
    }
}

#[repr(C)]
pub struct SandboxView {
    file: ByteSliceView
}

impl From<SandboxView> for SandboxCommand {
    fn from(val: SandboxView) -> Self {
        SandboxCommand::View {
            file: Option::<PathBuf>::from(val.file).unwrap()
        }
    }
}

#[repr(C)]
pub struct SandboxGenerate_StructLayoutOptions {
    /// Generate layout bindings for this struct.
    struct_: ByteSliceView,
    /// Generate layout bindings for `struct` bound to these type arguments.
    type_args: ByteSliceView,
    /// If set, generate bindings only for the struct passed in.
    /// When unset, generates bindings for the struct and all of its transitive dependencies.
    shallow: bool
}

impl From<SandboxGenerate_StructLayoutOptions> for StructLayoutOptions {
    fn from(val: SandboxGenerate_StructLayoutOptions) -> Self {
        Self {
            struct_: Option::<String>::from(val.struct_),
            type_args: Option::<Vec<TypeTag>>::from(val.type_args),
            shallow: val.shallow
        }
    }
}

#[repr(C)]
pub struct SandboxGenerate_StructLayouts {
    module: ByteSliceView,
    options: SandboxGenerate_StructLayoutOptions
}
*/