use move_deps::move_cli::Move;
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

// TODO: remove CoverageOption and CoverageSummaryOptions below here
//      and re-implement them via union. it's difficult to keep safe but more human readable and more clear than overlapped enums.

/// cbindgen:prefix-with-name
#[repr(u8)] // This makes it so the enum looks like a simple u32 to Go
#[derive(PartialEq)]
pub enum CoverageOption{
    // no 0 for the purpose
    /// Display a coverage summary for all modules in this package
    Summary = 1,
    /// Display coverage information about the module against source code
    Source = 2,
    /// Display coverage information about the module against disassembled bytecode
    Bytecode = 3,
}

// similar with the one from move-cli. we don't union in c but still it's useful anyway.
#[repr(C)] 
pub enum CoverageSummaryOptions {
    /// Display a coverage summary for all modules in this package
    Summary {
        /// Whether function coverage summaries should be displayed
        functions: bool,
        /// Output CSV data of coverage
        output_csv: bool,
    },
    /// Display coverage information about the module against source code
    Source {
        module_name: ByteSliceView,
    },
    /// Display coverage information about the module against disassembled bytecode
    Bytecode {
        module_name: ByteSliceView,
    },
}
