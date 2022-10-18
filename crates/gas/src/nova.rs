use nova_natives::GasParameters;

crate::natives::define_gas_parameters_for_natives!(GasParameters, "nova", [
    [.account.create_address.base_cost, "account.create_address.base", 300],
    [.account.create_signer.base_cost, "account.create_signer.base", 300],

    [.block.get_block_info.base_cost, "block.get_block_info.base", 500],

    [.type_info.type_of.base, "type_info.type_of.base", 300],
    [.type_info.type_of.unit, "type_info.type_of.unit", 5],
    [.type_info.type_name.base, "type_info.type_name.base", 300],
    [.type_info.type_name.unit, "type_info.type_name.unit", 5],

    [.util.from_bytes.base, "util.from_bytes.base", 300],
    [.util.from_bytes.unit, "util.from_bytes.unit", 5],


    [.code.request_publish.base, "code.request_publish.base", 500],
    [.code.request_publish.unit, "code.request_publish.unit", 2],

    // TODO(Gas): these should only be enabled when feature "testing" is present
    // TODO(Gas): rename these in the move repo
    [test_only .unit_test.create_signers_for_testing.base_cost, "unit_test.create_signers_for_testing.base", 1],
    [test_only .unit_test.create_signers_for_testing.unit_cost, "unit_test.create_signers_for_testing.unit", 1],
    [test_only .unit_test.set_block_info_for_testing.base_cost, "unit_test.set_block_info_for_testing.base", 1]
]);
