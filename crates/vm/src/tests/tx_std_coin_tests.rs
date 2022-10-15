use crate::message::{EntryFunction, Message, Module, ModuleBundle};

use move_deps::move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::parse_struct_tag,
    vm_status::VMStatus,
};

use crate::test_utils::mock_tx::{run_transaction, ExpectedOutput, MockTx};

#[cfg(test)]
impl Module {
    fn create_std_coin() -> Self {
        Self::new(
            include_bytes!("../../../move-test/build/test1/bytecode_modules/StdCoin.mv").to_vec(),
        )
    }
}

#[cfg(test)]
impl EntryFunction {}

#[test]
fn test_std_coin() {
    let account_one =
        AccountAddress::from_hex_literal("0x1").expect("0x1 account should be created");
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");
    let account_three =
        AccountAddress::from_hex_literal("0x3").expect("0x3 account should be created");

    let testcases: Vec<MockTx> = vec![
        MockTx::one(
            // publish
            Message::new_module(
                vec![1; 32],
                Some(account_two),
                ModuleBundle::from(Module::create_std_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        MockTx::one(
            // initialize coin and capabilities
            Message::new_entry_function(
                vec![2; 32],
                Some(account_two),
                EntryFunction::new(
                    ModuleId::new(account_two, Identifier::new("StdCoin").unwrap()),
                    Identifier::new("init").unwrap(),
                    vec![],
                    vec![],
                ),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
        MockTx::one(
            // register 0x3 to receive coin
            Message::new_entry_function(
                vec![3; 32],
                Some(account_three),
                EntryFunction::new(
                    ModuleId::new(account_two, Identifier::new("StdCoin").unwrap()),
                    Identifier::new("register").unwrap(),
                    vec![],
                    vec![],
                ),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
        MockTx::one(
            // mint coin to 0x3
            Message::new_entry_function(
                vec![4; 32],
                Some(account_two),
                EntryFunction::new(
                    ModuleId::new(account_two, Identifier::new("StdCoin").unwrap()),
                    Identifier::new("mint").unwrap(),
                    vec![],
                    vec![account_three.to_vec(), 100u64.to_le_bytes().to_vec()],
                ),
            ),
            ExpectedOutput::new(VMStatus::Executed, 2, Some(vec![])),
        ),
        MockTx::one(
            // get 0x3's balance
            Message::new_entry_function(
                vec![5; 32],
                None,
                EntryFunction::new(
                    ModuleId::new(account_one, Identifier::new("coin").unwrap()),
                    Identifier::new("balance").unwrap(),
                    vec![TypeTag::Struct(
                        parse_struct_tag("0x2::StdCoin::Std").unwrap(),
                    )],
                    vec![account_three.to_vec()],
                ),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![100, 0, 0, 0, 0, 0, 0, 0])),
        ),
    ];

    run_transaction(testcases);
}
