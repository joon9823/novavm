
use crate::vm::{
    message::{Message, Module},
};
use move_deps::move_core_types::{
    account_address::AccountAddress,
};
use move_deps::move_stdlib;
use move_deps::move_binary_format::CompiledModule;
use move_deps::move_compiler::{compiled_unit::AnnotatedCompiledUnit, Compiler};

pub fn compile_move_modules() -> Vec<CompiledModule> {
    let mut src_files = move_stdlib::move_stdlib_files();
    // src_files.append(&mut move_stdlib::move_nursery_files());
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


#[test]
fn test_publish_move_stdlib(){
    let messages = publish_move_stdlib();
    assert_eq!(messages.len(), 10);
}

#[test]
fn test_publish_move_stdlib_nursery(){
    let messages = publish_move_stdlib_nursery();
    assert_eq!(messages.len(), 11);
}