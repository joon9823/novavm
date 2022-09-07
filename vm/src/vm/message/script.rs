// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::account_config::core_code_address;
use crate::serde_helper::vec_bytes;

use move_deps::move_core_types::identifier::{IdentStr, Identifier};
use move_deps::move_core_types::language_storage::{ModuleId, TypeTag};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{Sample, CORE_CODE_ADDRESS};

/// Call a Move script.
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
    #[serde(with = "serde_bytes")]
    code: Vec<u8>,
    ty_args: Vec<TypeTag>,
    #[serde(with = "vec_bytes")]
    args: Vec<Vec<u8>>,
}

impl Script {
    pub fn new(code: Vec<u8>, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Script {
            code,
            ty_args,
            args,
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn ty_args(&self) -> &[TypeTag] {
        &self.ty_args
    }

    pub fn args(&self) -> &[Vec<u8>] {
        &self.args
    }

    pub fn into_inner(self) -> (Vec<u8>, Vec<TypeTag>, Vec<Vec<u8>>) {
        (self.code, self.ty_args, self.args)
    }
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Script")
            .field("code", &hex::encode(&self.code))
            .field("ty_args", &self.ty_args)
            .field("args", &self.args)
            .finish()
    }
}

impl Sample for Script {
    /// Sample script source code empty_script.move
    fn sample() -> Self {
        Self {
            code: hex::decode("a11ceb0b0500000007010002030206040802050a0707110c081d10062d0a0000000101020100000001030106090000056465627567057072696e740000000000000000000000000000000103080100000000000000000000070b000700160c010e01380002")
                .expect("Decode sample script should success."),
            ty_args: vec![],
            args: vec![(100 as u64).to_be_bytes().to_vec()],
        }
    }
}

/// Call a Move script function.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntryFunction {
    module: ModuleId,
    function: Identifier,
    ty_args: Vec<TypeTag>,
    #[serde(with = "vec_bytes")]
    args: Vec<Vec<u8>>,
}

impl EntryFunction {
    pub fn new(
        module: ModuleId,
        function: Identifier,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Self {
        EntryFunction {
            module,
            function,
            ty_args,
            args,
        }
    }

    pub fn module(&self) -> &ModuleId {
        &self.module
    }

    pub fn function(&self) -> &IdentStr {
        &self.function
    }

    pub fn ty_args(&self) -> &[TypeTag] {
        &self.ty_args
    }

    pub fn args(&self) -> &[Vec<u8>] {
        &self.args
    }
    pub fn into_inner(self) -> (ModuleId, Identifier, Vec<TypeTag>, Vec<Vec<u8>>) {
        (self.module, self.function, self.ty_args, self.args)
    }
}

impl Sample for EntryFunction {
    fn sample() -> Self {
        let amount: u64 = 100;
        Self {
            module: ModuleId::new(CORE_CODE_ADDRESS, Identifier::new("BasicCoin").unwrap()),
            function: Identifier::new("mint").unwrap(),
            ty_args: vec![],
            args: vec![amount.to_be_bytes().to_vec()],
        }
    }
}
