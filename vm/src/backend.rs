use crate::errors::BackendError;
use crate::storage::state_view::StateView;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Backend<A: BackendApi, S: StateView, Q: Querier> {
    pub api: A,
    pub storage: S,
    pub querier: Q,
}

/// A result type for calling into the backend. Such a call can cause
/// non-negligible computational cost in both success and faiure case and must always have gas information
/// attached.
pub type BackendResult<T> = core::result::Result<T, BackendError>;

/// Callbacks to system functions defined outside of the wasm modules.
/// This is a trait to allow Mocks in the test code.
///
/// Currently it just supports address conversion, we could add eg. crypto functions here.
/// These should all be pure (stateless) functions. If you need state, you probably want
/// to use the Querier.
///
/// We can use feature flags to opt-in to non-essential methods
/// for backwards compatibility in systems that don't have them all.
pub trait BackendApi: Copy + Clone + Send {
    fn bank_transfer(&self, recipient: &[u8], denom: &str, amount: &str) -> BackendResult<()>;
}

pub trait Querier {
    /// This is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types.
    ///
    /// The gas limit describes how much VM gas this particular query is allowed
    /// to comsume when measured separately from the rest of the contract.
    /// The returned gas info (in BackendResult) can exceed the gas limit in cases
    /// where the query could not be aborted exactly at the limit.
    fn query_raw(
        &self,
        request: &[u8],
    ) -> BackendResult<Vec<u8>>; /* FIXME : put Vec<u8> as stub */
}
