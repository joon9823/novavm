use move_deps::move_core_types::vm_status::VMStatus;

use nova_types::{message::Message, module::ModuleBundle, size_delta::SizeDelta};

use crate::test_utils::generate_account;
use crate::test_utils::mock_tx::{run_transaction, ExpectedOutput, ExpectedOutputItem, MockTx};
use crate::test_utils::{entry_function, module};

#[cfg(test)]
#[test]
fn test_tables() {
    type Item = ExpectedOutputItem;

    let account_two = generate_account("0x2");
    let account_three = generate_account("0x3");

    let testcases: Vec<MockTx> = vec![
        MockTx::one(
            // publish module
            Message::new_module(
                vec![1; 32],
                Some(account_two),
                ModuleBundle::from(module::create_table_test_data()),
            ),
            ExpectedOutput::new(VMStatus::Executed, None),
        ),
        MockTx::one(
            // make table and read, write, remove
            Message::new_entry_function(
                vec![2; 32],
                Some(account_two),
                entry_function::simple_read_write(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![2, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get table length
            Message::new_entry_function(
                vec![3; 32],
                Some(account_two),
                entry_function::table_len(),
            ),
            ExpectedOutput(vec![
                Item::VMStatusReturn(VMStatus::Executed),
                Item::ResultBytes(vec![3, 0, 0, 0, 0, 0, 0, 0]),
                Item::SizeChange(
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
                vec![4; 32],
                Some(account_three),
                entry_function::move_table(),
            ),
            ExpectedOutput(vec![
                Item::VMStatusReturn(VMStatus::Executed),
                Item::SizeChange(
                    [
                        (account_two, SizeDelta::decreasing(303)),
                        (account_three, SizeDelta::increasing(130 + 201 + 111)),
                        // access_path "0000000000000000000000000000000000000003/1/0x2::TableTestData::S<address, 0x1::table::Table<u64, u64>>" => 102
                        // S { address, u64 } => 28
                        // Total = 130

                        // previous Table = 201

                        // new Table
                        // access path => 83
                        // data Table => 28
                        // 1 row => 111
                    ]
                    .into(),
                ),
            ]),
        ),
        MockTx::one(
            // tables in table
            Message::new_entry_function(
                vec![5; 32],
                Some(account_two),
                entry_function::table_of_tables(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![3, 78, 11, 45])),
        ),
        MockTx::one(
            // borrow mut
            Message::new_entry_function(
                vec![6; 32],
                Some(generate_account("0x3")),
                entry_function::table_borrow_mut(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![3, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // borrow mut with default
            Message::new_entry_function(
                vec![7; 32],
                Some(generate_account("0x4")),
                entry_function::table_borrow_mut_with_default(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![232, 3, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // add after remove contents
            Message::new_entry_function(
                vec![8; 32],
                Some(generate_account("0x5")),
                entry_function::add_after_remove(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![55, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // borrow global
            Message::new_entry_function(
                vec![9; 32],
                Some(generate_account("0x5")),
                entry_function::table_borrow_global(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![55, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // tables in table
            Message::new_entry_function(
                vec![10; 32],
                Some(generate_account("0x7")),
                entry_function::table_move_from(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![245, 3, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // remove table
            Message::new_entry_function(
                vec![11; 32],
                Some(generate_account("0x7")),
                entry_function::table_remove(),
            ),
            ExpectedOutput(vec![
                Item::VMStatusReturn(VMStatus::Executed),
                Item::SizeChange([(generate_account("0x7"), SizeDelta::decreasing(236))].into()),
            ]),
        ),
    ];

    run_transaction(testcases);
}
