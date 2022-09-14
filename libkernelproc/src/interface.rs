use crate::UnmanagedVector;

// VM initializer
#[no_mangle]
pub extern "C" fn initialize() -> UnmanagedVector{
    let retval = UnmanagedVector::new(None);

    // TODO: call vm::initialize

    retval
}

/// exported function to publish a module
#[no_mangle]
pub extern "C" fn publish_module() -> UnmanagedVector{
    let retval = UnmanagedVector::new(None);

    // TODO: call vm::publish_module()

    retval
}

#[no_mangle]
// exported function to execute (an entrypoint of) contract
pub extern "C" fn execute_contract() -> UnmanagedVector {
    let retval = UnmanagedVector::new(None);

    // TODO: call vm::execute_contract()

    retval
}