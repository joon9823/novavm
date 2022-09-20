use crate::error::Error;
use crate::result::to_vec;
use crate::storage::Storage;
use crate::Db;
use crate::GoStorage;
use crate::result::generate_result;

use kernelvm::access_path::AccessPath;
use kernelvm::asset::{
    compile_kernel_stdlib_modules, compile_move_nursery_modules, compile_move_stdlib_modules,
};
use kernelvm::gas::Gas;
use kernelvm::storage::data_view_resolver::DataViewResolver;
use kernelvm::BackendResult;
use kernelvm::EntryFunction;
use kernelvm::KernelVM;
use kernelvm::Message;
use kernelvm::Module;
use kernelvm::ModuleBundle;

use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_core_types::effects::ChangeSet;
use move_deps::move_core_types::effects::Op;
use move_deps::move_core_types::language_storage::ModuleId;
use move_deps::move_core_types::vm_status::VMStatus;

use once_cell::sync::Lazy;

static mut INSTANCE: Lazy<KernelVM> = Lazy::new(|| KernelVM::new());

pub(crate) fn initialize_vm(db_handle: Db, payload: Vec<u8>) -> Result<Vec<u8>, Error> {
    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);

    // initialize stdlib
    let mut module_bundle: Vec<Vec<u8>> = vec![];
    let mut modules = compile_move_stdlib_modules();
    modules.append(&mut compile_move_nursery_modules());
    modules.append(&mut compile_kernel_stdlib_modules());
    for module in modules {
        let mut mod_blob = vec![];
        module.serialize(&mut mod_blob).unwrap();
        module_bundle.push(mod_blob);
    }

    // add passed custom module bundles
    let custom_module_bundle: ModuleBundle =
        serde_json::from_slice(payload.as_slice()).unwrap();
    module_bundle.extend(custom_module_bundle.into_inner());

    for module in module_bundle {
        let data_view = DataViewResolver::new(&storage);
        let (status, output, _retval) =
            unsafe { INSTANCE.initialize(module, &data_view) }.unwrap();

        match status {
            VMStatus::Executed => {
                push_write_set(&mut storage, output.change_set())?;
            },
            _ => Err(Error::from(status))?
        }
    }

    Ok(Vec::from("ok"))
}

pub(crate) fn publish_module(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    let gas_limit = Gas::new(gas);

    let module: ModuleBundle = ModuleBundle::from(Module::new(payload));
    let message: Message = Message::new_module(Some(sender), module);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) =
        unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };

    match status {
        VMStatus::Executed => {
            push_write_set(&mut storage, output.change_set())?;

            let res = generate_result(status, output, retval, false)?;
            to_vec(&res)
        },
        _ => Err(Error::from(status))
    }
}

pub(crate) fn execute_script(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry(Some(sender), payload, db_handle, gas, false)
}

pub(crate) fn execute_contract(
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry(Some(sender), payload, db_handle, gas, false)
}

// works as smart query
pub(crate) fn query_contract(
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry(None, payload, db_handle, gas, true)
}

fn execute_entry(
    sender: Option<AccountAddress>,
    payload: Vec<u8>,
    db_handle: Db,
    gas: u64,
    is_query: bool,
) -> Result<Vec<u8>, Error> {
    if !is_query && sender.is_none() {
        return Err(Error::unset_arg("sender"));
    }

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
            if !is_query { 
                push_write_set(&mut storage, output.change_set())?;
            }

            let res = generate_result(status, output, retval, is_query)?;
            to_vec(&res)
        },
        _ => Err(Error::from(status))
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
    for (addr, account_changeset) in changeset.accounts() {
        for (struct_tag, blob_opt) in account_changeset.resources() {
            let ap = AccessPath::resource_access_path(addr.clone(), struct_tag.clone());
            write_op(go_storage, &ap, &blob_opt)?;
        }

        for (name, blob_opt) in account_changeset.modules() {
            let ap = AccessPath::from(&ModuleId::new(addr.clone(), name.clone()));
            write_op(go_storage, &ap, &blob_opt)?;
        }
    }
    Ok(())
}

