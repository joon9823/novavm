//! This crate is the core of the gas metering system of the Aptos blockchain.
//!
//! More specifically, it
//!   - Is home to the gas meter implementation
//!   - Defines the gas parameters and formulae for instructions
//!   - Defines the gas parameters for transactions
//!   - Sets the initial values for all gas parameters, including the instruction, transaction
//!     move-stdlib and aptos-framework ones.
//!   - Defines a bi-directional mapping between the (Rust) gas parameter structs and their
//!     corresponding representation on-chain.
//!
//! The reason why we need two different representations is that they serve different purposes:
//!   - The Rust structs are used for quick (static) lookups by the gas meter and native functions
//!     when calculating gas costs.
//!   - The on-chain gas schedule needs to be extensible and unordered so we can upgrate it easily
//!     in the future.

#[macro_use]
mod natives;

#[macro_use]
mod params;

mod algebra;
mod algebra_ext;
mod instr;
mod meter;
mod misc;
mod move_stdlib;
mod nova;
mod table;
mod transaction;

pub use algebra::*;
pub use instr::InstructionGasParameters;
pub use meter::{
    FromOnChainGasSchedule, InitialGasSchedule, NativeGasParameters, NovaGasMeter,
    NovaGasParameters, ToOnChainGasSchedule,
};
pub use misc::{AbstractValueSizeGasParameters, MiscGasParameters};
pub use move_deps::move_core_types::gas_algebra::{
    Arg, Byte, GasQuantity, InternalGas, InternalGasPerArg, InternalGasPerByte, InternalGasUnit,
    NumArgs, NumBytes, UnitDiv,
};
