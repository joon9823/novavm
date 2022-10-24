// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use move_deps::move_core_types::gas_algebra::InternalGas;

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub unit: InternalGas,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub from_bytes: FromBytesGasParameters,
}
