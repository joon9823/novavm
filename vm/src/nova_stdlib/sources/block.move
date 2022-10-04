module nova_std::block {
    native fun get_block_info_internal(): (u64, u64);

    public fun get_block_info(): (u64, u64) {
        get_block_info_internal()
    }

    #[test]
    public fun test_get_block_info_internal(){
        let (height, timestamp) = get_block_info_internal();
        assert!(height == 100u64, 0);
        assert!(timestamp == 100u64, 1);
    }
}
