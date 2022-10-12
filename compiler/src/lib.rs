#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

pub mod clean;
pub mod compiler;
pub mod new;

pub use clean::Clean;
pub use compiler::{compile, Command};
pub use new::New;

mod mock;

#[cfg(test)]
mod tests;
