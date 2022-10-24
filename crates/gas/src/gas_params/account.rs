// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use move_deps::move_core_types::gas_algebra::InternalGas;

#[derive(Debug, Clone)]
pub struct CreateAddressGasParameters {
    pub base_cost: InternalGas,
}
#[derive(Debug, Clone)]
pub struct CreateSignerGasParameters {
    pub base_cost: InternalGas,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub create_address: CreateAddressGasParameters,
    pub create_signer: CreateSignerGasParameters,
}
