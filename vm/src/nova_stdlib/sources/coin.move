/// This module provides the foundation for typesafe Coins.
module nova_std::coin {
    use std::string;
    use std::error;
    use std::signer;
    use std::event::{Self, EventHandle};

    use nova_std::type_info;
    use nova_std::account::create_signer_for_friend;

    //
    // Errors.
    //

    /// Address of account which is used to initialize a coin `CoinType` doesn't match the deployer of module
    const ECOIN_INFO_ADDRESS_MISMATCH: u64 = 1;

    /// `CoinType` is already initialized as a coin
    const ECOIN_INFO_ALREADY_PUBLISHED: u64 = 2;

    /// `CoinType` hasn't been initialized as a coin
    const ECOIN_INFO_NOT_PUBLISHED: u64 = 3;

    /// Account already has `CoinStore` registered for `CoinType`
    const ECOIN_STORE_ALREADY_PUBLISHED: u64 = 4;

    /// Account hasn't registered `CoinStore` for `CoinType`
    const ECOIN_STORE_NOT_PUBLISHED: u64 = 5;

    /// Not enough coins to complete transaction
    const EINSUFFICIENT_BALANCE: u64 = 6;

    /// Cannot destroy non-zero coins
    const EDESTRUCTION_OF_NONZERO_TOKEN: u64 = 7;

    /// Coin amount cannot be zero
    const EZERO_COIN_AMOUNT: u64 = 9;

    /// CoinStore is frozen. Coins cannot be deposited or withdrawn
    const EFROZEN: u64 = 10;

    /// Cannot upgrade the total supply of coins to different implementation.
    const ECOIN_SUPPLY_UPGRADE_NOT_SUPPORTED: u64 = 11;

    /// Name of the coin is too long
    const ECOIN_NAME_TOO_LONG: u64 = 12;

    /// Symbol of the coin is too long
    const ECOIN_SYMBOL_TOO_LONG: u64 = 13;

    /// unauthorized
    const ECOIN_UNAUTHORIZED: u64 = 14;

    //
    // Constants
    //

    const MAX_COIN_NAME_LENGTH: u64 = 32;
    const MAX_COIN_SYMBOL_LENGTH: u64 = 10;

    /// Core data structures

    /// Main structure representing a coin/token in an account's custody.
    struct Coin<phantom CoinType> has store {
        /// Amount of coin this address has.
        value: u64,
    }

    /// A holder of a specific coin types and associated event handles.
    /// These are kept in a single resource to ensure locality of data.
    struct CoinStore<phantom CoinType> has key {
        coin: Coin<CoinType>,
        frozen: bool,
        deposit_events: EventHandle<DepositEvent>,
        withdraw_events: EventHandle<WithdrawEvent>,
    }

    /// Maximum possible coin supply.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    /// Information about a specific coin type. Stored on the creator of the coin's account.
    struct CoinInfo<phantom CoinType> has key {
        name: string::String,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: string::String,
        /// Number of decimals used to get its user representation.
        /// For example, if `decimals` equals `2`, a balance of `505` coins should
        /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
        decimals: u8,
        supply: u128,
    }

    /// Event emitted when some amount of a coin is deposited into an account.
    struct DepositEvent has drop, store {
        amount: u64,
    }

    /// Event emitted when some amount of a coin is withdrawn from an account.
    struct WithdrawEvent has drop, store {
        amount: u64,
    }

    /// Capability required to mint coins.
    struct MintCapability<phantom CoinType> has copy, store { }

    /// Capability required to freeze a coin store.
    struct FreezeCapability<phantom CoinType> has copy, store { }

    /// Capability required to burn coins.
    struct BurnCapability<phantom CoinType> has copy, store { }

    //
    // Getter functions
    //

    /// A helper function that returns the address of CoinType.
    fun coin_address<CoinType>(): address {
        let type_info = type_info::type_of<CoinType>();
        type_info::account_address(&type_info)
    }

    /// Returns the balance of `owner` for provided `CoinType`.
    public entry fun balance<CoinType>(owner: address): u64 acquires CoinStore {
        assert!(
            is_account_registered<CoinType>(owner),
            error::not_found(ECOIN_STORE_NOT_PUBLISHED),
        );
        borrow_global<CoinStore<CoinType>>(owner).coin.value
    }

