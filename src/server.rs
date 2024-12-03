use crate::plugin::GeyserPluginWebsocketError;
use crate::rpc_pubsub::GeyserPubSubServer;
use crate::types::account::MessageAccount;
use crate::types::channel_message::ChannelMessage;
use crate::types::filters::{
    FilterAccounts, FilterSlots, FilterTransactions, TransactionSubscribeFilter,
    TransactionSubscribeOptions,
};
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use jsonrpsee::core::{async_trait, SubscriptionResult};
use jsonrpsee::tracing::error;
use jsonrpsee::PendingSubscriptionSink;
use metrics::{counter, gauge, histogram};
use serde_json::json;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;
use tracing::{debug, warn};

pub struct GeyserPubSubImpl {
    pub shutdown: Arc<AtomicBool>,
    pub slot_stream: broadcast::Receiver<MessageSlotInfo>,
    pub transaction_stream: broadcast::Receiver<(Box<MessageTransaction>, CommitmentLevel)>,
    pub account_stream: broadcast::Receiver<(Box<MessageAccount>, CommitmentLevel)>,
}

impl GeyserPubSubImpl {
    pub fn new(shutdown: Arc<AtomicBool>, tx: tokio::sync::mpsc::Receiver<ChannelMessage>) -> Self {
        let (slot_stream_sender, slot_stream) = broadcast::channel(100);
        let (transaction_stream_sender, transaction_stream) = broadcast::channel(100);
        let (account_stream_sender, account_stream) = broadcast::channel(100);
        thread::spawn(move || {
            run_broadcast_with_commitment_cache_loop(
                tx,
                slot_stream_sender,
                transaction_stream_sender,
                account_stream_sender,
                100,
            )
        });
        Self {
            shutdown,
            slot_stream,
            transaction_stream,
            account_stream,
        }
    }
}

// keep a rolling cache of all recently received tx and account stream messages
// rebroadcast them on getting commitment updates for slot
fn run_broadcast_with_commitment_cache_loop(
    mut rx: tokio::sync::mpsc::Receiver<ChannelMessage>,
    slot_stream: broadcast::Sender<MessageSlotInfo>,
    transaction_stream: broadcast::Sender<(Box<MessageTransaction>, CommitmentLevel)>,
    account_stream: broadcast::Sender<(Box<MessageAccount>, CommitmentLevel)>,
    retain_slot_count: usize,
) {
    let mut transactions_cache = HashMap::<u64, Vec<Box<MessageTransaction>>>::new();
    let mut accounts_update_cache = HashMap::<u64, Vec<Box<MessageAccount>>>::new();
    loop {
        let message = rx.blocking_recv();
        if let Some(message) = message {
            gauge!("websocket_geyser_transactions_cache_size").set(transactions_cache.len() as f64);
            gauge!("websocket_geyser_accounts_cache_size").set(accounts_update_cache.len() as f64);
            match message {
                ChannelMessage::Slot(slot_msg) => {
                    let (transactions, account_updates) = match slot_msg.commitment {
                        CommitmentLevel::Processed => {
                            // remove old unconfirmed slot data,
                            // keep only retain_slot_count slots
                            let current_slot = slot_msg.slot;
                            let max_retain_slot = current_slot - retain_slot_count as u64;
                            transactions_cache.retain(|slot, _| *slot >= max_retain_slot);
                            accounts_update_cache.retain(|slot, _| *slot >= max_retain_slot);
                            (Vec::new(), Vec::new())
                        }
                        CommitmentLevel::Confirmed => {
                            let mut transactions = Vec::new();
                            let mut account_updates = Vec::new();
                            if let Some(messages) = transactions_cache.get(&slot_msg.slot) {
                                transactions = messages.clone();
                            }
                            if let Some(messages) = accounts_update_cache.get(&slot_msg.slot) {
                                account_updates = messages.clone();
                            }
                            (transactions, account_updates)
                        }
                        CommitmentLevel::Finalized => {
                            let mut transactions = Vec::new();
                            let mut account_updates = Vec::new();
                            if let Some(messages) = transactions_cache.remove(&slot_msg.slot) {
                                transactions = messages;
                            }
                            if let Some(messages) = accounts_update_cache.remove(&slot_msg.slot) {
                                account_updates = messages;
                            }
                            (transactions, account_updates)
                        }
                    };
                    if !transactions.is_empty() {
                        for transaction in transactions {
                            let _ = transaction_stream
                                .send((transaction, slot_msg.commitment))
                                .map_err(|e| {
                                    error!("Error sending transaction update: {:?}", e);
                                    e
                                });
                        }
                    }
                    if !account_updates.is_empty() {
                        for account in account_updates {
                            let _ =
                                account_stream
                                    .send((account, slot_msg.commitment))
                                    .map_err(|e| {
                                        error!("Error sending account update: {:?}", e);
                                        e
                                    });
                        }
                    }
                    let _ = slot_stream.send(slot_msg).map_err(|e| {
                        error!("Error sending slot update: {:?}", e);
                        e
                    });
                }
                ChannelMessage::Transaction(message_transaction) => {
                    //add to cache
                    transactions_cache
                        .entry(message_transaction.slot)
                        .and_modify(|txs| txs.push(message_transaction.clone()))
                        .or_insert(vec![message_transaction.clone()]);

                    let _ = transaction_stream
                        .send((message_transaction, CommitmentLevel::Processed))
                        .map_err(|e| {
                            error!("Error sending transaction update: {:?}", e);
                            e
                        });
                }
                ChannelMessage::AccountUpdate(message_account) => {
                    accounts_update_cache
                        .entry(message_account.slot)
                        .and_modify(|accounts| accounts.push(message_account.clone()))
                        .or_insert(vec![message_account.clone()]);

                    let _ = account_stream
                        .send((message_account, CommitmentLevel::Processed))
                        .map_err(|e| {
                            error!("Error sending account update: {:?}", e);
                            e
                        });
                }
            }
        }
    }
}

