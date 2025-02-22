use crate::types::filters::{TransactionSubscribeFilter, TransactionSubscribeOptions};
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;

#[rpc(server)]
pub trait GeyserPubSub {
    #[subscription(name = "getVersion", unsubscribe = "getVersionUnsubscribe", item = String)]
    async fn get_version(&self) -> SubscriptionResult;

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
        pubkey_str: String,
        config: Option<RpcAccountInfoConfig>,
    ) -> SubscriptionResult;
}
