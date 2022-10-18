use move_deps::{
    move_binary_format::CompiledModule,
    move_core_types::{
        account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    },
};
use nova_types::module::Module;

pub fn create_std_coin() -> Module {
    Module::new(
        include_bytes!("../../../move-test/build/test1/bytecode_modules/StdCoin.mv").to_vec(),
    )
}

pub fn create_table_test_data() -> Module {
    let s = Module::new(
        include_bytes!("../../../move-test/build/test1/bytecode_modules/TableTestData.mv").to_vec(),
    );
    let _compiled_module = CompiledModule::deserialize(s.code()).unwrap();

    s
}

pub fn create_basic_coin() -> Module {
    let s = Module::new(
        include_bytes!("../../../move-test/build/test1/bytecode_modules/BasicCoin.mv").to_vec(),
    );
    let _compiled_module = CompiledModule::deserialize(s.code()).unwrap();

    s
}

pub fn get_basic_coin_module_id() -> ModuleId {
    ModuleId::new(AccountAddress::ONE, Identifier::new("BasicCoin").unwrap())
}

pub fn get_table_test_data_module_id() -> ModuleId {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

    ModuleId::new(account_two, Identifier::new("TableTestData").unwrap())
}
