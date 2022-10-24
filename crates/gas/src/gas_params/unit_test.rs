// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_deps::move_core_types::gas_algebra::{InternalGas, InternalGasPerArg};

#[derive(Debug, Clone)]
pub struct CreateSignersForTestingGasParameters {
    pub base_cost: InternalGas,
    pub unit_cost: InternalGasPerArg,
}

#[derive(Debug, Clone)]
pub struct SetBlockInfoForTestingGasParameters {
    pub base_cost: InternalGas,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub create_signers_for_testing: CreateSignersForTestingGasParameters,
    pub set_block_info_for_testing: SetBlockInfoForTestingGasParameters,
}
