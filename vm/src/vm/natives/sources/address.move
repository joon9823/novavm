module kernel_std::address{

    native public fun humanize_address<T>(canonical_address: vector<u8>): T;
    native public fun canonicalize_address<T>(human_address: &T) : vector<u8>;
}