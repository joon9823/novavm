module 0x2::StdCoin {
    use 0x1::coin;
    use std::string;
    use std::signer;

    struct Std {}

    struct CapStore has key{
        burn: coin::BurnCapability<Std>,
        freeze: coin::FreezeCapability<Std>,
        mint: coin::MintCapability<Std>
    }

    entry fun init(sender: &signer) {
        let (burn, freeze, mint) = coin::initialize<Std>(sender, string::utf8(b"Std Coin"), string::utf8(b"STDC"), 8);

        move_to(sender, CapStore {
            burn, freeze, mint
        });
    }


    entry fun register(sender: &signer) {
        coin::register<Std>(sender);
    }

    entry fun mint(sender: &signer, account_to: address, amount: u64)  acquires CapStore {
        let sender_address = signer::address_of(sender);
        let caps = borrow_global<CapStore>(sender_address);
        
        let minted = coin::mint(amount, &caps.mint);
        coin::deposit(account_to, minted);
    }
}
