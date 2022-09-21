use anyhow::Result;
use move_deps::{
    move_core_types::{
        account_address::AccountAddress, effects::{ChangeSet, Event}, vm_status::StatusCode 
    },
    move_vm_runtime::{move_vm::MoveVM, session::{Session, SerializedReturnValues}},
    move_vm_types::gas::UnmeteredGasMeter,
};
use std::{sync::Arc};
use move_deps::{
    move_stdlib,
    move_core_types::language_storage::CORE_CODE_ADDRESS
};
pub use move_deps::move_core_types::{
    resolver::MoveResolver,
    vm_status::{KeptVMStatus, VMStatus},
};
pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};


use crate::{nova_stdlib, gas::{InitialGasSchedule}, NovaVMError};
use crate::storage::{data_view_resolver::DataViewResolver, state_view::StateView};
use crate::args_validator::validate_combine_signer_and_txn_args;
use crate::message::*;
use crate::gas::{
    NovaGasMeter, NovaGasParameters, Gas, NativeGasParameters,
};


#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NovaVM {
    move_vm: Arc<MoveVM>,
    gas_params: NovaGasParameters,
}

impl NovaVM {
    pub fn new() -> Self {

        let gas_params = NovaGasParameters::initial();
        let native_gas_parameters = NativeGasParameters::initial();

        let inner = MoveVM::new(
            move_stdlib::natives::all_natives(
            CORE_CODE_ADDRESS,
            native_gas_parameters.move_stdlib)
        .into_iter()
        .chain(
            move_stdlib::natives::nursery_natives(
            CORE_CODE_ADDRESS,
            move_stdlib::natives::NurseryGasParameters::zeros()))
        .into_iter()
        .chain(
            nova_stdlib::all_natives(
            CORE_CODE_ADDRESS, 
            native_gas_parameters.nova_stdlib
        )))
        .expect("should be able to create Move VM; check if there are duplicated natives");

        Self {
            move_vm: Arc::new(inner),
            gas_params
        }   
    }

    pub fn initialize<S: StateView>(
        &mut self,
        compiled_module: Vec<u8>,
        remote_cache: &DataViewResolver<'_, S>,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), NovaVMError> {
        let mut session = self.create_session(remote_cache);
        let mut gas_meter = UnmeteredGasMeter;
        session
                .publish_module(compiled_module, CORE_CODE_ADDRESS, &mut gas_meter)
                .map_err(|e| {
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    NovaVMError::from(e.into_vm_status())
                })?;

        let session_output = session.finish().map_err(|e| {
            NovaVMError::from(e.into_vm_status())
        })?;

        let output = get_message_output(session_output, Gas::zero(), KeptVMStatus::Executed).map_err(|e| {
            NovaVMError::from(e)
        })?;
        Ok((VMStatus::Executed, output, None))
    }

    /// Allows the adapter to announce to the VM that the code loading cache should be considered
    /// outdated. This can happen if the adapter executed a particular code publishing transaction
    /// but decided to not commit the result to the data store. Because the code cache currently
    /// does not support deletion, the cache will, incorrectly, still contain this module.
    pub fn invalidate_loader_cache(self) {
        self.move_vm.mark_loader_cache_as_invalid();
    }

    pub fn execute_message<S: StateView>(
        &mut self,
        msg: Message,
        remote_cache: &DataViewResolver<'_, S>,
        gas_limit : Gas
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), NovaVMError> {
        let sender = msg.sender();

        let gas_params = self.gas_params.clone();
        let mut gas_meter = NovaGasMeter::new(gas_params, gas_limit);

        // Charge for msg byte size
        gas_meter
            .charge_intrinsic_gas_for_transaction((msg.size() as u64).into())
            .map_err(|e| NovaVMError::from(e.into_vm_status()))?;
        
        let result = match msg.payload() {
            payload @ MessagePayload::Script(_) | payload @ MessagePayload::EntryFunction(_) => {
                self.execute_script_or_entry_function(sender, remote_cache, payload, &mut gas_meter)
            }
            MessagePayload::ModuleBundle(m) => {
                match sender {
                    Some(sender) => self.publish_module_bundle(sender, remote_cache, m, &mut gas_meter),
                    None => return Err(NovaVMError::generic_err("sender unset")),
                }
            },
        };

        // Charge for err msg        
        let gas_used = gas_limit.checked_sub(gas_meter.balance()).unwrap();

        match result {
            Ok(status_and_output) => Ok(status_and_output),
            Err(err) => {
                let txn_status = MessageStatus::from(err.clone());

                let (status, message_output) = match txn_status.is_discarded() {
                    true => discard_error_vm_status(err, gas_used),
                    false => self.failed_message_cleanup(err, remote_cache, gas_used ),
                };
                    
                Ok((status, message_output, None))
            }
        }
    }

