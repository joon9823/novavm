module kernel_std::address{
    use std::string::String;

    native fun humanize_address(canonical_address: vector<u8>): String;
}