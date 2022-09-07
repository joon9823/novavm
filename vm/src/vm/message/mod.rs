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

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub use module::Module;
pub use script::{EntryFunction, Script};

mod module;
mod script;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Sender's address.
    sender: AccountAddress,
    // The message script to execute.
    payload: MessagePayload,
}

impl Message {
    /// Create a new `Message` with a payload.
    ///
    /// It can be either to publish a module, to execute a script
    pub fn new(sender: AccountAddress, payload: MessagePayload) -> Self {
        Message { sender, payload }
    }

    pub fn new_with_default_gas_token(sender: AccountAddress, payload: MessagePayload) -> Self {
        Message { sender, payload }
    }

    /// Create a new `Message` with a script.
    ///
    /// A script message contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script(sender: AccountAddress, script: Script) -> Self {
        Message {
            sender,
            payload: MessagePayload::Script(script),
        }
    }

    /// Create a new `Message` with a script function.
    ///
    /// A script message contains only code to execute. No publishing is allowed in scripts.
    pub fn new_entry_function(sender: AccountAddress, entry_function: EntryFunction) -> Self {
        Message {
            sender,
            payload: MessagePayload::EntryFunction(entry_function),
        }
    }

    /// Create a new `Message` with a module to publish.
    ///
    /// A module message is the only way to publish code. Only one module per message
    /// can be published.
    pub fn new_module(sender: AccountAddress, module: Module) -> Self {
        Message {
            sender,
            payload: MessagePayload::Module(module),
        }
    }

    pub fn into_payload(self) -> MessagePayload {
        self.payload
    }

    /// Return the sender of this message.
    pub fn sender(&self) -> AccountAddress {
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
            sender,
            MessagePayload::Script(Script::new(vec![], vec![], vec![])),
        )
    }

    pub fn mock_from(compiled_script: Vec<u8>) -> Self {
        Self::new_with_default_gas_token(
            AccountAddress::ZERO,
            MessagePayload::Script(Script::new(compiled_script, vec![], vec![])),
        )
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessagePayload {
    /// A message that executes code.
    Script(Script),
    /// A message that publish or update module code by a package.
    Module(Module),
    /// A transaction that executes an existing entry function published on-chain.
    EntryFunction(EntryFunction),
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum MessagePayloadType {
    Script = 0,
    Module = 1,
    EntryFunction = 2,
}

impl MessagePayload {
    pub fn payload_type(&self) -> MessagePayloadType {
        match self {
            MessagePayload::Script(_) => MessagePayloadType::Script,
            MessagePayload::Module(_) => MessagePayloadType::Module,
            MessagePayload::EntryFunction(_) => MessagePayloadType::EntryFunction,
        }
    }
}

impl TryFrom<u8> for MessagePayloadType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessagePayloadType::Script),
            1 => Ok(MessagePayloadType::Module),
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

    /// The amount of gas used during execution.
    gas_used: u64,

    /// The execution status.
    status: MessageStatus,
}

impl MessageOutput {
    pub fn new(
        change_set: ChangeSet,
        events: Vec<Event>,
        gas_used: u64,
        status: MessageStatus,
    ) -> Self {
        MessageOutput {
            change_set,
            events,
            gas_used,
            status,
        }
    }

    pub fn change_set(&self) -> &ChangeSet {
        &self.change_set
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
