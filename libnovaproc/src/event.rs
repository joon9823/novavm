use move_deps::move_core_types::language_storage::TypeTag;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ContractEvent {
    /// The unique key that the event was emitted to
    #[serde(with = "serde_bytes")]
    key: Vec<u8>,
    /// The number of messages that have been emitted to the path previously
    sequence_number: u64,
    /// The type of the data
    type_tag: TypeTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    event_data: Vec<u8>,
}

impl ContractEvent {
    pub fn new(
        key: Vec<u8>,
        sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
    ) -> Self {
        Self {
            key,
            sequence_number,
            type_tag,
            event_data,
        }
    }

    pub fn _key(&self) -> &Vec<u8> {
        &self.key
    }

    pub fn _sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn _event_data(&self) -> &[u8] {
        &self.event_data
    }

    pub fn _type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}

impl std::fmt::Debug for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ContractEvent {{ key: {:?}, index: {:?}, type: {:?}, event_data: {:?} }}",
            self.key,
            self.sequence_number,
            self.type_tag,
            hex::encode(&self.event_data)
        )
    }
}

impl std::fmt::Display for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
