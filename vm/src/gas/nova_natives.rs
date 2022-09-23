use crate::natives::GasParameters;

crate::gas::natives::define_gas_parameters_for_natives!(GasParameters, "nova_natives", [
    [.signature.bls12381_validate_pubkey.base_cost, "signature.bls12381_validate_pubkey.base", 1],
    [.signature.ed25519_validate_pubkey.base_cost, "signature.ed25519_validate_pubkey.base", 1],
    [.signature.ed25519_verify.base_cost, "signature.ed25519_verify.base", 1],
    [.signature.ed25519_verify.unit_cost, "signature.ed25519_verify.unit", 1],
    [.signature.secp256k1_ecdsa_recover.base_cost, "signature.secp256k1_ecdsa_recover.base", 1],
    [.signature.bls12381_verify_signature.base_cost, "signature.bls12381_verify_signature.base", 1],
    [.signature.bls12381_verify_signature.unit_cost, "signature.bls12381_verify_signature.unit", 1],
    [.signature.bls12381_aggregate_pop_verified_pubkeys.base_cost, "signature.bls12381_aggregate_pop_verified_pubkeys.base", 1],
    [.signature.bls12381_aggregate_pop_verified_pubkeys.per_pubkey_cost, "signature.bls12381_aggregate_pop_verified_pubkeys.per_pubkey", 1],
    [.signature.bls12381_verify_proof_of_possession.base_cost, "signature.bls12381_verify_proof_of_possession.base", 1],

    [.type_info.type_of.base_cost, "type_info.type_of.base", 1],
    [.type_info.type_of.unit_cost, "type_info.type_of.unit", 1],
    [.type_info.type_name.base_cost, "type_info.type_name.base", 1],
    [.type_info.type_name.unit_cost, "type_info.type_name.unit", 1],

    [.util.from_bytes.base_cost, "util.from_bytes.base", 1],
    [.util.from_bytes.unit_cost, "util.from_bytes.unit", 1],

    [.code.request_publish.base_cost, "code.request_publish.base", 1],
    [.code.request_publish.unit_cost, "code.request_publish.unit", 1],
]);
