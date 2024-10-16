use crate::rpc_pubsub::GeyserPubSubServer;
use crate::types::account::{MessageAccount, MessageAccountInfo};
use crate::types::filters::{RpcAccountInfoConfig, RpcTransactionsConfig};
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use jsonrpsee::core::{async_trait, SubscriptionResult};
use jsonrpsee::tracing::error;
use jsonrpsee::PendingSubscriptionSink;
use log::{debug, warn};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

pub struct GeyserPubSubImpl {
    pub shutdown: Arc<AtomicBool>,
    pub slot_stream: broadcast::Receiver<MessageSlotInfo>,
    pub transaction_stream: broadcast::Receiver<MessageTransaction>,
    pub account_stream: broadcast::Receiver<MessageAccount>,
}

impl GeyserPubSubImpl {
    pub fn new(
        shutdown: Arc<AtomicBool>,
        slot_stream: broadcast::Receiver<MessageSlotInfo>,
        transaction_stream: broadcast::Receiver<MessageTransaction>,
        account_stream: broadcast::Receiver<MessageAccount>,
    ) -> Self {
        Self {
            shutdown,
            slot_stream,
            transaction_stream,
            account_stream,
        }
    }
}

#[async_trait]
impl GeyserPubSubServer for GeyserPubSubImpl {
    async fn slot_subscribe(
        &self,
        pending: PendingSubscriptionSink,
        config: Option<CommitmentConfig>,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let mut slot_stream = self.slot_stream.resubscribe();
        let stop = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                //check if shutdown is requested
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                let slot_response = slot_stream.recv().await;
                match slot_response {
                    Ok(slot) => {
                        if sink.is_closed() {
                            return;
                        }
                        //add filters here.
                        let resp = jsonrpsee::SubscriptionMessage::from_json(&slot).unwrap();
                        match sink.send(resp).await {
                            Ok(_) => {
                                continue;
                            }
                            Err(e) => {
                                error!("Error sending slot response: {:?}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => match e {
                        RecvError::Closed => {
                            debug!("slot subscription Closed");
                            return;
                        }
                        RecvError::Lagged(e) => {
                            warn!("slot subscription Lagged");
                            continue;
                        }
                    },
                }
            }
        });
        Ok(())
    }

    async fn transaction_subscribe(
        &self,
        pending: PendingSubscriptionSink,
        config: RpcTransactionsConfig,
    ) -> SubscriptionResult {
        todo!()
    }

    async fn account_update_subscribe(
        &self,
        pending: PendingSubscriptionSink,
        pub_key: Pubkey,
        config: RpcAccountInfoConfig,
    ) -> SubscriptionResult {
        todo!()
    }
}
