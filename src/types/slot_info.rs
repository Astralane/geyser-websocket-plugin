use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSlotInfo {
    slot: u64,
    parent: u64,
    commitment: CommitmentLevel,
}

impl MessageSlotInfo {
    pub fn new(slot: u64, parent: u64, commitment: CommitmentLevel) -> Self {
        Self {
            slot,
            parent,
            commitment,
        }
    }
}
