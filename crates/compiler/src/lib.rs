#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

pub mod clean;
pub mod command;
pub mod compiler;
pub mod new;

mod extensions;

pub use clean::Clean;
pub use command::Command;
pub use compiler::compile;
pub use new::New;

#[cfg(test)]
mod tests;
