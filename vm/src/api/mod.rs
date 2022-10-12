// Copyright (c) The NovaVM Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait ChainApi`](ChainApi).

use anyhow::Result;

/// Callbacks to system functions defined outside of the move modules.
/// This is a trait to allow Mocks in the test code.
pub trait ChainApi {
    fn get_block_info(&self) -> Result<(u64 /* height */, u64 /* timestamp */)>;
}
