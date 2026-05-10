//! Expandable **Nucleus backend** handle: entitlement/opt-in/persona orchestration that feeds Blaze stubs.
//! Wire structs live beside Blaze handlers; this type is the stable anchor for future RPC-facing helpers.

/// Game-agnostic nucleus façade (singleton usage optional — hook from Accounts / session bridges later).
#[derive(Debug, Default, Clone)]
pub struct NucleusBackend;

impl NucleusBackend {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}
