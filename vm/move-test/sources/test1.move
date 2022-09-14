module 0x1::BasicCoin {
    struct Coin has key, copy {
        value: u64,
        test: bool,
    }

    public entry fun mint(account: signer, value: u64) {
        move_to(&account, Coin { value, test: true })
    }

    public entry fun get(account: address): u64 acquires Coin{
        let c = borrow_global<Coin>(account);
        c.value
    }

    public entry fun number():u64 {
        123
    }

    public entry fun get_coin(addr: address): Coin acquires Coin {
        *borrow_global<Coin>(addr)
    }
}