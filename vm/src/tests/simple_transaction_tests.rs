use crate::{
    gas::Gas,
    message::{EntryFunction, Message, Module, ModuleBundle, Script},
    nova_vm::NovaVM,
    storage::data_view_resolver::DataViewResolver,
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

use super::mock_tx::MockTx;
use super::{mock_chain::MockChain, mock_tx::ExpectedOutput};

#[cfg(test)]
impl Module {
    fn create_basic_coin() -> Self {
        let s = Self::new(
            include_bytes!("../../move-test/build/test1/bytecode_modules/BasicCoin.mv").to_vec(),
        );
        let _compiled_module = CompiledModule::deserialize(s.code()).unwrap();

        s
    }

    fn get_basic_coin_module_id() -> ModuleId {
        ModuleId::new(AccountAddress::ONE, Identifier::new("BasicCoin").unwrap())
    }
}

#[cfg(test)]
impl EntryFunction {
    fn mint(amount: u64) -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("mint").unwrap(),
            vec![TypeTag::Struct(
                parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
            )],
            vec![amount.to_le_bytes().to_vec()],
        )
    }

    fn mint_with_wrong_module_address(amount: u64) -> Self {
        Self::new(
            ModuleId::new(AccountAddress::ZERO, Identifier::new("BasicCoin").unwrap()),
            Identifier::new("mint").unwrap(),
            vec![TypeTag::Struct(
                parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
            )],
            vec![amount.to_le_bytes().to_vec()],
        )
    }

    fn number() -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("number").unwrap(),
            vec![],
            vec![],
        )
    }

    fn get(addr: AccountAddress) -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("get").unwrap(),
            vec![TypeTag::Struct(
                parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
            )],
            vec![addr.to_vec()],
        )
    }

    fn get_coin_struct(addr: AccountAddress) -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("get_coin").unwrap(),
            vec![TypeTag::Struct(
                parse_struct_tag("0x1::BasicCoin::Nova").unwrap(),
            )],
            vec![addr.to_vec()],
        )
    }
}

#[cfg(test)]
impl Script {
    fn mint_200() -> Self {
        Self::new(
            include_bytes!("../../move-test/build/test1/bytecode_scripts/main.mv").to_vec(),
            vec![
                TypeTag::Struct(parse_struct_tag("0x1::BasicCoin::Nova").unwrap()),
                TypeTag::Bool,
            ],
            vec![],
        )
    }
}

fn run_transaction(testcases: Vec<MockTx>) {
    let mut chain = MockChain::new();
    let mut vm = NovaVM::new();

    // publish move_stdlib and move_nursery and nova_stdlib modules
    // let mut modules = compile_move_stdlib_modules();
    // modules.append(&mut compile_move_nursery_modules());
    // modules.append(&mut compile_nova_stdlib_modules());

    // let module_bundle = ModuleBundle::new(
    //     modules
    //         .iter()
    //         .map(|m| {
    //             let mut mod_blob = vec![];
    //             m.serialize(&mut mod_blob)
    //                 .expect("Module serialization error");
    //             mod_blob
    //         })
    //         .collect(),
    // );

    let mut state = chain.create_state();
    let resolver = DataViewResolver::new(&state);
    let (status, output, _) = vm.initialize(&resolver, None).expect("Module must load");
    assert!(status == VMStatus::Executed);
    state.push_write_set(output.change_set().clone());
    chain.commit(state);

    let gas_limit = Gas::new(100_000u64);
    for MockTx {
        msg_tests,
        should_commit,
    } in testcases
    {
        let mut state = chain.create_state();

        for (msg, exp_output) in msg_tests {
            let resolver = DataViewResolver::new(&state);
            let (status, output, result) = vm
                .execute_message(msg, &resolver, gas_limit)
                .expect("nova vm failure");

            println!("gas used: {}", output.gas_used());
            println!("got:{}, exp:{}", status, exp_output.vm_status());
            assert!(status == *exp_output.vm_status());
            assert!(output.change_set().accounts().len() == exp_output.changed_accounts());

            let result_bytes =
                result.map(|r| r.return_values.first().map_or(vec![], |m| m.0.clone()));
            assert!(result_bytes == *exp_output.result_bytes());

            if status != VMStatus::Executed {
                continue;
            }
            // apply output into state
            state.push_write_set(output.change_set().clone());
        }

        if should_commit {
            chain.commit(state);
        }
    }
}

#[test]
fn test_abandon_tx_loader_cache() {
    let testcases: Vec<MockTx> = vec![
        MockTx::new_skip_commit(vec![
            (
                // upgrade module
                Message::new_module(
                    vec![1; 32],
                    Some(AccountAddress::ONE),
                    ModuleBundle::from(Module::create_basic_coin()),
                ),
                ExpectedOutput::new(VMStatus::Executed, 1, None),
            ),
            (
                // get 123
                Message::new_entry_function(
                    vec![2; 32],
                    Some(AccountAddress::ZERO),
                    EntryFunction::number(),
                ),
                ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
            ),
        ]),
        MockTx::one(
            // should fail since module has been disposed
            Message::new_entry_function(
                vec![3; 32],
                Some(AccountAddress::ZERO),
                EntryFunction::number(),
            ),
            ExpectedOutput::new(VMStatus::Error(StatusCode::LINKER_ERROR), 0, None),
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
            ExpectedOutput::new(VMStatus::Executed, 1, None)
        ),
        MockTx::one(
            // by calling this, loader caches module
            Message::new_entry_function(vec![2; 32],Some(AccountAddress::ZERO), EntryFunction::number()),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // upgrade module
            Message::new_module(
                vec![3; 32],
                Some(AccountAddress::ONE),
                ModuleBundle::from(Module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        MockTx::one(
            // mint with entry function
            // should work with new module
            Message::new_entry_function(vec![4; 32],Some(account_two), EntryFunction::mint(100)),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
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
                ModuleBundle::from(Module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        MockTx::one(
            // mint with script
            Message::new_script(vec![2; 32], Some(AccountAddress::ONE), Script::mint_200()),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
        MockTx::one(
            // mint with entry function
            Message::new_entry_function(vec![3; 32], Some(account_two), EntryFunction::mint(100)),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
        MockTx::one(
            // linker error
            Message::new_entry_function(
                vec![4; 32],
                Some(AccountAddress::ZERO),
                EntryFunction::mint_with_wrong_module_address(100),
            ),
            ExpectedOutput::new(VMStatus::Error(StatusCode::LINKER_ERROR), 0, None),
        ),
        MockTx::one(
            // get 123
            Message::new_entry_function(
                vec![5; 32],
                Some(AccountAddress::ZERO),
                EntryFunction::number(),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get coin amount for 0x1
            Message::new_entry_function(
                vec![6; 32],
                Some(AccountAddress::ZERO),
                EntryFunction::get(AccountAddress::ONE),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![200, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get coin amount for 0x0
            Message::new_entry_function(
                vec![7; 32],
                Some(AccountAddress::ZERO),
                EntryFunction::get(account_two),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![100, 0, 0, 0, 0, 0, 0, 0])),
        ),
        MockTx::one(
            // get Coin structure
            Message::new_entry_function(
                vec![8; 32],
                Some(AccountAddress::ZERO),
                EntryFunction::get_coin_struct(account_two),
            ),
            ExpectedOutput::new(
                VMStatus::Executed,
                0,
                Some(vec![100, 0, 0, 0, 0, 0, 0, 0, 1]),
            ),
        ),
    ];

    run_transaction(testcases);
}
