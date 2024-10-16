use crate::rpc_pubsub::GeyserPubSubServer;
use crate::server::GeyserPubSubImpl;
use crate::types::account::{MessageAccount, MessageAccountInfo};
use crate::types::channel_message::ChannelMessage;
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use agave_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use log::{error, info, warn};
use solana_sdk::commitment_config::CommitmentConfig;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct GeyserWebsocketPlugin {
    inner: Option<GeyserPluginWebsocketInner>,
}
impl GeyserWebsocketPlugin {
    pub fn new() -> Self {
        Self { inner: None }
    }
}

#[derive(Debug)]
pub struct GeyserPluginWebsocketInner {
    pub shutdown: Arc<AtomicBool>,
    pub slot_updates_tx: tokio::sync::broadcast::Sender<MessageSlotInfo>,
    pub transaction_updates_tx: tokio::sync::broadcast::Sender<MessageTransaction>,
    pub account_updates_tx: tokio::sync::broadcast::Sender<MessageAccount>,
    pub server_hdl: ServerHandle,
    pub runtime: Runtime,
}

#[derive(Error, Debug)]
pub enum GeyserPluginWebsocketError {
    #[error("Generic Error message: ({msg})")]
    GenericError { msg: String },

    #[error("channel send error")]
    ChannelSendError(#[from] crossbeam_channel::SendError<usize>),

    #[error("version  not supported anymore")]
    VersionNotSupported,
}

impl GeyserPlugin for GeyserWebsocketPlugin {
    fn name(&self) -> &'static str {
        "GeyserWebsocketPlugin"
    }

    fn on_load(
        &mut self,
        config_file: &str,
        is_reload: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        solana_logger::setup_with_default("info");
        info!(target: "geyser", "on_load: config_file: {:?}", config_file);

        //run socket server in a tokio runtime
        let (slot_updates_tx, slot_updates_rx) = tokio::sync::broadcast::channel(16);
        let (transaction_updates_tx, transaction_updates_rx) = tokio::sync::broadcast::channel(16);
        let (account_updates_tx, account_updates_rx) = tokio::sync::broadcast::channel(16);
        let shutdown = Arc::new(AtomicBool::new(false));

        let runtime = Runtime::new().unwrap();

        let pubsub = GeyserPubSubImpl::new(
            shutdown.clone(),
            slot_updates_rx,
            transaction_updates_rx,
            account_updates_rx,
        );

        let ws_server_handle = runtime.block_on(async move {
            let hdl = ServerBuilder::default()
                .ws_only()
                .build("127.0.0.1:8999")
                .await
                .unwrap()
                .start(pubsub.into_rpc());
            Ok::<_, GeyserPluginError>(hdl)
        })?;

        let inner = GeyserPluginWebsocketInner {
            shutdown,
            slot_updates_tx,
            transaction_updates_tx,
            account_updates_tx,
            server_hdl: ws_server_handle,
            runtime,
        };

        self.inner = Some(inner);
        Ok(())
    }

    fn on_unload(&mut self) {
        info!(target: "geyser", "on_unload");
        //do cleanup
        if let Some(inner) = self.inner.take() {
            inner.shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = inner.server_hdl.stop();
            inner.runtime.shutdown_timeout(Duration::from_secs(30));
        }
    }

    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!(target: "geyser", update_account: account");
        let account = match account {
            ReplicaAccountInfoVersions::V0_0_1(_info) => {
                unreachable!("ReplicaAccountInfoVersions::V0_0_1 is not supported")
            }
            ReplicaAccountInfoVersions::V0_0_2(_info) => {
                unreachable!("ReplicaAccountInfoVersions::V0_0_2 is not supported")
            }
            ReplicaAccountInfoVersions::V0_0_3(info) => info,
        };
        if let Some(inner) = self.inner.as_ref() {
            if let Err(e) = inner
                .account_updates_tx
                .send((account, slot, is_startup).into())
            {
                error!(target: "geyser", "Error sending account update: {:?}", e);
            }
        }
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
        let commitment = match status {
            SlotStatus::Processed => CommitmentConfig::processed(),
            SlotStatus::Confirmed => CommitmentConfig::confirmed(),
            SlotStatus::Rooted => CommitmentConfig::finalized(),
        };
        let message = MessageSlotInfo::new(slot, parent, commitment.commitment);
        if let Some(inner) = self.inner.as_ref() {
            if let Err(e) = inner.slot_updates_tx.send(message) {
                error!(target: "geyser", "Error sending slot update: {:?}", e);
            }
        }
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
        if let Some(inner) = self.inner.as_ref() {
            if let Err(e) = inner.transaction_updates_tx.send(transaction_message) {
                error!(target: "geyser", "Error sending transaction update: {:?}", e);
            }
        }
        Ok(())
    }

    fn notify_block_metadata(
        &self,
        _blockinfo: ReplicaBlockInfoVersions,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    fn transaction_notifications_enabled(&self) -> bool {
        true
    }
}
