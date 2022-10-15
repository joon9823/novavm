// Copyright (c) The Kernel Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait TableMetaResolver`](TableMetaResolver).

use anyhow::Result;
use nova_natives::table::TableHandle;

use super::TableMeta;

/// `TableMetaResolver` is a trait that defines a read-only snapshot of the global state.
pub trait TableMetaResolver {
    fn get_table_meta(&self, table_handle: &TableHandle) -> Result<Option<TableMeta>>;
}
