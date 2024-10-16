use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, TransactionDetails};

#[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSubscribeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_required: Option<Vec<String>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionCommitment {
    Processed,
    Confirmed,
    Finalized,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiEncoding {
    Base58,
    Base64,
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
    JsonParsed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSubscribeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<TransactionCommitment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<UiEncoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<TransactionDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_rewards: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supported_transaction_version: Option<u8>,
}

impl Default for TransactionSubscribeOptions {
    fn default() -> Self {
        Self {
            commitment: Some(TransactionCommitment::Confirmed),
            encoding: Some(UiEncoding::JsonParsed),
            transaction_details: Some(TransactionDetails::Full),
            show_rewards: Some(true),
            max_supported_transaction_version: Some(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionsConfig {
    pub filter: TransactionSubscribeFilter,
    pub options: TransactionSubscribeOptions,
}

// Websocket transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionNotification {
    pub transaction: EncodedTransactionWithStatusMeta,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountInfoConfig {
    pub encoding: Option<UiEncoding>, // supported values: base58, base64, base64+zstd, jsonParsed
    pub commitment: Option<CommitmentConfig>, // supported values: finalized, confirmed, processed - defaults to finalized
}

pub trait Filter {
    fn filter(&self) -> bool;
}
