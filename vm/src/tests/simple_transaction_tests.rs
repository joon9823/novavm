use crate::vm::{
    access_path::AccessPath,
    gas_meter::Gas,
    message::{EntryFunction, Message, Module, Script},
    storage::data_view_resolver::DataViewResolver,
    storage::state_view::StateView,
    KernelVM,
};
use std::collections::BTreeMap;

use move_deps::move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    identifier::Identifier,
    language_storage::ModuleId,
    vm_status::{StatusCode, VMStatus},
};

//faking chain db
struct MockDB {
    map: BTreeMap<AccessPath, Option<Vec<u8>>>,
}

impl MockDB {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    fn write_op(&mut self, ref ap: AccessPath, ref blob_opt: Op<Vec<u8>>) {
        match blob_opt {
            Op::New(blob) | Op::Modify(blob) => {
                self.map.insert(ap.clone(), Some(blob.clone()));
            }
            Op::Delete => {
                self.map.remove(ap);
                self.map.insert(ap.clone(), None);
            }
        }
    }

    pub fn push_write_set(&mut self, changeset: ChangeSet) {
        for (addr, account_changeset) in changeset.into_inner() {
            let (modules, resources) = account_changeset.into_inner();
            for (struct_tag, blob_opt) in resources {
                let ap = AccessPath::resource_access_path(addr, struct_tag);
                self.write_op(ap, blob_opt)
            }

            for (name, blob_opt) in modules {
                let ap = AccessPath::from(&ModuleId::new(addr, name));
                self.write_op(ap, blob_opt)
            }
        }
    }
}

impl StateView for MockDB {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.map.get(access_path) {
            Some(opt_data) => Ok(opt_data.clone()),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[test]
fn test_simple_trasaction() {
    let mut db = MockDB::new();
    let mut vm = KernelVM::new();

    let account_two =
        AccountAddress::from_hex_literal("0x2").expect("0x2 account should be created");

    let testcases: Vec<(Message, VMStatus, usize, Option<Vec<u8>>)> = vec![
        (
            // publish module
            Message::new_module(AccountAddress::ONE, Module::create_basic_coin()),
            VMStatus::Executed,
            1,
            None,
        ),
        (
            // mint with script
            Message::new_script(AccountAddress::ONE, Script::mint_200()),
            VMStatus::Executed,
            1,
            Some(vec![]),
        ),
        (
            // mint with entry function
            Message::new_entry_function(account_two, EntryFunction::mint(100)),
            VMStatus::Executed,
            1,
            Some(vec![]),
        ),
        (
            // linker error
            Message::new_entry_function(
                AccountAddress::ZERO,
                EntryFunction::mint_with_wrong_module_address(100),
            ),
            VMStatus::Error(StatusCode::LINKER_ERROR),
            0,
            None,
        ),
        (
            // get 123
            Message::new_entry_function(AccountAddress::ZERO, EntryFunction::number()),
            VMStatus::Executed,
            0,
            Some(vec![123, 0, 0, 0, 0, 0, 0, 0]),
        ),
        (
            // get coin amount for 0x1
            Message::new_entry_function(
                AccountAddress::ZERO,
                EntryFunction::get(AccountAddress::ONE),
            ),
            VMStatus::Executed,
            0,
            Some(vec![200, 0, 0, 0, 0, 0, 0, 0]),
        ),
        (
            // get coin amount for 0x0
            Message::new_entry_function(AccountAddress::ZERO, EntryFunction::get(account_two)),
            VMStatus::Executed,
            0,
            Some(vec![100, 0, 0, 0, 0, 0, 0, 0]),
        ),
        (
            // get Coin structure
            Message::new_entry_function(
                AccountAddress::ZERO,
                EntryFunction::getCoinStruct(account_two),
            ),
            VMStatus::Executed,
            0,
            Some(vec![100, 0, 0, 0, 0, 0, 0, 0, 1]),
        ),
    ];

    let gas_left = Gas::new(100_000u64);
    for (tx, exp_status, exp_changed_accounts, exp_result) in testcases {
        let resolver = DataViewResolver::new(&db);
        let (status, output, result) = vm.execute_message(tx, &resolver, gas_left);

        assert!(status == exp_status);
        assert!(output.change_set().accounts().len() == exp_changed_accounts);

        let result_bytes = result.map(|r| r.return_values.first().map_or(vec![], |m| m.0.clone()));
        assert!(result_bytes == exp_result);

        if output.status().is_discarded() {
            continue;
        }
        // apply output into db
        db.push_write_set(output.change_set().clone());
    }
}

#[cfg(test)]
impl Module {
    fn create_basic_coin() -> Self {
        Self::new(
            include_bytes!("../../move-test/build/test1/bytecode_modules/BasicCoin.mv").to_vec(),
        )
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
            vec![],
            vec![amount.to_le_bytes().to_vec()],
        )
    }

    fn mint_with_wrong_module_address(amount: u64) -> Self {
        Self::new(
            ModuleId::new(AccountAddress::ZERO, Identifier::new("BasicCoin").unwrap()),
            Identifier::new("mint").unwrap(),
            vec![],
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
            vec![],
            vec![addr.to_vec()],
        )
    }

    fn getCoinStruct(addr: AccountAddress) -> Self {
        Self::new(
            Module::get_basic_coin_module_id(),
            Identifier::new("getCoin").unwrap(),
            vec![],
            vec![addr.to_vec()],
        )
    }
}

#[cfg(test)]
impl Script {
    fn mint_200() -> Self {
        Self::new(
            include_bytes!("../../move-test/build/test1/bytecode_scripts/main.mv").to_vec(),
            vec![],
            vec![],
        )
    }
}