#[async_trait]
impl GeyserPubSubServer for GeyserPubSubImpl {
    async fn get_version(&self, pending: PendingSubscriptionSink) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let resp_json = json!({"version": "1.0.0"});
        let resp = jsonrpsee::SubscriptionMessage::from_json(&resp_json).unwrap();
        sink.send(resp).await?;
        Ok(())
    }

    async fn slot_subscribe(
        &self,
        pending: PendingSubscriptionSink,
        config: Option<CommitmentConfig>,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let mut slot_stream = self.slot_stream.resubscribe();
        let stop = self.shutdown.clone();
        let filter = FilterSlots::new(config);
        gauge!("websocket_geyser_total_active_subscriptions").increment(1);
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
                            gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                            return;
                        }
                        //add filters here.
                        if !filter.allows(&slot) {
                            continue;
                        }
                        let resp = jsonrpsee::SubscriptionMessage::from_json(&slot).unwrap();
                        let timer = Instant::now();
                        match sink.send(resp).await {
                            Ok(_) => {
                                counter!("websocket_geyser_total_messages_sent").increment(1);
                                histogram!("websocket_geyser_slot_send_to_client_latency")
                                    .record(timer.elapsed().as_millis() as f64);
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
                            gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                            return;
                        }
                        RecvError::Lagged(skipped) => {
                            warn!("slot subscription Lagged skipped {}", skipped);
                            counter!("websocket_geyser_error", "receiver_lagged"=>"slot_subscribe")
                                .increment(1);
                            //send lagged error message
                            let resp = GeyserPluginWebsocketError::Lagged(skipped);
                            let resp = jsonrpsee::SubscriptionMessage::from_json(&resp).unwrap();
                            let sink_ = sink.clone();
                            tokio::spawn(async move {
                                let _ = sink_.send(resp).await;
                            });
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
        filter: TransactionSubscribeFilter,
        options: TransactionSubscribeOptions,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let mut transaction_stream = self.transaction_stream.resubscribe();
        let stop = self.shutdown.clone();
        let filter = FilterTransactions::new(filter, options.commitment.map(Into::into));
        gauge!("websocket_geyser_total_active_subscriptions").increment(1);
        tokio::spawn(async move {
            loop {
                //check if shutdown is requested
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                if sink.is_closed() {
                    gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                    return;
                }
                let transaction_response = transaction_stream.recv().await;
                match transaction_response {
                    Ok((transaction, commitment)) => {
                        if !filter.allows(&transaction, commitment) {
                            continue;
                        }

                        if let Ok(notification) = transaction.to_notification(
                            options.encoding,
                            options.max_supported_transaction_version,
                            options.show_rewards.unwrap_or_default(),
                        ) {
                            //This might be a bottleneck.
                            let resp =
                                jsonrpsee::SubscriptionMessage::from_json(&notification).unwrap();
                            let timer = Instant::now();
                            match sink.send(resp).await {
                                Ok(_) => {
                                    counter!("websocket_geyser_total_messages_sent").increment(1);
                                    histogram!("transactions_send_to_client_latency")
                                        .record(timer.elapsed().as_millis() as f64);
                                    continue;
                                }
                                Err(e) => {
                                    error!("Error sending transaction response: {:?}", e);
                                    gauge!("websocket_geyser_total_active_subscriptions")
                                        .decrement(1);
                                    return;
                                }
                            }
                        } else {
                            error!("Error encoding transaction");
                            continue;
                        }
                    }
                    Err(e) => match e {
                        RecvError::Closed => {
                            debug!("transaction subscription Closed");
                            gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                            return;
                        }
                        RecvError::Lagged(skipped) => {
                            warn!("slot subscription Lagged skipped {}", skipped);
                            counter!("websocket_geyser_error", "receiver_lagged"=>"transaction_subscribe")
                                .increment(1);
                            continue;
                        }
                    },
                }
            }
        });
        Ok(())
    }

    async fn account_subscribe(
        &self,
        pending: PendingSubscriptionSink,
        pubkey_str: String,
        config: Option<RpcAccountInfoConfig>,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let mut account_stream = self.account_stream.resubscribe();
        let stop = self.shutdown.clone();
        let pubkey = Pubkey::from_str(&pubkey_str)?;
        let commitment = config
            .as_ref()
            .and_then(|c| c.commitment)
            .map(|c| c.commitment);
        let filter = FilterAccounts::new(pubkey, commitment);
        gauge!("websocket_geyser_total_active_subscriptions").increment(1);
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
                    Ok((message, commitment)) => {
                        if sink.is_closed() {
                            gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                            return;
                        }
                        //add filters here.
                        if !filter.allows(&message, commitment) {
                            continue;
                        }
                        let resp = jsonrpsee::SubscriptionMessage::from_json(
                            &message.to_notification(config.as_ref()),
                        )
                        .unwrap();
                        let timer = Instant::now();
                        match sink.send(resp).await {
                            Ok(_) => {
                                counter!("websocket_geyser_total_messages_sent").increment(1);
                                histogram!("transactions_send_to_client_latency")
                                    .record(timer.elapsed().as_millis() as f64);
                                continue;
                            }
                            Err(e) => {
                                gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                                error!("Error sending account response: {:?}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => match e {
                        RecvError::Closed => {
                            debug!("account subscription Closed");
                            gauge!("websocket_geyser_total_active_subscriptions").decrement(1);
                            return;
                        }
                        RecvError::Lagged(skipped) => {
                            warn!("slot subscription Lagged skipped {}", skipped);
                            counter!("websocket_geyser_error", "receiver_lagged"=>"account_subscribe").increment(1);
                            continue;
                        }
                    },
                }
            }
        });
        Ok(())
    }
}