    /// Returns `true` if the type `CoinType` is an initialized coin.
    public entry fun is_coin_initialized<CoinType>(): bool {
        exists<CoinInfo<CoinType>>(coin_address<CoinType>())
    }

    /// Returns `true` if `account_addr` is registered to receive `CoinType`.
    public entry fun is_account_registered<CoinType>(account_addr: address): bool {
        exists<CoinStore<CoinType>>(account_addr)
    }

    /// Returns the name of the coin.
    public entry fun name<CoinType>(): string::String acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).name
    }

    /// Returns the symbol of the coin, usually a shorter version of the name.
    public entry fun symbol<CoinType>(): string::String acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).symbol
    }

    /// Returns the number of decimals used to get its user representation.
    /// For example, if `decimals` equals `2`, a balance of `505` coins should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    public entry fun decimals<CoinType>(): u8 acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).decimals
    }

    /// Returns the amount of coin in existence.
    public entry fun supply<CoinType>(): u128 acquires CoinInfo {
        borrow_global<CoinInfo<CoinType>>(coin_address<CoinType>()).supply
    }

    // Chain functions
    /// Creates a new Coin with given `CoinType`
    public entry fun native_initialize<CoinType>(
        chain: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ) {
        assert!(signer::address_of(chain) == @nova_std, error::invalid_argument(ECOIN_UNAUTHORIZED));
        assert!(!exists<CoinInfo<CoinType>>(@nova_std), error::already_exists(ECOIN_INFO_ALREADY_PUBLISHED));

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ECOIN_NAME_TOO_LONG));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ECOIN_SYMBOL_TOO_LONG));

        let coin_info = CoinInfo<CoinType> {
            name,
            symbol,
            decimals,
            supply: (0 as u128),
        };
        move_to(chain, coin_info);
    }

    /// Burn native coins to the recipient
    public entry fun native_burn_from<CoinType>(chain: &signer, account_addr: address, amount: u64) acquires CoinStore, CoinInfo {
        assert!(signer::address_of(chain) == @nova_std, ECOIN_UNAUTHORIZED);
        assert!(amount > 0, error::invalid_argument(EZERO_COIN_AMOUNT));

        let coin_info = borrow_global_mut<CoinInfo<CoinType>>(coin_address<CoinType>());
        coin_info.supply = coin_info.supply - (amount as u128);

        let Coin { value: _ } = withdraw_internal<CoinType>(account_addr, amount);
    }

    /// Mint native coins to the recipient
    public entry fun native_mint_to<CoinType>(chain: &signer, account_addr: address, amount: u64) acquires CoinStore, CoinInfo {
        assert!(signer::address_of(chain) == @nova_std, ECOIN_UNAUTHORIZED);
        assert!(amount > 0, error::invalid_argument(EZERO_COIN_AMOUNT));

        if (!is_account_registered<CoinType>(account_addr)) {
            let account = create_signer_for_friend(account_addr);
            register<CoinType>(&account);
        };
        
        deposit<CoinType>(account_addr, Coin<CoinType> { value: amount });

        let coin_info = borrow_global_mut<CoinInfo<CoinType>>(coin_address<CoinType>());
        coin_info.supply = coin_info.supply + (amount as u128);
    }

    // Public functions
    /// Burn `coin` with capability.
    /// The capability `_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    public fun burn<CoinType>(
        coin: Coin<CoinType>,
        _cap: &BurnCapability<CoinType>,
    ) acquires CoinInfo {
        let Coin { value: amount } = coin;
        assert!(amount > 0, error::invalid_argument(EZERO_COIN_AMOUNT));

        let coin_info = borrow_global_mut<CoinInfo<CoinType>>(coin_address<CoinType>());
        coin_info.supply = coin_info.supply - (amount as u128);
    }

    /// Burn `coin` from the specified `account` with capability.
    /// The capability `burn_cap` should be passed as a reference to `BurnCapability<CoinType>`.
    /// This function shouldn't fail as it's called as part of transaction fee burning.
    ///
    /// Note: This bypasses CoinStore::frozen -- coins within a frozen CoinStore can be burned.
    public fun burn_from<CoinType>(
        account_addr: address,
        amount: u64,
        burn_cap: &BurnCapability<CoinType>,
    ) acquires CoinInfo, CoinStore {
        // Skip burning if amount is zero. This shouldn't error out as it's called as part of transaction fee burning.
        if (amount == 0) {
            return
        };

        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        let coin_to_burn = extract(&mut coin_store.coin, amount);
        burn(coin_to_burn, burn_cap);
    }

    /// Deposit the coin balance into the recipient's account and emit an event.
    public fun deposit<CoinType>(account_addr: address, coin: Coin<CoinType>) acquires CoinStore {
        assert!(
            is_account_registered<CoinType>(account_addr),
            error::not_found(ECOIN_STORE_NOT_PUBLISHED),
        );

        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        assert!(
            !coin_store.frozen,
            error::permission_denied(EFROZEN),
        );

        event::emit_event<DepositEvent>(
            &mut coin_store.deposit_events,
            DepositEvent { amount: coin.value },
        );

        merge(&mut coin_store.coin, coin);
    }

    /// Destroys a zero-value coin. Calls will fail if the `value` in the passed-in `token` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Coin` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(zero_coin: Coin<CoinType>) {
        let Coin { value } = zero_coin;
        assert!(value == 0, error::invalid_argument(EDESTRUCTION_OF_NONZERO_TOKEN))
    }

    /// Extracts `amount` from the passed-in `coin`, where the original token is modified in place.
    public fun extract<CoinType>(coin: &mut Coin<CoinType>, amount: u64): Coin<CoinType> {
        assert!(coin.value >= amount, error::invalid_argument(EINSUFFICIENT_BALANCE));
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    /// Extracts the entire amount from the passed-in `coin`, where the original token is modified in place.
    public fun extract_all<CoinType>(coin: &mut Coin<CoinType>): Coin<CoinType> {
        let total_value = coin.value;
        coin.value = 0;
        Coin { value: total_value }
    }

    /// Freeze a CoinStore to prevent transfers
    public entry fun freeze_coin_store<CoinType>(
        account_addr: address,
        _freeze_cap: &FreezeCapability<CoinType>,
    ) acquires CoinStore {
        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        coin_store.frozen = true
    }

    /// Creates a new Coin with given `CoinType` and returns minting/burning capabilities.
    /// The given signer also becomes the account hosting the information  about the coin
    /// (name, supply, etc.). Supply is initialized as non-parallelizable integer.
    public fun initialize<CoinType>(
        account: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        initialize_internal(account, name, symbol, decimals)
    }

    fun initialize_internal<CoinType>(
        account: &signer,
        name: string::String,
        symbol: string::String,
        decimals: u8,
    ): (BurnCapability<CoinType>, FreezeCapability<CoinType>, MintCapability<CoinType>) {
        let account_addr = signer::address_of(account);

        assert!(
            coin_address<CoinType>() == account_addr,
            error::invalid_argument(ECOIN_INFO_ADDRESS_MISMATCH),
        );

        assert!(
            !exists<CoinInfo<CoinType>>(account_addr),
            error::already_exists(ECOIN_INFO_ALREADY_PUBLISHED),
        );

        assert!(string::length(&name) <= MAX_COIN_NAME_LENGTH, error::invalid_argument(ECOIN_NAME_TOO_LONG));
        assert!(string::length(&symbol) <= MAX_COIN_SYMBOL_LENGTH, error::invalid_argument(ECOIN_SYMBOL_TOO_LONG));

        let coin_info = CoinInfo<CoinType> {
            name,
            symbol,
            decimals,
            supply: (0 as u128),
        };
        move_to(account, coin_info);

        (BurnCapability<CoinType>{ }, FreezeCapability<CoinType>{ }, MintCapability<CoinType>{ })
    }

    /// "Merges" the two given coins.  The coin passed in as `dst_coin` will have a value equal
    /// to the sum of the two tokens (`dst_coin` and `source_coin`).
    public fun merge<CoinType>(dst_coin: &mut Coin<CoinType>, source_coin: Coin<CoinType>) {
        spec {
            assume dst_coin.value + source_coin.value <= MAX_U64;
        };
        dst_coin.value = dst_coin.value + source_coin.value;
        let Coin { value: _ } = source_coin;
    }

    /// Mint new `Coin` with capability.
    /// The capability `_cap` should be passed as reference to `MintCapability<CoinType>`.
    /// Returns minted `Coin`.
    public fun mint<CoinType>(
        amount: u64,
        _cap: &MintCapability<CoinType>,
    ): Coin<CoinType> acquires CoinInfo {
        if (amount == 0) {
            return zero<CoinType>()
        };

        let coin_info = borrow_global_mut<CoinInfo<CoinType>>(coin_address<CoinType>());
        coin_info.supply = coin_info.supply + (amount as u128);

        Coin<CoinType> { value: amount }
    }

    public entry fun register<CoinType>(account: &signer) {
        let account_addr = signer::address_of(account);
        assert!(
            !is_account_registered<CoinType>(account_addr),
            error::already_exists(ECOIN_STORE_ALREADY_PUBLISHED),
        );

        let coin_store = CoinStore<CoinType> {
            coin: Coin { value: 0 },
            frozen: false,
            deposit_events: event::new_event_handle<DepositEvent>(account),
            withdraw_events: event::new_event_handle<WithdrawEvent>(account),
        };
        move_to(account, coin_store);
    }

    /// Transfers `amount` of coins `CoinType` from `from` to `to`.
    public entry fun transfer<CoinType>(
        from: &signer,
        to: address,
        amount: u64,
    ) acquires CoinStore {
        let coin = withdraw<CoinType>(from, amount);
        deposit(to, coin);
    }

    /// Returns the `value` passed in `coin`.
    public fun value<CoinType>(coin: &Coin<CoinType>): u64 {
        coin.value
    }

    /// Withdraw specifed `amount` of coin `CoinType` from the signing account.
    public fun withdraw<CoinType>(
        account: &signer,
        amount: u64,
    ): Coin<CoinType> acquires CoinStore {
        withdraw_internal<CoinType>(signer::address_of(account), amount)
    }

    fun withdraw_internal<CoinType>(
        account_addr: address,
        amount: u64,
    ): Coin<CoinType> acquires CoinStore {
        assert!(
            is_account_registered<CoinType>(account_addr),
            error::not_found(ECOIN_STORE_NOT_PUBLISHED),
        );

        let coin_store = borrow_global_mut<CoinStore<CoinType>>(account_addr);
        assert!(
            !coin_store.frozen,
            error::permission_denied(EFROZEN),
        );

        event::emit_event<WithdrawEvent>(
            &mut coin_store.withdraw_events,
            WithdrawEvent { amount },
        );

        extract(&mut coin_store.coin, amount)
    }

    /// Create a new `Coin<CoinType>` with a value of `0`.
    public fun zero<CoinType>(): Coin<CoinType> {
        Coin<CoinType> {
            value: 0
        }
    }

    /// Destroy a freeze capability. Freeze capability is dangerous and therefore should be destroyed if not used.
    public fun destroy_freeze_cap<CoinType>(freeze_cap: FreezeCapability<CoinType>) {
        let FreezeCapability<CoinType> { } = freeze_cap;
    }

    /// Destroy a mint capability.
    public fun destroy_mint_cap<CoinType>(mint_cap: MintCapability<CoinType>) {
        let MintCapability<CoinType> { } = mint_cap;
    }

    /// Destroy a burn capability.
    public fun destroy_burn_cap<CoinType>(burn_cap: BurnCapability<CoinType>) {
        let BurnCapability<CoinType> { } = burn_cap;
    }

    #[test_only]
    struct FakeMoney { }

    #[test_only]
    struct FakeMoneyCapabilities has key {
        burn_cap: BurnCapability<FakeMoney>,
        freeze_cap: FreezeCapability<FakeMoney>,
        mint_cap: MintCapability<FakeMoney>,
    }

    #[test_only]
    fun initialize_fake_money(
        account: &signer,
        decimals: u8,
    ): (BurnCapability<FakeMoney>, FreezeCapability<FakeMoney>, MintCapability<FakeMoney>) {
        initialize<FakeMoney>(
            account,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            decimals,
        )
    }

    #[test_only]
    fun initialize_and_register_fake_money(
        account: &signer,
        decimals: u8,
    ): (BurnCapability<FakeMoney>, FreezeCapability<FakeMoney>, MintCapability<FakeMoney>) {
        let (burn_cap, freeze_cap, mint_cap) = initialize_fake_money(
            account,
            decimals,
        );
        register<FakeMoney>(account);
        (burn_cap, freeze_cap, mint_cap)
    }

    #[test_only]
    public entry fun create_fake_money(
        source: &signer,
        destination: &signer,
        amount: u64
    ) acquires CoinInfo, CoinStore {
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(source, 18);

        register<FakeMoney>(destination);
        let coins_minted = mint<FakeMoney>(amount, &mint_cap);
        deposit(signer::address_of(source), coins_minted);
        move_to(source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    public entry fun end_to_end_native_coins(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore {
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);

        let name = string::utf8(b"Fake money");
        let symbol = string::utf8(b"FMD");

        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            &source,
            name,
            symbol,
            18
        );
        register<FakeMoney>(&source);
        register<FakeMoney>(&destination);
        assert!(supply<FakeMoney>() == 0, 0);

        assert!(name<FakeMoney>() == name, 1);
        assert!(symbol<FakeMoney>() == symbol, 2);
        assert!(decimals<FakeMoney>() == 18, 3);

        native_mint_to<FakeMoney>(&source, destination_addr, 100);

        assert!(balance<FakeMoney>(source_addr) == 0, 4);
        assert!(balance<FakeMoney>(destination_addr) == 100, 5);
        assert!(supply<FakeMoney>() == 100, 6);

        native_burn_from<FakeMoney>(&source, destination_addr, 10);
        assert!(supply<FakeMoney>() == 90, 7);
        assert!(balance<FakeMoney>(destination_addr) == 90, 8);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    public entry fun end_to_end(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore {
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);

        let name = string::utf8(b"Fake money");
        let symbol = string::utf8(b"FMD");

        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            &source,
            name,
            symbol,
            18
        );
        register<FakeMoney>(&source);
        register<FakeMoney>(&destination);
        assert!(supply<FakeMoney>() == 0, 0);

        assert!(name<FakeMoney>() == name, 1);
        assert!(symbol<FakeMoney>() == symbol, 2);
        assert!(decimals<FakeMoney>() == 18, 3);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(source_addr, coins_minted);
        transfer<FakeMoney>(&source, destination_addr, 50);

        assert!(balance<FakeMoney>(source_addr) == 50, 4);
        assert!(balance<FakeMoney>(destination_addr) == 50, 5);
        assert!(supply<FakeMoney>() == 100, 6);

        let coin = withdraw<FakeMoney>(&source, 10);
        assert!(value(&coin) == 10, 7);
        burn(coin, &burn_cap);
        assert!(supply<FakeMoney>() == 90, 8);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    #[expected_failure(abort_code = 14)]
    public entry fun fail_native_mint_to(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore {
        let destination_addr = signer::address_of(&destination);

        let name = string::utf8(b"Fake money");
        let symbol = string::utf8(b"FMD");

        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            &source,
            name,
            symbol,
            18
        );
        register<FakeMoney>(&source);
        register<FakeMoney>(&destination);
        assert!(supply<FakeMoney>() == 0, 0);

        assert!(name<FakeMoney>() == name, 1);
        assert!(symbol<FakeMoney>() == symbol, 2);
        assert!(decimals<FakeMoney>() == 18, 3);

        native_mint_to<FakeMoney>(&destination, destination_addr, 100);
        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    #[expected_failure(abort_code = 14)]
    public entry fun fail_native_burn_from(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore {
        let destination_addr = signer::address_of(&destination);

        let name = string::utf8(b"Fake money");
        let symbol = string::utf8(b"FMD");

        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            &source,
            name,
            symbol,
            18
        );
        register<FakeMoney>(&source);
        register<FakeMoney>(&destination);
        assert!(supply<FakeMoney>() == 0, 0);

        assert!(name<FakeMoney>() == name, 1);
        assert!(symbol<FakeMoney>() == symbol, 2);
        assert!(decimals<FakeMoney>() == 18, 3);

        native_mint_to<FakeMoney>(&source, destination_addr, 100);
        native_burn_from<FakeMoney>(&destination, destination_addr, 10);
        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x2, nova_std = @nova_std)]
    #[expected_failure(abort_code = 0x10001)]
    public fun fail_initialize(source: signer) {
        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            &source,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            1,
        );

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    #[expected_failure(abort_code = 0x60005)]
    public entry fun fail_transfer(
        source: signer,
        destination: signer,
    ) acquires CoinInfo, CoinStore {
        let source_addr = signer::address_of(&source);
        let destination_addr = signer::address_of(&destination);

        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1);
        assert!(supply<FakeMoney>() == 0, 0);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(source_addr, coins_minted);
        transfer<FakeMoney>(&source, destination_addr, 50);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1, destination = @0x2)]
    public entry fun test_burn_from_with_capability(
        source: signer,
    ) acquires CoinInfo, CoinStore {
        let source_addr = signer::address_of(&source);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1);
        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(source_addr, coins_minted);
        assert!(balance<FakeMoney>(source_addr) == 100, 0);
        assert!(supply<FakeMoney>() == 100, 1);

        burn_from<FakeMoney>(source_addr, 10, &burn_cap);
        assert!(balance<FakeMoney>(source_addr) == 90, 2);
        assert!(supply<FakeMoney>() == 90, 3);

        move_to(&source, FakeMoneyCapabilities{
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    #[expected_failure(abort_code = 0x10007)]
    public fun test_destroy_non_zero(
        source: signer,
    ) acquires CoinInfo {
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1);
        let coins_minted = mint<FakeMoney>( 100, &mint_cap);
        destroy_zero(coins_minted);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    public entry fun test_extract(
        source: signer,
    ) acquires CoinInfo, CoinStore {
        let source_addr = signer::address_of(&source);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&source, 1);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);

        let extracted = extract(&mut coins_minted, 25);
        assert!(value(&coins_minted) == 75, 0);
        assert!(value(&extracted) == 25, 1);

        deposit(source_addr, coins_minted);
        deposit(source_addr, extracted);

        assert!(balance<FakeMoney>(source_addr) == 100, 2);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(source = @0x1)]
    public fun test_is_coin_initialized(source: signer) {
        assert!(!is_coin_initialized<FakeMoney>(), 0);

        let (burn_cap, freeze_cap, mint_cap) = initialize_fake_money(&source, 1);
        assert!(is_coin_initialized<FakeMoney>(), 1);

        move_to(&source, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test]
    fun test_zero() {
        let zero = zero<FakeMoney>();
        assert!(value(&zero) == 0, 1);
        destroy_zero(zero);
    }

    #[test(account = @0x1)]
    public entry fun burn_frozen(account: signer) acquires CoinInfo, CoinStore {
        let account_addr = signer::address_of(&account);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        deposit(account_addr, coins_minted);

        freeze_coin_store(account_addr, &freeze_cap);
        burn_from(account_addr, 100, &burn_cap);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @0x1)]
    #[expected_failure(abort_code = 0x5000A)]
    public entry fun withdraw_frozen(account: signer) acquires CoinStore {
        let account_addr = signer::address_of(&account);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18);

        freeze_coin_store(account_addr, &freeze_cap);
        let coin = withdraw<FakeMoney>(&account, 10);
        deposit(account_addr, coin);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test(account = @0x1)]
    #[expected_failure(abort_code = 0x5000A)]
    public entry fun deposit_frozen(account: signer) acquires CoinInfo, CoinStore {
        let account_addr = signer::address_of(&account);
        let (burn_cap, freeze_cap, mint_cap) = initialize_and_register_fake_money(&account, 18);

        let coins_minted = mint<FakeMoney>(100, &mint_cap);
        freeze_coin_store(account_addr, &freeze_cap);
        deposit(account_addr, coins_minted);

        move_to(&account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }

    #[test_only]
    fun initialize_with_integer(account: &signer) {
        let (burn_cap, freeze_cap, mint_cap) = initialize<FakeMoney>(
            account,
            string::utf8(b"Fake money"),
            string::utf8(b"FMD"),
            1,
        );
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }
}
