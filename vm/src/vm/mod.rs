use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Event},
    value::MoveValue,
    vm_status::StatusCode,
};
pub use move_core_types::{
    resolver::MoveResolver,
    vm_status::{KeptVMStatus, VMStatus},
};
use move_vm_runtime::{move_vm::MoveVM, session::Session};
use move_vm_types::gas::UnmeteredGasMeter;

pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};

use std::sync::Arc;

pub mod transaction;
use transaction::*;

use self::storage::{data_view_resolver::DataViewResolver, state_view::StateView};

pub mod access_path;

pub mod storage;

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

    pub fn execute_user_transaction<S: StateView>(
        &mut self,
        txn: Transaction,
        remote_cache: &DataViewResolver<'_, S>,
    ) -> (VMStatus, TransactionOutput) {
        // let gas_schedule = match self.get_gas_schedule() {
        //     Ok(gas_schedule) => gas_schedule,
        //     Err(e) => {
        //         if remote_cache.is_genesis() {
        //             &G_LATEST_GAS_SCHEDULE
        //         } else {
        //             return discard_error_vm_status(e);
        //         }
        //     }
        // };
        let sender = txn.sender();

        let result = match txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::ScriptFunction(_) => {
                self.execute_script_or_script_function(sender, remote_cache, txn.payload())
            }
            TransactionPayload::Module(m) => self.publish_module(sender, remote_cache, m),
        };

        match result {
            Ok(status_and_output) => status_and_output,
            Err(err) => {
                let txn_status = TransactionStatus::from(err.clone());
                if txn_status.is_discarded() {
                    discard_error_vm_status(err)
                } else {
                    self.failed_transaction_cleanup(err, remote_cache)
                }
            }
        }
    }

    fn publish_module<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        module: &Module,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);
        let mut cost_strategy = UnmeteredGasMeter;

        {
            // Run the validation logic
            // cost_strategy.set_metering(false);
            // genesis txn skip check gas and txn prologue.
            // if !remote_cache.is_genesis() {
            //     //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
            //     self.check_gas(txn_data)?;
            //     self.run_prologue(&mut session, cost_strategy, txn_data)?;
            // }
        }
        {
            // // Genesis txn not enable gas charge.
            // if !remote_cache.is_genesis() {
            //     cost_strategy.set_metering(true);
            // }
            // cost_strategy
            //     .charge_intrinsic_gas(txn_data.transaction_size())
            //     .map_err(|e| e.into_vm_status())?;

            session
                .publish_module(module.code().to_vec(), sender, &mut cost_strategy)
                .map_err(|e| {
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

            // after publish the modules, we need to clear loader cache, to make init script function and
            // epilogue use the new modules.
            // session.empty_loader_cache()?;

            // charge_global_write_gas_usage(cost_strategy, &session, &txn_data.sender())?;

            // cost_strategy.set_metering(false);
            self.success_transaction_cleanup(session)
        }
    }

    fn execute_script_or_script_function<S: StateView>(
        &self,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        payload: &TransactionPayload,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        let mut cost_strategy = UnmeteredGasMeter;

        // Run the validation logic
        {
            // cost_strategy.set_metering(false);
            // //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
            // run prologue
        }

        // Run the execution logic
        {
            match payload {
                TransactionPayload::Script(script) => {
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
                TransactionPayload::ScriptFunction(script_function) => {
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
                TransactionPayload::Module(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e|
                {
                    println!("[VM] execute_script_function error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

            self.success_transaction_cleanup(session)
        }
    }

    fn success_transaction_cleanup<R: MoveResolver>(
        &self,
        mut session: Session<R>,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        Ok((
            VMStatus::Executed,
            get_transaction_output(session, KeptVMStatus::Executed)?,
        ))
    }

    fn failed_transaction_cleanup<S: StateView>(
        &self,
        error_code: VMStatus,
        remote_cache: &DataViewResolver<'_, S>,
    ) -> (VMStatus, TransactionOutput) {
        // let mut gas_status = {
        //     let mut gas_status = GasStatus::new(gas_schedule, gas_left);
        //     gas_status.set_metering(false);
        //     gas_status
        // };
        let mut session: Session<_> = self.move_vm.new_session(remote_cache).into();

        // init_script doesn't need run epilogue
        // if remote_cache.is_genesis() {
        //     return discard_error_vm_status(error_code);
        // }

        match TransactionStatus::from(error_code.clone()) {
            TransactionStatus::Keep(status) => {
                let txn_output = get_transaction_output(session, status)
                    .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            TransactionStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
        }
    }
}

pub struct TransactionOutput {
    change_set: ChangeSet,
    events: Vec<Event>,

    /// The amount of gas used during execution.
    gas_used: u64,

    /// The execution status.
    status: TransactionStatus,
}

impl TransactionOutput {
    pub fn new(
        change_set: ChangeSet,
        events: Vec<Event>,
        gas_used: u64,
        status: TransactionStatus,
    ) -> Self {
        TransactionOutput {
            change_set,
            events,
            gas_used,
            status,
        }
    }

    pub fn change_set(&self) -> &ChangeSet {
        &self.change_set
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }

    pub fn into_inner(self) -> (ChangeSet, Vec<Event>, u64, TransactionStatus) {
        (self.change_set, self.events, self.gas_used, self.status)
    }
}

pub(crate) fn discard_error_output(err: StatusCode) -> TransactionOutput {
    info!("discard error output: {:?}", err);
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(ChangeSet::new(), vec![], 0, TransactionStatus::Discard(err))
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, TransactionOutput) {
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

pub(crate) fn get_transaction_output<R: MoveResolver>(
    session: Session<R>,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    let gas_used: u64 = 1;

    let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;

    Ok(TransactionOutput::new(
        changeset,
        events,
        gas_used,
        TransactionStatus::Keep(status),
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
