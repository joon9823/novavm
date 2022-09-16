module std::BasicCoin {
    struct Kernel {}

    struct Coin<phantom CoinType> has key, copy {
        value: u64,
        test: bool,
    }

    public entry fun mint<CoinType>(account: signer, value: u64) {
        move_to(&account, Coin<CoinType> { value, test: true })
    }

    public entry fun get<CoinType>(account: address): u64 acquires Coin{
        let c = borrow_global<Coin<CoinType>>(account);
        c.value
    }

    public entry fun number():u64 {
        123
    }

    public entry fun getCoin<CoinType>(addr: address): Coin<CoinType> acquires Coin {
        *borrow_global<Coin<CoinType>>(addr)
    }
}