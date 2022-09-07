// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::account_config::core_code_address;
use crate::serde_helper::vec_bytes;

use move_deps::move_core_types::identifier::{IdentStr, Identifier};
use move_deps::move_core_types::language_storage::{ModuleId, TypeTag};
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
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

#[cfg(test)]
impl Sample for Script {
    fn sample() -> Self {
        Self {
            code: include_bytes!("../../../move-test/build/test1/bytecode_scripts/main.mv")
                .to_vec(),
            ty_args: vec![],
            args: vec![],
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

#[cfg(test)]
impl Sample for EntryFunction {
    fn sample() -> Self {
        let amount: u64 = 100;
        Self {
            module: ModuleId::new(CORE_CODE_ADDRESS, Identifier::new("BasicCoin").unwrap()),
            function: Identifier::new("mint").unwrap(),
            ty_args: vec![],
            args: vec![amount.to_le_bytes().to_vec()],
        }
    }
}
