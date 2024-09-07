use solana_sdk::commitment_config::CommitmentConfig;
use crate::types::transaction::Transaction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMessage {
    Slot(u64, u64, CommitmentConfig),
    Transaction(Box<Transaction>),
}