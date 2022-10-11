/// Contains functions for:
///
///  1. [Ed25519](https://en.wikipedia.org/wiki/EdDSA#Ed25519) digital signatures
///
///  2. ECDSA digital signatures over secp256k1 elliptic curves
///
///  3. The minimum-pubkey-size variant of [Boneh-Lynn-Shacham (BLS) signatures](https://en.wikipedia.org/wiki/BLS_digital_signature),
///     where public keys are BLS12-381 elliptic-curve points in $\mathbb{G}_1$ and signatures are in $\mathbb{G}_2$,
///     as per the [IETF BLS draft standard](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature#section-2.1)
module nova_std::signature {

    /// Return `true` if the elliptic curve point serialized in `signature`:
    ///  (1) is NOT the identity point, and
    ///  (2) is a BLS12-381 elliptic curve point, and
    ///  (3) is a prime-order point
    /// Return `false` otherwise.
    /// Does not abort.
    native public fun bls12381_signature_subgroup_check(signature: vector<u8>): bool;

    /// Given a vector of serialized public keys, combines them into an aggregated public key which can be used to verify
    /// multisignatures using `bls12381_verify_signature`.
    /// Returns 'None' if no public keys are given as input.
    /// This function assumes that the caller verified all public keys have a valid proof-of-possesion (PoP) using
    /// `bls12381_verify_proof_of_possession`.
    /// Does not abort.
    native public fun bls12381_aggregate_pop_verified_pubkeys(
        public_keys: vector<vector<u8>>,
    ): (vector<u8>, bool);

    /// CRYPTOGRAPHY WARNING: This function can be safely called without verifying that the input signatures are elements
    /// of the prime-order subgroup of the BLS12-381 curve.
    ///
    /// Given a vector of serialized signatures, combines them into an aggregate signature, returning `(bytes, true)`,
    /// where `bytes` store the serialized signature.
    /// Does not check the input signatures nor the final aggregated signatures for prime-order subgroup membership.
    /// Returns `(_, false)` if no signatures are given as input.
    /// Does not abort.
    native public fun bls12381_aggregate_signatures(signatures: vector<vector<u8>>): (vector<u8>, bool);

    /// Return `true` if the bytes in `public_key` are a valid bls12381 public key: it is in the prime-order subgroup
    /// and it is different from the identity group element.
    /// Return `false` otherwise.
    /// Does not abort.
    native public fun bls12381_validate_pubkey(public_key: vector<u8>): bool;

    /// Return `true` if the bytes in `public_key` are a valid bls12381 public key (as per `bls12381_validate_pubkey`)
    /// *and* has a valid proof-of-possesion (PoP).
    /// Return `false` otherwise.
    /// Does not abort.
    native public fun bls12381_verify_proof_of_possession(public_key: vector<u8>, proof_of_possesion: vector<u8>): bool;
    spec bls12381_verify_proof_of_possession { // TODO: temporary mockup.
        pragma opaque;
    }

    /// CRYPTOGRAPHY WARNING: First, this function assumes all public keys have a valid proof-of-possesion (PoP).
    /// This prevents both small-subgroup attacks and rogue-key attacks. Second, this function can be safely called
    /// without verifying that the aggregate signature is in the prime-order subgroup of the BLS12-381 curve.
    ///
    /// Returns `true` if the aggregate signature `aggsig` on `messages` under `public_keys` verifies (where `messages[i]`
    /// should be signed by `public_keys[i]`).
    ///
    /// Returns `false` if either:
    /// - no public keys or messages are given as input,
    /// - number of messages does not equal number of public keys
    /// - `aggsig` (1) is the identity point, or (2) is NOT a BLS12-381 elliptic curve point, or (3) is NOT a
    ///   prime-order point
    /// Does not abort.
    native public fun bls12381_verify_aggregate_signature(
        aggsig: vector<u8>,
        public_keys: vector<vector<u8>>,
        messages: vector<vector<u8>>,
    ): bool;

