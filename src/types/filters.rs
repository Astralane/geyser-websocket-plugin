use crate::types::account::MessageAccount;
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::{
    EncodedTransactionWithStatusMeta, TransactionDetails, UiTransactionEncoding,
};
use std::collections::HashSet;
use std::str::FromStr;

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
    pub encoding: Option<UiTransactionEncoding>,
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
            encoding: Some(UiTransactionEncoding::Base64),
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

#[derive(Debug, Clone)]
pub struct FilterTransactions {
    vote: Option<bool>,
    failed: Option<bool>,
    signature: Option<Signature>,
    account_include: Vec<Pubkey>,
    account_exclude: Vec<Pubkey>,
    account_required: Vec<Pubkey>,
}

impl FilterTransactions {
    pub fn new(filter: TransactionSubscribeFilter) -> Self {
        let mut account_include = Vec::new();
        let mut account_exclude = Vec::new();
        let mut account_required = Vec::new();
        if let Some(account_include_str) = filter.account_include {
            account_include = account_include_str
                .iter()
                .map(|s| Pubkey::from_str(s).expect("valid Pubkey"))
                .collect();
        }
        if let Some(account_exclude_str) = filter.account_exclude {
            account_exclude = account_exclude_str
                .iter()
                .map(|s| Pubkey::from_str(s).expect("valid Pubkey"))
                .collect();
        }
        if let Some(account_required_str) = filter.account_required {
            account_required = account_required_str
                .iter()
                .map(|s| Pubkey::from_str(s).expect("valid Pubkey"))
                .collect();
        }
        Self {
            vote: filter.vote,
            failed: filter.failed,
            signature: filter
                .signature
                .map(|s| Signature::from_str(&s).expect("valid Signature")),
            account_include,
            account_exclude,
            account_required,
        }
    }

    pub fn allows(&self, message: &MessageTransaction) -> bool {
        if let Some(is_vote) = self.vote {
            if is_vote != message.transaction.is_vote {
                return false;
            }
        }

        if let Some(is_failed) = self.failed {
            if is_failed != message.transaction.meta.status.is_err() {
                return false;
            }
        }

        if let Some(signature) = &self.signature {
            if signature != message.transaction.transaction.signature() {
                return false;
            }
        }

        if !self.account_include.is_empty()
            && message
                .transaction
                .transaction
                .message()
                .account_keys()
                .iter()
                .all(|pubkey| self.account_include.binary_search(pubkey).is_err())
        {
            return false;
        }

        if !self.account_exclude.is_empty()
            && message
                .transaction
                .transaction
                .message()
                .account_keys()
                .iter()
                .any(|pubkey| self.account_exclude.binary_search(pubkey).is_ok())
        {
            return false;
        }

        if !self.account_required.is_empty() {
            let mut other: Vec<&Pubkey> = message
                .transaction
                .transaction
                .message()
                .account_keys()
                .iter()
                .collect();

            let is_subset = if self.account_required.len() <= other.len() {
                other.sort();
                self.account_required
                    .iter()
                    .all(|pubkey| other.binary_search(&pubkey).is_ok())
            } else {
                false
            };

            if !is_subset {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Default, Clone)]
pub struct FilterAccounts {
    owner: Option<Pubkey>,
    accounts: Option<HashSet<Pubkey>>,
}

impl FilterAccounts {
    pub fn new(account: Pubkey) -> Self {
        //add account to set for now
        let mut accounts = HashSet::new();
        accounts.insert(account);
        Self {
            owner: None,
            accounts: Some(accounts),
        }
    }

    pub fn allows(&self, message: &MessageAccount) -> bool {
        if let Some(owner) = self.owner {
            if owner != message.account.owner {
                return false;
            }
        }

        if let Some(accounts) = self.accounts.as_ref() {
            if !accounts.contains(&message.account.pubkey) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Default, Clone)]
pub struct FilterSlots {
    commitment_level: CommitmentLevel,
}

impl FilterSlots {
    pub fn new(commitment_level: Option<CommitmentConfig>) -> Self {
        let commitment_level = commitment_level
            .map(|config| config.commitment)
            .unwrap_or(CommitmentLevel::Processed);
        Self { commitment_level }
    }

    pub fn allows(&self, message: &MessageSlotInfo) -> bool {
        if self.commitment_level != message.commitment {
            return false;
        }
        true
    }
}
