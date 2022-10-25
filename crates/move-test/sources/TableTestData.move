/// This module provides test tables of various key / value types, for use in API tests
module TestAccount::TableTestData {
    use std::vector;
    use std::signer;
    use std::option;
    use nova_std::table as T;

    struct S<phantom K: copy + drop,phantom V> has key {
        t: T::Table<K, V>
    }

    public entry fun simple_read_write(s: signer): u64{
        let t = T::new<u64, u64>(&s);
        T::add(&mut t, 1, 2);
        let two = *T::borrow(&t, 1);
        T::remove(&mut t, 1);
        T::destroy_empty(t);
        two
    }

    public entry fun table_len(s: signer): u64{
        let t = T::new<u64, u64>(&s);
        T::add(&mut t, 1, 1);
        T::add(&mut t, 2, 2);
        T::add(&mut t, 3, 3);
        let len = T::length(&t);
        move_to(&s, S { t });
        len
    }

    public entry fun move_table(s:signer, from:address) acquires S {
        let S { t } = move_from<S<u64, u64>>(from); 

        let tt = T::new<address, T::Table<u64, u64>>(&s);
        T::set_payer(&t, &s);
        T::add(&mut tt, @0xAA, t);

        move_to(&s, S {t: tt});
    }

    public entry fun table_of_tables(s: signer): vector<u8>{
        let t = T::new<address, T::Table<address, u8>>(&s);
        let val_1 = 11;
        let val_2 = 45;
        let val_3 = 78;

        // Create two small tables
        let t1 = T::new<address, u8>(&s);
        T::add(&mut t1, @0xAB, val_1);

        let t2 = T::new<address, u8>(&s);
        T::add(&mut t2, @0xCD, val_2);

        // Insert two small tables into the big table
        T::add(&mut t, @0x12, t1);
        T::add(&mut t, @0x34, t2);

        T::add(T::borrow_mut(&mut t, @0x12), @0xEF, val_3);

        let val = T::remove(T::borrow_mut(&mut t, @0x34), @0xCD);
        T::add(T::borrow_mut(&mut t, @0x34), @0xEE, 22);

        let vec_u8 = vector::empty<u8>();
        vector::push_back(&mut vec_u8, *T::borrow(T::borrow(&t, @0x12), @0xEF)); // == val_3
        vector::push_back(&mut vec_u8, *T::borrow(T::borrow(&t, @0x12), @0xAB)); // == val_1
        vector::push_back(&mut vec_u8, val); // == val_2
        
        move_to(&s, S { t });
        vec_u8
    }

    public entry fun table_borrow_mut(s: signer): u64 {
        let t = T::new<u64, u64>(&s);
        T::add(&mut t, 10, 2);
        *T::borrow_mut(&mut t, 10) = 3 ;
        let updated = *T::borrow(&t, 10);
        move_to(&s, S { t });
        updated
    }
   
    public entry fun table_borrow_mut_with_default(s: signer): u64{
        let t = T::new<u64, u64>(&s);
        let updated = *T::borrow_mut_with_default(&mut t, 10, 1000);
        move_to(&s, S { t });
        updated
    }

    public entry fun add_after_remove(s: signer): u64 {
        let t = T::new<u64, u64>(&s);
        T::add(&mut t, 42, 55);
        let fifty_five = *T::borrow(&t, 42);
        move_to(&s, S { t });
        fifty_five 
    }

    public entry fun table_borrow_global(s: signer): u64 acquires S {
        let acc = signer::address_of(&s);
        let t_ref = &borrow_global<S<u64, u64>>(acc).t;
        let v = *T::borrow(t_ref, 42);
        v
    }

    public entry fun table_move_from(s: signer): u64 acquires S {
        let t = T::new<u64, u64>(&s);
        T::add(&mut t, 42, 1012);
        T::add(&mut t, 43, 1013);
        move_to(&s, S { t });
        let acc = signer::address_of(&s);
        let S { t: local_t } = move_from<S<u64, u64>>(acc);
    
        let v = *T::borrow(&local_t, 43);
        move_to(&s, S { t: local_t });
        v
    }

    public entry fun table_remove(s: signer) acquires S {
        let acc = signer::address_of(&s);
        
        let S {t: tt} = move_from<S<u64, u64>>(acc);

        T::remove(&mut tt, 42);
        T::remove(&mut tt, 43);
        T::destroy_empty(tt);
    }

    public entry fun prepare_table_for_iterator(s: signer) {
        let t = T::new<u64, u64>(&s);

        T::add(&mut t, 1, 1);
        T::add(&mut t, 2, 2);
        T::add(&mut t, 3, 3);
        T::add(&mut t, 4, 4);
        T::add(&mut t, 5, 5);
        T::add(&mut t, 6, 6);
        T::add(&mut t, 7, 7);
        T::add(&mut t, 8, 8);
        T::add(&mut t, 9, 9);
        T::add(&mut t, 10, 10);

        move_to(&s, S { t });
    }

    public entry fun iterate_ascending(acc: address) acquires S {
        let t_ref = &borrow_global<S<u64, u64>>(acc).t;

        let iter = T::iter<u64, u64>(t_ref, option::none(), option::none(), 1);
        
        let i = 1;
        while(i < 11) {
            assert!(T::prepare<u64, u64>(&mut iter), 101);
            let (key, value) = T::next<u64, u64>(&mut iter);
            assert!(key == i, 101);
            assert!(value == &i, 101);

            i = i + 1;
        };
        assert!(!T::prepare<u64, u64>(&mut iter), 101);

        let iter = T::iter(t_ref, option::some(2), option::some(5), 1);
        
        let i = 2;
        while(i < 5) {
            assert!(T::prepare<u64, u64>(&mut iter), 102);
            let (key, value) = T::next(&mut iter);
            assert!(key == i, 102);
            assert!(value == &i, 102);

            i = i + 1;
        };
        assert!(!T::prepare<u64, u64>(&mut iter), 102);
    }

    public entry fun iterate_descending(acc: address) acquires S {
        let t_ref = &borrow_global<S<u64, u64>>(acc).t;

        let iter = T::iter<u64, u64>(t_ref, option::none(), option::none(), 2);
        
        let i = 10;
        while(i > 0) {
            assert!(T::prepare<u64, u64>(&mut iter), 101);
            let (key, value) = T::next(&mut iter);
            assert!(key == i, 101);
            assert!(value == &i, 101);

            i = i - 1;
        };
        assert!(!T::prepare<u64, u64>(&mut iter), 101);

        let iter = T::iter(t_ref, option::some(2), option::some(5), 2);
        
        let i = 4;
        while(i > 1) {
            assert!(T::prepare<u64, u64>(&mut iter), 102);
            let (key, value) = T::next(&mut iter);
            assert!(key == i, 102);
            assert!(value == &i, 102);

            i = i - 1;
        };
        assert!(!T::prepare<u64, u64>(&mut iter), 102);
    }
}
