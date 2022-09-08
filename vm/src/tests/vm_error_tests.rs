use crate::VmError;

// constructors

#[test]
fn gas_depletion_works() {
    let error = VmError::gas_depletion();
    match error {
        VmError::GasDepletion { .. } => {}
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn generic_err_works() {
    let guess = 7;
    let error = VmError::generic_err(format!("{} is too low", guess));
    match error {
        VmError::GenericErr { msg, .. } => {
            assert_eq!(msg, String::from("7 is too low"));
        }
        e => panic!("Unexpected error: {:?}", e),
    }
}

