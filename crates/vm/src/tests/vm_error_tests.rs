use nova_types::errors::NovaVMError;

// constructors

#[test]
fn gas_depletion_works() {
    let error = NovaVMError::gas_depletion();
    match error {
        NovaVMError::GasDepletion { .. } => {}
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn generic_err_works() {
    let guess = 7;
    let error = NovaVMError::generic_err(format!("{} is too low", guess));
    match error {
        NovaVMError::GenericErr { msg, .. } => {
            assert_eq!(msg, String::from("7 is too low"));
        }
        e => panic!("Unexpected error: {:?}", e),
    }
}
