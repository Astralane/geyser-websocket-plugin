use agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfoV3;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccount;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAccount {
    pub account: MessageAccountInfo,
    pub slot: u64,
    pub is_startup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAccountInfo {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
    pub write_version: u64,
    pub txn_signature: Option<Signature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountData {
    pub pubkey: Pubkey,
    pub account: Account,
    pub write_version: u64,
}

impl<'a> From<(&'a ReplicaAccountInfoV3<'a>, u64, bool)> for MessageAccount {
    fn from((account, slot, is_startup): (&'a ReplicaAccountInfoV3<'a>, u64, bool)) -> Self {
        Self {
            account: MessageAccountInfo {
                pubkey: Pubkey::try_from(account.pubkey).expect("valid Pubkey"),
                lamports: account.lamports,
                owner: Pubkey::try_from(account.owner).expect("valid Pubkey"),
                executable: account.executable,
                rent_epoch: account.rent_epoch,
                data: account.data.into(),
                write_version: account.write_version,
                txn_signature: account.txn.map(|txn| *txn.signature()),
            },
            slot,
            is_startup,
        }
    }
}

impl From<MessageAccount> for AccountData {
    fn from(account: MessageAccount) -> Self {
        Self {
            pubkey: account.account.pubkey,
            account: Account {
                lamports: account.account.lamports,
                owner: account.account.owner,
                executable: account.account.executable,
                rent_epoch: account.account.rent_epoch,
                data: account.account.data,
            },
            write_version: account.account.write_version,
        }
    }
}

impl MessageAccount {
    pub fn to_notification(&self, config: Option<&RpcAccountInfoConfig>) -> UiAccount {
        let encoding = config
            .as_ref()
            .map(|c| c.encoding)
            .unwrap_or_default()
            .unwrap_or(solana_account_decoder::UiAccountEncoding::Base64);
        let data_slice = config.as_ref().map(|c| c.data_slice).unwrap_or_default();
        let account: AccountData = self.clone().into();
        UiAccount::encode(
            &account.pubkey,
            &account.account,
            encoding,
            None,
            data_slice,
        )
    }
}
