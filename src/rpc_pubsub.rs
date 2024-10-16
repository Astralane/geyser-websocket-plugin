use crate::types::filters::{
    RpcAccountInfoConfig, RpcTransactionsConfig, TransactionSubscribeFilter,
    TransactionSubscribeOptions,
};
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

#[rpc(server)]
pub trait GeyserPubSub {
    #[subscription(name = "slotSubscribe", unsubscribe = "slotUnsubscribe", item = String)]
    async fn slot_subscribe(&self, config: Option<CommitmentConfig>) -> SubscriptionResult;

    #[subscription(
        name = "transactionSubscribe", unsubscribe = "transactionUnsubscribe", item = String
    )]
    async fn transaction_subscribe(&self, config: RpcTransactionsConfig) -> SubscriptionResult;

    #[subscription(
        name = "accountUpdateSubscribe", unsubscribe = "accountUpdateUnsubscribe", item = String
    )]
    async fn account_update_subscribe(
        &self,
        pubkey: Pubkey,
        config: RpcAccountInfoConfig,
    ) -> SubscriptionResult;
}
