use super::{mock_chain::MockChain};

use crate::{
    message::{Message},
    gas::Gas,
    nova_vm::NovaVM,
    storage::data_view_resolver::DataViewResolver,
};

use move_deps::{
    move_core_types::{
        vm_status::{VMStatus},
    },
};


pub struct MockTx {
    pub msg_tests: Vec<(Message, ExpectedOutput)>,
    pub should_commit: bool,
}

#[allow(dead_code)]
impl MockTx {
    pub fn one(msg: Message, exp_output: ExpectedOutput) -> Self {
        Self {
            msg_tests: vec![(msg, exp_output)],
            should_commit: true,
        }
    }

    pub fn one_skip_commit(msg: Message, exp_output: ExpectedOutput) -> Self {
        Self {
            msg_tests: vec![(msg, exp_output)],
            should_commit: false,
        }
    }

    pub fn new(msg_tests: Vec<(Message, ExpectedOutput)>) -> Self {
        Self {
            msg_tests,
            should_commit: true,
        }
    }

    pub fn new_skip_commit(msg_tests: Vec<(Message, ExpectedOutput)>) -> Self {
        Self {
            msg_tests,
            should_commit: false,
        }
    }
}

pub struct ExpectedOutput {
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


pub fn run_transaction(testcases: Vec<MockTx>) {
    let mut chain = MockChain::new();
    let mut vm = NovaVM::new();

    let mut state = chain.create_state();
    let resolver = DataViewResolver::new(&state);
    let (status, output, _) = vm.initialize(&resolver, None).expect("Module must load");
    assert!(status == VMStatus::Executed);
    state.push_write_set(output.change_set().clone(), output.table_change_set());
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
            
            println!("result_bytes: {:?}", result_bytes);            
            assert!(result_bytes == *exp_output.result_bytes());

            if status != VMStatus::Executed {
                continue;
            }
            // apply output into state
            state.push_write_set(output.change_set().clone(),output.table_change_set());
        }

        if should_commit {
            chain.commit(state);
        }
    }
}
