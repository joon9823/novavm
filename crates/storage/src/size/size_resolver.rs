// Copyright (c) The Kernel Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait SizeResolver`](SizeResolver).

use anyhow::Result;
use nova_types::access_path::AccessPath;

/// `SizeResolver` is a trait that defines a read-only snapshot of the global state.
pub trait SizeResolver {
    fn get_size(&self, access_path: &AccessPath) -> Result<usize>;
}
