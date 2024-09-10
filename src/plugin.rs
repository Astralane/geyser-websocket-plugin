use crate::server::{SubscriptionStore, WebsocketServer};
use crate::types::channel_message::ChannelMessage;
use crate::types::rpc::ServerResponse;
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use agave_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use log::{error, info};
use solana_sdk::commitment_config::CommitmentConfig;
use std::fmt::Debug;
use std::ops::Deref;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::Message;

/// This is the main object returned bu our dynamic library in entrypoint.rs
#[derive(Debug)]
pub struct GeyserPluginWebsocket {
    pub subscription_store: SubscriptionStore,
    pub runtime: Runtime,
}

#[derive(Error, Debug)]
pub enum GeyserPluginPostgresError {
    #[error("Generic Error message: ({msg})")]
    GenericError { msg: String },

    #[error("channel send error")]
    ChannelSendError(#[from] crossbeam_channel::SendError<usize>),

    #[error("version  not supported anymore")]
    VersionNotSupported,
}

impl GeyserPlugin for GeyserPluginWebsocket {
    fn name(&self) -> &'static str {
        "GeyserWebsocketPlugin"
    }

    fn on_load(
        &mut self,
        config_file: &str,
        _is_reload: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        solana_logger::setup_with_default("info");
        info!("on_load: config_file: {:?}", config_file);
        //run socket server in a tokio runtime
        info!("starting runtime of ws server");
        self.runtime.spawn(WebsocketServer::serve(
            "127.0.0.1:9002",
            self.subscription_store.clone(),
        ));
        Ok(())
    }

    fn on_unload(&mut self) {
        info!("on_unload");
    }

    fn update_account(
        &self,
        _account: ReplicaAccountInfoVersions,
        _slot: u64,
        _is_startup: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!("update_account: account");
        Ok(())
    }

    fn notify_end_of_startup(
        &self,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("notify_end_of_startup");
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("update_slot_status: slot: {:?}", slot);
        let commitment_level = match status {
            SlotStatus::Processed => CommitmentConfig::processed(),
            SlotStatus::Rooted => CommitmentConfig::finalized(),
            SlotStatus::Confirmed => CommitmentConfig::confirmed(),
        };
        let message = ChannelMessage::Slot(MessageSlotInfo::new(
            slot,
            parent.unwrap_or(0),
            commitment_level.commitment,
        ));
        self.notify_clients(message);
        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("notify_transaction: transaction for {:?}", slot);
        //get validator for this slot
        let ReplicaTransactionInfoVersions::V0_0_2(solana_transaction) = transaction else {
            return Err(GeyserPluginError::TransactionUpdateError {
                msg: "Unsupported transaction version".to_string(),
            });
        };
        let transaction_message: MessageTransaction = solana_transaction.into();
        let message = ChannelMessage::Transaction(Box::new(transaction_message));
        self.notify_clients(message);
        Ok(())
    }

    fn notify_block_metadata(
        &self,
        _blockinfo: ReplicaBlockInfoVersions,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        false
    }

    fn transaction_notifications_enabled(&self) -> bool {
        true
    }
}

impl GeyserPluginWebsocket {
    pub fn new() -> Self {
        solana_logger::setup_with_default("info");
        info!("creating client");
        Self {
            subscription_store: Default::default(),
            runtime: Runtime::new().unwrap(),
        }
    }

    pub fn notify_clients(&self, message: ChannelMessage) {
        info!("sending slot update to clients");
        match message {
            ChannelMessage::Slot(slot) => {
                {
                    let subscribers = self.subscription_store.slot_subscribers.lock().unwrap();
                    for (_, tx) in subscribers.iter() {
                        //give back server_response
                        let response = ServerResponse::try_from(slot.clone());
                        if let Ok(response) = response {
                            let message = serde_json::to_string(&response).unwrap();
                            let _ = tx.send(Message::Text(message));
                        } else {
                            error!("error converting slot update to server response");
                        }
                    }
                }
            }
            ChannelMessage::Transaction(transaction_update) => {
                {
                    let subscribers = self
                        .subscription_store
                        .transaction_subscribers
                        .lock()
                        .unwrap();
                    for (_, tx) in subscribers.iter() {
                        //give back server_response
                        let response = ServerResponse::try_from(transaction_update.deref().clone());
                        if let Ok(response) = response {
                            let message = serde_json::to_string(&response).unwrap();
                            let _ = tx.send(Message::Text(message));
                        } else {
                            error!("error converting transaction update to server response");
                        }
                    }
                }
            }
        }
    }

    pub fn shutdown(&self) {
        info!("shutting down");
    }
}
