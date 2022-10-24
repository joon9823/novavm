use crate::InternalGasPerAbstractValueUnit;
use move_deps::move_core_types::gas_algebra::InternalGas;

#[derive(Debug, Clone)]
pub struct WriteToEventStoreGasParameters {
    pub base: InternalGas,
    pub per_abstract_value_unit: InternalGasPerAbstractValueUnit,
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub write_to_event_store: WriteToEventStoreGasParameters,
}
