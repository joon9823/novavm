use crate::{
    message::{EntryFunction, Message, Module, ModuleBundle, Script},
};

use move_deps::{
    move_binary_format::CompiledModule,
    move_core_types::{
        account_address::AccountAddress,
        identifier::Identifier,
        language_storage::{ModuleId, TypeTag},
        parser::parse_struct_tag,
        vm_status::{StatusCode, VMStatus},
    },
};

use super::mock_tx::{MockTx, run_transaction};
use super::{mock_tx::ExpectedOutput};

#[cfg(test)]
impl Module {
    fn create_table_test_data() -> Self {
        let s = Self::new(
            include_bytes!("../../move-test/build/test1/bytecode_modules/TableTestData.mv").to_vec(),
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
    fn table_of_tables() -> Self {
        Self::new(
            Module::get_table_test_data_module_id(),
            Identifier::new("table_of_tables").unwrap(),
            vec![],
            vec![],
        )
    }
}


#[test]
fn test_tables() {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

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
            Message::new_entry_function(
                vec![3; 32],
                Some(account_two),
                EntryFunction::table_len(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![3, 0, 0, 0, 0, 0, 0, 0])),
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
    ];

    run_transaction(testcases);
}
