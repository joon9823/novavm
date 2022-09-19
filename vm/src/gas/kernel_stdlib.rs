use crate::kernel_stdlib::GasParameters;

crate::gas::natives::define_gas_parameters_for_natives!(GasParameters, "kernel_stdlib", [
    [.bank.transfer.base, "bank.transfer.base", 10],
    [.bank.balance.base, "bank.balance.base",10],
]);
