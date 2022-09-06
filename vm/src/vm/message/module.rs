// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::fmt;

use super::Sample;
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Module {
    code: Vec<u8>,
}
impl From<Module> for Vec<u8> {
    fn from(m: Module) -> Self {
        m.code
    }
}
impl Module {
    pub fn new(code: Vec<u8>) -> Module {
        Module { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Module")
            .field("code", &hex::encode(&self.code))
            .finish()
    }
}

impl Sample for Module {
    ///Sample module's source code:
    /// ```move
    /// address 0x1{
    ///     module M{
    ///     }
    /// }
    /// ```
    ///
    fn sample() -> Self {
        Self {
            code: hex::decode(
                "a11ceb0b0500000008010002020204030605050b04070f1a0829100a39050c3e0f0000000108000002000100020c0300094261736963436f696e04436f696e046d696e740576616c75650000000000000000000000000000000100020103030001000001050e000b0112002d000200",
            )
            .expect("decode sample module should success"),
        }
    }
}