    /// Return true if the BLS `signature` on `message` verifiers against the BLS public key `public_key`.
    /// Returns `false` if:
    /// - `signature` is not 96 bytes
    /// - `public_key` is not 48 bytes
    /// - `signature` or `public_key` are not valid: i.e., (1) they are the identity point, or (2) they are not valid
    ///    points on the BLS12-381 elliptic curve or (3) they are not prime-order points.
    /// - `signature` and `public key` are valid but the signature on `message` is not valid.
    /// This function can be used to verify either:
    ///     (1) signature shares for a BLS multisignature scheme or for a BLS aggregate signature scheme,
    ///     (2) BLS multisignatures (for this the `public_key` needs to be aggregated via `bls12381_aggregate_pubkey`).
    /// Does not abort.
    native public fun bls12381_verify_signature(
        signature: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    /// CRYPTOGRAPHY WARNING: This function assumes verified proofs-of-possesion (PoP) for the public keys used in
    /// computing the aggregate public key. This prevents small-subgroup attacks and rogue-key attacks.
    ///
    /// Return `true` if the BLS `multisignature` on `message` verifies against the BLS aggregate public key `agg_public_key`.
    /// Returns `false` otherwise.
    /// Does not abort.
    native public fun bls12381_verify_multisig(
        multisignature: vector<u8>,
        agg_public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    /// CRYPTOGRAPHY WARNING: Assumes the public key has a valid proof-of-possesion (PoP). This prevents rogue-key
    /// attacks later on during signature aggregation.
    ///
    /// Returns `true` if the `signature_share` on `message` verifies under `public key`.
    /// Returns `false` otherwise, similar to `verify_multisignature`.
    /// Does not abort.
    native public fun bls12381_verify_signature_share(
        signature_share: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    /// Return `true` if the bytes in `public_key` can be parsed as a valid Ed25519 public key.
    /// Returns `false` if `public_key` is not 32 bytes OR is 32 bytes, but does not pass
    /// points-on-curve or small subgroup checks. This function should NOT be needed for most users
    /// since ed25519_verify already does all these checks. We leave it here just in case.
    /// See the Rust `aptos_crypto::Ed25519PublicKey` type for more details.
    /// Does not abort.
    native public fun ed25519_validate_pubkey(public_key: vector<u8>): bool;

    /// Return true if the Ed25519 `signature` on `message` verifies against the Ed25519 public key
    /// `public_key`.
    /// Returns `false` if:
    /// - `signature` is not 64 bytes
    /// - `public_key` is not 32 bytes
    /// - `public_key` does not pass points-on-curve or small subgroup checks,
    /// - `signature` and `public_key` are valid, but the signature on `message` does not verify.
    /// Does not abort.
    native public fun ed25519_verify(
        signature: vector<u8>,
        public_key: vector<u8>,
        message: vector<u8>
    ): bool;

    /// Recovers the signer's public key from a secp256k1 ECDSA `signature` provided the `recovery_id` and signed
    /// `message` (32 byte digest).
    /// Returns `(public_key, true)` if inputs are valid and `([], false)` if invalid.
    native public fun secp256k1_ecdsa_recover(
        message: vector<u8>,
        recovery_id: u8,
        signature: vector<u8>
    ): (vector<u8>, bool);

    #[test_only]
    use std::vector;

    #[test]
    /// Test on a valid secp256k1 ECDSA signature created using sk = x"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    fun test_secp256k1_recover() {
        use std::hash;

        let (pk, ok) = secp256k1_ecdsa_recover(
            hash::sha2_256(b"test aptos secp256k1"),
            0,
            x"f7ad936da03f948c14c542020e3c5f4e02aaacd1f20427c11aa6e2fbf8776477646bba0e1a37f9e7c777c423a1d2849baafd7ff6a9930814a43c3f80d59db56f",
        );
        assert!(ok == true, 1);
        assert!(pk == x"4646ae5047316b4230d0086c8acec687f00b1cd9d1dc634f6cb358ac0a9a8ffffe77b4dd0a4bfb95851f3b7355c781dd60f8418fc8a65d14907aff47c903a559", 2);

        // Flipped bits; Signature stays valid
        let (pk, ok) = secp256k1_ecdsa_recover(
            hash::sha2_256(b"test aptos secp256k1"),
            0,
            x"f7ad936da03f948c14c542020e3c5f4e02aaacd1f20427c11aa6e2fbf8776477646bba0e1a37f9e7c7f7c423a1d2849baafd7ff6a9930814a43c3f80d59db56f",
        );
        assert!(ok == true, 3);
        assert!(pk != x"4646ae5047316b4230d0086c8acec687f00b1cd9d1dc634f6cb358ac0a9a8ffffe77b4dd0a4bfb95851f3b7355c781dd60f8418fc8a65d14907aff47c903a559", 4);

        // Flipped bits; Signature becomes invalid
        let (_, ok) = secp256k1_ecdsa_recover(
            hash::sha2_256(b"test aptos secp256k1"),
            0,
            x"ffad936da03f948c14c542020e3c5f4e02aaacd1f20427c11aa6e2fbf8776477646bba0e1a37f9e7c7f7c423a1d2849baafd7ff6a9930814a43c3f80d59db56f",
        );
        assert!(ok == false, 5);
    }

    #[test]
    fun test_empty_pubkey_aggregation() {
        let (_, ok) = bls12381_aggregate_pop_verified_pubkeys(std::vector::empty());
        assert!(ok == false, 1)
    }

    #[test]
    fun test_empty_signature_aggregation() {
        let (_, ok) = bls12381_aggregate_signatures(std::vector::empty());
        assert!(ok == false, 1)
    }

    #[test]
    fun test_pubkey_aggregation() {
        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- sample_aggregate_pk_and_multisig --nocapture --include-ignored
        let pks = vector[
            x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a",
            x"ab9df801c6f96ade1c0490c938c87d5bcc2e52ccb8768e1b5d14197c5e8bfa562783b96711b702dda411a1a9f08ebbfa",
            x"b698c932cf7097d99c17bd6e9c9dc4eeba84278c621700a8f80ec726b1daa11e3ab55fc045b4dbadefbeef05c4182494",
            x"934706a8b876d47a996d427e1526ce52c952d5ec0858d49cd262efb785b62b1972d06270b0a7adda1addc98433ad1843",
            x"a4cd352daad3a0651c1998dfbaa7a748e08d248a54347544bfedd51a197e016bb6008e9b8e45a744e1a030cc3b27d2da" ,
        ];

        // agg_pks[i] = \sum_{j <= i}  pk[j]
        let agg_pks = vector[
            x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a",
            x"b79ad47abb441d7eda9b220a626df2e4e4910738c5f777947f0213398ecafae044ec0c20d552d1348347e9abfcf3eca1",
            x"b5f5eb6153ab5388a1a76343d714e4a2dcf224c5d0722d1e8e90c6bcead05c573fffe986460bd4000645a655bf52bc60",
            x"b922006ec14c183572a8864c31dc6632dccffa9f9c86411796f8b1b5a93a2457762c8e2f5ef0a2303506c4bca9a4e0bf",
            x"b53df1cfee2168f59e5792e710bf22928dc0553e6531dae5c7656c0a66fc12cb82fbb04863938c953dc901a5a79cc0f3",
        ];

        let i = 0;
        let accum_pk = std::vector::empty<vector<u8>>();
        while (i < std::vector::length(&pks)) {
            std::vector::push_back(&mut accum_pk, *std::vector::borrow(&pks, i));

            let (apk, ok) = bls12381_aggregate_pop_verified_pubkeys(accum_pk);
            assert!(ok == true, 1);

            // Make sure PKs were aggregated correctly
            assert!(apk == *std::vector::borrow(&agg_pks, i), 1);
            assert!(bls12381_validate_pubkey(apk), 1);

            i = i + 1;
        };
    }

    #[test]
    /// Tests verification of random BLS proofs-of-possession (PoPs)
    fun test_verify_pop() {
        // Test case generated by running `cargo test -- sample_pop --nocapture --include-ignored` in `crates/aptos-crypto`
        // =============================================================================================================

        let pks = vector[
            x"808864c91ae7a9998b3f5ee71f447840864e56d79838e4785ff5126c51480198df3d972e1e0348c6da80d396983e42d7",
            x"8843843c76d167c02842a214c21277bad0bfd83da467cb5cf2d3ee67b2dcc7221b9fafa6d430400164012580e0c34d27",
            x"a23b524d4308d46e43ee8cbbf57f3e1c20c47061ad9c3f915212334ea6532451dd5c01d3d3ada6bea10fe180b2c3b450",
            x"a2aaa3eae1df3fc36365491afa1da5181acbb03801afd1430f04bb3b3eb18036f8b756b3508e4caee04beff50d455d1c",
            x"84985b7e983dbdaddfca1f0b7dad9660bb39fff660e329acec15f69ac48c75dfa5d2df9f0dc320e4e7b7658166e0ac1c",
        ];

        let pops = vector[
            x"ab42afff92510034bf1232a37a0d31bc8abfc17e7ead9170d2d100f6cf6c75ccdcfedbd31699a112b4464a06fd636f3f190595863677d660b4c5d922268ace421f9e86e3a054946ee34ce29e1f88c1a10f27587cf5ec528d65ba7c0dc4863364",
            x"a6da5f2bc17df70ce664cff3e3a3e09d17162e47e652032b9fedc0c772fd5a533583242cba12095602e422e579c5284b1735009332dbdd23430bbcf61cc506ae37e41ff9a1fc78f0bc0d99b6bc7bf74c8f567dfb59079a035842bdc5fa3a0464",
            x"b8eef236595e2eab34d3c1abdab65971f5cfa1988c731ef62bd63c9a9ad3dfc9259f4f183bfffbc8375a38ba62e1c41a11173209705996ce889859bcbb3ddd7faa3c4ea3d8778f30a9ff814fdcfea1fb163d745c54dfb4dcc5a8cee092ee0070",
            x"a03a12fab68ad59d85c15dd1528560eff2c89250070ad0654ba260fda4334da179811d2ecdaca57693f80e9ce977d62011e3b1ee7bb4f7e0eb9b349468dd758f10fc35d54e0d0b8536ca713a77a301944392a5c192b6adf2a79ae2b38912dc98",
            x"8899b294f3c066e6dfb59bc0843265a1ccd6afc8f0f38a074d45ded8799c39d25ee0376cd6d6153b0d4d2ff8655e578b140254f1287b9e9df4e2aecc5b049d8556a4ab07f574df68e46348fd78e5298b7913377cf5bb3cf4796bfc755902bfdd",
        ];

        assert!(std::vector::length(&pks) == std::vector::length(&pops), 1);

        let i = 0;
        while (i < std::vector::length(&pks)) {
            let ok = bls12381_verify_proof_of_possession(*vector::borrow(&pks, i), *vector::borrow(&pops, i));
            assert!(ok == true, 1);
            i = i + 1;
        };

        // assert first PK's PoP does not verify against modifed PK' = 0xa0 | PK[1:]
        let ok = bls12381_verify_proof_of_possession(
            x"a08864c91ae7a9998b3f5ee71f447840864e56d79838e4785ff5126c51480198df3d972e1e0348c6da80d396983e42d7",
            x"ab42afff92510034bf1232a37a0d31bc8abfc17e7ead9170d2d100f6cf6c75ccdcfedbd31699a112b4464a06fd636f3f190595863677d660b4c5d922268ace421f9e86e3a054946ee34ce29e1f88c1a10f27587cf5ec528d65ba7c0dc4863364");
        assert!(ok == false, 2);

        // assert first PK's PoP does not verify if modifed as pop' = 0xb0 | pop[1:]
        let ok = bls12381_verify_proof_of_possession(
            x"808864c91ae7a9998b3f5ee71f447840864e56d79838e4785ff5126c51480198df3d972e1e0348c6da80d396983e42d7",
            x"bb42afff92510034bf1232a37a0d31bc8abfc17e7ead9170d2d100f6cf6c75ccdcfedbd31699a112b4464a06fd636f3f190595863677d660b4c5d922268ace421f9e86e3a054946ee34ce29e1f88c1a10f27587cf5ec528d65ba7c0dc4863364");
        assert!(ok == false, 3);
    }

    #[test]
    /// Tests verification of a random BLS signature created using sk = x""
    fun test_bls12381_verify_individual_signature() {
        // Test case generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in `crates/aptos-crypto`
        // =============================================================================================================
        // SK:        077c8a56f26259215a4a245373ab6ddf328ac6e00e5ea38d8700efa361bdc58d

        let ok = bls12381_verify_signature(
            x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7",
            x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858",
            b"Hello Aptos!",
        );

        assert!(ok == true, 1);
    }

    #[test]
    fun test_bls12381_pubkey_validation() {
        // test low order points (in group for PK)
        assert!(bls12381_validate_pubkey(x"ae3cd9403b69c20a0d455fd860e977fe6ee7140a7f091f26c860f2caccd3e0a7a7365798ac10df776675b3a67db8faa0") == false, 1);
        assert!(bls12381_validate_pubkey(x"928d4862a40439a67fd76a9c7560e2ff159e770dcf688ff7b2dd165792541c88ee76c82eb77dd6e9e72c89cbf1a56a68") == false, 1);
        assert!(bls12381_validate_pubkey(x"b3e4921277221e01ed71284be5e3045292b26c7f465a6fcdba53ee47edd39ec5160da3b229a73c75671024dcb36de091") == true, 1);
    }

    #[test]
    fun test_signature_aggregation() {
        // Signatures of each signer i
        let sigs = vector[
            x"a55ac2d64b4c1d141b15d876d3e54ad1eea07ee488e8287cce7cdf3eec551458ab5795ab196f8c112590346f7bc7c97e0053cd5be0f9bd74b93a87cd44458e98d125d6d5c6950ea5e62666beb34422ead79121f8cb0815dae41a986688d03eaf",
            x"90a639a44491191c46379a843266c293de3a46197714ead2ad3886233dd5c2b608b6437fa32fbf9d218b20f1cbfa7970182663beb9c148e2e9412b148e16abf283ffa51b8a536c0e55d61b2e5c849edc49f636c0ef07cb99f125cbcf602e22bb",
            x"9527d81aa15863ef3a3bf96bea6d58157d5063a93a6d0eb9d8b4f4bbda3b31142ec4586cb519da2cd7600941283d1bad061b5439703fd584295b44037a969876962ae1897dcc7cadf909d06faae213c4fef8e015dfb33ec109af02ab0c3f6833",
            x"a54d264f5cab9654b1744232c4650c42b29adf2b19bd00bbdaf4a4d792ee4dfd40a1fe1b067f298bcfd8ae4fdc8250660a2848bd4a80d96585afccec5c6cfa617033dd7913c9acfdf98a72467e8a5155d4cad589a72d6665be7cb410aebc0068",
            x"8d22876bdf73e6ad36ed98546018f6258cd47e45904b87c071e774a6ef4b07cac323258cb920b2fe2b07cca1f2b24bcb0a3194ec76f32edb92391ed2c39e1ada8919f8ea755c5e39873d33ff3a8f4fba21b1261c1ddb9d1688c2b40b77e355d1",
        ];

        // multisigs[i] is a signature on "Hello, Aptoverse!" from signers 1 through i (inclusive)
        let multisigs = vector[
            x"a55ac2d64b4c1d141b15d876d3e54ad1eea07ee488e8287cce7cdf3eec551458ab5795ab196f8c112590346f7bc7c97e0053cd5be0f9bd74b93a87cd44458e98d125d6d5c6950ea5e62666beb34422ead79121f8cb0815dae41a986688d03eaf",
            x"8f1949a06b95c3cb62898d861f889350c0d2cb740da513bfa195aa0ab8fa006ea2efe004a7bbbd9bb363637a279aed20132efd0846f520e7ee0e8ed847a1c6969bb986ad2239bcc9af561b6c2aa6d3016e1c722146471f1e28313de189fe7ebc",
            x"ab5ad42bb8f350f8a6b4ae897946a05dbe8f2b22db4f6c37eff6ff737aebd6c5d75bd1abdfc99345ac8ec38b9a449700026f98647752e1c99f69bb132340f063b8a989728e0a3d82a753740bf63e5d8f51e413ebd9a36f6acbe1407a00c4b3e7",
            x"ae307a0d055d3ba55ad6ec7094adef27ed821bdcf735fb509ab2c20b80952732394bc67ea1fd8c26ea963540df7448f8102509f7b8c694e4d75f30a43c455f251b6b3fd8b580b9228ffeeb9039834927aacefccd3069bef4b847180d036971cf",
            x"8284e4e3983f29cb45020c3e2d89066df2eae533a01cb6ca2c4d466b5e02dd22467f59640aa120db2b9cc49e931415c3097e3d54ff977fd9067b5bc6cfa1c885d9d8821aef20c028999a1d97e783ae049d8fa3d0bbac36ce4ca8e10e551d3461",
        ];

        let i = 0;
        let accum_sigs = std::vector::empty<vector<u8>>();
        while (i < std::vector::length(&sigs)) {
            std::vector::push_back(&mut accum_sigs, *std::vector::borrow(&sigs, i));
            
            let (multisig, ok) = bls12381_aggregate_signatures(accum_sigs);
            assert!(ok == true, 1);

            // Make sure sigs were aggregated correctly
            assert!(multisig == *std::vector::borrow(&multisigs, i), 1);
            assert!(bls12381_signature_subgroup_check(multisig), 1);

            i = i + 1;
        };
    }

    /// Random signature generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in `crates/aptos-crypto`.
    /// The message signed is "Hello Aptos!" and the associated SK is 07416693b6b32c84abe45578728e2379f525729e5b94762435a31e65ecc728da.
    const RANDOM_SIGNATURE: vector<u8> = x"a01a65854f987d3434149b7f08f70730e30b241984e8712bc2aca885d632aafced4c3f661209debb6b1c8601326623cc16ca2f6c9edc53b7b88b7435fb6b05ddece418d2c34dc6aca2f5a11a79e67774582c14084a01dcb7820e4cb4bad0ea8d";

    /// Random signature generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in `crates/aptos-crypto`.
    /// The associated SK is 07416693b6b32c84abe45578728e2379f525729e5b94762435a31e65ecc728da.
    const RANDOM_PK: vector<u8> = x"8a53e7ae5270e3e765cd8a4032c2e77c6f7e87a44ebb85bf28a4d7865565698f975346714262f9e47c6f3e0d5d951660";


    #[test]
    fun test_verify_aggsig() {
        // First, make sure verification returns None when no inputs are given or |pks| != |msgs|
        assert!(bls12381_verify_aggregate_signature(RANDOM_SIGNATURE, vector[], vector[]) == false, 1);
        assert!(bls12381_verify_aggregate_signature(RANDOM_SIGNATURE, vector[RANDOM_PK], vector[]) == false, 1);
        assert!(bls12381_verify_aggregate_signature(RANDOM_SIGNATURE, vector[], vector[ x"ab" ]) == false, 1);
        assert!(bls12381_verify_aggregate_signature(RANDOM_SIGNATURE, vector[RANDOM_PK], vector[x"cd", x"ef"]) == false, 1);
        assert!(bls12381_verify_aggregate_signature(RANDOM_SIGNATURE, vector[RANDOM_PK, RANDOM_PK, RANDOM_PK], vector[x"cd", x"ef"]) == false, 1);

        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- bls12381_sample_aggregate_pk_and_aggsig --nocapture --ignored

        // The signed messages are "Hello, Aptos <i>!", where <i> \in {1, ..., 5}
        let msgs = vector[
        x"48656c6c6f2c204170746f73203121",
        x"48656c6c6f2c204170746f73203221",
        x"48656c6c6f2c204170746f73203321",
        x"48656c6c6f2c204170746f73203421",
        x"48656c6c6f2c204170746f73203521",
        ];

        // Public key of signer i
        let pks = vector[
        x"b93d6aabb2b83e52f4b8bda43c24ea920bbced87a03ffc80f8f70c814a8b3f5d69fbb4e579ca76ee008d61365747dbc6",
        x"b45648ceae3a983bcb816a96db599b5aef3b688c5753fa20ce36ac7a4f2c9ed792ab20af6604e85e42dab746398bb82c",
        x"b3e4921277221e01ed71284be5e3045292b26c7f465a6fcdba53ee47edd39ec5160da3b229a73c75671024dcb36de091",
        x"8463b8671c9775a7dbd98bf76d3deba90b5a90535fc87dc8c13506bb5c7bbd99be4d257e60c548140e1e30b107ff5822",
        x"a79e3d0e9d04587a3b27d05efe5717da05fd93485dc47978c866dc70a01695c2efd247d1dd843a011a4b6b24079d7384",
        ];

        // aggsigs[i] = \sum_{j <= i}  sigs[j], where sigs[j] is a signature on msgs[j] under pks[j]
        let aggsigs = vector[
        x"a2bc8bdebe6215ba74b5b53c5ed2aa0c68221a4adf868989ccdcfb62bb0eecc6537def9ee686a7960169c5917d25e5220177ed1c5e95ecfd68c09694062e76efcb00759beac874e4f9a715fd144210883bf9bb272f156b0a1fa15d0e9460f01f",
        x"a523aa3c3f1f1074d968ffecf017c7b93ae5243006bf0abd2e45c036ddbec99302984b650ebe5ba306cda4071d281ba50a99ef0e66c3957fab94163296f9d673fc58a36de4276f82bfb1d9180b591df93b5c2804d40dd68cf0f72cd92f86442e",
        x"abed10f464de74769121fc09715e59a3ac96a5054a43a9d43cc890a2d4d332614c74c7fb4cceef6d25f85c65dee337330f062f89f23fec9ecf7ce3193fbba2c886630d753be6a4513a4634428904b767af2f230c5cadbcb53a451dd9c7d977f6",
        x"8362871631ba822742a31209fa4abce6dc94b741ac4725995459da2951324b51efbbf6bc3ab4681e547ebfbadd80e0360dc078c04188198f0acea26c12645ace9107a4a23cf8db46abc7a402637f16a0477c72569fc9966fe804ef4dc0e5e758",
        x"a44d967935fbe63a763ce2dd2b16981f967ecd31e20d3266eef5517530cdc233c8a18273b6d9fd7f61dd39178826e3f115df4e7b304f2de17373a95ea0c9a14293dcfd6f0ef416e06fa23f6a3c850d638e4d8f97ab4562ef55d49a96a50baa13",
        ];

        let i = 0;
        let msg_subset = std::vector::empty<vector<u8>>();
        let pk_subset = std::vector::empty<vector<u8>>();
        while (i < std::vector::length(&pks)) {

            let aggsig = *std::vector::borrow(&aggsigs, i);

            std::vector::push_back(&mut pk_subset, *std::vector::borrow(&pks, i));
            std::vector::push_back(&mut msg_subset, *std::vector::borrow(&msgs, i));

            assert!(bls12381_verify_aggregate_signature(aggsig, pk_subset, msg_subset), 1);

            i = i + 1;
        };
    }

    #[test]
    /// Tests verification of a random BLS signature created using sk = x""
    fun test_verify_normal_and_verify_sigshare() {
        // Test case generated by running `cargo test -- bls12381_sample_signature --nocapture --include-ignored` in
        // `crates/aptos-crypto`
        // =============================================================================================================
        // SK:        077c8a56f26259215a4a245373ab6ddf328ac6e00e5ea38d8700efa361bdc58d

        let message = b"Hello Aptos!";

        // First, test signatures that verify
        let ok = bls12381_verify_signature(
            x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7",
            x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858",
            message,
        );
        assert!(ok == true, 1);

        let pk_with_pop = x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858";

        let ok = bls12381_verify_signature_share(
            x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7",
            pk_with_pop,
            message,
        );
        assert!(ok == true, 1);

        // Second, test signatures that do NOT verify
        let sigs = vector[
            x"a01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7",
            x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7",
            x"b01ce4632e94d8c611736e96aa2ad8e0528a02f927a81a92db8047b002a8c71dc2d6bfb94729d0973790c10b6ece446817e4b7543afd7ca9a17c75de301ae835d66231c26a003f11ae26802b98d90869a9e73788c38739f7ac9d52659e1f7cf7",
        ];
        let pks = vector[
            x"94209a296b739577cb076d3bfb1ca8ee936f29b69b7dae436118c4dd1cc26fd43dcd16249476a006b8b949bf022a7858",
            x"ae4851bb9e7782027437ed0e2c026dd63b77a972ddf4bd9f72bcc218e327986568317e3aa9f679c697a2cb7cebf992f3",
            x"82ed7bb5528303a2e306775040a7309e0bd597b70d9949d8c6198a01a7be0b00079320ebfeaf7bbd5bfe86809940d252",
        ];
        let messages = vector[
            b"Hello Aptos!",
            b"Hello Aptos!",
            b"Bello Aptos!",
        ];

        let i = 0;
        while (i < std::vector::length(&pks)) {
            let sig = *std::vector::borrow(&sigs, i);
            let pk = *std::vector::borrow(&pks, i);
            let msg = *std::vector::borrow(&messages, i);

            let notok = bls12381_verify_signature(sig, pk, msg);
            assert!(notok == false, 1);

            let notok = bls12381_verify_signature_share(sig, pk, msg);
            assert!(notok == false, 1);

            i = i + 1;
        }
    }

    #[test]
    fun test_verify_multisig() {
        // Second, try some test-cases generated by running the following command in `crates/aptos-crypto`:
        //  $ cargo test -- sample_aggregate_pk_and_multisig --nocapture --include-ignored
        let pks = vector[
            x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a",
            x"ab9df801c6f96ade1c0490c938c87d5bcc2e52ccb8768e1b5d14197c5e8bfa562783b96711b702dda411a1a9f08ebbfa",
            x"b698c932cf7097d99c17bd6e9c9dc4eeba84278c621700a8f80ec726b1daa11e3ab55fc045b4dbadefbeef05c4182494",
            x"934706a8b876d47a996d427e1526ce52c952d5ec0858d49cd262efb785b62b1972d06270b0a7adda1addc98433ad1843",
            x"a4cd352daad3a0651c1998dfbaa7a748e08d248a54347544bfedd51a197e016bb6008e9b8e45a744e1a030cc3b27d2da",
        ];

        // agg_pks[i] = \sum_{j <= i}  pk[j]
        let agg_pks = vector[
            x"92e201a806af246f805f460fbdc6fc90dd16a18d6accc236e85d3578671d6f6690dde22134d19596c58ce9d63252410a",
            x"b79ad47abb441d7eda9b220a626df2e4e4910738c5f777947f0213398ecafae044ec0c20d552d1348347e9abfcf3eca1",
            x"b5f5eb6153ab5388a1a76343d714e4a2dcf224c5d0722d1e8e90c6bcead05c573fffe986460bd4000645a655bf52bc60",
            x"b922006ec14c183572a8864c31dc6632dccffa9f9c86411796f8b1b5a93a2457762c8e2f5ef0a2303506c4bca9a4e0bf",
            x"b53df1cfee2168f59e5792e710bf22928dc0553e6531dae5c7656c0a66fc12cb82fbb04863938c953dc901a5a79cc0f3",
        ];

        // multisigs[i] is a signature on "Hello, Aptoverse!" under agg_pks[i]
        let multisigs = vector[
            x"ade45c67bff09ae57e0575feb0be870f2d351ce078e8033d847615099366da1299c69497027b77badb226ff1708543cd062597030c3f1553e0aef6c17e7af5dd0de63c1e4f1f9da68c966ea6c1dcade2cdc646bd5e8bcd4773931021ec5be3fd",
            x"964af3d83436f6a9a382f34590c0c14e4454dc1de536af205319ce1ed417b87a2374863d5df7b7d5ed900cf91dffa7a105d3f308831d698c0d74fb2259d4813434fb86425db0ded664ae8f85d02ec1d31734910317d4155cbf69017735900d4d",
            x"b523a31813e771e55aa0fc99a48db716ecc1085f9899ccadb64e759ecb481a2fb1cdcc0b266f036695f941361de773081729311f6a1bca9d47393f5359c8c87dc34a91f5dae335590aacbff974076ad1f910dd81750553a72ccbcad3c8cc0f07",
            x"a945f61699df58617d37530a85e67bd1181349678b89293951ed29d1fb7588b5c12ebb7917dfc9d674f3f4fde4d062740b85a5f4927f5a4f0091e46e1ac6e41bbd650a74dd49e91445339d741e3b10bdeb9bc8bba46833e0011ff91fa5c77bd2",
            x"b627b2cfd8ae59dcf5e58cc6c230ae369985fd096e1bc3be38da5deafcbed7d939f07cccc75383539940c56c6b6453db193f563f5b6e4fe54915afd9e1baea40a297fa7eda74abbdcd4cc5c667d6db3b9bd265782f7693798894400f2beb4637",
        ];

        let i = 0;
        let accum_pk = std::vector::empty<vector<u8>>();
        while (i < std::vector::length(&pks)) {
            std::vector::push_back(&mut accum_pk, *std::vector::borrow(&pks, i));

            let (apk, succ) = bls12381_aggregate_pop_verified_pubkeys(accum_pk);
            assert!(succ == true, 1);

            assert!(apk == *std::vector::borrow(&agg_pks, i), 1);

            let msig = vector::borrow(&multisigs, i);
            assert!(bls12381_verify_multisig(*msig, apk, b"Hello, Aptoverse!"), 1);

            i = i + 1;
        };
    }
}