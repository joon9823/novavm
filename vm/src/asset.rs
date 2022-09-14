use move_deps::move_stdlib;
use move_deps::move_binary_format::CompiledModule;
use move_deps::move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler};
use move_deps::move_compiler::shared::NumericalAddress;
use std::{collections::BTreeMap, path::PathBuf};
use move_deps::move_command_line_common::files::{extension_equals,find_filenames};

pub fn compile_move_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_stdlib::move_stdlib_files();
    let (_files, compiled_units) = Compiler::from_files(
        src_files,
        vec![],
        move_stdlib::move_stdlib_named_addresses(),
    )
    .build_and_report()
    .expect("Error compiling...");
    compiled_units
        .into_iter()
        .map(|unit| match unit {
            AnnotatedCompiledUnit::Module(annot_unit) => annot_unit.named_module.module,
            AnnotatedCompiledUnit::Script(_) => {
                panic!("Expected a module but received a script")
            }
        })
        .collect()
}

pub fn compile_move_nursery_modules() -> Vec<CompiledModule> {
    let src_files = move_stdlib::move_nursery_files();
    let deps_files = move_stdlib::move_stdlib_files();
    let (_files, compiled_units) = Compiler::from_files(
        src_files,
        deps_files,
        move_stdlib::move_stdlib_named_addresses(),
    )
    .build_and_report()
    .expect("Error compiling...");
    compiled_units
        .into_iter()
        .map(|unit| match unit {
            AnnotatedCompiledUnit::Module(annot_unit) => annot_unit.named_module.module,
            AnnotatedCompiledUnit::Script(_) => {
                panic!("Expected a module but received a script")
            }
        })
        .collect()
}

pub fn compile_kernel_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_kernel_stdlib_files();
    let (_files, compiled_units) = Compiler::from_files(
        src_files,
        vec![],
        kernel_stdlib_named_addresses(),
    )
    .build_and_report()
    .expect("Error compiling...");
    compiled_units
        .into_iter()
        .map(|unit| match unit {
            AnnotatedCompiledUnit::Module(annot_unit) => annot_unit.named_module.module,
            AnnotatedCompiledUnit::Script(_) => {
                panic!("Expected a module but received a script")
            }
        })
        .collect()
}

fn kernel_stdlib_named_addresses() -> BTreeMap<String, NumericalAddress> {
    let mapping = [("kernel_std", "0x1")];
    mapping
        .iter()
        .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap()))
        .collect()
}

fn move_kernel_stdlib_files() -> Vec<String> {
    // FIXME : make this configurable or cosnt var
    let path = std::path::PathBuf::from("src/natives/sources");
    find_filenames(&[path], |p| extension_equals(p, "move")).unwrap()
}
