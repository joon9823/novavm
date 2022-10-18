use move_deps::{
    move_core_types::{
        account_address::AccountAddress,
        resolver::MoveResolver,
        value::MoveValue,
        vm_status::{StatusCode, VMStatus},
    },
    move_vm_runtime::session::{LoadedFunctionInstantiation, Session},
    move_vm_types::loaded_data::runtime_types::Type,
};
use once_cell::sync::Lazy;
use std::collections::BTreeSet;

static ALLOWED_STRUCTS: Lazy<BTreeSet<String>> = Lazy::new(|| {
    ["0x1::string::String"]
        .iter()
        .map(|s| s.to_string())
        .collect()
});

/// Validate and generate args with senders and non-signer arguments
///
/// validation includes:
/// 1. number of signers is same as the number of senders
/// 2. check arg types are allowed after signers
///
/// after validation, add senders and non-signer arguments to generate the final args
pub(crate) fn validate_combine_signer_and_txn_args<S: MoveResolver>(
    session: &Session<S>,
    senders: Vec<AccountAddress>,
    args: Vec<Vec<u8>>,
    func: &LoadedFunctionInstantiation,
) -> Result<Vec<Vec<u8>>, VMStatus> {
    let mut signer_param_cnt = 0;
    // find all signer params at the beginning
    for ty in func.parameters.iter() {
        match ty {
            Type::Signer => signer_param_cnt += 1,
            Type::Reference(inner_type) => {
                if matches!(&**inner_type, Type::Signer) {
                    signer_param_cnt += 1;
                }
            }
            _ => (),
        }
    }
    // validate all non_signer params
    for ty in func.parameters[signer_param_cnt..].iter() {
        if !is_valid_txn_arg(session, ty) {
            return Err(VMStatus::Error(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE));
        }
    }

    if (signer_param_cnt + args.len()) != func.parameters.len() {
        return Err(VMStatus::Error(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH));
    }
    // if function doesn't require signer, we reuse txn args
    // if the function require signer, we check senders number same as signers
    // and then combine senders with txn args.
    let combined_args = if signer_param_cnt == 0 {
        args
    } else {
        // the number of txn senders should be the same number of signers
        if senders.len() != signer_param_cnt {
            return Err(VMStatus::Error(
                StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH,
            ));
        }
        senders
            .into_iter()
            .map(|s| MoveValue::Signer(s).simple_serialize().unwrap())
            .chain(args)
            .collect()
    };
    Ok(combined_args)
}

fn is_valid_txn_arg<S: MoveResolver>(session: &Session<S>, typ: &Type) -> bool {
    use move_deps::move_vm_types::loaded_data::runtime_types::Type::*;
    match typ {
        Bool | U8 | U64 | U128 | Address => true,
        Vector(inner) => is_valid_txn_arg(session, inner),
        Struct(idx) | StructInstantiation(idx, _) => {
            if let Some(st) = session.get_struct_type(*idx) {
                let full_name = format!("{}::{}", st.module.short_str_lossless(), st.name);
                ALLOWED_STRUCTS.contains(&full_name)
            } else {
                false
            }
        }
        Signer | Reference(_) | MutableReference(_) | TyParam(_) => false,
    }
}
