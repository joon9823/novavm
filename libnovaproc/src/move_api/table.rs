use crate::move_api::move_types::MoveType;
use crate::move_api::{VerifyInput, VerifyInputWithRecursion};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Table Item request for the GetTableItem API
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableItemRequest {
    pub key_type: MoveType,
    pub value_type: MoveType,
    /// The value of the table item's key
    pub key: Value,
}

impl VerifyInput for TableItemRequest {
    fn verify(&self) -> anyhow::Result<()> {
        self.key_type.verify(0)?;
        self.value_type.verify(0)
    }
}