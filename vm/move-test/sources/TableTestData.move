/// This module provides test tables of various key / value types, for use in API tests
module TestAccount::TableTestData {
    use std::vector;
    use std::table as T;

    struct S<phantom K: copy + drop, phantom V> has key {
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

    public entry fun table_len(account: signer): u64{
        let t = T::new<u64, u64>();
        T::add(&mut t, 1, 1);
        T::add(&mut t, 2, 2);
        T::add(&mut t, 3, 3);
        let len = T::length(&t);
        move_to(&account,S { t });
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
        vec_u8 // vec![val_3, val_1, val_2]
    }
    
}
