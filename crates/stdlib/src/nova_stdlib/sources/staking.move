module nova_std::staking {
    use std::error;
    use std::event::{Self, EventHandle};
    use std::fixed_point32::{Self, FixedPoint32};
    use std::string::String;
    use std::table::{Self, Table};
    use std::signer;
    use std::coin::{Self, Coin};

    /// TODO - temporal coin structure should be changed
    struct StakeCoin {}
    
    /// Account already has `DelegationStore` registered
    const EDELEGATION_STORE_ALREADY_PUBLISHED: u64 = 1;
    
    /// Validator of delegation which is used as operand doesn't match the other operand one
    const EDELEGATOIN_VALIDATOR_MISMATCH: u64 = 2;

    /// Store a reward index which is represending rewards per share
    /// and collected reward coins
    struct Reward has store {
        // TODO - change FixedPoint32 to the data type with more precision
        index: FixedPoint32,
        coin: Coin<StakeCoin>,
    }

    /// Global storage for the rewards
    struct RewardStore has key {
        rewards: Table<String, Reward>,
        reward_events: EventHandle<RewardEvent>,
    }

    /// Define a delegation entry which can be transferred.
    struct Delegation has store {
        share: u64,
        validator: String,
        reward: Reward,
    }

    /// A holder of a delegation and associated event handles.
    /// These are kept in a single resource to ensure locality of data.
    struct DelegationStore has key {
        delegations: Table<String, Delegation>,
        delegate_events: EventHandle<DelegateEvent>,
        undelegate_events: EventHandle<UndelegateEvent>,
    }

    /// Event emitted when some amount of reward is deposited.
    struct RewardEvent has drop, store {
        amount: u64
    }

    /// Event emitted when some share of a coin is delegated from an account.
    struct DelegateEvent has drop, store {
        share: u64,
        validator: String,
    }

    /// Event emitted when some share of a coin is undelegated from an account.
    struct UndelegateEvent has drop, store {
        share: u64,
        validator: String,
    }

    /// Check the DelegationStore is already exist
    public fun is_account_registered(account_addr: address): bool {
        exists<DelegationStore>(account_addr)
    }

    /// Register delegation store for the account
    public fun register(account: &signer) {
        let account_addr = signer::address_of(account);
        assert!(
            !is_account_registered(account_addr),
            error::already_exists(EDELEGATION_STORE_ALREADY_PUBLISHED),
        );

        let delegation_store = DelegationStore {
            delegations: table::new<String, Delegation>(account),
            delegate_events: event::new_event_handle<DelegateEvent>(account),
            undelegate_events: event::new_event_handle<UndelegateEvent>(account),
        };

        move_to(account, delegation_store);
    }

    /// return empty delegation resource
    public fun empty(validator: String): Delegation {
        Delegation {
            share: 0,
            validator,
            reward: Reward { 
                index: fixed_point32::create_from_rational(0, 1), 
                coin: coin::zero(),
            },
        }
    }

    /// "Merges" the two given delegations.  The delegation passed in as `dst_delegation` will have a value equal
    /// to the sum of the two shares (`dst_delegation` and `source_delegation`).
    public fun merge(dst_delegation: &mut Delegation, source_delegation: Delegation) {
        assert!(dst_delegation.validator == source_delegation.validator, EDELEGATOIN_VALIDATOR_MISMATCH);

        spec {
            assume dst_delegation.share + source_delegation.share <= MAX_U64;
        };

        // TODO - before_share_changes
        
        dst_delegation.share = dst_delegation.share + source_delegation.share;
        let Delegation { share: _, validator: _, reward } = source_delegation;
        let Reward { index: _, coin } = reward;
        coin::merge(&mut dst_delegation.reward.coin, coin);
    }

    /// Delegate a coin to a validator of the given Delegation object.
    // public fun delegate(delegation: &mut Delegation, amount: Coin<StakeCoin>) {

    // }

    // fun before_share_change(delegation: &mut Delegation): acquires 
}