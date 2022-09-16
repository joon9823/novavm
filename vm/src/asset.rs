use move_deps::move_stdlib;
use move_deps::move_binary_format::CompiledModule;
use move_deps::move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler};
use move_deps::move_compiler::shared::NumericalAddress;
use std::env;
use std::{collections::BTreeMap, path::PathBuf};
use move_deps::move_command_line_common::files::{extension_equals,find_filenames};

pub fn compile_move_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_stdlib::move_stdlib_files();
    let deps_files = vec![];
    let name_address_map = move_stdlib::move_stdlib_named_addresses();
    compile_modules(src_files, deps_files, name_address_map)
}

pub fn compile_move_nursery_modules() -> Vec<CompiledModule> {
    let src_files = move_stdlib::move_nursery_files();
    let deps_files = move_stdlib::move_stdlib_files();
    let name_address_map = move_stdlib::move_stdlib_named_addresses();
    compile_modules(src_files, deps_files, name_address_map)
}

pub fn compile_kernel_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_kernel_stdlib_files();
    let deps_files = vec![];
    let mapping = [("kernel_std", "0x1")];
    let name_address_map = mapping
        .iter()
        .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap()))
        .collect();

    compile_modules(src_files, deps_files, name_address_map)
}

fn compile_modules(src_files : Vec<String>, deps_files :Vec<String>, name_address_map : BTreeMap<String, NumericalAddress> ) -> Vec<CompiledModule> {
    let (_files, compiled_units) = Compiler::from_files(
        src_files,
        deps_files,
        name_address_map,
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

const MODULES_DIR: &str = "src/kernel_stdlib/sources";

fn move_kernel_stdlib_files() -> Vec<String> {
    let path = path_in_crate(MODULES_DIR);
    find_filenames(&[path], |p| extension_equals(p, "move")).unwrap()
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}
