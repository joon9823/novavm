pub mod serde_helper;
mod vm;
use std::collections::BTreeMap;

use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Op},
    identifier::Identifier,
    language_storage::ModuleId,
};
use vm::access_path::AccessPath;
use vm::storage::state_view::StateView;
use vm::transaction::Transaction;
use vm::KernelVM;

use crate::vm::{
    storage::data_view_resolver::DataViewResolver,
    transaction::{Sample, Script, ScriptFunction},
};

fn main() {
    println!("Hello, world!");

    let mut db = DB::new();

    let mut vm = KernelVM::new();

    let txs = vec![
        Transaction::sample(),
        Transaction::new_script(AccountAddress::ONE, Script::sample()),
        Transaction::new_script_function(
            AccountAddress::ZERO,
            ScriptFunction::new(
                ModuleId::new(AccountAddress::ZERO, Identifier::new("BasicCoin").unwrap()),
                Identifier::new("mint").unwrap(),
                vec![],
                vec![(100 as u64).to_be_bytes().to_vec()],
            ),
        ),
        Transaction::new_script_function(AccountAddress::ONE, ScriptFunction::sample()),
        // Transaction::new_script_function(AccountAddress::ONE, ScriptFunction::sample()),
    ];

    for tx in txs {
        let resolver = DataViewResolver::new(&db);
        let (status, output) = vm.execute_user_transaction(tx, &resolver);

        println!("\nstatus {:?}", status);
        println!(
            "tx output's affected account count {:?}",
            output.change_set().accounts().len()
        );

        for (acco, accc) in output.change_set().accounts().into_iter() {
            println!("account - {:?}", acco);
            accc.modules()
                .iter()
                .for_each(|(id, ops)| println!("id {:?} ops {:?}", id, ops));

            accc.resources()
                .iter()
                .for_each(|(id, ops)| println!("id {:?} ops {:?}", id, ops));
        }

        if output.status().is_discarded() {
            continue;
        }

        // apply output into db
        db.push_write_set(output.change_set().clone())
    }

    println!("end");
}

//faking chain db
struct DB {
    map: BTreeMap<AccessPath, Option<Vec<u8>>>,
}

impl DB {
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

impl StateView for DB {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.map.get(access_path) {
            Some(opt_data) => Ok(opt_data.clone()),
            None => Ok(None),
        }
    }
}
