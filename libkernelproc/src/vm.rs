use crate::error::Error;
use crate::storage::Storage;
use crate::Db;
use crate::GoStorage;

use kernelvm::Module;
use kernelvm::access_path::AccessPath;
use kernelvm::gas_meter::Gas;
use kernelvm::storage::data_view_resolver::DataViewResolver;
use kernelvm::BackendResult;
use kernelvm::EntryFunction;
use kernelvm::GasInfo;
use kernelvm::KernelVM;
use kernelvm::Message;
use kernelvm::ModuleBundle;

use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_core_types::effects::ChangeSet;
use move_deps::move_core_types::effects::Op;
use move_deps::move_core_types::language_storage::ModuleId;
use move_deps::move_core_types::vm_status::VMStatus;

use once_cell::sync::Lazy;

static mut INSTANCE: Lazy<KernelVM> = Lazy::new(|| KernelVM::new());

pub(crate) fn initialize_vm(module_bundle: Vec<u8>, db_handle: Db) -> Result<Vec<u8>, Error> {
    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, _retval) =
        unsafe { INSTANCE.initialize(module_bundle, &data_view) }.unwrap();
    // let gas_used: u64 = 0;

    match status {
        VMStatus::Executed => {
            let (res, _gas_info) = push_write_set(&mut storage, output.change_set());
            // TODO: deduct gas
            res?;

            Ok(Vec::from(status.to_string()))
        }
        _ => Err(Error::vm_err("failed to initialize")),
    }
}

pub(crate) fn publish_module(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    let gas_limit = Gas::new(gas);

    let module: ModuleBundle = ModuleBundle::from(Module::new(payload));
    let message: Message = Message::new_module(sender, module);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, _retval) =
        unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };

    match status {
        VMStatus::Executed => {
            let (res, _gas_info) = push_write_set(&mut storage, output.change_set());
            // TODO: deduct gas
            res?;

            // FIXME: TBD whether return retval or not
            Ok(Vec::from(status.to_string()))
        }
        _ => Err(Error::vm_err("failed to publish")),
    }
}

pub(crate) fn execute_script(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry(sender, payload, db_handle, gas, false)
}

pub(crate) fn execute_contract(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry(sender, payload, db_handle, gas, false)
}

// works as smart query
pub(crate) fn query_contract(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry(sender, payload, db_handle, gas, true)
}

fn execute_entry(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
    is_read_only: bool,
) -> Result<Vec<u8>, Error> {
    let gas_limit = Gas::new(gas);

    let entry: EntryFunction = serde_json::from_slice(payload.as_slice()).unwrap();
    let message: Message = Message::new_entry_function(sender, entry);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) =
        unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };

    match status {
        VMStatus::Executed => {
            if is_read_only {
                return match retval {
                    // FIXME: retval or output?
                    Some(val) => {
                        // allow only single return values
                        if Vec::len(&val.mutable_reference_outputs) == 0
                            && Vec::len(&val.return_values) == 1
                        {
                            // ignore _move_type_layout
                            // a client should handle deserialize
                            let (blob, _move_type_layout) = val.return_values.first().unwrap();
                            Ok(blob.to_vec())
                        } else {
                            Err(Error::vm_err("only one value is allowed to be returned."))
                        }
                    }
                    None => Ok(Vec::from("no data")),
                };
            }

            let (res, _gas_info) = push_write_set(&mut storage, output.change_set());
            // TODO: deduct gas
            res?;

            // FIXME: TBD whether return retval or not
            Ok(Vec::from(status.to_string()))
        }
        _ => Err(Error::vm_err("failed to execute")),
    }
}

fn write_op(
    go_storage: &mut GoStorage,
    ap: &AccessPath,
    blob_opt: &Op<Vec<u8>>,
) -> BackendResult<()> {
    match blob_opt {
        Op::New(blob) | Op::Modify(blob) => go_storage.set(ap.to_string().as_bytes(), &blob),
        Op::Delete => go_storage.remove(ap.to_string().as_bytes()),
    }
}

pub fn push_write_set(go_storage: &mut GoStorage, changeset: &ChangeSet) -> BackendResult<()> {
    let mut used_gas = 0u64;
    for (addr, account_changeset) in changeset.accounts() {
        for (struct_tag, blob_opt) in account_changeset.resources() {
            let ap = AccessPath::resource_access_path(addr.clone(), struct_tag.clone());
            let (res, gas_info) = write_op(go_storage, &ap, &blob_opt);
            if res.is_err() {
                return (res, gas_info);
            }

            used_gas += gas_info.externally_used
        }

        for (name, blob_opt) in account_changeset.modules() {
            let ap = AccessPath::from(&ModuleId::new(addr.clone(), name.clone()));
            let (res, gas_info) = write_op(go_storage, &ap, &blob_opt);
            if res.is_err() {
                return (res, gas_info);
            }

            used_gas += gas_info.externally_used
        }
    }

    (Ok(()), GasInfo::with_externally_used(used_gas))
}
