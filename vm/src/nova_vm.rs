use anyhow::Result;
pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};
pub use move_deps::move_core_types::{
    resolver::MoveResolver,
    vm_status::{KeptVMStatus, VMStatus},
};
use move_deps::{
    move_binary_format::CompiledModule,
    move_binary_format::{
        access::ModuleAccess,
        errors::{Location, PartialVMError, VMError, VMResult},
    },
    move_bytecode_utils::Modules,
    move_core_types::{
        account_address::AccountAddress, effects::ChangeSet,
        vm_status::StatusCode,
    },
    move_vm_runtime::{
        move_vm::MoveVM,
        native_extensions::NativeContextExtensions,
        session::{SerializedReturnValues, Session},
    },
    move_vm_types::gas::UnmeteredGasMeter,
};
use crate::{
    natives::table::{NativeTableContext, TableChangeSet, TableResolver}, 
    args_validator::check_args_address, 
    size_change_set::AccountSizeChangeSet
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::args_validator::validate_combine_signer_and_txn_args;
use crate::asset::{
    compile_move_nursery_modules, compile_move_stdlib_modules, compile_nova_stdlib_modules,
};
use crate::gas::{Gas, NativeGasParameters, NovaGasMeter, NovaGasParameters};
use crate::message::*;
use crate::storage::{data_view_resolver::DataViewResolver, state_view::StateView};
use crate::{
    gas::InitialGasSchedule,
    natives::{
        code::{NativeCodeContext, PublishRequest},
        nova_natives,
    },
    session::{SessionExt, SessionOutput},
    storage::data_view_resolver::{StoredSizeResolver},
    table_meta::{resolve_table_ownership, resolve_table_size_change_by_account , TableMetaChangeSet, TableOwnerDataCache},
    NovaVMError,
};

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NovaVM {
    move_vm: Arc<MoveVM>,
    gas_params: NovaGasParameters,
}

impl NovaVM {
    pub fn new() -> Self {
        let inner = MoveVM::new(nova_natives(NativeGasParameters::initial()))
            .expect("should be able to create Move VM; check if there are duplicated natives");

        Self {
            move_vm: Arc::new(inner),
            gas_params: NovaGasParameters::initial(),
        }
    }

    fn create_session<'r, S: MoveResolver + TableResolver + StoredSizeResolver>(
        &self,
        remote: &'r S,
        session_id: Vec<u8>,
    ) -> SessionExt<'r, '_, S> {
        let mut extensions = NativeContextExtensions::default();
        let txn_hash: [u8; 32] = session_id
            .try_into()
            .expect("HashValue should convert to [u8; 32]");
        extensions.add(NativeTableContext::new(txn_hash, remote));
        extensions.add(NativeCodeContext::default());

        self.move_vm.flush_loader_cache_if_invalidated();
        SessionExt::new(
            self.move_vm.new_session_with_extensions(remote, extensions),
            remote,
        )
    }

    pub fn initialize<S: StateView>(
        &mut self,
        resolver: &DataViewResolver<'_, S>,
        custom_module_bundle: Option<ModuleBundle>,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), NovaVMError> {
        // publish move_stdlib and nova_stdlib modules
        let mut modules = compile_move_stdlib_modules();
        modules.append(&mut compile_move_nursery_modules());
        modules.append(&mut compile_nova_stdlib_modules());

        if let Some(module_bundle) = custom_module_bundle {
            let custom_modules = self
                .deserialize_module_bundle(&module_bundle)
                .map_err(|e| e.into_vm_status())?;
            modules.extend(custom_modules.into_iter());
        }

        let mut session = self.create_session(resolver, vec![0; 32]);

        let lib = Modules::new(&modules);
        let dep_graph = lib.compute_dependency_graph();
        let mut addr_opt: Option<AccountAddress> = None;
        let modules = dep_graph
            .compute_topological_order()
            .unwrap()
            .map(|m| {
                let addr = *m.self_id().address();
                if let Some(a) = addr_opt {
                    assert_eq!(
                        a,
                        addr,
                        "All genesis modules must be published under the same address, but found modules under both {} and {}",
                        a.short_str_lossless(),
                        addr.short_str_lossless(),
                    );
                } else {
                    addr_opt = Some(addr)
                }
                let mut bytes = vec![];
                m.serialize(&mut bytes).unwrap();
                bytes
            })
            .collect::<Vec<Vec<u8>>>();

        session
                .publish_module_bundle(modules, addr_opt.unwrap(), &mut UnmeteredGasMeter)
                .map_err(|e| {
                    self.move_vm.mark_loader_cache_as_invalid();
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    NovaVMError::from(e.into_vm_status())
                })?;

        self.move_vm.mark_loader_cache_as_invalid();

        let session_output = session.finish()?;

        let output = get_message_output(session_output, None, Gas::zero(), KeptVMStatus::Executed)
            .map_err(|e| NovaVMError::from(e))?;
        Ok((VMStatus::Executed, output, None))
    }

    pub fn execute_message<S: StateView>(
        &mut self,
        msg: Message,
        remote_cache: &DataViewResolver<'_, S>,
        gas_limit: Gas,
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
                self.execute_script_or_entry_function(
                    msg.session_id().to_vec(),
                    sender,
                    remote_cache,
                    payload,
                    &mut gas_meter,
                )
            }
            MessagePayload::ModuleBundle(m) => match sender {
                Some(sender) => self.publish_module_bundle(
                    msg.session_id().to_vec(),
                    sender,
                    remote_cache,
                    m,
                    &mut gas_meter,
                ),
                None => return Err(NovaVMError::generic_err("sender unset")),
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
                    false => self.failed_message_cleanup(
                        msg.session_id().to_vec(),
                        err,
                        remote_cache,
                        gas_used,
                    ),
                };

                Ok((status, message_output, None))
            }
        }
    }

    fn publish_module_bundle<S: StateView>(
        &self,
        session_id: Vec<u8>,
        sender: AccountAddress,
        remote_cache: &DataViewResolver<'_, S>,
        modules: &ModuleBundle,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.create_session(remote_cache, session_id);

        let module_bin_list = modules.clone().into_inner();
        session
                .publish_module_bundle(module_bin_list, sender, gas_meter)
                .map_err(|e| {
                    self.move_vm.mark_loader_cache_as_invalid();
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

        self.move_vm.mark_loader_cache_as_invalid();

        let session_output = session.finish()?;
        let (status, output) = self.success_message_cleanup(session_output, None, gas_meter)?;
        Ok((status, output, None))
    }

    fn execute_script_or_entry_function<S: StateView>(
        &self,
        session_id: Vec<u8>,
        sender: Option<AccountAddress>,
        remote_cache: &DataViewResolver<'_, S>,
        payload: &MessagePayload,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.create_session(remote_cache, session_id.clone());

        let senders = match sender {
            Some(s) => vec![s],
            None => vec![],
        };

        let res = match payload {
                MessagePayload::Script(script) => {
                    // we only use the ok path, let move vm handle the wrong path.
                    // let Ok(s) = CompiledScript::deserialize(script.code());
                    let loaded_func =
                        session.load_script(script.code(), script.ty_args().to_vec())?;
                    let args = validate_combine_signer_and_txn_args(&session, senders, script.args().to_vec(), &loaded_func)?;
                    
                    check_args_address(remote_cache, &loaded_func.parameters, &args).map_err(|e| e.finish(Location::Undefined))?;


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
                    check_args_address(remote_cache, &function.parameters, &args).map_err(|e| e.finish(Location::Undefined))?;

                    
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
                    self.move_vm.mark_loader_cache_as_invalid();
                    println!("[VM] execute_entry_function error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

        self.move_vm.mark_loader_cache_as_invalid();

        // Handler for NativeCodeContext - to allow a module publish other module
        self.resolve_pending_code_publish(&mut session, gas_meter)?;

        let mut session_output = session.finish()?;

        let temporary_session = self.create_session(remote_cache, session_id);
        let mut data_cache = TableOwnerDataCache::new(remote_cache);

        resolve_table_ownership(
                temporary_session,
                &session_output.0,
                &session_output.2, 
                &mut data_cache
            )
            .map_err(|e| {
                println!("[VM] resolve_table_ownership error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                e.into_vm_status()
            })?;

        let accounts_table_size_changes = resolve_table_size_change_by_account(
            &session_output.2, 
            &session_output.4,
            &mut data_cache   
        )
                .map_err(|e| {
            println!("[VM] resolve_table_size_change_by_account error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
            e.into_vm_status()
        })?;
        session_output.3.merge(accounts_table_size_changes);

        let meta_change_set = data_cache.into_change_set()            
        .map_err(|e| {
            let e = e.finish(Location::Undefined);
            println!("[VM] table meta change set making error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
            e.into_vm_status()
        })?;

        
        // Charge for change set
        gas_meter.charge_change_set_gas(session_output.0.accounts())?;
        let (status, output) =
            self.success_message_cleanup(session_output, Some(meta_change_set), gas_meter)?;

        Ok((status, output, res.into()))
    }

    fn success_message_cleanup(
        &self,
        session_output: SessionOutput, // session: Session<R>,
        table_owner_change_set: Option<TableMetaChangeSet>,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput), VMStatus> {
        let gas_limit = gas_meter.gas_limit();
        let gas_used = gas_limit.checked_sub(gas_meter.balance()).unwrap();
        Ok((
            VMStatus::Executed,
            get_message_output(
                session_output,
                table_owner_change_set,
                gas_used,
                KeptVMStatus::Executed,
            )?,
        ))
    }

    fn failed_message_cleanup<S: StateView>(
        &self,
        session_id: Vec<u8>,
        error_code: VMStatus,
        remote_cache: &DataViewResolver<'_, S>,
        gas_used: Gas,
    ) -> (VMStatus, MessageOutput) {
        // TODO - in aptos vm, they rerun tx in simulation mode and get the used gas to charge cost
        // even the tx failed. should we follow this?
        let session: SessionExt<_> = self.create_session(remote_cache, session_id).into();
        let session_output = session.finish().unwrap();

        match MessageStatus::from(error_code.clone()) {
            MessageStatus::Keep(status) => {
                let txn_output = get_message_output(session_output, None, gas_used, status)
                    .unwrap_or_else(|e| discard_error_vm_status(e, gas_used).1);
                (error_code, txn_output)
            }
            MessageStatus::Discard(status) => (
                VMStatus::Error(status),
                discard_error_output(status, gas_used),
            ),
        }
    }

    /// Deserialize a module bundle.
    fn deserialize_module_bundle(&self, modules: &ModuleBundle) -> VMResult<Vec<CompiledModule>> {
        let mut result = vec![];
        for module_blob in modules.iter() {
            match CompiledModule::deserialize(module_blob.code()) {
                Ok(module) => {
                    result.push(module);
                }
                Err(_err) => {
                    return Err(PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                        .finish(Location::Undefined))
                }
            }
        }
        Ok(result)
    }

    /// Resolve a pending code publish request registered via the NativeCodeContext.
    fn resolve_pending_code_publish<'r, S: MoveResolver + TableResolver>(
        &self,
        session: &mut Session<'r, '_, S>,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(), VMStatus> {
        let ctx = session
            .get_native_extensions()
            .get_mut::<NativeCodeContext>();

        if let Some(PublishRequest {
            destination,
            bundle,
            expected_modules,
            allowed_deps,
            check_compat,
        }) = ctx.requested_module_bundle.take()
        {
            // TODO: unfortunately we need to deserialize the entire bundle here to handle
            // `init_module` and verify some deployment conditions, while the VM need to do
            // the deserialization again. Consider adding an API to MoveVM which allows to
            // directly pass CompiledModule.
            let modules = self.deserialize_module_bundle(&bundle)?;

            // Validate the module bundle
            self.validate_publish_request(&modules, expected_modules, allowed_deps)?;

            // Publish the bundle
            if check_compat {
                session.publish_module_bundle(bundle.into_inner(), destination, gas_meter)?
            } else {
                session.publish_module_bundle_relax_compatibility(
                    bundle.into_inner(),
                    destination,
                    gas_meter,
                )?
            }

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Validate a publish request.
    fn validate_publish_request(
        &self,
        modules: &[CompiledModule],
        mut expected_modules: BTreeSet<String>,
        allowed_deps: Option<BTreeMap<AccountAddress, BTreeSet<String>>>,
    ) -> VMResult<()> {
        for m in modules {
            if !expected_modules.remove(m.self_id().name().as_str()) {
                return Err(Self::metadata_validation_error(&format!(
                    "unregistered module: '{}'",
                    m.self_id().name()
                )));
            }
            if let Some(allowed) = &allowed_deps {
                for dep in m.immediate_dependencies() {
                    if !allowed
                        .get(dep.address())
                        .map(|modules| {
                            modules.contains("") || modules.contains(dep.name().as_str())
                        })
                        .unwrap_or(false)
                    {
                        return Err(Self::metadata_validation_error(&format!(
                            "unregistered dependency: '{}'",
                            dep
                        )));
                    }
                }
            }
        }
        if !expected_modules.is_empty() {
            return Err(Self::metadata_validation_error(
                "not all registered modules published",
            ));
        }
        Ok(())
    }

    fn metadata_validation_error(msg: &str) -> VMError {
        PartialVMError::new(StatusCode::CONSTRAINT_NOT_SATISFIED)
            .with_message(format!("metadata and code bundle mismatch: {}", msg))
            .finish(Location::Undefined)
    }
}

pub(crate) fn discard_error_output(err: StatusCode, gas_used: Gas) -> MessageOutput {
    info!("discard error output: {:?}", err);
    // Since this message will be discarded, no writeset will be included.
    MessageOutput::new(
        ChangeSet::new(),
        vec![],
        TableChangeSet::default(),
        AccountSizeChangeSet::default(),
        TableMetaChangeSet::default(),
        gas_used.into(),
        MessageStatus::Discard(err),
    )
}

pub(crate) fn discard_error_vm_status(err: VMStatus, gas_used: Gas) -> (VMStatus, MessageOutput) {
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
    session_output: SessionOutput,
    table_owner_change_set: Option<TableMetaChangeSet>,
    gas_used: Gas,
    status: KeptVMStatus,
) -> Result<MessageOutput, VMStatus> {
    let (change_set, events, table_change_set, account_size_changes, _) = session_output;

    Ok(MessageOutput::new(
        change_set,
        events,
        table_change_set,
        account_size_changes,
        table_owner_change_set.unwrap_or_default(),
        gas_used.into(),
        MessageStatus::Keep(status),
    ))
}
