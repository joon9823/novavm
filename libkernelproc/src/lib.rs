#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

//mod api;
mod db;
mod gas_meter;
mod memory;
mod version;
mod vm;
mod interface;
mod error;


pub use db::{db_t, Db};
pub use memory::{
    destroy_unmanaged_vector, new_unmanaged_vector, ByteSliceView, U8SliceView, UnmanagedVector,
};

#[cfg(test)]
mod tests;
