use move_deps::move_core_types::vm_status::VMStatus;

use crate::Message;

pub struct MockTx {
    pub msg_tests: Vec<(Message, ExpectedOutput)>,
    pub should_commit: bool,
}

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
