use crate::types::filters::TransactionNotification;
use agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use anyhow::Context;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::SanitizedTransaction;
use solana_transaction_status::{
    TransactionStatusMeta, UiTransactionEncoding, VersionedTransactionWithStatusMeta,
};

#[derive(Debug, Clone)]
pub struct MessageTransactionInfo {
    pub signature: Signature,
    pub is_vote: bool,
    pub transaction: SanitizedTransaction,
    pub meta: TransactionStatusMeta,
    pub index: usize,
}
#[derive(Debug, Clone)]
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
                transaction: transaction.transaction.clone(),
                meta: transaction.transaction_status_meta.clone(),
                index: transaction.index,
            },
            slot,
        }
    }
}

impl MessageTransaction {
    pub fn to_notification(
        &self,
        ui_encoding: Option<UiTransactionEncoding>,
        max_supported_transaction_version: Option<u8>,
        show_rewards: bool,
    ) -> anyhow::Result<TransactionNotification> {
        let versioned = self.transaction.transaction.to_versioned_transaction();
        let versioned_with_meta = VersionedTransactionWithStatusMeta {
            transaction: versioned,
            meta: self.transaction.meta.clone(),
        };
        Ok(TransactionNotification {
            signature: self.transaction.transaction.signature().to_string(),
            transaction: versioned_with_meta
                .encode(
                    ui_encoding.unwrap_or(UiTransactionEncoding::Base64),
                    max_supported_transaction_version,
                    show_rewards,
                )
                .context("cannot encode transaction")?,
        })
    }
}
