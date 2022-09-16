#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

mod api;
mod db;
mod error;
mod interface;
mod memory;
mod querier;
mod storage;
mod version;
mod vm;


pub use db::{db_t, Db};
pub use memory::{
    destroy_unmanaged_vector, new_unmanaged_vector, ByteSliceView, U8SliceView, UnmanagedVector,
};
pub use storage::GoStorage;

#[cfg(test)]
mod tests;
