use crate::vm::{
    access_path::AccessPath,
    gas_meter::Gas,
    message::Message,
    message::{EntryFunction, Sample, Script},
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

#[test]
fn test_simple_trasaction() {
    let mut db = MockDB::new();

    let mut vm = KernelVM::new();

    let testcases = vec![
        (Message::sample(), VMStatus::Executed),
        (Message::new_script(AccountAddress::ONE, Script::sample()), VMStatus::Error(StatusCode::LINKER_ERROR)),
        (Message::new_entry_function(
                AccountAddress::ZERO,
                EntryFunction::new(
                    ModuleId::new(AccountAddress::ZERO, Identifier::new("BasicCoin").unwrap()),
                    Identifier::new("mint").unwrap(),
                    vec![],
                    vec![(100 as u64).to_be_bytes().to_vec()],
                ),
        ), VMStatus::Error(StatusCode::LINKER_ERROR)),
        (Message::new_entry_function(AccountAddress::ONE, EntryFunction::sample()), VMStatus::Executed),
    ];

    let gas_left = Gas::new(100_000u64);
    for (tx, exp_status) in testcases {
        let resolver = DataViewResolver::new(&db);
        let (status, output) = vm.execute_message(tx, &resolver, gas_left);

        assert!(status == exp_status);
        match status {
            VMStatus::Executed => {
                assert!(output.change_set().accounts().len() > 0)
            },
            _ => assert!(output.change_set().accounts().len() == 0),
        }

        if output.status().is_discarded() {
            continue;
        }
        // apply output into db
        db.push_write_set(output.change_set().clone());
    }
}
