use crate::mocks::{BlankTableViewImpl, MockApi};
use move_deps::move_unit_test;
use move_deps::move_vm_runtime::native_extensions::NativeContextExtensions;
use nova_natives::{block::NativeBlockContext, code::NativeCodeContext, table::NativeTableContext};

static mut BLANK_TABLE_RESOLVER: BlankTableViewImpl = BlankTableViewImpl;

pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    exts.add(NativeCodeContext::default());
    exts.add(NativeTableContext::new([0; 32], unsafe {
        &mut BLANK_TABLE_RESOLVER
    }));
    exts.add(NativeBlockContext::new(&MockApi {
        height: 0,
        timestamp: 0,
    }));
}
