use anyhow::Result;
use log::info;
use move_deps::move_core_types::{
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
    move_core_types::{account_address::AccountAddress, vm_status::StatusCode},
    move_vm_runtime::{
        move_vm::MoveVM,
        native_extensions::NativeContextExtensions,
        session::{SerializedReturnValues, Session},
    },
    move_vm_types::gas::UnmeteredGasMeter,
};
use nova_stdlib::{
    compile_move_nursery_modules, compile_move_stdlib_modules, compile_nova_stdlib_modules,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use nova_gas::AbstractValueSizeGasParameters;
use nova_gas::{Gas, InitialGasSchedule, NativeGasParameters, NovaGasMeter, NovaGasParameters};
use nova_natives::all_natives;
use nova_natives::{
    block::BlockInfoResolver,
    block::NativeBlockContext,
    code::{NativeCodeContext, PublishRequest},
    table::{NativeTableContext, TableResolver},
};
use nova_storage::{
    size::size_resolver::SizeResolver, state_view::StateView, state_view_impl::StateViewImpl,
    table_meta::table_meta_resolver::TableMetaResolver, table_view::TableView,
    table_view_impl::TableViewImpl,
};
use nova_types::{
    errors::NovaVMError,
    message::{Message, MessageOutput, MessagePayload, MessageStatus},
    module::ModuleBundle,
    size_change_set::SizeChangeSet,
    write_set::WriteSet,
};

use crate::{
    arguments::validate_combine_signer_and_txn_args,
    session::{empty_session_output, SessionExt, SessionOutput},
};

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NovaVM {
    move_vm: Arc<MoveVM>,
    gas_params: NovaGasParameters,
}

impl NovaVM {
    pub fn new() -> Self {
        let gas_params = NativeGasParameters::initial();
        let abs_val_size_gas_params = AbstractValueSizeGasParameters::initial();
        let inner = MoveVM::new(all_natives(
            gas_params.move_stdlib,
            gas_params.nova_stdlib,
            gas_params.table,
            abs_val_size_gas_params,
        ))
        .expect("should be able to create Move VM; check if there are duplicated natives");

        Self {
            move_vm: Arc::new(inner),
            gas_params: NovaGasParameters::initial(),
        }
    }

    fn create_session<'r, S: MoveResolver + SizeResolver + TableMetaResolver, T: TableResolver>(
        &self,
        resolver: &'r S,
        table_resolver: &'r mut T,
        session_id: Vec<u8>,
    ) -> SessionExt<'r, '_, S> {
        let mut extensions = NativeContextExtensions::default();
        let txn_hash: [u8; 32] = session_id
            .try_into()
            .expect("HashValue should convert to [u8; 32]");
        extensions.add(NativeTableContext::new(txn_hash, table_resolver));
        extensions.add(NativeCodeContext::default());

        self.move_vm.flush_loader_cache_if_invalidated();
        SessionExt::new(
            self.move_vm
                .new_session_with_extensions(resolver, extensions),
            resolver,
        )
    }

    fn create_session_with_api<
        'r,
        S: MoveResolver + SizeResolver + TableMetaResolver,
        T: TableResolver,
        A: BlockInfoResolver,
    >(
        &self,
        resolver: &'r S,
        table_resolver: &'r mut T,
        api: &'r A,
        session_id: Vec<u8>,
    ) -> SessionExt<'r, '_, S> {
        let mut session = self.create_session(resolver, table_resolver, session_id);
        session
            .get_native_extensions()
            .add(NativeBlockContext::new(api));
        session
    }

    pub fn initialize<S: StateView, T: TableView>(
        &mut self,
        state_view_impl: &StateViewImpl<'_, S>,
        table_view_impl: &mut TableViewImpl<'_, T>,
        custom_module_bundle: Option<ModuleBundle>,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), NovaVMError> {
        // publish move_stdlib and nova_stdlib modules
        let mut compiled_modules = compile_move_stdlib_modules();
        compiled_modules.append(&mut compile_move_nursery_modules());
        compiled_modules.append(&mut compile_nova_stdlib_modules());

        if let Some(module_bundle) = custom_module_bundle {
            let custom_modules = self
                .deserialize_module_bundle(&module_bundle)
                .map_err(|e| e.into_vm_status())?;
            compiled_modules.extend(custom_modules.into_iter());
        }

        let mut session = self.create_session(state_view_impl, table_view_impl, vec![0; 32]);
        let modules = Modules::new(&compiled_modules);
        let dep_graph = modules.compute_dependency_graph();
        let mut addr: Option<AccountAddress> = None;
        let modules = dep_graph
            .compute_topological_order()
            .unwrap()
            .map(|m| {
                addr = Some(*m.self_id().address());
                let mut bytes = vec![];
                m.serialize(&mut bytes).unwrap();
                bytes
            })
            .collect::<Vec<Vec<u8>>>();

        session
                .publish_module_bundle(modules, addr.unwrap(), &mut UnmeteredGasMeter)
                .map_err(|e| {
                    self.move_vm.mark_loader_cache_as_invalid();
                    println!("[VM] publish_module error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    NovaVMError::from(e.into_vm_status())
                })?;

        self.move_vm.mark_loader_cache_as_invalid();

        let session_output = session.finish()?;

        let output = get_message_output(session_output, Gas::zero(), KeptVMStatus::Executed)
            .map_err(|e| NovaVMError::from(e))?;
        Ok((VMStatus::Executed, output, None))
    }

    pub fn execute_message<S: StateView, T: TableView, A: BlockInfoResolver>(
        &mut self,
        msg: Message,
        state_view_impl: &StateViewImpl<'_, S>,
        table_view_impl: &mut TableViewImpl<'_, T>,
        api: Option<&A>,
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
                let api = match api {
                    Some(api) => Ok(api),
                    None => Err(NovaVMError::generic_err("need BlockInfoResolver")),
                }?;

                self.execute_script_or_entry_function(
                    msg.session_id().to_vec(),
                    sender,
                    state_view_impl,
                    table_view_impl,
                    api,
                    payload,
                    &mut gas_meter,
                )
            }
            MessagePayload::ModuleBundle(m) => match sender {
                Some(sender) => self.publish_module_bundle(
                    msg.session_id().to_vec(),
                    sender,
                    state_view_impl,
                    table_view_impl,
                    m,
                    &mut gas_meter,
                ),
                None => return Err(NovaVMError::generic_err("sender unset")),
            },
        };

        // Charge gas for error handling
        let gas_used = gas_limit.checked_sub(gas_meter.balance()).unwrap();

        match result {
            Ok(status_and_output) => Ok(status_and_output),
            Err(err) => {
                let txn_status = MessageStatus::from(err.clone());

                let (status, message_output) = match txn_status.is_discarded() {
                    true => discard_error_vm_status(err, gas_used),
                    false => self.failed_message_cleanup(err, gas_used),
                };

                Ok((status, message_output, None))
            }
        }
    }

    fn publish_module_bundle<S: StateView, T: TableView>(
        &self,
        session_id: Vec<u8>,
        sender: AccountAddress,
        state_view_impl: &StateViewImpl<'_, S>,
        table_view_impl: &mut TableViewImpl<'_, T>,
        modules: &ModuleBundle,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session = self.create_session(state_view_impl, table_view_impl, session_id);

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
        let (status, output) = self.success_message_cleanup(session_output, gas_meter)?;
        Ok((status, output, None))
    }

    fn execute_script_or_entry_function<S: StateView, T: TableView, A: BlockInfoResolver>(
        &self,
        session_id: Vec<u8>,
        sender: Option<AccountAddress>,
        state_view_impl: &StateViewImpl<'_, S>,
        table_view_impl: &mut TableViewImpl<'_, T>,
        api: &A,
        payload: &MessagePayload,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput, Option<SerializedReturnValues>), VMStatus> {
        let mut session =
            self.create_session_with_api(state_view_impl, table_view_impl, api, session_id.clone());

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
                    let args = validate_combine_signer_and_txn_args(&session, senders, entry_fn.args().to_vec(), &function)?;

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

        let session_output = session.finish()?;

        // Charge for gas cost for write set ops
        gas_meter.charge_write_set_gas(&session_output.1)?;
        let (status, output) = self.success_message_cleanup(session_output, gas_meter)?;

        Ok((status, output, res.into()))
    }

    fn success_message_cleanup(
        &self,
        session_output: SessionOutput,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(VMStatus, MessageOutput), VMStatus> {
        let gas_limit = gas_meter.gas_limit();
        let gas_used = gas_limit.checked_sub(gas_meter.balance()).unwrap();
        Ok((
            VMStatus::Executed,
            get_message_output(session_output, gas_used, KeptVMStatus::Executed)?,
        ))
    }

    fn failed_message_cleanup(
        &self,
        error_code: VMStatus,
        gas_used: Gas,
    ) -> (VMStatus, MessageOutput) {
        match MessageStatus::from(error_code.clone()) {
            MessageStatus::Keep(status) => {
                let txn_output = get_message_output(empty_session_output(), gas_used, status)
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
    fn resolve_pending_code_publish<'r, S: MoveResolver>(
        &self,
        session: &mut Session<'r, '_, S>,
        gas_meter: &mut NovaGasMeter,
    ) -> Result<(), VMStatus> {
        let ctx = session
            .get_native_extensions()
            .get_mut::<NativeCodeContext>();

        if let Some(PublishRequest {
            destination,
            modules,
            expected_modules,
            allowed_deps,
            check_compat,
        }) = ctx.requested_module_bundle.take()
        {
            // TODO: unfortunately we need to deserialize the entire bundle here to handle
            // `init_module` and verify some deployment conditions, while the VM need to do
            // the deserialization again. Consider adding an API to MoveVM which allows to
            // directly pass CompiledModule.
            let bundle = ModuleBundle::new(modules);
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
        vec![],
        WriteSet::default(),
        SizeChangeSet::default(),
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
    gas_used: Gas,
    status: KeptVMStatus,
) -> Result<MessageOutput, VMStatus> {
    let (events, write_set, size_change_set) = session_output;

    Ok(MessageOutput::new(
        events,
        write_set,
        size_change_set,
        gas_used.into(),
        MessageStatus::Keep(status),
    ))
}
