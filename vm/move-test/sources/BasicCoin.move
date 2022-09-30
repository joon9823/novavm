module std::BasicCoin {
    use std::debug;
    use std::signer;
    use std::event::{Self, EventHandle};

    struct Nova {}

    struct Coin<phantom CoinType> has key, copy {
        value: u64,
        test: bool,
    }

    struct TestEvents<phantom CoinType> has key {
        mint_events: EventHandle<MintEvent>,
    }

    /// Event emitted when some amount of coins are withdrawn from an Collateral.
    struct MintEvent has drop, store {
        amount: u64,
    }

    public entry fun mint<CoinType>(account: signer, value: u64) acquires Coin,TestEvents {
        let account_addr = signer::address_of(&account);
        if (!exists<Coin<CoinType>>(account_addr)) {
            move_to(&account, Coin<CoinType> { value, test: true });
        } else {
            let coin = borrow_global_mut<Coin<CoinType>>(account_addr);
            coin.value = coin.value + value;
        };

        if (!exists<TestEvents<CoinType>>(account_addr)) {
            move_to(&account, TestEvents<CoinType> {
                mint_events: event::new_event_handle<MintEvent>(&account),
            });
        };

        let test_events = borrow_global_mut<TestEvents<CoinType>>(account_addr);
        
        // emit event
        event::emit_event<MintEvent>(
            &mut test_events.mint_events,
            MintEvent {
                amount: value,
            }
        );
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