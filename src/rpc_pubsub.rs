use crate::types::filters::{TransactionSubscribeFilter, TransactionSubscribeOptions};
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

#[rpc(server)]
pub trait GeyserPubSub {
    #[subscription(name = "slotSubscribe", unsubscribe = "slotUnsubscribe", item = String)]
    async fn slot_subscribe(&self, config: Option<CommitmentConfig>) -> SubscriptionResult;

    #[subscription(
        name = "transactionSubscribe", unsubscribe = "transactionUnsubscribe", item = String
    )]
    async fn transaction_subscribe(
        &self,
        filter: TransactionSubscribeFilter,
        options: TransactionSubscribeOptions,
    ) -> SubscriptionResult;

    #[subscription(
        name = "accountSubscribe", unsubscribe = "accountUnsubscribe", item = String
    )]
    async fn account_subscribe(
        &self,
        pubkey: Pubkey,
        config: Option<RpcAccountInfoConfig>,
    ) -> SubscriptionResult;
}
