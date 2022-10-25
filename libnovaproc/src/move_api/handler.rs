use crate::move_api::convert::MoveConverter;
use crate::move_api::move_types::{MoveModuleBytecode, MoveScriptBytecode};
use crate::{error::Error, Db, GoStorage};

use nova_storage::state_view_impl::StateViewImpl;

pub(crate) fn decode_move_resource(
    db_handle: Db,
    struct_tag: String,
    data_bytes: &[u8],
) -> Result<Vec<u8>, Error> {
    let storage = GoStorage::new(&db_handle);

    let state_view_impl = StateViewImpl::new(&storage);
    let converter = MoveConverter::new(&state_view_impl);
    let resource = converter
        .try_into_resource(&struct_tag, data_bytes)
        .map_err(|e| Error::BackendFailure { msg: e.to_string() })?;

    // serialize response as json
    serde_json::to_vec(&resource).map_err(|e| Error::BackendFailure { msg: e.to_string() })
}

pub(crate) fn decode_script_bytes(script_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let script: MoveScriptBytecode = MoveScriptBytecode::new(script_bytes);
    let abi = script
        .try_parse_abi()
        .map_err(|e| Error::BackendFailure { msg: e.to_string() })?;

    // serialize response as json
    serde_json::to_vec(&abi).map_err(|e| Error::BackendFailure { msg: e.to_string() })
}

pub(crate) fn decode_module_bytes(module_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    // deserialized request from the json
    let module: MoveModuleBytecode = MoveModuleBytecode::new(module_bytes);
    let abi = module
        .try_parse_abi()
        .map_err(|e| Error::BackendFailure { msg: e.to_string() })?;
    // serialize response as json
    serde_json::to_vec(&abi).map_err(|e| Error::BackendFailure { msg: e.to_string() })
}
