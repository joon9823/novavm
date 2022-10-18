use move_deps::move_core_types::{language_storage::TypeTag, parser::parse_struct_tag};
use nova_types::script::Script;

pub fn mint_200() -> Script {
    Script::new(
        include_bytes!("../../../move-test/build/test1/bytecode_scripts/main.mv").to_vec(),
        vec![
            TypeTag::Struct(parse_struct_tag("0x1::BasicCoin::Nova").unwrap()),
            TypeTag::Bool,
        ],
        vec![],
    )
}
