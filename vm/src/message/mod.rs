// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub const CORE_CODE_ADDRESS: AccountAddress = AccountAddress::ONE;
pub fn genesis_address() -> AccountAddress {
    CORE_CODE_ADDRESS
}

use anyhow::{format_err, Error, Result};

use move_deps::move_core_types::effects::Event;
use move_deps::move_core_types::vm_status::*;
use move_deps::move_core_types::{account_address::AccountAddress, effects::ChangeSet};

use move_deps::move_table_extension::TableChangeSet;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub use module::{Module, ModuleBundle};
pub use script::{EntryFunction, Script};

use self::size_change_set::SizeChangeSet;

mod module;
mod script;
pub mod size_change_set;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// SessionID is a seed for global unique ID of Table extension.
    /// Ex) transaction hash
    session_id: Vec<u8>,
    /// Sender's address.
    sender: Option<AccountAddress>,
    /// The message script to execute.
    payload: MessagePayload,
}

impl Message {
    /// Create a new `Message` with a payload.
    ///
    /// It can be either to publish a module, to execute a script
    pub fn new(
        session_id: Vec<u8>,
        sender: Option<AccountAddress>,
        payload: MessagePayload,
    ) -> Self {
        Message {
            session_id,
            sender,
            payload,
        }
    }

    pub fn new_with_default_gas_token(
        session_id: Vec<u8>,
        sender: Option<AccountAddress>,
        payload: MessagePayload,
    ) -> Self {
        Message {
            session_id,
            sender,
            payload,
        }
    }

    /// Create a new `Message` with a script.
    ///
    /// A script message contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script(session_id: Vec<u8>, sender: Option<AccountAddress>, script: Script) -> Self {
        Message {
            session_id,
            sender,
            payload: MessagePayload::Script(script),
        }
    }

    /// Create a new `Message` with a script function.
    ///
    /// A script message contains only code to execute. No publishing is allowed in scripts.
    pub fn new_entry_function(
        session_id: Vec<u8>,
        sender: Option<AccountAddress>,
        entry_function: EntryFunction,
    ) -> Self {
        Message {
            session_id,
            sender,
            payload: MessagePayload::EntryFunction(entry_function),
        }
    }

    /// Create a new `Message` with a module to publish.
    ///
    /// A module message is the only way to publish code.
    pub fn new_module(
        session_id: Vec<u8>,
        sender: Option<AccountAddress>,
        modules: ModuleBundle,
    ) -> Self {
        Message {
            session_id,
            sender,
            payload: MessagePayload::ModuleBundle(modules),
        }
    }

    pub fn into_payload(self) -> MessagePayload {
        self.payload
    }

    /// Return session_id
    pub fn session_id(&self) -> &[u8] {
        &self.session_id
    }

    /// Return the sender of this message.
    pub fn sender(&self) -> Option<AccountAddress> {
        self.sender
    }

    pub fn payload(&self) -> &MessagePayload {
        &self.payload
    }

    pub fn mock() -> Self {
        Self::mock_by_sender(AccountAddress::random())
    }

    pub fn mock_by_sender(sender: AccountAddress) -> Self {
        Self::new_with_default_gas_token(
            vec![0; 32],
            Some(sender),
            MessagePayload::Script(Script::new(vec![], vec![], vec![])),
        )
    }

    pub fn mock_from(compiled_script: Vec<u8>) -> Self {
        Self::new_with_default_gas_token(
            vec![0; 32],
            None,
            MessagePayload::Script(Script::new(compiled_script, vec![], vec![])),
        )
    }

    pub fn size(&self) -> usize {
        bcs::to_bytes(&self.payload())
            .expect("Unable to serialize payload")
            .len()
            + bcs::to_bytes(&self.sender())
                .expect("Unable to serialize sender")
                .len()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessagePayload {
    /// A message that executes code.
    Script(Script),
    /// A message that publishes multiple modules at the same time.
    ModuleBundle(ModuleBundle),
    /// A transaction that executes an existing entry function published on-chain.
    EntryFunction(EntryFunction),
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum MessagePayloadType {
    Script = 0,
    ModuleBundle = 1,
    EntryFunction = 2,
}

impl TryFrom<u8> for MessagePayloadType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessagePayloadType::Script),
            1 => Ok(MessagePayloadType::ModuleBundle),
            _ => Err(format_err!("invalid PayloadType")),
        }
    }
}

impl From<MessagePayloadType> for u8 {
    fn from(t: MessagePayloadType) -> Self {
        t as u8
    }
}

/// The status of executing a message. The VM decides whether or not we should `Keep` the
/// message output or `Discard` it based upon the execution of the message. We wrap these
/// decisions around a `VMStatus` that provides more detail on the final execution state of the VM.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageStatus {
    /// Discard the message output
    Discard(DiscardedVMStatus),

    /// Keep the message output
    Keep(KeptVMStatus),
}

impl MessageStatus {
    pub fn status(&self) -> Result<KeptVMStatus, StatusCode> {
        match self {
            MessageStatus::Keep(status) => Ok(status.clone()),
            MessageStatus::Discard(code) => Err(*code),
        }
    }

    pub fn is_discarded(&self) -> bool {
        match self {
            MessageStatus::Discard(_) => true,
            MessageStatus::Keep(_) => false,
        }
    }
}

impl From<VMStatus> for MessageStatus {
    fn from(vm_status: VMStatus) -> Self {
        match vm_status.keep_or_discard() {
            Ok(recorded) => MessageStatus::Keep(recorded),
            Err(code) => MessageStatus::Discard(code),
        }
    }
}

pub struct MessageOutput {
    change_set: ChangeSet,
    events: Vec<Event>,
    table_change_set: TableChangeSet,
    size_change_set: SizeChangeSet,

    /// The amount of gas used during execution.
    gas_used: u64,

    /// The execution status.
    status: MessageStatus,
}

impl MessageOutput {
    pub fn new(
        change_set: ChangeSet,
        events: Vec<Event>,
        table_change_set: TableChangeSet,
        size_change_set: SizeChangeSet,
        gas_used: u64,
        status: MessageStatus,
    ) -> Self {
        MessageOutput {
            change_set,
            table_change_set,
            size_change_set,
            events,
            gas_used,
            status,
        }
    }

    pub fn change_set(&self) -> &ChangeSet {
        &self.change_set
    }

    pub fn table_change_set(&self) -> &TableChangeSet {
        &self.table_change_set
    }

    pub fn size_change_set(&self) -> &SizeChangeSet {
        &self.size_change_set
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn status(&self) -> &MessageStatus {
        &self.status
    }

    pub fn into_inner(self) -> (ChangeSet, Vec<Event>, u64, MessageStatus) {
        (self.change_set, self.events, self.gas_used, self.status)
    }
}
