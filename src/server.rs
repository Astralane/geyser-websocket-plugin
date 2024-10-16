use crate::rpc_pubsub::GeyserPubSubServer;
use crate::types::account::{MessageAccount, MessageAccountInfo};
use crate::types::filters::{RpcAccountInfoConfig, RpcTransactionsConfig};
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use jsonrpsee::core::{async_trait, SubscriptionResult};
use jsonrpsee::tracing::error;
use jsonrpsee::PendingSubscriptionSink;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};

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
                        RecvError::Lagged(skipped) => {
                            warn!("slot subscription Lagged skipped {}", skipped);
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
        let sink = pending.accept().await?;
        let mut transaction_stream = self.transaction_stream.resubscribe();
        let stop = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                //check if shutdown is requested
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                if sink.is_closed() {
                    return;
                }
                let transaction_response = transaction_stream.recv().await;
                match transaction_response {
                    Ok(transaction) => {
                        if sink.is_closed() {
                            return;
                        }
                        //add filters here.
                        let resp = jsonrpsee::SubscriptionMessage::from_json(&transaction).unwrap();
                        match sink.send(resp).await {
                            Ok(_) => {
                                continue;
                            }
                            Err(e) => {
                                error!("Error sending transaction response: {:?}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => match e {
                        RecvError::Closed => {
                            debug!("transaction subscription Closed");
                            return;
                        }
                        RecvError::Lagged(skipped) => {
                            warn!("slot subscription Lagged skipped {}", skipped);
                            continue;
                        }
                    },
                }
            }
        });
        Ok(())
    }

    async fn account_update_subscribe(
        &self,
        pending: PendingSubscriptionSink,
        pub_key: Pubkey,
        config: RpcAccountInfoConfig,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let mut account_stream = self.account_stream.resubscribe();
        let stop = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                //check if shutdown is requested
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                if sink.is_closed() {
                    return;
                }
                let account_response = account_stream.recv().await;
                match account_response {
                    Ok(account) => {
                        if sink.is_closed() {
                            return;
                        }
                        //add filters here.
                        let resp = jsonrpsee::SubscriptionMessage::from_json(&account).unwrap();
                        match sink.send(resp).await {
                            Ok(_) => {
                                continue;
                            }
                            Err(e) => {
                                error!("Error sending account response: {:?}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => match e {
                        RecvError::Closed => {
                            debug!("account subscription Closed");
                            return;
                        }
                        RecvError::Lagged(skipped) => {
                            warn!("slot subscription Lagged skipped {}", skipped);
                            continue;
                        }
                    },
                }
            }
        });
        Ok(())
    }
}
