use agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::Signature,
};
use solana_sdk::transaction::SanitizedTransaction;
use solana_transaction_status::{TransactionStatusMeta, UiTransactionStatusMeta};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,)]
pub struct MessageTransactionInfo {
    pub signature: Signature,
    pub is_vote: bool,
    //pub transaction: SanitizedTransaction,
    pub meta: UiTransactionStatusMeta,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,)]
pub struct MessageTransaction {
    pub transaction: MessageTransactionInfo,
    pub slot: u64,
}
impl<'a> From<(&'a ReplicaTransactionInfoV2<'a>, u64)> for MessageTransaction {
    fn from((transaction, slot): (&'a ReplicaTransactionInfoV2<'a>, u64)) -> Self {
        Self {
            transaction: MessageTransactionInfo {
                signature: *transaction.signature,
                is_vote: transaction.is_vote,
                //transaction: transaction.transaction.clone(),
                meta: transaction.transaction_status_meta.clone().into(),
                index: transaction.index,
            },
            slot,
        }
    }
}