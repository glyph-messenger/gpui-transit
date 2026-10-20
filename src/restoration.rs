//! Navigation state serialization and restoration for deep linking and session recovery.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Serializable navigation snapshot.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersistedNavigationState {
    /// Serialized route configurations in stack order.
    pub stack_routes: Vec<String>,
    /// Serialized active slot route if present.
    pub active_slot: Option<String>,
    /// Last active tab or pane index.
    pub active_tab: Option<usize>,
}

impl PersistedNavigationState {
    /// Creates a new empty navigation snapshot.
    pub fn new() -> Self {
        Self::default()
    }

    /// Serializes this snapshot to a JSON string if the `serde` feature is active.
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a navigation snapshot from a JSON string if the `serde` feature is active.
    #[cfg(feature = "serde")]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
