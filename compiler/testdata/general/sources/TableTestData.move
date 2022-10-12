/// This module provides test tables of various key / value types, for use in API tests
module TestAccount::TableTestData {
    use std::vector;
    use std::signer;
    use nova_std::table as T;

    struct S<phantom K: copy + drop,phantom V> has key {
        t: T::Table<K, V>
    }

    public entry fun simple_read_write(): u64{
        let t = T::new<u64, u64>();
        T::add(&mut t, 1, 2);
        let two = *T::borrow(&t, 1);
        T::remove(&mut t,1);
        T::destroy_empty(t);
        two
    }

    public entry fun table_len(s: signer): u64{
        let t = T::new<u64, u64>();
        T::add(&mut t, 1, 1);
        T::add(&mut t, 2, 2);
        T::add(&mut t, 3, 3);
        let len = T::length(&t);
        move_to(&s,S { t });
        len
    }

    public entry fun table_of_tables(s: signer): vector<u8>{
        let t = T::new<address, T::Table<address, u8>>();
        let val_1 = 11;
        let val_2 = 45;
        let val_3 = 78;

        // Create two small tables
        let t1 = T::new<address, u8>();
        T::add(&mut t1, @0xAB, val_1);

        let t2 = T::new<address, u8>();
        T::add(&mut t2, @0xCD, val_2);

        // Insert two small tables into the big table
        T::add(&mut t, @0x12, t1);
        T::add(&mut t, @0x34, t2);

        T::add(T::borrow_mut(&mut t, @0x12), @0xEF, val_3);

        let val = T::remove(T::borrow_mut(&mut t, @0x34), @0xCD);

        let vec_u8 = vector::empty<u8>();
        vector::push_back(&mut vec_u8, *T::borrow(T::borrow(&t, @0x12), @0xEF)); // == val_3
        vector::push_back(&mut vec_u8, *T::borrow(T::borrow(&t, @0x12), @0xAB)); // == val_1
        vector::push_back(&mut vec_u8, val); // == val_2
        
        move_to(&s, S { t });
        vec_u8
    }

    public entry fun table_borrow_mut(s: signer): u64 {
        let t = T::new<u64, u64>();
        T::add(&mut t, 10, 2);
        *T::borrow_mut(&mut t, 10) = 3 ;
        let updated = *T::borrow(&t, 10);
        move_to(&s, S { t });
        updated
    }
   
    public entry fun table_borrow_mut_with_default(s: signer): u64{
        let t = T::new<u64, u64>();
        let updated = *T::borrow_mut_with_default(&mut t, 10, 1000);
        move_to(&s, S { t });
        updated
    }

    public entry fun add_after_remove(s: signer): u64 {
        let t = T::new<u64, u64>();
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
        let t = T::new<u64, u64>();
        T::add(&mut t, 42, 1012);
        T::add(&mut t, 43, 1013);
        move_to(&s, S { t });
        let acc = signer::address_of(&s);
        let S { t: local_t } = move_from<S<u64, u64>>(acc);
    
        let v = *T::borrow(&local_t, 43);
        move_to(&s, S { t: local_t });
        v
    }
}
