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

use crate::asset::{
    compile_move_nursery_modules, compile_move_stdlib_modules, compile_nova_stdlib_modules,
};

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
    fn balance(addr: AccountAddress) -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("balance").unwrap(),
            vec![],
            vec![addr.to_vec()],
        )
    }

    fn transfer(from: AccountAddress, to: AccountAddress, amount: u64) -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("transfer").unwrap(),
            vec![],
            vec![from.to_vec(), to.to_vec(), amount.to_le_bytes().to_vec()],
        )
    }

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

struct ExpectedOutput {
    vm_status: VMStatus,
    changed_accounts: usize,
    result_bytes: Option<Vec<u8>>,
}
impl ExpectedOutput {
    pub fn new(
        vm_status: VMStatus,
        changed_accounts: usize,
        result_bytes: Option<Vec<u8>>,
    ) -> Self {
        ExpectedOutput {
            vm_status,
            changed_accounts,
            result_bytes,
        }
    }
    pub fn vm_status(&self) -> &VMStatus {
        &self.vm_status
    }
    pub fn changed_accounts(&self) -> usize {
        self.changed_accounts
    }
    pub fn result_bytes(&self) -> &Option<Vec<u8>> {
        &self.result_bytes
    }
}

fn run_transaction(testcases: Vec<(Message, ExpectedOutput)>) {
    use super::mock_chain::MockChain;

    let mut chain = MockChain::new();
    let mut vm = NovaVM::new();

    // publish move_stdlib and move_nursery and nova_stdlib modules
    let mut modules = compile_move_stdlib_modules();
    modules.append(&mut compile_move_nursery_modules());
    modules.append(&mut compile_nova_stdlib_modules());

    let mut state = chain.create_state();
    for module in modules {
        let resolver = DataViewResolver::new(&state);
        let mut mod_blob = vec![];
        module
            .serialize(&mut mod_blob)
            .expect("Module serialization error");
        let (status, output, _) = vm
            .initialize(mod_blob, &resolver)
            .expect("Module must load");
        assert!(status == VMStatus::Executed);
        state.push_write_set(output.change_set().clone());
    }
    chain.commit(state);

    let gas_limit = Gas::new(100_000u64);
    for (msg, exp_output) in testcases {
        let mut state = chain.create_state();
        let resolver = DataViewResolver::new(&state);
        let (status, output, result) = vm
            .execute_message(msg, &resolver, gas_limit)
            .expect("nova vm failure");
        println!("gas used: {}", output.gas_used());
        println!("got:{}, exp:{}", status, exp_output.vm_status());
        assert!(status == *exp_output.vm_status());
        assert!(output.change_set().accounts().len() == exp_output.changed_accounts());

        let result_bytes = result.map(|r| r.return_values.first().map_or(vec![], |m| m.0.clone()));
        assert!(result_bytes == *exp_output.result_bytes());

        if output.status().is_discarded() {
            continue;
        }
        // apply output into db
        state.push_write_set(output.change_set().clone());

        chain.commit(state);

        // TODO: mock tx rollback
        vm.invalidate_loader_cache();
    }
}

#[cfg(test)]
#[test]
fn test_deps_transaction() {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");
    let account_three =
        AccountAddress::from_hex_literal("0x3").expect("0x3 account should be created");

    let testcases: Vec<(Message, ExpectedOutput)> = vec![
        (
            // publish module
            Message::new_module(
                Some(AccountAddress::ONE),
                ModuleBundle::from(Module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        (
            // bank module : balance
            Message::new_entry_function(
                Some(AccountAddress::ONE),
                EntryFunction::balance(account_two),
            ),
            ExpectedOutput::new(
                VMStatus::Executed,
                0,
                Some(vec![160, 134, 1, 0, 0, 0, 0, 0]),
            ),
        ),
        (
            // bank module : transfer
            Message::new_entry_function(
                Some(AccountAddress::ONE),
                EntryFunction::transfer(account_two, account_three, 100),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![])),
        ),
    ];
    run_transaction(testcases);
}

#[test]
fn test_linker_cache_flush() {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

    let testcases: Vec<(Message, ExpectedOutput)> = vec![
        (
            // module have only one function that get number 123
            Message::new_module(
                Some(AccountAddress::ONE),
                ModuleBundle::singleton(hex::decode("a11ceb0b0500000006010002030205050703070a11081b140c2f1000000001000100000103094261736963436f696e066e756d6265720000000000000000000000000000000000000001000104000002067b000000000000000200").expect("ms"))
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None)
        ),
        (
            // by calling this, loader caches module
            Message::new_entry_function(Some(AccountAddress::ZERO), EntryFunction::number()),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
        ),
        (
            // upgrade module
            Message::new_module(
                Some(AccountAddress::ONE),
                ModuleBundle::from(Module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        (
            // mint with entry function
            // should work with new module
            Message::new_entry_function(Some(account_two), EntryFunction::mint(100)),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
    ];

    run_transaction(testcases);
}

#[cfg(test)]
#[test]
fn test_simple_trasaction() {
    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

    let testcases: Vec<(Message, ExpectedOutput)> = vec![
        (
            // publish module
            Message::new_module(
                Some(AccountAddress::ONE),
                ModuleBundle::from(Module::create_basic_coin()),
            ),
            ExpectedOutput::new(VMStatus::Executed, 1, None),
        ),
        (
            // mint with script
            Message::new_script(Some(AccountAddress::ONE), Script::mint_200()),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
        (
            // mint with entry function
            Message::new_entry_function(Some(account_two), EntryFunction::mint(100)),
            ExpectedOutput::new(VMStatus::Executed, 1, Some(vec![])),
        ),
        (
            // linker error
            Message::new_entry_function(
                Some(AccountAddress::ZERO),
                EntryFunction::mint_with_wrong_module_address(100),
            ),
            ExpectedOutput::new(VMStatus::Error(StatusCode::LINKER_ERROR), 0, None),
        ),
        (
            // get 123
            Message::new_entry_function(Some(AccountAddress::ZERO), EntryFunction::number()),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![123, 0, 0, 0, 0, 0, 0, 0])),
        ),
        (
            // get coin amount for 0x1
            Message::new_entry_function(
                Some(AccountAddress::ZERO),
                EntryFunction::get(AccountAddress::ONE),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![200, 0, 0, 0, 0, 0, 0, 0])),
        ),
        (
            // get coin amount for 0x0
            Message::new_entry_function(
                Some(AccountAddress::ZERO),
                EntryFunction::get(account_two),
            ),
            ExpectedOutput::new(VMStatus::Executed, 0, Some(vec![100, 0, 0, 0, 0, 0, 0, 0])),
        ),
        (
            // get Coin structure
            Message::new_entry_function(
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
