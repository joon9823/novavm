#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

pub mod compiler;
pub mod clean;
pub mod new;

pub use compiler::{compile, Command};
pub use clean::Clean;
pub use new::New;

#[cfg(test)]
mod tests;
