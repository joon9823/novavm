use std::sync::Arc;

use pool::Pool;

use kernelvm::vm::KernelVM;


pub fn create_vm_pool(cap: usize) -> Arc<Pool<KernelVM>> {
   // Arc::new(Pool::new(cap, || KernelVM::new()))
    Arc::new(Pool::with_capacity(cap, 0, || KernelVM::new()))
}