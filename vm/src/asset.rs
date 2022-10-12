use move_deps::move_binary_format::CompiledModule;
use move_deps::move_compiler::shared::NumericalAddress;
use move_deps::move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler};

use std::collections::BTreeMap;
use tempfile::TempPath;

use crate::move_modules;

pub fn compile_move_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_modules::move_stdlib_files();
    let deps_files = vec![];
    let name_address_map = named_addresses();
    compile_modules(src_files, deps_files, name_address_map)
}

pub fn compile_move_nursery_modules() -> Vec<CompiledModule> {
    let src_files = move_modules::move_nursery_files();
    let deps_files = move_modules::move_stdlib_files();
    let name_address_map = named_addresses();
    compile_modules(src_files, deps_files, name_address_map)
}

pub fn compile_nova_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_modules::nova_stdlib_files();
    let deps_files = move_modules::move_stdlib_files()
        .into_iter()
        .chain(move_modules::move_nursery_files())
        .collect();
    let name_address_map = named_addresses();

    compile_modules(src_files, deps_files, name_address_map)
}

fn compile_modules(
    src_files: Vec<TempPath>,
    deps_files: Vec<TempPath>,
    name_address_map: BTreeMap<String, NumericalAddress>,
) -> Vec<CompiledModule> {
    let (_files, compiled_units) = Compiler::from_files(
        src_files
            .iter()
            .map(|f| f.as_os_str().to_str().unwrap())
            .collect(),
        deps_files
            .iter()
            .map(|f| f.as_os_str().to_str().unwrap())
            .collect(),
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

pub fn named_addresses() -> BTreeMap<String, NumericalAddress> {
    let mapping = [("nova_std", "0x1"), ("std", "0x1")];
    mapping
        .iter()
        .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap()))
        .collect()
}

#[test]
fn test_compile_nova_stdlib_modules() {
    let modules = compile_nova_stdlib_modules();
    assert!(!modules.is_empty()); // TODO: check that all modules are compiled
}
