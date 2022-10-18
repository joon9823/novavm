use nova_natives::GasParameters;

crate::natives::define_gas_parameters_for_natives!(GasParameters, "nova", [
    [.account.create_address.base_cost, "account.create_address.base", 1],
    [.account.create_signer.base_cost, "account.create_signer.base", 1],

    [.block.get_block_info.base_cost, "block.get_block_info.base", 1],

    [.type_info.type_of.base, "type_info.type_of.base", 1],
    [.type_info.type_of.unit, "type_info.type_of.unit", 1],
    [.type_info.type_name.base, "type_info.type_name.base", 1],
    [.type_info.type_name.unit, "type_info.type_name.unit", 1],

    [.util.from_bytes.base, "util.from_bytes.base", 1],
    [.util.from_bytes.unit, "util.from_bytes.unit", 1],

    [.code.request_publish.base, "code.request_publish.base", 1],
    [.code.request_publish.unit, "code.request_publish.unit", 1],

    // TODO(Gas): these should only be enabled when feature "testing" is present
    // TODO(Gas): rename these in the move repo
    [test_only .unit_test.create_signers_for_testing.base_cost, "unit_test.create_signers_for_testing.base", 1],
    [test_only .unit_test.create_signers_for_testing.unit_cost, "unit_test.create_signers_for_testing.unit", 1],
    [test_only .unit_test.set_block_info_for_testing.base_cost, "unit_test.set_block_info_for_testing.base", 1]
]);
