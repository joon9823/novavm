// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub const CORE_CODE_ADDRESS: AccountAddress = AccountAddress::ONE;
pub fn genesis_address() -> AccountAddress {
    CORE_CODE_ADDRESS
}

use anyhow::{format_err, Error, Result};

use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::*;

use serde::{Deserialize, Deserializer, Serialize};
use std::{convert::TryFrom, fmt};

pub use module::Module;
pub use script::{Script, ScriptABI, ScriptFunction, ScriptFunctionABI, TransactionScriptABI};

mod module;
mod script;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Sender's address.
    sender: AccountAddress,
    // The transaction script to execute.
    payload: TransactionPayload,
}

impl Transaction {
    /// Create a new `RawUserTransaction` with a payload.
    ///
    /// It can be either to publish a module, to execute a script, or to issue a writeset
    /// transaction.
    pub fn new(sender: AccountAddress, payload: TransactionPayload) -> Self {
        Transaction { sender, payload }
    }

    pub fn new_with_default_gas_token(sender: AccountAddress, payload: TransactionPayload) -> Self {
        Transaction { sender, payload }
    }

    /// Create a new `RawUserTransaction` with a script.
    ///
    /// A script transaction contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script(sender: AccountAddress, script: Script) -> Self {
        Transaction {
            sender,
            payload: TransactionPayload::Script(script),
        }
    }

    /// Create a new `RawTransaction` with a script function.
    ///
    /// A script transaction contains only code to execute. No publishing is allowed in scripts.
    pub fn new_script_function(sender: AccountAddress, script_function: ScriptFunction) -> Self {
        Transaction {
            sender,
            payload: TransactionPayload::ScriptFunction(script_function),
        }
    }

    /// Create a new `RawUserTransaction` with a module to publish.
    ///
    /// A module transaction is the only way to publish code. Only one module per transaction
    /// can be published.
    pub fn new_module(sender: AccountAddress, module: Module) -> Self {
        Transaction {
            sender,
            payload: TransactionPayload::Module(module),
        }
    }

    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }

    /// Return the sender of this transaction.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    pub fn payload(&self) -> &TransactionPayload {
        &self.payload
    }

    pub fn mock() -> Self {
        Self::mock_by_sender(AccountAddress::random())
    }

    pub fn mock_by_sender(sender: AccountAddress) -> Self {
        Self::new_with_default_gas_token(
            sender,
            TransactionPayload::Script(Script::new(vec![], vec![], vec![])),
        )
    }

    pub fn mock_from(compiled_script: Vec<u8>) -> Self {
        Self::new_with_default_gas_token(
            AccountAddress::ZERO,
            TransactionPayload::Script(Script::new(compiled_script, vec![], vec![])),
        )
    }
}

impl Sample for Transaction {
    fn sample() -> Self {
        Self::new_module(genesis_address(), Module::sample())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// A transaction that executes code.
    Script(Script),
    /// A transaction that publish or update module code by a package.
    Module(Module),
    /// A transaction that executes an existing script function published on-chain.
    ScriptFunction(ScriptFunction),
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum TransactionPayloadType {
    Script = 0,
    Module = 1,
    ScriptFunction = 2,
}

impl TransactionPayload {
    pub fn payload_type(&self) -> TransactionPayloadType {
        match self {
            TransactionPayload::Script(_) => TransactionPayloadType::Script,
            TransactionPayload::Module(_) => TransactionPayloadType::Module,
            TransactionPayload::ScriptFunction(_) => TransactionPayloadType::ScriptFunction,
        }
    }
}

impl TryFrom<u8> for TransactionPayloadType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TransactionPayloadType::Script),
            1 => Ok(TransactionPayloadType::Module),
            _ => Err(format_err!("invalid PayloadType")),
        }
    }
}

impl From<TransactionPayloadType> for u8 {
    fn from(t: TransactionPayloadType) -> Self {
        t as u8
    }
}

/// The status of executing a transaction. The VM decides whether or not we should `Keep` the
/// transaction output or `Discard` it based upon the execution of the transaction. We wrap these
/// decisions around a `VMStatus` that provides more detail on the final execution state of the VM.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TransactionStatus {
    /// Discard the transaction output
    Discard(DiscardedVMStatus),

    /// Keep the transaction output
    Keep(KeptVMStatus),
}

impl TransactionStatus {
    pub fn status(&self) -> Result<KeptVMStatus, StatusCode> {
        match self {
            TransactionStatus::Keep(status) => Ok(status.clone()),
            TransactionStatus::Discard(code) => Err(*code),
        }
    }

    pub fn is_discarded(&self) -> bool {
        match self {
            TransactionStatus::Discard(_) => true,
            TransactionStatus::Keep(_) => false,
        }
    }
}

impl From<VMStatus> for TransactionStatus {
    fn from(vm_status: VMStatus) -> Self {
        match vm_status.keep_or_discard() {
            Ok(recorded) => TransactionStatus::Keep(recorded),
            Err(code) => TransactionStatus::Discard(code),
        }
    }
}

/// Pool transactions status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TxStatus {
    /// Added transaction
    Added,
    /// Rejected transaction
    Rejected,
    /// Dropped transaction
    Dropped,
    /// Invalid transaction
    Invalid,
    /// Canceled transaction
    Canceled,
    /// Culled transaction
    Culled,
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TxStatus::Added => "added",
            TxStatus::Rejected => "rejected",
            TxStatus::Dropped => "dropped",
            TxStatus::Invalid => "invalid",
            TxStatus::Canceled => "canceled",
            TxStatus::Culled => "culled",
        };
        write!(f, "{}", s)
    }
}

pub trait Sample {
    /// A default construct for generate type Sample data for test or document.
    /// Please ensure return same data when call sample fn.
    fn sample() -> Self;
}
