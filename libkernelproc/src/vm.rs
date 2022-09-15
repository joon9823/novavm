use crate::Db;
use crate::storage::Storage;
use crate::error::Error;
use crate::GoStorage;

use kernelvm::EntryFunction;
use kernelvm::ModuleBundle;
use kernelvm::storage::data_view_resolver::DataViewResolver;
use kernelvm::KernelVM;
use kernelvm::Message;
use kernelvm::gas_meter::Gas;

use move_deps::move_core_types::account_address::AccountAddress;
use move_deps::move_core_types::effects::Op;
use move_deps::move_core_types::vm_status::VMStatus;

use once_cell::sync::Lazy;

static mut INSTANCE: Lazy<KernelVM> = Lazy::new(|| KernelVM::new());

pub(crate) fn initialize_vm(module_bundle: Vec<u8>, db_handle: Db) -> Result<Vec<u8>, Error> {
    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = unsafe { INSTANCE.initialize(module_bundle, &data_view) }.unwrap();
    let gas_used: u64 = 0;

    match status {
        VMStatus::Executed => {
            for (addr, cset) in output.change_set().accounts() {
                for (id, module) in cset.modules() {
                    match module {
                        Op::New(v) | Op::Modify(v) => { storage.set(id.as_bytes(), v.as_ref())  },
                        Op::Delete => { storage.remove(id.as_bytes()) }
                    }.0?;
                }
            }
            Ok(Vec::from(status.to_string()))
        },
        _ => { Err(Error::vm_err("failed to intitialize"))}
    }
}

pub(crate) fn publish_module(sender: AccountAddress, payload: Vec<u8>, db_handle: Db, gas: u64) -> Result<Vec<u8>, Error>{
    let gas_limit = Gas::new(gas);

    let module: ModuleBundle = serde_json::from_slice(payload.as_slice()).unwrap();
    let message: Message = Message::new_module(sender, module);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };

    match status {
        VMStatus::Executed => {
            for (addr, cset) in output.change_set().accounts() {
                for (id, module) in cset.modules() {
                    let (res, gas_used) = match module {
                        Op::New(v) | Op::Modify(v) => { storage.set(id.as_bytes(), v.as_ref())  },
                        Op::Delete => { storage.remove(id.as_bytes()) }
                    };
                    // TODO: deduct gas
                    res?;
                }
            }
            // FIXME: TBD whether return retval or not
            Ok(Vec::from(status.to_string()))
        },
        _ => { Err(Error::vm_err("failed to intitialize"))}
    }
}

pub(crate) fn execute_contract(sender: AccountAddress, payload: Vec<u8>, db_handle: Db, gas: u64) -> Result<Vec<u8>, Error> {
    execute_entry(sender, payload, db_handle, gas, false)
}

// works as smart query
pub(crate) fn query_contract(sender: AccountAddress, payload: Vec<u8>, db_handle: Db, gas: u64) -> Result<Vec<u8>, Error> {
    execute_entry(sender, payload, db_handle, gas, true)
}

fn execute_entry(sender: AccountAddress, payload: Vec<u8>, db_handle: Db, gas: u64, is_read_only: bool) -> Result<Vec<u8>, Error> {
    let gas_limit = Gas::new(gas);

    let entry: EntryFunction = serde_json::from_slice(payload.as_slice()).unwrap();
    let message: Message = Message::new_entry_function(sender, entry);

    //let cv = CosmosView::new(&db_handle);
    let mut storage = GoStorage::new(db_handle);
    let data_view = DataViewResolver::new(&storage);

    let (status, output, retval) = unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };

    match status {
        VMStatus::Executed => {
            if is_read_only {
                return Ok(Vec::from(status.to_string()))
            }
            for (addr, cset) in output.change_set().accounts() {
                for (id, module) in cset.modules() {
                    let (res, gas_used) = match module {
                        Op::New(v) | Op::Modify(v) => { storage.set(id.as_bytes(), v.as_ref())  },
                        Op::Delete => { storage.remove(id.as_bytes()) }
                    };
                    // TODO: deduct gas
                    res?;
                }
                 for (id, module) in cset.resources() {
                    let (res, gas_used) = match module {
                        Op::New(v) | Op::Modify(v) => { storage.set(&Vec::from(id.to_string()), v.as_ref())  },
                        Op::Delete => { storage.remove(&Vec::from(id.to_string())) }
                    };
                    // TODO: deduct gas
                    res?;
                }
            }
            
            // FIXME: TBD whether return retval or not
            Ok(Vec::from(status.to_string()))
        },
        _ => { Err(Error::vm_err("failed to intitialize"))}
    }
}
