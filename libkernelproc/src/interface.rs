use crate::{UnmanagedVector, vm, Db, ByteSliceView};

// VM initializer
#[no_mangle]
pub extern "C" fn initialize(db: Db, module_bundle: ByteSliceView) -> UnmanagedVector{
    let retval = UnmanagedVector::new(None);

    // TODO: call vm::initialize
    //vm::initialize_vm(module_bundle, db_handle).unwrap_or_else(|| {});

    retval
}

/// exported function to publish a module
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn publish_module(db: Db, sender: ByteSliceView, module_bundle: ByteSliceView) -> UnmanagedVector{
    let retval = UnmanagedVector::new(None);

    // TODO: call vm::publish_module()
    //vm::publish_module(sender, payload, db_handle, gas).unwrap_or_else(||{});

    retval
}

// exported function to execute (an entrypoint of) contract
/// TODO: wrap sender after PoC: make Context including sender, funds and other contextual information
#[no_mangle]
pub extern "C" fn execute_contract(db: Db, sender: ByteSliceView, message: ByteSliceView) -> UnmanagedVector {
    let retval = UnmanagedVector::new(None);

    // TODO: call vm::execute_contract()
    //vm::execute_contract(sender, payload, db_handle, gas).unwrap_or_else(||{});

    retval
}