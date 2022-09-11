use std::sync::Arc;

use crate::gas_meter;
use kernelvm::vm::KernelVM;
use kernelvm::MessageOutput;

use once_cell::sync::Lazy;




static INSTANCE: Lazy<Arc<KernelVM>> = Lazy::new(|| Arc::new(KernelVM::new()));

/*  TODO: put VM into pool
static pool_capacity: i32 = 1;
static pool_extra: i32 = 0;
use pool::Pool;

static VM_POOL : Lazy<Pool<KernelVM>> = Lazy::new(|| {
    Pool::with_capacity(pool_capacity, pool_extra, || KernelVM::new())
});
*/

pub fn publish_module() -> MessageOutput {
    _ = INSTANCE.execute_message(msg, remote_cache, gas_limit);
}

pub fn execute_entry_function() -> MessageOutput {
    _ = INSTANCE.execute_message(msg, remote_cache, gas_limit);
}