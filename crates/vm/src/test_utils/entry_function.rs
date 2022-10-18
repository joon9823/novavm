use move_deps::move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_struct_tag,
};
use nova_types::entry_function::EntryFunction;

use super::{generate_account, module};

pub fn simple_read_write() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("simple_read_write").unwrap(),
        vec![],
        vec![],
    )
}
pub fn table_len() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_len").unwrap(),
        vec![],
        vec![],
    )
}

pub fn move_table() -> EntryFunction {
    let account_two = generate_account("0x2");
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("move_table").unwrap(),
        vec![],
        vec![account_two.to_vec()],
    )
}

pub fn table_of_tables() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_of_tables").unwrap(),
        vec![],
        vec![],
    )
}

pub fn table_borrow_mut() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_borrow_mut").unwrap(),
        vec![],
        vec![],
    )
}

pub fn table_borrow_mut_with_default() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_borrow_mut_with_default").unwrap(),
        vec![],
        vec![],
    )
}

pub fn add_after_remove() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("add_after_remove").unwrap(),
        vec![],
        vec![],
    )
}

pub fn table_borrow_global() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_borrow_global").unwrap(),
        vec![],
        vec![],
    )
}

pub fn table_move_from() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_move_from").unwrap(),
        vec![],
        vec![],
    )
}

pub fn table_remove() -> EntryFunction {
    EntryFunction::new(
        module::get_table_test_data_module_id(),
        Identifier::new("table_remove").unwrap(),
        vec![],
        vec![],
    )
}

pub fn mint(amount: u64) -> EntryFunction {
    EntryFunction::new(
        module::get_basic_coin_module_id(),
        Identifier::new("mint").unwrap(),
        vec![TypeTag::Struct(
            parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
        )],
        vec![amount.to_le_bytes().to_vec()],
    )
}

pub fn mint_with_wrong_module_address(amount: u64) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(AccountAddress::ZERO, Identifier::new("BasicCoin").unwrap()),
        Identifier::new("mint").unwrap(),
        vec![TypeTag::Struct(
            parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
        )],
        vec![amount.to_le_bytes().to_vec()],
    )
}

pub fn number() -> EntryFunction {
    EntryFunction::new(
        module::get_basic_coin_module_id(),
        Identifier::new("number").unwrap(),
        vec![],
        vec![],
    )
}

pub fn get(addr: AccountAddress) -> EntryFunction {
    EntryFunction::new(
        module::get_basic_coin_module_id(),
        Identifier::new("get").unwrap(),
        vec![TypeTag::Struct(
            parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
        )],
        vec![addr.to_vec()],
    )
}

pub fn get_coin_struct(addr: AccountAddress) -> EntryFunction {
    EntryFunction::new(
        module::get_basic_coin_module_id(),
        Identifier::new("get_coin").unwrap(),
        vec![TypeTag::Struct(
            parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
        )],
        vec![addr.to_vec()],
    )
}
