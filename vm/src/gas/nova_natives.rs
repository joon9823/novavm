use crate::nova_natives::GasParameters;

crate::gas::natives::define_gas_parameters_for_natives!(GasParameters, "nova_natives", [
    [.signature.ed25519.base, "signature.ed25519.base", 1],
    [.signature.ed25519.per_pubkey_deserialize, "signature.ed25519_verify.per_pubkey_deserialize", 1],
    [.signature.ed25519.per_pubkey_small_order_check, "signature.ed25519.per_pubkey_small_order_check", 1],
    [.signature.ed25519.per_sig_deserialize, "signature.ed25519.per_sig_deserialize", 1],
    [.signature.ed25519.per_sig_strict_verify, "signature.ed25519.per_sig_strict_verify", 1],
    [.signature.ed25519.per_msg_hashing_base, "signature.ed25519.per_msg_hashing_base", 1],
    [.signature.ed25519.per_msg_byte_hashing, "signature.ed25519.per_msg_byte_hashing", 1],

    [.signature.secp256k1.base, "signature.secp256k1.base", 1],
    [.signature.secp256k1.ecdsa_recover, "signature.secp256k1.ecdsa_recover", 1],

    [.signature.bls12381.base, "signature.bls12381.base", 1],
    [.signature.bls12381.per_pairing, "signature.bls12381.per_pairing", 1],
	[.signature.bls12381.per_msg, "signature.bls12381.per_msg", 1],
	[.signature.bls12381.per_byte, "signature.bls12381.per_bytes", 1],
    [.signature.bls12381.per_pubkey_deserialize, "signature.bls12381.per_pubkey_deserialize", 1],
    [.signature.bls12381.per_pubkey_aggregate, "signature.bls12381.per_aggregate", 1],
    [.signature.bls12381.per_pubkey_subgroup_check, "signature.bls12381.per_subgroup_check", 1],
    [.signature.bls12381.per_sig_verify, "signature.bls12381.per_sig_verify", 1],
    [.signature.bls12381.per_sig_deserialize, "signature.bls12381.per_sig_deserialize", 1],
    [.signature.bls12381.per_sig_aggregate, "signature.bls12381.per_sig_aggregate", 1],
    [.signature.bls12381.per_sig_subgroup_check, "signature.bls12381.per_sig_subgroup_check", 1],
    [.signature.bls12381.per_proof_of_possession_verify, "signature.bls12381.proof_of_possession_verify", 1],


    [.type_info.type_of.base, "type_info.type_of.base", 1],
    [.type_info.type_of.unit, "type_info.type_of.unit", 1],
    [.type_info.type_name.base, "type_info.type_name.base", 1],
    [.type_info.type_name.unit, "type_info.type_name.unit", 1],

    [.util.from_bytes.base, "util.from_bytes.base", 1],
    [.util.from_bytes.unit, "util.from_bytes.unit", 1],

    [.code.request_publish.base, "code.request_publish.base", 1],
    [.code.request_publish.unit, "code.request_publish.unit", 1],
]);
