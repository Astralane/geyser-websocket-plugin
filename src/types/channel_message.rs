use solana_sdk::commitment_config::CommitmentConfig;
use crate::types::transaction::MessageTransaction;

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    Slot(u64, u64, CommitmentConfig),
    Transaction(Box<MessageTransaction>),
}