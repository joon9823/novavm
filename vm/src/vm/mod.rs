use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, effects::ChangeSet, value::MoveValue, vm_status::StatusCode,
};
pub use move_core_types::{
    resolver::MoveResolver,
    vm_status::{KeptVMStatus, VMStatus},
};
use move_vm_runtime::{move_vm::MoveVM, session::Session};
use move_vm_types::gas::UnmeteredGasMeter;

pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};

use std::sync::Arc;

pub mod message;
use message::*;

use self::storage::{data_view_resolver::DataViewResolver, state_view::StateView};

use gas_meter::{GasStatus, Gas, unit_cost_table};

pub mod access_path;
pub mod storage;
pub mod gas_meter;


#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct KernelVM {
    move_vm: Arc<MoveVM>,
}

impl KernelVM {
    pub fn new() -> Self {
        let inner = MoveVM::new(vec![])
            .expect("should be able to create Move VM; check if there are duplicated natives");
        Self {
            move_vm: Arc::new(inner),
        }
    }

    pub fn execute_message<S: StateView>(
        &mut self,
        msg: Message,
        remote_cache: &DataViewResolver<'_, S>,
    ) -> (VMStatus, MessageOutput) {
        let sender = msg.sender();

        let result = match msg.payload() {
            payload @ MessagePayload::Script(_) | payload @ MessagePayload::EntryFunction(_) => {
                self.execute_script_or_entry_function(sender, remote_cache, payload)
            }
            MessagePayload::Module(m) => self.publish_module(sender, remote_cache, m),
        };

        match result {
            Ok(status_and_output) => status_and_output,
            Err(err) => {
                let txn_status = MessageStatus::from(err.clone());
                if txn_status.is_discarded() {
                    discard_error_vm_status(err)
                } else {
                    self.failed_message_cleanup(err, remote_cache)
                }
            }
        }
    }

    fn publish_module<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        module: &Module,
    ) -> Result<(VMStatus, MessageOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        // TODO : set gas_left with params
        let cost_schedule = unit_cost_table();
        let mut cost_strategy =  GasStatus::new(&cost_schedule,Gas::new(100_000u64));

        session
                .publish_module(module.code().to_vec(), sender, &mut cost_strategy)
                .map_err(|e| {
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

        // after publish the modules, we need to clear loader cache, to make init script function and
        // epilogue use the new modules.
        // session.empty_loader_cache()?;

        self.success_message_cleanup(session)
    }

    fn execute_script_or_entry_function<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        payload: &MessagePayload,
    ) -> Result<(VMStatus, MessageOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        // TODO : set gas_left with params
        let cost_schedule = unit_cost_table();
        let mut cost_strategy =  GasStatus::new(&cost_schedule,Gas::new(100_000u64));

        match payload {
                MessagePayload::Script(script) => {
                    // we only use the ok path, let move vm handle the wrong path.
                    // let Ok(s) = CompiledScript::deserialize(script.code());
                    let args = combine_signers_and_args(vec![sender], script.args().to_vec());

                    session.execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        args,
                        &mut cost_strategy,
                    )
                }
                MessagePayload::EntryFunction(script_function) => {
                    let args = combine_signers_and_args(vec![sender], script_function.args().to_vec());
                    println!("num {:?}", script_function.ty_args().len());
                    session.execute_function_bypass_visibility(
                        script_function.module(),
                        script_function.function(),
                        script_function.ty_args().to_vec(),
                        args,
                        &mut cost_strategy,
                    )
                }
                MessagePayload::Module(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e|
                {
                    println!("[VM] execute_entry_function error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

        self.success_message_cleanup(session)
    }

    fn success_message_cleanup<R: MoveResolver>(
        &self,
        mut session: Session<R>,
    ) -> Result<(VMStatus, MessageOutput), VMStatus> {
        Ok((
            VMStatus::Executed,
            get_message_output(session, KeptVMStatus::Executed)?,
        ))
    }

    fn failed_message_cleanup<S: StateView>(
        &self,
        error_code: VMStatus,
        remote_cache: &DataViewResolver<'_, S>,
    ) -> (VMStatus, MessageOutput) {
        let mut session: Session<_> = self.move_vm.new_session(remote_cache).into();

        match MessageStatus::from(error_code.clone()) {
            MessageStatus::Keep(status) => {
                let txn_output = get_message_output(session, status)
                    .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            MessageStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
        }
    }
}

pub(crate) fn discard_error_output(err: StatusCode) -> MessageOutput {
    info!("discard error output: {:?}", err);
    // Since this message will be discarded, no writeset will be included.
    MessageOutput::new(ChangeSet::new(), vec![], 0, MessageStatus::Discard(err))
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, MessageOutput) {
    info!("discard error vm_status output: {:?}", err);
    let vm_status = err.clone();
    let error_code = match err.keep_or_discard() {
        Ok(_) => {
            debug_assert!(false, "discarding non-discardable error: {:?}", vm_status);
            vm_status.status_code()
        }
        Err(code) => code,
    };
    (vm_status, discard_error_output(error_code))
}

pub(crate) fn get_message_output<R: MoveResolver>(
    session: Session<R>,
    status: KeptVMStatus,
) -> Result<MessageOutput, VMStatus> {
    let gas_used: u64 = 1;

    let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;

    Ok(MessageOutput::new(
        changeset,
        events,
        gas_used,
        MessageStatus::Keep(status),
    ))
}

fn combine_signers_and_args(
    signers: Vec<AccountAddress>,
    non_signer_args: Vec<Vec<u8>>,
) -> Vec<Vec<u8>> {
    signers
        .into_iter()
        .map(|s| MoveValue::Signer(s).simple_serialize().unwrap())
        .chain(non_signer_args)
        .collect()
}
