module 0x77d96ae5e7885b19b5bf4e680e129ace8fd58fb1::TestCoin {
    struct Kernel {}

    struct Coin<phantom CoinType> has key, copy {
        value: u64,
        test: bool,
    }

    use std::debug;

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

    public entry fun get_coin<CoinType>(addr: address): Coin<CoinType> acquires Coin {
        *borrow_global<Coin<CoinType>>(addr)
    }

    public entry fun print_number(number: u64) {
        debug::print(&number)
    }
}