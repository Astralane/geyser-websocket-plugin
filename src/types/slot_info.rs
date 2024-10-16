use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::CommitmentLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSlotInfo {
    pub(crate) slot: u64,
    pub(crate) parent: Option<u64>,
    pub(crate) commitment: CommitmentLevel,
}

impl MessageSlotInfo {
    pub fn new(slot: u64, parent: Option<u64>, commitment: CommitmentLevel) -> Self {
        Self {
            slot,
            parent,
            commitment,
        }
    }
}
