use crate::server::{ServerConfig, WebsocketServer};
use crate::types::channel_message::ChannelMessage;
use crate::types::rpc::ServerResponse;
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use agave_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use log::{error, info, warn};
use solana_sdk::commitment_config::CommitmentConfig;
use std::fmt::Debug;
use std::ops::Deref;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::Message;

/// This is the main object returned bu our dynamic library in entrypoint.rs
#[derive(Debug)]
pub struct GeyserPluginWebsocket {
    inner: Option<GeyserPluginWebsocketInner>,
}

#[derive(Debug)]
pub struct GeyserPluginWebsocketInner {
    pub slot_updates_tx: tokio::sync::broadcast::Sender<ChannelMessage>,
    pub transaction_updates_tx: tokio::sync::broadcast::Sender<ChannelMessage>,
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
        info!(target: "geyser", "on_load: config_file: {:?}", config_file);
        //run socket server in a tokio runtime
        let (slot_updates_tx, _) = tokio::sync::broadcast::channel(16);
        let (transaction_updates_tx, _) = tokio::sync::broadcast::channel(16);

        let runtime = Runtime::new().unwrap();
        let config = ServerConfig {
            slot_subscribers: slot_updates_tx.clone(),
            transaction_subscribers: transaction_updates_tx.clone(),
        };
        runtime.spawn(async move {
            WebsocketServer::serve("127.0.0.1:9002", config).await;
        });

        let inner = GeyserPluginWebsocketInner {
            slot_updates_tx,
            transaction_updates_tx,
            runtime,
        };

        self.inner = Some(inner);
        Ok(())
    }

    fn on_unload(&mut self) {
        info!(target: "geyser", "on_unload");
    }

    fn update_account(
        &self,
        _account: ReplicaAccountInfoVersions,
        _slot: u64,
        _is_startup: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!(target: "geyser", update_account: account");
        Ok(())
    }

    fn notify_end_of_startup(
        &self,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!(target: "geyser", "notify_end_of_startup");
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!(target: "geyser", "update_slot_status: slot: {:?}", slot);
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
        info!(target: "geyser", "notify_transaction: transaction for {:?}", slot);
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
        Self { inner: None }
    }

    pub fn notify_clients(&self, message: ChannelMessage) {
        info!(target: "geyser", "sending slot update to clients");
        let result = match (message, self.inner.as_ref()) {
            (ChannelMessage::Slot(slot), Some(_)) => {
                let inner = self.inner.as_ref().unwrap();
                Some(inner.slot_updates_tx.send(ChannelMessage::Slot(slot)))
            }
            (ChannelMessage::Transaction(transaction_update), Some(_)) => {
                let inner = self.inner.as_ref().unwrap();
                Some(
                    inner
                        .transaction_updates_tx
                        .send(ChannelMessage::Transaction(transaction_update)),
                )
            }
            _ => {
                error!("Error sending message to clients");
                None
            }
        };
        if let Some(result) = result {
            match result {
                Ok(_) => info!(target: "geyser", "message sent"),
                Err(e) => error!("Error sending message: {:?}", e),
            }
        }
    }

    pub fn shutdown(&self) {
        warn!("shutting down");
    }
}