    fn create_session<'r, S: MoveResolver>(&self, remote: &'r S) -> Session<'r, '_, S> {
        self.move_vm.flush_loader_cache_if_invalidated();
        self.move_vm.new_session(remote)
    }

    fn publish_module_bundle<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        modules: &ModuleBundle,
        gas_meter : &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.create_session(remote_cache);

        // TODO: verification

        let module_bin_list = modules.clone().into_inner();
        session
                .publish_module_bundle(module_bin_list, sender, gas_meter)
                .map_err(|e| {
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;
            
        // after publish the modules, we need to clear loader cache, to make init script function and
        // epilogue use the new modules.
        // session.empty_loader_cache()?;

        let session_output = session.finish().map_err(|e| e.into_vm_status())?;
        let (status,output) = self.success_message_cleanup(session_output, gas_meter)?;
        Ok((status, output, None))
    }

    fn execute_script_or_entry_function<S: StateView>(
        &self,
        sender: Option<AccountAddress>,
        remote_cache: &DataViewResolver<'_, S>,
        payload: &MessagePayload,
        gas_meter : &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.create_session(remote_cache);

        // TODO: verification

        let senders = match sender {
            Some(s) => vec![s],
            None => vec![]
        };

        let res = match payload {
                MessagePayload::Script(script) => {
                    // we only use the ok path, let move vm handle the wrong path.
                    // let Ok(s) = CompiledScript::deserialize(script.code());
                    let loaded_func =
                        session.load_script(script.code(), script.ty_args().to_vec())?;
                    let args = validate_combine_signer_and_txn_args(&session, senders, script.args().to_vec(), &loaded_func)?;

                    session.execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        args,
                        gas_meter,
                    )
                }
                MessagePayload::EntryFunction(entry_fn) => {
                    let function = session.load_function(
                        entry_fn.module(),
                        entry_fn.function(),
                        entry_fn.ty_args(),
                    )?;
                    let args = validate_combine_signer_and_txn_args(&session,senders, entry_fn.args().to_vec(), &function)?;
                    
                    session.execute_entry_function(
                        entry_fn.module(),
                        entry_fn.function(),
                        entry_fn.ty_args().to_vec(),
                        args,
                        gas_meter,
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

        // Charge for change set
        let session_output = session.finish().map_err(|e| e.into_vm_status())?;
        gas_meter.charge_change_set_gas(session_output.0.accounts())?;
        let (status, output) = self.success_message_cleanup(session_output, gas_meter)?;
        
        Ok((status, output, res.into()))
    }

    fn success_message_cleanup(
        &self,
        session_output : (ChangeSet, Vec<Event>),// session: Session<R>,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput), VMStatus> {
        let gas_limit = gas_meter.gas_limit();
        let gas_used = gas_limit.checked_sub(gas_meter.balance()).unwrap();
        Ok((
            VMStatus::Executed,
            get_message_output(session_output, gas_used, KeptVMStatus::Executed)?,
        ))
    }

    fn failed_message_cleanup<S: StateView>(
        &self,
        error_code: VMStatus,
        remote_cache: &DataViewResolver<'_, S>,
        gas_used : Gas
    ) -> (VMStatus, MessageOutput) {
        let session: Session<_> = self.create_session(remote_cache).into();
        let session_output = session.finish().map_err(|e| e.into_vm_status()).unwrap();
        // TODO: check if we should keep output on failure
        match MessageStatus::from(error_code.clone()) {
            MessageStatus::Keep(status) => {
                let txn_output = get_message_output(session_output, gas_used, status)
                    .unwrap_or_else(|e| discard_error_vm_status(e, gas_used).1);
                (error_code, txn_output)
            }
            MessageStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status, gas_used))
            }
        }
    }
}

pub(crate) fn discard_error_output(err: StatusCode, gas_used : Gas) -> MessageOutput {
    info!("discard error output: {:?}", err);
    // Since this message will be discarded, no writeset will be included.
    MessageOutput::new(ChangeSet::new(), vec![], gas_used.into(), MessageStatus::Discard(err))
}

pub(crate) fn discard_error_vm_status(err: VMStatus, gas_used : Gas) -> (VMStatus, MessageOutput) {
    info!("discard error vm_status output: {:?}", err);
    let vm_status = err.clone();
    let error_code = match err.keep_or_discard() {
        Ok(_) => {
            debug_assert!(false, "discarding non-discardable error: {:?}", vm_status);
            vm_status.status_code()
        }
        Err(code) => code,
    };
    (vm_status, discard_error_output(error_code, gas_used))
}

pub(crate) fn get_message_output(
    session_output : (ChangeSet, Vec<Event>),
    gas_used: Gas,
    status: KeptVMStatus,
) -> Result<MessageOutput, VMStatus> {
    let (changeset, events) = session_output;

    Ok(MessageOutput::new(
        changeset,
        events,
        gas_used.into(),
        MessageStatus::Keep(status),
    ))
}