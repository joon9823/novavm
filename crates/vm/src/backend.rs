use nova_natives::block::BlockInfoResolver;
use nova_storage::state_view::StateView;
use nova_types::errors::BackendError;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Backend<A: BlockInfoResolver, S: StateView> {
    pub api: A,
    pub storage: S,
}

/// A result type for calling into the backend. Such a call can cause
/// non-negligible computational cost in both success and failure case and must always have gas information
/// attached.
pub type BackendResult<T> = core::result::Result<T, BackendError>;
