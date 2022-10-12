use crate::api::GoApi;
use crate::error::Error;
use crate::result::generate_result;
use crate::result::to_vec;
use crate::storage::Storage;
use crate::Db;
use crate::GoStorage;

use novavm::access_path::AccessPath;
use novavm::gas::Gas;
use novavm::natives::table::TableChangeSet;
use novavm::storage::data_view_resolver::DataViewResolver;
use novavm::table_meta::TableMetaChangeSet;
use novavm::table_meta::TableMetaType;
use novavm::BackendResult;
use novavm::Message;
use novavm::ModuleBundle;
use novavm::NovaVM;
use novavm::{EntryFunction, Script};

use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_core_types::effects::ChangeSet;
use move_deps::move_core_types::effects::Op;
use move_deps::move_core_types::language_storage::ModuleId;
use move_deps::move_core_types::vm_status::VMStatus;

pub(crate) fn initialize_vm(vm: &mut NovaVM, db_handle: Db, payload: &[u8]) -> Result<(), Error> {
    let mut storage = GoStorage::new(db_handle);

    // add passed custom module bundles
    let custom_module_bundle: ModuleBundle = serde_json::from_slice(payload).unwrap();

    let data_view = DataViewResolver::new(&storage);
    let (status, output, _retval) = vm
        .initialize(&data_view, Some(custom_module_bundle))
        .unwrap();

    match status {
        VMStatus::Executed => {
            push_write_set(
                &mut storage,
                output.change_set(),
                output.table_change_set(),
                output.table_meta_change_set(),
            )?;
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

    let module_bundle: ModuleBundle = serde_json::from_slice(payload).unwrap();
    let sorted_module_bundle = module_bundle.sorted_code_and_modules();

    let message: Message = Message::new_module(session_id, Some(sender), sorted_module_bundle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = vm
        .execute_message::<GoStorage, GoApi>(message, &data_view, None, gas_limit)
        .map_err(|e| Error::from(e))?;

    match status {
        VMStatus::Executed => {
            push_write_set(
                &mut storage,
                output.change_set(),
                output.table_change_set(),
                output.table_meta_change_set(),
            )?;

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

    let ef: EntryFunction = serde_json::from_slice(payload.as_slice()).unwrap();
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
                push_write_set(
                    &mut storage,
                    output.change_set(),
                    output.table_change_set(),
                    output.table_meta_change_set(),
                )?;
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

pub fn push_write_set(
    go_storage: &mut GoStorage,
    changeset: &ChangeSet,
    table_change_set: &TableChangeSet,
    table_meta_change_set: &TableMetaChangeSet,
) -> BackendResult<()> {
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

    for (handle, change) in &table_change_set.changes {
        for (key, val) in &change.entries {
            let ap = AccessPath::table_item_access_path(handle.0, key.to_vec());
            write_op(go_storage, &ap, &val)?;
        }
    }

    for (handle, op) in &table_meta_change_set.owner {
        let ap = AccessPath::table_meta_access_path(handle.0, TableMetaType::Owner);
        write_op(go_storage, &ap, &op)?;
    }

    for (handle, op) in &table_meta_change_set.size {
        let ap = AccessPath::table_meta_access_path(handle.0, TableMetaType::Size);
        write_op(go_storage, &ap, &op)?;
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

    let script: Script = serde_json::from_slice(payload.as_slice()).unwrap();
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
                push_write_set(
                    &mut storage,
                    output.change_set(),
                    output.table_change_set(),
                    output.table_meta_change_set(),
                )?;
            }

            let res = generate_result(status, output, retval, is_query)?;
            to_vec(&res)
        }
        _ => Err(Error::from(status)),
    }
}
