use move_deps::move_core_types::gas_algebra::{InternalGas, InternalGasPerByte};

#[derive(Clone, Debug)]
pub struct RequestPublishGasParameters {
    pub base: InternalGas,
    pub unit: InternalGasPerByte,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub request_publish: RequestPublishGasParameters,
}
