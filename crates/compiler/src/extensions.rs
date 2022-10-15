use once_cell::sync::Lazy;
use move_deps::move_unit_test;
use move_deps::move_vm_runtime::native_extensions::NativeContextExtensions;
use nova_natives::{block::NativeBlockContext, code::NativeCodeContext, table::NativeTableContext};
use novavm::test_utils::mock_chain::{BlankStorage, MockApi};

pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

static DUMMY_RESOLVER: Lazy<BlankStorage> = Lazy::new(|| BlankStorage);

fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    exts.add(NativeCodeContext::default());
    exts.add(NativeTableContext::new([0; 32], &*DUMMY_RESOLVER));
    exts.add(NativeBlockContext::new(&MockApi {
        height: 100,
        timestamp: 100,
    }));
}
