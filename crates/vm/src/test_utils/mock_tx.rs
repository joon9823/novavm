use std::collections::BTreeMap;

use super::mock_chain::MockChain;

use nova_gas::Gas;
use nova_storage::{state_view_impl::StateViewImpl, table_view_impl::TableViewImpl};
use nova_types::{message::Message, message::MessageOutput, size_delta::SizeDelta};

use crate::{nova_vm::NovaVM, test_utils::mock_chain::MockTableState};

use move_deps::{
    move_core_types::{account_address::AccountAddress, vm_status::VMStatus},
    move_vm_runtime::session::SerializedReturnValues,
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

    #[allow(dead_code)]
    pub fn one_skip_commit(msg: Message, exp_output: ExpectedOutput) -> Self {
        Self {
            msg_tests: vec![(msg, exp_output)],
            should_commit: false,
        }
    }

    #[allow(dead_code)]
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

type VMOutput = (VMStatus, MessageOutput, Option<SerializedReturnValues>);

pub struct ExpectedOutput(pub Vec<ExpectedOutputItem>);

impl ExpectedOutput {
    // for compatibility with previous tests
    pub fn new(vm_status: VMStatus, result_bytes: Option<Vec<u8>>) -> Self {
        let mut items = vec![ExpectedOutputItem::VMStatusReturn(vm_status)];
        if let Some(b) = result_bytes {
            items.push(ExpectedOutputItem::ResultBytes(b));
        }
        Self(items)
    }

    pub fn check_output(&self, vm_output: &VMOutput) {
        for exp in &self.0 {
            exp.check_output(vm_output);
        }
    }
}

pub enum ExpectedOutputItem {
    VMStatusReturn(VMStatus),
    ResultBytes(Vec<u8>),
    SizeChange(BTreeMap<AccountAddress, SizeDelta>),
}

impl ExpectedOutputItem {
    pub fn check_output(&self, vm_output: &VMOutput) {
        let (status, output, result) = vm_output;
        match self {
            ExpectedOutputItem::VMStatusReturn(exp_status) => {
                println!("got:{}, exp:{}", status, exp_status);
                assert!(status == exp_status);
            }
            ExpectedOutputItem::ResultBytes(exp_bytes) => {
                let result_bytes = result
                    .as_ref()
                    .map(|r| r.return_values.first().map_or(vec![], |m| m.0.clone()))
                    .expect("expected some bytes return");

                println!("result_bytes: {:?}", result_bytes);
                assert!(result_bytes == *exp_bytes);
            }
            ExpectedOutputItem::SizeChange(exp_map) => {
                assert!(
                    *output.size_change_set().changes() == *exp_map,
                    "expected\n{:?}\n\noutput\n{:?}",
                    exp_map,
                    output.size_change_set().changes()
                );
            }
        };
    }
}

pub fn run_transaction(testcases: Vec<MockTx>) {
    let mut chain = MockChain::new();
    let mut vm = NovaVM::new();

    let mut state = chain.create_state();
    let mut table_state = MockTableState::new(&state);

    let api = chain.create_api(0, 0);
    let resolver = StateViewImpl::new(&state);
    let mut table_resolver = TableViewImpl::new(&mut table_state);

    let (status, output, _) = vm
        .initialize(&resolver, &mut table_resolver, None)
        .expect("Module must load");
    assert!(status == VMStatus::Executed);
    let inner_output = output.into_inner();
    state.push_write_set(inner_output.1);
    chain.commit(state);

    let gas_limit = Gas::new(100_000u64);
    let mut num = 0;
    for MockTx {
        msg_tests,
        should_commit,
    } in testcases
    {
        num += 1;
        println!("\n\ntx #{} start", num);
        let mut state = chain.create_state();

        for (msg, exp_output) in msg_tests {
            let resolver = StateViewImpl::new(&state);

            let mut table_state = MockTableState::new(&state);
            let mut table_resolver = TableViewImpl::new(&mut table_state);

            let vm_output = vm
                .execute_message(msg, &resolver, &mut table_resolver, Some(&api), gas_limit)
                .expect("nova vm failure");

            exp_output.check_output(&vm_output);

            let (status, output, _result) = vm_output;
            println!("gas used: {}", output.gas_used());

            if status != VMStatus::Executed {
                continue;
            }

            // apply output into state
            let inner_output = output.into_inner();
            state.push_write_set(inner_output.1);

            println!("size change of accounts {:?}", &inner_output.3);
        }

        if should_commit {
            chain.commit(state);
        }
    }
}
