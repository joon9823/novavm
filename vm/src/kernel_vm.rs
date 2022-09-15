use anyhow::Result;
use move_deps::{
    move_core_types::{
        account_address::AccountAddress, effects::ChangeSet, vm_status::StatusCode, 
    },
    move_vm_runtime::{move_vm::MoveVM, session::{Session, SerializedReturnValues}},
};
use std::sync::Arc;

use move_deps::move_stdlib;
pub use move_deps::move_core_types::{
    resolver::MoveResolver,
    vm_status::{KeptVMStatus, VMStatus},
};
pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};


use crate::kernel_stdlib;
use crate::storage::{data_view_resolver::DataViewResolver, state_view::StateView};
use crate::gas_meter::{GasStatus, Gas, unit_cost_table};
use crate::args_validator::validate_combine_signer_and_txn_args;
use crate::message::*;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct KernelVM {
    move_vm: Arc<MoveVM>,
}

impl KernelVM {
    pub fn new() -> Self {
        let inner = MoveVM::new(
            move_stdlib::natives::all_natives(
            AccountAddress::from_hex_literal("0x1").unwrap(),
            move_stdlib::natives::GasParameters::zeros())
        .into_iter()
        .chain(
            move_stdlib::natives::nursery_natives(
            AccountAddress::from_hex_literal("0x1").unwrap(),
            move_stdlib::natives::NurseryGasParameters::zeros()))
        .into_iter()
        .chain(
            kernel_stdlib::all_natives(
                AccountAddress::from_hex_literal("0x1").unwrap(), 
                kernel_stdlib::GasParameters::zeros()
        )))
        .expect("should be able to create Move VM; check if there are duplicated natives");

        Self {
            move_vm: Arc::new(inner),
        }
    }

    pub fn initialize<S: StateView>(
        &mut self,
        compiled_module: Vec<u8>,
        remote_cache: &DataViewResolver<'_, S>,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);
        let mut cost_strategy =  GasStatus::new_unmetered();

        session
                .publish_module(compiled_module, AccountAddress::ONE, &mut cost_strategy)
                .map_err(|e| {
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;
        let (status,output) = self.success_message_cleanup(session)?;
        Ok((status, output, None))
    }

    pub fn execute_message<S: StateView>(
        &mut self,
        msg: Message,
        remote_cache: &DataViewResolver<'_, S>,
        gas_limit : Gas
    ) -> (VMStatus, MessageOutput, Option<SerializedReturnValues>) {
        let sender = msg.sender();

        let cost_schedule = unit_cost_table();
        let cost_strategy =  GasStatus::new(&cost_schedule, gas_limit);

        let result = match msg.payload() {
            payload @ MessagePayload::Script(_) | payload @ MessagePayload::EntryFunction(_) => {
                self.execute_script_or_entry_function(sender, remote_cache, payload, cost_strategy)
            }
            MessagePayload::ModuleBundle(m) => self.publish_module_bundle(sender, remote_cache, m, cost_strategy),
        };

        match result {
            Ok(status_and_output) => status_and_output,
            Err(err) => {
                let txn_status = MessageStatus::from(err.clone());

                let (status, message_output) = match txn_status.is_discarded() {
                    true => discard_error_vm_status(err),
                    false => self.failed_message_cleanup(err, remote_cache),
                };
                    
                (status, message_output, None)
            }
        }
    }
    fn publish_module_bundle<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        modules: &ModuleBundle,
        mut cost_strategy : GasStatus
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        // TODO: verification

        let module_bin_list = modules.clone().into_inner();
        session
                .publish_module_bundle(module_bin_list, sender, &mut cost_strategy)
                .map_err(|e| {
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

        // after publish the modules, we need to clear loader cache, to make init script function and
        // epilogue use the new modules.
        // session.empty_loader_cache()?;

        let (status,output) = self.success_message_cleanup(session)?;
        Ok((status, output, None))
    }

    fn execute_script_or_entry_function<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        payload: &MessagePayload,
        mut cost_strategy : GasStatus
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        // TODO: verification

        let res = match payload {
                MessagePayload::Script(script) => {
                    // we only use the ok path, let move vm handle the wrong path.
                    // let Ok(s) = CompiledScript::deserialize(script.code());
                    let loaded_func =
                        session.load_script(script.code(), script.ty_args().to_vec())?;
                    let args = validate_combine_signer_and_txn_args(&session, vec![sender], script.args().to_vec(), &loaded_func)?;

                    session.execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        args,
                        &mut cost_strategy,
                    )
                }
                MessagePayload::EntryFunction(entry_fn) => {
                    let function = session.load_function(
                        entry_fn.module(),
                        entry_fn.function(),
                        entry_fn.ty_args(),
                    )?;
                    let args = validate_combine_signer_and_txn_args(&session,vec![sender], entry_fn.args().to_vec(), &function)?;
                    
                    session.execute_entry_function(
                        entry_fn.module(),
                        entry_fn.function(),
                        entry_fn.ty_args().to_vec(),
                        args,
                        &mut cost_strategy,
                    )
                }
                MessagePayload::ModuleBundle(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e|
                {
                    println!("[VM] execute_entry_function error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

        let (status, output) = self.success_message_cleanup(session)?;
        Ok((status, output, res.into()))
    }

    fn success_message_cleanup<R: MoveResolver>(
        &self,
        session: Session<R>,
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
        let session: Session<_> = self.move_vm.new_session(remote_cache).into();

        // TODO: check if we should keep output on failure
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