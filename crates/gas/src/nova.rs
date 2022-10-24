use crate::meter::EXECUTION_GAS_MULTIPLIER as MUL;

crate::natives::define_gas_parameters_for_natives!(GasParameters, "nova", [
    [.account.create_address.base_cost, "account.create_address.base", 300 * MUL],
    [.account.create_signer.base_cost, "account.create_signer.base", 300 * MUL],

    [.block.get_block_info.base_cost, "block.get_block_info.base", 500 * MUL],

    [.type_info.type_of.base, "type_info.type_of.base", 300 * MUL],
    [.type_info.type_of.unit, "type_info.type_of.unit", 5 * MUL],
    [.type_info.type_name.base, "type_info.type_name.base", 300 * MUL],
    [.type_info.type_name.unit, "type_info.type_name.unit", 5 * MUL],

    [.util.from_bytes.base, "util.from_bytes.base", 300 * MUL],
    [.util.from_bytes.unit, "util.from_bytes.unit", 5 * MUL],


    [.code.request_publish.base, "code.request_publish.base", 500 * MUL],
    [.code.request_publish.unit, "code.request_publish.unit", 2 * MUL],

    // Note(Gas): These are storage operations so the values should not be multiplied.
    [.event.write_to_event_store.base, "event.write_to_event_store.base", 500_000],
    // TODO(Gas): the on-chain name is wrong...
    [.event.write_to_event_store.per_abstract_value_unit, "event.write_to_event_store.per_abstract_memory_unit", 5_000],


    // TODO(Gas): these should only be enabled when feature "testing" is present
    // TODO(Gas): rename these in the move repo
    [test_only .unit_test.create_signers_for_testing.base_cost, "unit_test.create_signers_for_testing.base", 1],
    [test_only .unit_test.create_signers_for_testing.unit_cost, "unit_test.create_signers_for_testing.unit", 1],
    [test_only .unit_test.set_block_info_for_testing.base_cost, "unit_test.set_block_info_for_testing.base", 1]
]);

use crate::gas_params::*;

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub account: account::GasParameters,
    pub block: block::GasParameters,
    pub type_info: type_info::GasParameters,
    pub util: util::GasParameters,
    pub code: code::GasParameters,
    pub event: event::GasParameters,
    pub unit_test: unit_test::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            account: account::GasParameters {
                create_address: account::CreateAddressGasParameters {
                    base_cost: 0.into(),
                },
                create_signer: account::CreateSignerGasParameters {
                    base_cost: 0.into(),
                },
            },
            block: block::GasParameters {
                get_block_info: block::GetBlockInfoGasParameters {
                    base_cost: 0.into(),
                },
            },
            type_info: type_info::GasParameters {
                type_of: type_info::TypeOfGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
                type_name: type_info::TypeNameGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
            },
            util: util::GasParameters {
                from_bytes: util::FromBytesGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
            },
            code: code::GasParameters {
                request_publish: code::RequestPublishGasParameters {
                    base: 0.into(),
                    unit: 0.into(),
                },
            },
            event: event::GasParameters {
                write_to_event_store: event::WriteToEventStoreGasParameters {
                    base: 0.into(),
                    per_abstract_value_unit: 0.into(),
                },
            },
            unit_test: unit_test::GasParameters {
                create_signers_for_testing: unit_test::CreateSignersForTestingGasParameters {
                    base_cost: 0.into(),
                    unit_cost: 0.into(),
                },
                set_block_info_for_testing: unit_test::SetBlockInfoForTestingGasParameters {
                    base_cost: 0.into(),
                },
            },
        }
    }
}
