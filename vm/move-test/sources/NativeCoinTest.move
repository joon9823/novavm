module TestAccount::NativeCoinTest {
    use std::string;
    use std::signer;
    use nova_std::coin::{Self, BurnCapability, FreezeCapability, MintCapability};

    const ERR_UNAUTHORIZED: u64 = 0;
    const ERR_UNINITIALIZED: u64 = 1;
    const ERR_INITIALIZED: u64 = 2;

    struct TestCoin {}

    struct Capabilities has key {
        burn_capability: BurnCapability<TestCoin>,
        freeze_capability: FreezeCapability<TestCoin>,
        mint_capability: MintCapability<TestCoin>,
    }

    public fun initialized(): bool {
        exists<Capabilities>(@TestAccount)
    }

    public entry fun initialize(account: &signer) {
        assert!(!initialized(), ERR_INITIALIZED);
        assert!(signer::address_of(account) == @TestAccount, ERR_UNAUTHORIZED);

        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<TestCoin>(
            account,
            string::utf8(b"TestCoin"),
            string::utf8(b"TC"),
            8,
        );

        move_to(account, Capabilities {
            burn_capability: burn_cap,
            freeze_capability: freeze_cap,
            mint_capability: mint_cap
        })
    }

    public entry fun mint(account: &signer, recipient_addr: address, amount: u64) acquires Capabilities {
        assert!(initialized(), ERR_UNINITIALIZED);
        assert!(signer::address_of(account) == @TestAccount, ERR_UNAUTHORIZED);

        let cap = borrow_global<Capabilities>(@TestAccount);
        let test_coin = coin::mint(amount, &cap.mint_capability);
        coin::deposit(recipient_addr, test_coin);
    }
}