use move_deps::move_core_types::{
    account_address::AccountAddress,
    vm_status::{StatusCode, VMStatus},
};

use nova_types::{message::Message, module::ModuleBundle};

use crate::test_utils::mock_tx::{run_transaction, ExpectedOutput, MockTx};
use crate::test_utils::{entry_function, module, script};

#[test]
fn test_abandon_tx_loader_cache() {
    let testcases: Vec<MockTx> = vec![
        MockTx::new_skip_commit(vec![
            (
                // upgrade module
                Message::new_module(
                    vec![1; 32],
                    Some(AccountAddress::ONE),
                    ModuleBundle::from(module::create_basic_coin()),
                ),
                ExpectedOutput::new(VMStatus::Executed, None),
            ),
            (
                // get 123
                Message::new_entry_function(
                    vec![2; 32],
                    Some(AccountAddress::ZERO),
                    entry_function::number(),
                ),
                ExpectedOutput::new(VMStatus::Executed, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
            ),
        ]),
        MockTx::one(
            // should fail since module has been disposed
            Message::new_entry_function(
                vec![3; 32],
                Some(AccountAddress::ZERO),
                entry_function::number(),
            ),
            ExpectedOutput::new(VMStatus::Error(StatusCode::LINKER_ERROR), None),
        ),
    ];

    run_transaction(testcases);
}

#[test]
fn test_module_upgrade_loader_cache() {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

    let testcases: Vec<MockTx> = vec![

        MockTx::one(
            // module have only one function that get number 123
            Message::new_module(
                vec![1; 32],
                Some(AccountAddress::ONE),
                ModuleBundle::singleton(hex::decode("a11ceb0b0500000006010002030205050703070a11081b140c2f1000000001000100000103094261736963436f696e066e756d6265720000000000000000000000000000000000000001000104000002067b000000000000000200").expect("ms")),
            ),
            ExpectedOutput::new(VMStatus::Executed, None)
        ),
        MockTx::one(
            // by calling this, loader caches module
            Message::new_entry_function(vec![2; 32],Some(AccountAddress::ZERO), entry_function::number()),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // upgrade module
            Message::new_module(
                vec![3; 32],
                Some(AccountAddress::ONE),
                ModuleBundle::from(module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, None),
        ),
        MockTx::one(
            // mint with entry function
            // should work with new module
            Message::new_entry_function(vec![4; 32],Some(account_two), entry_function::mint(100)),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![])),
        ),
    ];

    run_transaction(testcases);
}

#[test]
fn test_simple_trasaction() {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

    let testcases: Vec<MockTx> = vec![
        MockTx::one(
            // publish module
            Message::new_module(
                vec![1; 32],
                Some(AccountAddress::ONE),
                ModuleBundle::from(module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, None),
        ),
        MockTx::one(
            // mint with script
            Message::new_script(vec![2; 32], Some(AccountAddress::ONE), script::mint_200()),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![])),
        ),
        MockTx::one(
            // mint with entry function
            Message::new_entry_function(vec![3; 32], Some(account_two), entry_function::mint(100)),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![])),
        ),
        MockTx::one(
            // linker error
            Message::new_entry_function(
                vec![4; 32],
                Some(AccountAddress::ZERO),
                entry_function::mint_with_wrong_module_address(100),
            ),
            ExpectedOutput::new(VMStatus::Error(StatusCode::LINKER_ERROR), None),
        ),
        MockTx::one(
            // get 123
            Message::new_entry_function(
                vec![5; 32],
                Some(AccountAddress::ZERO),
                entry_function::number(),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get coin amount for 0x1
            Message::new_entry_function(
                vec![6; 32],
                Some(AccountAddress::ZERO),
                entry_function::get(AccountAddress::ONE),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![200, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get coin amount for 0x0
            Message::new_entry_function(
                vec![7; 32],
                Some(AccountAddress::ZERO),
                entry_function::get(account_two),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![100, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get Coin structure
            Message::new_entry_function(
                vec![8; 32],
                Some(AccountAddress::ZERO),
                entry_function::get_coin_struct(account_two),
            ),
            ExpectedOutput::new(VMStatus::Executed, Some(vec![100, 0, 0, 0, 0, 0, 0, 0, 1])),
        ),
    ];

    run_transaction(testcases);
}
