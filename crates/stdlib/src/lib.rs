use std::collections::BTreeMap;
use std::io::Write;
use move_deps::move_compiler::Flags;
use tempfile::{NamedTempFile, TempPath};

use move_deps::move_binary_format::CompiledModule;
use move_deps::move_compiler::shared::NumericalAddress;
use move_deps::move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler};

fn move_stdlib_files() -> Vec<TempPath> {
    let files: Vec<&str> = vec![
        include_str!("move_stdlib/sources/ascii.move"),
        include_str!("move_stdlib/sources/bcs.move"),
        include_str!("move_stdlib/sources/bit_vector.move"),
        include_str!("move_stdlib/sources/error.move"),
        include_str!("move_stdlib/sources/fixed_point32.move"),
        include_str!("move_stdlib/sources/hash.move"),
        include_str!("move_stdlib/sources/option.move"),
        include_str!("move_stdlib/sources/signer.move"),
        include_str!("move_stdlib/sources/string.move"),
        include_str!("move_stdlib/sources/vector.move"),

        #[cfg(feature = "testing")]
        include_str!("move_stdlib/sources/unit_test.move"),
    ];

    files
        .iter()
        .map(|contents| {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();

            file.into_temp_path()
        })
        .collect()
}

fn move_nursery_files() -> Vec<TempPath> {
    let files: Vec<&str> = vec![
        include_str!("move_nursery/sources/acl.move"),
        include_str!("move_nursery/sources/capability.move"),
        include_str!("move_nursery/sources/compare.move"),
        include_str!("move_nursery/sources/debug.move"),
        include_str!("move_nursery/sources/errors.move"),
        include_str!("move_nursery/sources/event.move"),
        include_str!("move_nursery/sources/guid.move"),
        include_str!("move_nursery/sources/offer.move"),
        include_str!("move_nursery/sources/role.move"),
        include_str!("move_nursery/sources/vault.move"),
    ];

    files
        .iter()
        .map(|contents| {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();

            file.into_temp_path()
        })
        .collect()
}

fn nova_stdlib_files() -> Vec<TempPath> {
    let files: Vec<&str> = vec![
        include_str!("nova_stdlib/sources/account.move"),
        include_str!("nova_stdlib/sources/block.move"),
        include_str!("nova_stdlib/sources/code.move"),
        include_str!("nova_stdlib/sources/coin.move"),
        include_str!("nova_stdlib/sources/comparator.move"),
        include_str!("nova_stdlib/sources/simple_map.move"),
        include_str!("nova_stdlib/sources/table_with_length.move"),
        include_str!("nova_stdlib/sources/table.move"),
        include_str!("nova_stdlib/sources/type_info.move"),
        include_str!("nova_stdlib/sources/util.move"),
    ];

    files
        .iter()
        .map(|contents| {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", contents).unwrap();

            file.into_temp_path()
        })
        .collect()
}

pub fn compile_move_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = move_stdlib_files();
    let deps_files = vec![];
    let name_address_map = named_addresses();
    compile_modules(src_files, deps_files, name_address_map)
}

pub fn compile_move_nursery_modules() -> Vec<CompiledModule> {
    let src_files = move_nursery_files();
    let deps_files = move_stdlib_files();
    let name_address_map = named_addresses();
    compile_modules(src_files, deps_files, name_address_map)
}

pub fn compile_nova_stdlib_modules() -> Vec<CompiledModule> {
    let src_files = nova_stdlib_files();
    let deps_files = move_stdlib_files()
        .into_iter()
        .chain(move_nursery_files())
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
    ).set_flags(Flags::empty())
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

fn named_addresses() -> BTreeMap<String, NumericalAddress> {
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
