use crate::error::Error;
use crate::event::ContractEvent;

use kernelvm::MessageOutput;

use move_deps::move_core_types::vm_status::VMStatus;
use move_deps::move_vm_runtime::session::SerializedReturnValues;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

pub struct ExecutionResult{
    result: Vec<u8>,
    events: Vec<ContractEvent>,
    gas_used: u64,
}

pub fn to_vec<T>(data: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    serde_json::to_vec(data).map_err(|e| Error::invalid_utf8("failed to serialize"))
}

impl Serialize for  ExecutionResult{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("ExecutionResult", 3)?;
        state.serialize_field("result", &self.result)?;
        state.serialize_field("events", &self.events)?;
        state.serialize_field("gas_used", &self.gas_used)?;
        state.end()
    }
}


pub fn generate_result(status: VMStatus, output: MessageOutput, retval: Option<SerializedReturnValues>, is_query: bool) -> Result<ExecutionResult, Error> {

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
        },
        None => Ok(Vec::from(status.to_string()))
    }?;

    let mut events = Vec::new();
    for (guid, sec, ty, dat) in output.events() {
        events.push(ContractEvent::new(guid.clone(), sec.clone(), ty.clone(), dat.clone()));
    }

    Ok(ExecutionResult{result, events, gas_used: output.gas_used()})
}