//! Nexus backend handle for persona/account data used by Blaze stubs.

/// Game-agnostic Nexus façade.
#[derive(Debug, Default, Clone)]
pub struct NexusBackend;

impl NexusBackend {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}
