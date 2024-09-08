use agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::Signature,
};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};
use solana_transaction_status::{EncodableWithMeta, EncodedTransaction, TransactionStatusMeta, UiTransaction, UiTransactionEncoding, UiTransactionStatusMeta};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,)]
pub struct MessageTransactionInfo {
    pub signature: Signature,
    pub is_vote: bool,
    pub transaction: EncodedTransaction,
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
                transaction: transaction.transaction.to_versioned_transaction().encode_with_meta(UiTransactionEncoding::JsonParsed, transaction.transaction_status_meta),
                index: transaction.index,
            },
            slot,
        }
    }
}