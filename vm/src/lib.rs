//#![cfg_attr(feature = "backtraces", feature(backtrace))]

pub mod serde_helper;
pub mod vm;
pub mod errors;


pub use crate::errors::VmError;
pub use crate::errors::BackendError;
pub use crate::vm::{KernelVM, gas_meter};
pub use crate::vm::message::*;

#[cfg(test)]
mod tests;
