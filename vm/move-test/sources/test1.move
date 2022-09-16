module 0x1::BasicCoin {

    use std::debug;
    use std::vector;
    use kernel_std::bank;
    use std::string::String;
    use std::signer;

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