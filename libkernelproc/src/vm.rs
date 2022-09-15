use std::sync::Arc;

use crate::{gas_meter, Db};
use crate::view::CosmosView;

use move_deps::move_core_types::{account_address::AccountAddress};
use kernelvm::{EntryFunction, ModuleBundle};
use kernelvm::Module;
use kernelvm::storage::data_view_resolver::DataViewResolver;
use kernelvm::KernelVM;
use kernelvm::Message;
use kernelvm::gas_meter::Gas;

use once_cell::sync::Lazy;

struct ExecutionResult {
    
}

static mut INSTANCE: Lazy<KernelVM> = Lazy::new(|| KernelVM::new());

fn publish_module(sender: AccountAddress, payload: Vec<u8>, db_handle: Db, gas: u64) -> ExecutionResult {
    let gas_limit = Gas::new(gas);

    let module: Module = serde_json::from_slice(payload.as_slice()).unwrap();
    let bundle: ModuleBundle = ModuleBundle::from(module);
    let message: Message = Message::new_module(sender, bundle);

	let cv = CosmosView::new(db_handle);
	let data_view = DataViewResolver::new(&cv);

    let (status, output, retval) = unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };
    // TODO handle results
    ExecutionResult {  } // just stub
}

fn execute_contract(sender: AccountAddress, payload: Vec<u8>, db_handle: Db, gas: u64) -> ExecutionResult {
    let gas_limit = Gas::new(gas);

    let entry: EntryFunction = serde_json::from_slice(payload.as_slice()).unwrap();
    let message: Message = Message::new_entry_function(sender, entry);

    let cv = CosmosView::new(db_handle);
    let data_view = DataViewResolver::new(&cv);


    let (status, output, retval) = unsafe { INSTANCE.execute_message(message, &data_view, gas_limit) };
    // TODO handle results
    ExecutionResult {  } // just stub
}