module nova_std::block {
    native fun get_block_info_internal(): (u64, u64);

    public fun get_block_info(): (u64, u64) {
        get_block_info_internal()
    }

    #[test_only]
    use std::unit_test::set_block_info_for_testing;

    #[test]
    public fun test_get_block_info_internal(){
        set_block_info_for_testing(12321u64, 9999999u64);

        let (height, timestamp) = get_block_info_internal();
        assert!(height == 12321u64, 0);
        assert!(timestamp == 9999999u64, 1);
    }
}
