#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

pub mod compiler;

pub use compiler::{compile, Command};

#[cfg(test)]
mod tests;
