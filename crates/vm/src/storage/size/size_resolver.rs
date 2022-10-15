// Copyright (c) The Kernel Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait SizeResolver`](SizeResolver).

use crate::access_path::AccessPath;
use anyhow::Result;

/// `SizeResolver` is a trait that defines a read-only snapshot of the global state.
pub trait SizeResolver {
    fn get_size(&self, access_path: &AccessPath) -> Result<usize>;
}
