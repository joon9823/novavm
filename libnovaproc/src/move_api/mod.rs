pub mod handler;
pub mod compiler;

mod address;
mod bytecode;
mod convert;
mod move_types;
mod wrappers;

/// For verifying a given struct
pub trait VerifyInput {
    fn verify(&self) -> anyhow::Result<()>;
}

/// For verifying a given struct that needs to limit recursion
pub trait VerifyInputWithRecursion {
    fn verify(&self, recursion_count: u8) -> anyhow::Result<()>;
}
