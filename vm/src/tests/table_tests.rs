use crate::{
    message::{EntryFunction, Message, Module, ModuleBundle},
    size_change_set::SizeDelta,
};
use move_deps::{
    move_binary_format::CompiledModule,
    move_core_types::{
        account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
        vm_status::VMStatus,
    },
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use super::mock_tx::{run_transaction, MockTx};
use super::mock_tx::{ExpectedOutput, ExpectedOutputItem};

#[cfg(test)]
impl Module {
    fn create_table_test_data() -> Self {
        let s = Self::new(
            include_bytes!("../../move-test/build/test1/bytecode_modules/TableTestData.mv")
                .to_vec(),
        );
        let _compiled_module = CompiledModule::deserialize(s.code()).unwrap();

        s
    }

    fn get_table_test_data_module_id() -> ModuleId {
        let account_two =
            AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

        ModuleId::new(account_two, Identifier::new("TableTestData").unwrap())
    }
}

#[cfg(test)]
impl EntryFunction {
    fn simple_read_write() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("simple_read_write").unwrap(),
            vec![],
            vec![],
        )
    }
    fn table_len() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_len").unwrap(),
            vec![],
            vec![],
        )
    }
    fn move_table() -> Self {
        let account_two = generate_account("0x2");
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("move_table").unwrap(),
            vec![],
            vec![account_two.to_vec()],
        )
    }
    fn table_of_tables() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_of_tables").unwrap(),
            vec![],
            vec![],
        )
    }

    fn table_borrow_mut() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_borrow_mut").unwrap(),
            vec![],
            vec![],
        )
    }

    fn table_borrow_mut_with_default() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_borrow_mut_with_default").unwrap(),
            vec![],
            vec![],
        )
    }

    fn add_after_remove() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("add_after_remove").unwrap(),
            vec![],
            vec![],
        )
    }

    fn table_borrow_global() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_borrow_global").unwrap(),
            vec![],
            vec![],
        )
    }

    fn table_move_from() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_move_from").unwrap(),
            vec![],
            vec![],
        )
    }
}

fn generate_account(literal: &str) -> AccountAddress {
    AccountAddress::from_hex_literal(literal).expect("account should be created")
}
#[test]
fn test_tables() {
    let account_two = generate_account("0x2");
    let account_three = generate_account("0x3");

    let testcases: Vec<MockTx> = vec![
        MockTx::one(
            // publish module
            Message::new_module(
                vec![1; 32],
                Some(account_two),
                ModuleBundle::from(Module::create_table_test_data()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        MockTx::one(
            // make table and read, write, remove
            Message::new_entry_function(
                vec![2; 32],
                Some(account_two),
                EntryFunction::simple_read_write(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![2, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get table length
            Message::new_entry_function(vec![3; 32], Some(account_two), EntryFunction::table_len()),
            ExpectedOutput(vec![
                ExpectedOutputItem::VMStatusReturn(VMStatus::Executed),
                ExpectedOutputItem::ChangedAccountCount(1),
                ExpectedOutputItem::ResultBytes(vec![3, 0, 0, 0, 0, 0, 0, 0]),
                ExpectedOutputItem::AccountSizeChange(
                    [(account_two, SizeDelta::increasing(102 + 201))].into(),
                    // access_path "0000000000000000000000000000000000000002/1/0x2::TableTestData::S<u64, u64>" => 74
                    // S { address, u64 } => 20 + 8
                    // Total = 102
                    //
                    // Table access path "90abac00da7103273074b8d42c458cb96b309867/2/0100000000000000" => 59
                    // Table data u64 => 8
                    // Per Item = 59 + 8 = 67
                    // Table len 3 => 67 * 3 = 201
                ),
            ]),
        ),
        MockTx::one(
            Message::new_entry_function(
                vec![10; 32],
                Some(account_three),
                EntryFunction::move_table(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 2, Some(vec![])),
        ),
        MockTx::one(
            // tables in table
            Message::new_entry_function(
                vec![4; 32],
                Some(account_two),
                EntryFunction::table_of_tables(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![3, 78, 11, 45])),
        ),
        MockTx::one(
            // borrow mut
            Message::new_entry_function(
                vec![5; 32],
                Some(generate_account("0x3")),
                EntryFunction::table_borrow_mut(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![3, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // borrow mut with default
            Message::new_entry_function(
                vec![6; 32],
                Some(generate_account("0x4")),
                EntryFunction::table_borrow_mut_with_default(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![232, 3, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // add after remove contents
            Message::new_entry_function(
                vec![7; 32],
                Some(generate_account("0x5")),
                EntryFunction::add_after_remove(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![55, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // borrow global
            Message::new_entry_function(
                vec![8; 32],
                Some(generate_account("0x5")),
                EntryFunction::table_borrow_global(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![55, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // tables in table
            Message::new_entry_function(
                vec![9; 32],
                Some(generate_account("0x7")),
                EntryFunction::table_move_from(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![245, 3, 0, 0, 0, 0, 0, 0])),
        ),
    ];

    run_transaction(testcases);
}
