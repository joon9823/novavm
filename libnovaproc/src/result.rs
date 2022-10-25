use crate::error::Error;
use crate::event::ContractEvent;
use crate::size_delta::SizeDelta;

use nova_types::message::MessageOutput;

use move_deps::move_core_types::vm_status::VMStatus;
use move_deps::move_vm_runtime::session::SerializedReturnValues;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ExecutionResult {
    result: Vec<u8>,
    events: Vec<ContractEvent>,
    size_deltas: Vec<SizeDelta>,
    gas_used: u64,
}

#[allow(dead_code)]
pub fn to_vec<T>(data: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    bcs::to_bytes(data).map_err(|_| Error::invalid_utf8("failed to serialize"))
}

#[allow(dead_code)]
pub fn generate_result(
    status: VMStatus,
    output: MessageOutput,
    retval: Option<SerializedReturnValues>,
    is_query: bool,
) -> Result<ExecutionResult, Error> {
    let result = match retval {
        Some(val) => {
            if !is_query {
                Ok(Vec::from(status.to_string()))
            } else
            // allow only single return values
            if Vec::len(&val.mutable_reference_outputs) == 0
                && Vec::len(&val.return_values) == 1
            {
                // ignore _move_type_layout
                // a client should handle deserialize
                let (blob, _move_type_layout) = val.return_values.first().unwrap();
                Ok(blob.to_vec())
            } else {
                Err(Error::vm_err("only one value is allowed to be returned."))
            }
        }
        None => Ok(Vec::from(status.to_string())),
    }?;

    let mut events = Vec::new();
    for (guid, sec, ty, dat) in output.events() {
        events.push(ContractEvent::new(
            guid.clone(),
            sec.clone(),
            ty.clone(),
            dat.clone(),
        ));
    }

    let mut size_deltas = Vec::new();
    for (account_addr, size_delta) in output.size_change_set().changes() {
        size_deltas.push(SizeDelta::new(
            *account_addr,
            size_delta.amount as u64,
            size_delta.is_decrease,
        ));
    }

    Ok(ExecutionResult {
        result,
        events,
        size_deltas,
        gas_used: output.gas_used(),
    })
}
