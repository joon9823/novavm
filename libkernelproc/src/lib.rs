#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

mod api;
mod db;
mod error;
mod gas_meter;
mod interface;
mod iterator;
mod memory;
mod querier;
mod version;
mod view;
mod vm;


pub use db::{db_t, Db};
pub use memory::{
    destroy_unmanaged_vector, new_unmanaged_vector, ByteSliceView, U8SliceView, UnmanagedVector,
};
pub use view::CosmosView;

#[cfg(test)]
mod tests;
