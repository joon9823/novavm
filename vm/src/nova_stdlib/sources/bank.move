module nova_std::bank{
    native public fun balance(addr : address): u64;
    // TODO: from should be a signer
    native public fun transfer(from: address, to: address,  amount: u64);
}