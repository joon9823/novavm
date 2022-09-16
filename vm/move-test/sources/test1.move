module std::BasicCoin {
    struct Kernel {}

    struct Coin<phantom CoinType> has key, copy {
        value: u64,
        test: bool,
    }

    use std::debug;
    use kernel_std::bank;

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

    public entry fun balance(addr: address) : u64 {
        bank::balance(addr)
    }

    public entry fun transfer(from: address, to: address, amount: u64) {
        bank::transfer(from, to, amount)
    }
}