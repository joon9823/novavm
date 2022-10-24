use crate::api::GoApi;
use crate::error::Error;
use crate::result::generate_result;
use crate::result::to_vec;
use crate::storage::Storage;
use crate::Db;
use crate::GoStorage;

use nova_gas::Gas;
use nova_storage::data_view_resolver::DataViewResolver;
use nova_types::access_path::AccessPath;
use nova_types::write_set::WriteSet;
use nova_types::{
    entry_function::EntryFunction, message::Message, module::ModuleBundle, script::Script,
};
use novavm::BackendResult;
use novavm::NovaVM;

use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_core_types::effects::Op;
use move_deps::move_core_types::vm_status::VMStatus;

pub(crate) fn initialize_vm(vm: &mut NovaVM, db_handle: Db, payload: &[u8]) -> Result<(), Error> {
    let mut storage = GoStorage::new(db_handle);

    // add passed custom module bundles
    let custom_module_bundle: ModuleBundle = bcs::from_bytes(payload).unwrap();

    let data_view = DataViewResolver::new(&storage);
    let (status, output, _retval) = vm
        .initialize(&data_view, Some(custom_module_bundle))
        .unwrap();

    match status {
        VMStatus::Executed => {
            push_write_set(&mut storage, output.write_set())?;
        }
        _ => Err(Error::from(status))?,
    }

    Ok(())
}

pub(crate) fn publish_module_bundle(
    vm: &mut NovaVM,
    session_id: Vec<u8>, // seed for global unique session id
    sender: AccountAddress,
    payload: &[u8],
    db_handle: Db,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    let gas_limit = Gas::new(gas);

    let module_bundle: ModuleBundle = bcs::from_bytes(payload).unwrap();
    let sorted_module_bundle = module_bundle.sorted_code_and_modules();

    let message: Message = Message::new_module(session_id, Some(sender), sorted_module_bundle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = vm
        .execute_message::<GoStorage, GoApi>(message, &data_view, None, gas_limit)
        .map_err(|e| Error::from(e))?;

    match status {
        VMStatus::Executed => {
            push_write_set(&mut storage, output.write_set())?;

            let res = generate_result(status, output, retval, false)?;
            to_vec(&res)
        }
        _ => Err(Error::from(status)),
    }
}

pub(crate) fn execute_script(
    vm: &mut NovaVM,
    session_id: Vec<u8>, // seed for global unique session id
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    api: GoApi,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_script_internal(
        vm,
        session_id,
        Some(sender),
        payload,
        db_handle,
        api,
        gas,
        false,
    )
}

pub(crate) fn execute_contract(
    vm: &mut NovaVM,
    session_id: Vec<u8>, // seed for global unique session id
    sender: AccountAddress,
    payload: Vec<u8>,
    db_handle: Db,
    api: GoApi,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry_function_internal(
        vm,
        session_id,
        Some(sender),
        payload,
        db_handle,
        api,
        gas,
        false,
    )
}

// works as smart query
pub(crate) fn query_contract(
    vm: &mut NovaVM,
    payload: Vec<u8>,
    db_handle: Db,
    api: GoApi,
    gas: u64,
) -> Result<Vec<u8>, Error> {
    execute_entry_function_internal(vm, vec![0; 32], None, payload, db_handle, api, gas, true)
}

/////////////////////////////////////////
/// Entry Function //////////////////////
/////////////////////////////////////////

fn execute_entry_function_internal(
    vm: &mut NovaVM,
    session_id: Vec<u8>, // seed for global unique session id
    sender: Option<AccountAddress>,
    payload: Vec<u8>,
    db_handle: Db,
    api: GoApi,
    gas: u64,
    is_query: bool,
) -> Result<Vec<u8>, Error> {
    if !is_query && sender.is_none() {
        return Err(Error::unset_arg("sender"));
    }

    let gas_limit = Gas::new(gas);

    let ef: EntryFunction = bcs::from_bytes(&payload.to_vec()).unwrap();
    let message: Message = Message::new_entry_function(session_id, sender, ef);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = vm
        .execute_message(message, &data_view, Some(&api), gas_limit)
        .map_err(|e| Error::from(e))?;

    match status {
        VMStatus::Executed => {
            if !is_query {
                push_write_set(&mut storage, output.write_set())?;
            }

            let res = generate_result(status, output, retval, is_query)?;
            to_vec(&res)
        }
        _ => Err(Error::from(status)),
    }
}

/////////////////////////////////////////
/// Storage Operation ///////////////////
/////////////////////////////////////////

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

pub fn push_write_set(go_storage: &mut GoStorage, write_set: &WriteSet) -> BackendResult<()> {
    for (ap, blob_opt) in write_set {
        write_op(go_storage, &ap, blob_opt)?;
    }

    Ok(())
}

/////////////////////////////////////////
/// Script //////////////////////////////
/////////////////////////////////////////

fn execute_script_internal(
    vm: &mut NovaVM,
    session_id: Vec<u8>, // seed for global unique session id
    sender: Option<AccountAddress>,
    payload: Vec<u8>,
    db_handle: Db,
    api: GoApi,
    gas: u64,
    is_query: bool,
) -> Result<Vec<u8>, Error> {
    if !is_query && sender.is_none() {
        return Err(Error::unset_arg("sender"));
    }

    let gas_limit = Gas::new(gas);

    let script: Script = bcs::from_bytes(&payload.to_vec()).unwrap();
    let message: Message = Message::new_script(session_id, sender, script);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = vm
        .execute_message(message, &data_view, Some(&api), gas_limit)
        .map_err(|e| Error::from(e))?;

    match status {
        VMStatus::Executed => {
            if !is_query {
                push_write_set(&mut storage, output.write_set())?;
            }

            let res = generate_result(status, output, retval, is_query)?;
            to_vec(&res)
        }
        _ => Err(Error::from(status)),
    }
}
