use move_deps::move_cli::Move;
use nova_compiler::compile as nova_compile;
use crate::error::Error;

pub use nova_compiler::Command;


pub fn compile(
    move_args: Move,
    cmd: Command,
) -> Result<Vec<u8>, Error> {
    let action = cmd.to_string();

    match nova_compile(move_args, cmd) {
        Ok(_) => Ok(Vec::from("ok")),
        Err(e) => Err(Error::backend_failure(format!("failed to {}: {}", action, e))),
    }
}
