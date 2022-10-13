module nova_std::nft {
    use std::string::String;
    use std::error;
    use std::signer;
    use std::option::{Self, Option};
    use std::event::{Self, EventHandle};

    use nova_std::table::{Self, Table};
    use nova_std::type_info;

    //
    // Errors.
    //

    const ETOKEN_ADDRESS_MISMATCH: u64 = 0;

    const ECOLLECTION_ALREADY_EXISTS: u64 = 1;
    const ECOLLECTION_NOT_FOUND: u64 = 2;

    const ETOKEN_STORE_ALREADY_EXISTS: u64 = 3;
    const ETOKEN_STORE_NOT_FOUND: u64 = 4;

    const ETOKEN_ID_ALREADY_EXISTS: u64 = 5;
    const ETOKEN_ID_NOT_FOUND: u64 = 6;

    const ENOT_MUTABLE: u64 = 7;

    // Data structures

    /// Capability required to mint nfts.
    struct MintCapability<phantom Extension: store + drop + copy> has copy, store { }

    /// Collection information, store on the creator
    struct Collection<Extension: store + drop + copy> has key {
        /// Name of the collection
        name: String,
        /// Symbol of the collection
        symbol: String,
        /// Total supply of NFT
        token_count: u64,
        /// Mutability of extension And token_uri
        is_mutable: bool,
        /// All token information
        tokens: Table<String, TokenInfo<Extension>>,
    }

    /// The holder storage for specific nft collection 
    struct TokenStore<Extension: store + drop + copy> has key {
        tokens: Table<String, Token<Extension>>,
        deposit_events: EventHandle<DepositEvent>,
        withdraw_events: EventHandle<WithdrawEvent>,
        mint_events: EventHandle<MintEvent>,
        burn_events: EventHandle<BurnEvent>,
        update_events: EventHandle<UpdateEvent>,
    }

    /// Token Information
    struct TokenInfo<Extension: store + drop + copy> has store {
        token_id: String,
        token_uri: String,
        extension: Extension,
        /// Current owner of Token.
        /// If none, token is not in TokenStore
        owner: Option<address>,
    }

    struct Token<Extension: store + drop + copy> has store {
        token_id: String,
        token_uri: String,
        extension: Extension,
    }

    struct DepositEvent has drop, store {
        extension_type: String,
        token_id: String,
    }

    struct WithdrawEvent has drop, store {
        extension_type: String,
        token_id: String,
    }

    struct MintEvent has drop, store {
        extension_type: String,
        token_id: String,
    }

    struct BurnEvent has drop, store {
        extension_type: String,
        token_id: String,
    }

    struct UpdateEvent has drop, store {
        extension_type: String,
        token_id: String,
    }

    public entry fun register<Extension: store + drop + copy>(account: &signer) {
        assert!(
            !exists<TokenStore<Extension>>(signer::address_of(account)),
            error::not_found(ETOKEN_STORE_ALREADY_EXISTS),
        );

        let token_store = TokenStore<Extension> {
            tokens: table::new<String, Token<Extension>>(),
            deposit_events: event::new_event_handle<DepositEvent>(account),
            withdraw_events: event::new_event_handle<WithdrawEvent>(account),
            mint_events: event::new_event_handle<MintEvent>(account),
            burn_events: event::new_event_handle<BurnEvent>(account),
            update_events: event::new_event_handle<UpdateEvent>(account),
        };

        move_to(account, token_store);
    }

    ///
    /// Query entry functions(TODO)
    /// 

    public entry fun is_account_registered<Extension: store + drop + copy>(addr: address): bool {
        exists<TokenStore<Extension>>(addr)
    }

    public entry fun is_exists<Extension: store + drop + copy>(
        token_id: String
    ): bool acquires Collection {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        let collection = borrow_global<Collection<Extension>>(creator);

        table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id)
    }

    // TODO: check is safe?
    public entry fun get_extension<Extension: store + drop + copy>(
        token_id: String,
    ): Extension acquires Collection {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        let collection = borrow_global<Collection<Extension>>(creator);
        
        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        let token_info = table::borrow<String, TokenInfo<Extension>>(&collection.tokens, token_id);

        token_info.extension
    }

    public entry fun owner_of<Extension: store + drop + copy>(
        token_id: String,
    ): Option<address> acquires Collection {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        let collection = borrow_global<Collection<Extension>>(creator);
        
        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        let token_info = table::borrow<String, TokenInfo<Extension>>(&collection.tokens, token_id);

        token_info.owner
    }

    ///
    /// Execute entry functions
    /// 

    public entry fun burn<Extension: store + drop + copy>(
        account: &signer, 
        token_id: String,
    ) acquires Collection, TokenStore {
        let creator = token_address<Extension>();
        let addr = signer::address_of(account);

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        assert!(
            exists<TokenStore<Extension>>(addr),
            error::not_found(ETOKEN_STORE_NOT_FOUND),
        );

        let collection = borrow_global_mut<Collection<Extension>>(creator);

        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        let token_store = borrow_global_mut<TokenStore<Extension>>(addr);

        assert!(
            table::contains<String, Token<Extension>>(&token_store.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        let TokenInfo { token_id: _, token_uri: _, extension: _, owner: _ } 
            = table::remove<String, TokenInfo<Extension>>(&mut collection.tokens, token_id);

        let Token { token_id: _, token_uri: _, extension: _ } 
            = table::remove<String, Token<Extension>>(&mut token_store.tokens, token_id);

        collection.token_count = collection.token_count - 1;

        event::emit_event<BurnEvent>(
            &mut token_store.burn_events,
            BurnEvent { token_id, extension_type: type_info::type_name<Extension>() },
        );
    }

    public entry fun transfer<Extension: store + drop + copy>(
        account: &signer,
        to: address,
        token_id: String,
    ) acquires Collection, TokenStore {
        let token = withdraw<Extension>(account, token_id);
        deposit<Extension>(to, token);
    }

    public entry fun update<Extension: store + drop + copy>(
        account: &signer,
        token_id: String,
    ) acquires Collection, TokenStore {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        let collection = borrow_global<Collection<Extension>>(creator);

        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        assert!(
            collection.is_mutable,
            error::permission_denied(ENOT_MUTABLE),
        );

        let token_info = table::borrow<String, TokenInfo<Extension>>(&collection.tokens, token_id);

        let token_store = borrow_global_mut<TokenStore<Extension>>(signer::address_of(account));

        assert!(
            table::contains<String, Token<Extension>>(&token_store.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );
        
        let token = table::borrow_mut<String, Token<Extension>>(&mut token_store.tokens, token_id);

        token.token_uri = token_info.token_uri;
        token.extension = token_info.extension;

        event::emit_event<UpdateEvent>(
            &mut token_store.update_events,
            UpdateEvent { token_id, extension_type: type_info::type_name<Extension>() },
        );
    }

    ///
    /// Public functions
    /// 

    public fun make_collection<Extension: store + drop + copy>(
        account: &signer,
        name: String,
        symbol: String,
        is_mutable: bool
    ): MintCapability<Extension> {
        let creator = signer::address_of(account);

        assert!(
            token_address<Extension>() == creator,
            error::invalid_argument(ETOKEN_ADDRESS_MISMATCH),
        );

        assert!(
            !exists<Collection<Extension>>(creator),
            error::already_exists(ECOLLECTION_ALREADY_EXISTS),
        );

        let collection = Collection<Extension>{
            name,
            symbol,
            token_count: 0,
            is_mutable,
            tokens: table::new<String, TokenInfo<Extension>>(),
        };

        move_to(account, collection);

        MintCapability<Extension>{ }
    }

    public fun mint<Extension: store + drop + copy>(
        _account: &signer,
        to: address, 
        token_id: String,
        token_uri: String,
        extension: Extension,
        _mint_capability: &MintCapability<Extension>,
    ) acquires Collection, TokenStore {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        assert!(
            exists<TokenStore<Extension>>(to),
            error::not_found(ETOKEN_STORE_NOT_FOUND),
        );

        let collection = borrow_global_mut<Collection<Extension>>(creator);

        collection.token_count = collection.token_count + 1;

        assert!(
            !table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::already_exists(ETOKEN_ID_ALREADY_EXISTS),
        );

        let token_info = TokenInfo<Extension> { token_id, token_uri, extension, owner: option::some(to) };

        let token = Token<Extension> { token_id, token_uri, extension };

        table::add<String, TokenInfo<Extension>>(&mut collection.tokens, token_id, token_info);

        let token_store = borrow_global_mut<TokenStore<Extension>>(to);

        table::add<String, Token<Extension>>(&mut token_store.tokens, token_id, token);

        event::emit_event<MintEvent>(
            &mut token_store.mint_events,
            MintEvent { token_id, extension_type: type_info::type_name<Extension>() },
        );
    }

    public fun mutate_nft<Extension: store + drop + copy>(
        token_id: String,
        token_uri: Option<String>,
        extension: Option<Extension>,
        _mint_capability: &MintCapability<Extension>,
    ) acquires Collection {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        let collection = borrow_global_mut<Collection<Extension>>(creator);

        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        assert!(
            collection.is_mutable,
            error::permission_denied(ENOT_MUTABLE),
        );

        let token_info = table::borrow_mut<String, TokenInfo<Extension>>(&mut collection.tokens, token_id);

        if (option::is_some<String>(&token_uri)) {
            let new_token_uri = option::extract<String>(&mut token_uri);
            token_info.token_uri = new_token_uri;
        };

        if (option::is_some<Extension>(&extension)) {
            let new_extension = option::extract<Extension>(&mut extension);
            token_info.extension = new_extension;
        };
    }

    public fun withdraw<Extension: store + drop + copy>(
        account: &signer, 
        token_id: String,
    ): Token<Extension> acquires Collection, TokenStore {
        let creator = token_address<Extension>();
        let addr = signer::address_of(account);

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        assert!(
            exists<TokenStore<Extension>>(addr),
            error::not_found(ETOKEN_STORE_NOT_FOUND),
        );

        let collection = borrow_global_mut<Collection<Extension>>(creator);

        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        let token_info = table::borrow_mut<String, TokenInfo<Extension>>(&mut collection.tokens, token_id);

        token_info.owner = option::none<address>();

        let token_store = borrow_global_mut<TokenStore<Extension>>(addr);

        assert!(
            table::contains<String, Token<Extension>>(&token_store.tokens, token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        event::emit_event<WithdrawEvent>(
            &mut token_store.withdraw_events,
            WithdrawEvent { token_id, extension_type: type_info::type_name<Extension>() },
        );
        
        table::remove<String, Token<Extension>>(&mut token_store.tokens, token_id)
    }

    public fun deposit<Extension: store + drop + copy>(
        addr: address,
        token: Token<Extension>
    ) acquires Collection, TokenStore {
        let creator = token_address<Extension>();

        assert!(
            exists<Collection<Extension>>(creator),
            error::not_found(ECOLLECTION_NOT_FOUND),
        );

        assert!(
            exists<TokenStore<Extension>>(addr),
            error::not_found(ETOKEN_STORE_NOT_FOUND),
        );

        let collection = borrow_global_mut<Collection<Extension>>(creator);

        assert!(
            table::contains<String, TokenInfo<Extension>>(&collection.tokens, token.token_id),
            error::not_found(ETOKEN_ID_NOT_FOUND),
        );

        let token_info = table::borrow_mut<String, TokenInfo<Extension>>(&mut collection.tokens, token.token_id);

        token_info.owner = option::some<address>(addr);

        let token_store = borrow_global_mut<TokenStore<Extension>>(addr);

        event::emit_event<DepositEvent>(
            &mut token_store.deposit_events,
            DepositEvent { token_id: token.token_id, extension_type: type_info::type_name<Extension>() },
        );

        table::add<String, Token<Extension>>(&mut token_store.tokens, token.token_id, token); 
    }

    fun token_address<Extension: store + drop + copy>(): address {
        let type_info = type_info::type_of<Extension>();
        type_info::account_address(&type_info)
    }

    #[test_only]
    use std::string;

    #[test_only]
    struct Metadata has store, drop, copy{ 
        power: u64,
    }

    #[test_only]
    fun make_collection_for_test(account: &signer): MintCapability<Metadata> {
        // make collection
        let name = string::utf8(b"Collection");
        let symbol = string::utf8(b"COL");
        let is_mutable = true;
        make_collection<Metadata>(
            account,
            name,
            symbol,
            is_mutable,
        )
    }

    #[test(source = @0x1, destination = @0x2)]
    fun end_to_end(
        source: signer,
        destination: signer,
    ): MintCapability<Metadata> acquires Collection, TokenStore {
        let cap = make_collection_for_test(&source);

        let name = string::utf8(b"Collection");
        let symbol = string::utf8(b"COL");
        let is_mutable = true;

        // check collection
        let collection = borrow_global<Collection<Metadata>>(@nova_std);
        assert!(collection.name == name, 0);
        assert!(collection.symbol == symbol, 1);
        assert!(collection.is_mutable == is_mutable, 2);
        assert!(collection.token_count == 0, 3);

        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);
        
        // register
        register<Metadata>(&source);
        register<Metadata>(&destination);

        let token_id = string::utf8(b"id:1");
        let token_uri = string::utf8(b"https://url.com");
        let extension = Metadata { power: 1234 };

        mint<Metadata>(
            &source,
            source_addr,
            token_id,
            token_uri,
            extension,
            &cap,
        );

        // check minted token
        let token_store = borrow_global<TokenStore<Metadata>>(source_addr);
        let token = table::borrow(&token_store.tokens, string::utf8(b"id:1"));

        assert!(token.token_id == token_id, 4);
        assert!(token.token_uri == token_uri, 5);
        assert!(token.extension == extension, 6);

        // check token_count
        let collection = borrow_global<Collection<Metadata>>(@nova_std);

        assert!(collection.token_count == 1, 7);

        transfer<Metadata>(
            &source,
            destination_addr,
            string::utf8(b"id:1"),
        );
        // check transfered
        let token_store = borrow_global<TokenStore<Metadata>>(destination_addr);
        assert!(table::contains(&token_store.tokens, string::utf8(b"id:1")), 8);
        let token_store = borrow_global<TokenStore<Metadata>>(source_addr);
        assert!(!table::contains(&token_store.tokens, string::utf8(b"id:1")), 9);

        let token = withdraw<Metadata>(&destination, string::utf8(b"id:1"));
        // check withdrawn
        let token_store = borrow_global<TokenStore<Metadata>>(destination_addr);
        assert!(!table::contains(&token_store.tokens, string::utf8(b"id:1")), 10);

        let new_uri = string::utf8(b"https://new_url.com");
        let new_metadata = Metadata { power: 4321 };

        mutate_nft<Metadata>(
            string::utf8(b"id:1"),
            option::some<String>(new_uri),
            option::some<Metadata>(new_metadata),
            &cap,
        );

        // check token info
        let collection = borrow_global<Collection<Metadata>>(@nova_std);
        let token_info = table::borrow(&collection.tokens, string::utf8(b"id:1"));

        assert!(token_info.token_uri == new_uri, 11);
        assert!(token_info.extension == new_metadata, 12);

        deposit<Metadata>(destination_addr, token);

        // check deposit
        let token_store = borrow_global<TokenStore<Metadata>>(destination_addr);
        assert!(table::contains(&token_store.tokens, string::utf8(b"id:1")), 13);

        update<Metadata>(&destination, string::utf8(b"id:1"));

        // check update
        let token_store = borrow_global<TokenStore<Metadata>>(destination_addr);
        let token = table::borrow(&token_store.tokens, string::utf8(b"id:1"));
        
        assert!(token.token_uri == new_uri, 14);
        assert!(token.extension == new_metadata, 15);

        burn<Metadata>(&destination, string::utf8(b"id:1"));

        let token_store = borrow_global<TokenStore<Metadata>>(destination_addr);
        assert!(!table::contains(&token_store.tokens, string::utf8(b"id:1")), 16);

        // check burn

        cap
    }

    #[test(not_source = @0x2)]
    #[expected_failure(abort_code = 0x10000)]
    fun fail_make_collection_address_mismatch(not_source: signer): MintCapability<Metadata> {
        make_collection_for_test(&not_source)
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x80001)]
    fun fail_make_collection_collection_already_exists(
        source: signer
    ): (MintCapability<Metadata>, MintCapability<Metadata>) {
        let cap_1 = make_collection_for_test(&source);

        let cap_2 = make_collection_for_test(&source);

        (cap_1, cap_2)
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x60003)]
    fun fail_register(source: signer){
        register<Metadata>(&source);
        register<Metadata>(&source);
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x60002)]
    fun fail_mint_collection_not_found(source: signer): MintCapability<Metadata> acquires Collection, TokenStore {
        // It is impossible to get MintCapability without make_collection, but somehow..
        let cap = MintCapability<Metadata>{ };
        let token_id = string::utf8(b"id:1");
        let token_uri = string::utf8(b"https://url.com");
        let extension = Metadata { power: 1234 };

        mint<Metadata>(
            &source,
            signer::address_of(&source),
            token_id,
            token_uri,
            extension,
            &cap,
        );

        cap
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x60004)]
    fun fail_mint_token_store_not_found(source: signer): MintCapability<Metadata> acquires Collection, TokenStore {
        let cap = make_collection_for_test(&source);

        let token_id = string::utf8(b"id:1");
        let token_uri = string::utf8(b"https://url.com");
        let extension = Metadata { power: 1234 };

        mint<Metadata>(
            &source,
            signer::address_of(&source),
            token_id,
            token_uri,
            extension,
            &cap,
        );

        cap
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x80005)]
    fun fail_mint_token_id_exists(source: signer): MintCapability<Metadata> acquires Collection, TokenStore {
        let cap = make_collection_for_test(&source);

        let token_id = string::utf8(b"id:1");
        let token_uri = string::utf8(b"https://url.com");
        let extension = Metadata { power: 1234 };

        register<Metadata>(&source);

        mint<Metadata>(
            &source,
            signer::address_of(&source),
            token_id,
            token_uri,
            extension,
            &cap,
        );

        mint<Metadata>(
            &source,
            signer::address_of(&source),
            token_id,
            token_uri,
            extension,
            &cap,
        );

        cap
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x50007)]
    fun fail_mutate_not_mutable(source: signer): MintCapability<Metadata> acquires Collection, TokenStore {
        // make collection
        let name = string::utf8(b"Collection");
        let symbol = string::utf8(b"COL");
        let is_mutable = false;

        let cap = make_collection<Metadata>(
            &source,
            name,
            symbol,
            is_mutable,
        );

        let token_id = string::utf8(b"id:1");
        let token_uri = string::utf8(b"https://url.com");
        let extension = Metadata { power: 1234 };

        register<Metadata>(&source);

        mint<Metadata>(
            &source,
            signer::address_of(&source),
            token_id,
            token_uri,
            extension,
            &cap,
        );

        let new_uri = string::utf8(b"https://new_url.com");
        let new_metadata = Metadata { power: 4321 };

        mutate_nft<Metadata>(
            string::utf8(b"id:1"),
            option::some<String>(new_uri),
            option::some<Metadata>(new_metadata),
            &cap,
        );

        cap
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x60006)]
    fun fail_mutate_token_id_not_found(source: signer): MintCapability<Metadata> acquires Collection, TokenStore {
        let cap = make_collection_for_test(&source);

        let token_id = string::utf8(b"id:1");
        let token_uri = string::utf8(b"https://url.com");
        let extension = Metadata { power: 1234 };

        register<Metadata>(&source);

        mint<Metadata>(
            &source,
            signer::address_of(&source),
            token_id,
            token_uri,
            extension,
            &cap,
        );

        let new_uri = string::utf8(b"https://new_url.com");
        let new_metadata = Metadata { power: 4321 };

        mutate_nft<Metadata>(
            string::utf8(b"id:2"),
            option::some<String>(new_uri),
            option::some<Metadata>(new_metadata),
            &cap,
        );

        cap
    }
}