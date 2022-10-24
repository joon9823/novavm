// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use move_deps::move_core_types::gas_algebra::{InternalGas, InternalGasPerByte};

#[derive(Debug, Clone)]
pub struct TypeOfGasParameters {
    pub base: InternalGas,
    pub unit: InternalGasPerByte,
}

#[derive(Debug, Clone)]
pub struct TypeNameGasParameters {
    pub base: InternalGas,
    pub unit: InternalGasPerByte,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub type_of: TypeOfGasParameters,
    pub type_name: TypeNameGasParameters,
}
