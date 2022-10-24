use std::env;

use std::fs::File;

use serde_reflection::{Tracer, TracerConfig};
use serde_generate::{Encoding, golang::CodeGenerator, CodeGeneratorConfig};

use move_deps::move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, ResourceKey, StructTag, TypeTag},
};
use nova_types::{entry_function::EntryFunction, module::ModuleBundle, script::Script };
#[path = "src/memory.rs"]
mod memory;
#[path = "src/size_delta.rs"]
mod size_delta;
#[path = "src/move_api/mod.rs"]
mod move_api;
#[path = "src/error/mod.rs"]
mod error;
#[path = "src/event.rs"]
mod event;

#[path = "src/db.rs"]
mod db;
use db::Db;

#[path = "src/storage.rs"]
mod storage;
use storage::GoStorage;

#[path = "src/result.rs"]
mod result;


fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");

    let mut tracer = Tracer::new(TracerConfig::default());
    tracer.trace_simple_type::<Identifier>().unwrap();
    tracer.trace_simple_type::<StructTag>().unwrap();
    tracer.trace_simple_type::<TypeTag>().unwrap();
    tracer.trace_simple_type::<ModuleId>().unwrap();
    tracer.trace_simple_type::<ResourceKey>().unwrap();
    tracer.trace_simple_type::<AccountAddress>().unwrap();
    tracer.trace_simple_type::<size_delta::SizeDelta>().unwrap();
    tracer.trace_simple_type::<result::ExecutionResult>().unwrap();
    tracer.trace_simple_type::<EntryFunction>().unwrap();
    tracer.trace_simple_type::<ModuleBundle>().unwrap();
    tracer.trace_simple_type::<Script>().unwrap();

    let registry = tracer.registry().unwrap();

    let buffer = File::create("../types/bcs.go").unwrap();

    // Create class definitions in Go
    let config = CodeGeneratorConfig::new("types".to_string())
        .with_encodings(vec![Encoding::Bcs]);
    let generator = CodeGenerator::new(&config);
    generator.output(&mut &buffer, &registry).unwrap();
}
