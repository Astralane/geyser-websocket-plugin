use crate::config::Config;
use crate::metrics::spawn_metrics_client;
use crate::rpc_pubsub::GeyserPubSubServer;
use crate::server::GeyserPubSubImpl;
use crate::types::channel_message::ChannelMessage;
use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use agave_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use serde::Serialize;
use socket2::{Domain, Socket, Type};
use solana_sdk::commitment_config::CommitmentConfig;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::runtime::Runtime;
use tracing::{error, info};

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
    pub sender: tokio::sync::mpsc::Sender<ChannelMessage>,
    pub server_hdl: ServerHandle,
    pub runtime: Runtime,
}

#[derive(Error, Debug, Serialize)]
pub enum GeyserPluginWebsocketError {
    #[error("Generic Error message: ({msg})")]
    GenericError { msg: String },

    #[error("version  not supported anymore")]
    VersionNotSupported,

    #[error("internal server error")]
    Lagged(u64),
}

impl From<GeyserPluginWebsocketError> for GeyserPluginError {
    fn from(e: GeyserPluginWebsocketError) -> Self {
        GeyserPluginError::Custom(Box::new(e))
    }
}

impl GeyserPlugin for GeyserWebsocketPlugin {
    fn name(&self) -> &'static str {
        "GeyserWebsocketPlugin"
    }

    fn on_load(
        &mut self,
        config_file: &str,
        _is_reload: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!(target: "geyser", "on_load: config_file: {:?}", config_file);

        let config = Config::load_from_file(config_file)?;
        solana_logger::setup_with_default(&config.log.level);
        spawn_metrics_client(config.prometheus_address).map_err(|e| {
            error!(target: "geyser", "Error running prometheus serve: {:?}", e);
            GeyserPluginWebsocketError::GenericError {
                msg: "Error running prometheus server".to_string(),
            }
        })?;
        //run socket server in a tokio runtime
        let (sender, reciever) = tokio::sync::mpsc::channel(64);
        let shutdown = Arc::new(AtomicBool::new(false));

        let runtime = Runtime::new().map_err(|e| {
            error!(target: "geyser", "Error creating runtime: {:?}", e);
            GeyserPluginWebsocketError::GenericError {
                msg: "Error creating runtime".to_string(),
            }
        })?;

        let pubsub = GeyserPubSubImpl::new(shutdown.clone(), reciever);
        let addr = config.websocket.address;
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
        socket.set_nonblocking(true)?;
        socket.reuse_address()?;
        socket.set_nodelay(true)?;
        let address = addr.into();
        socket.bind(&address)?;
        socket.listen(4096)?;

        let ws_server_handle = runtime.block_on(async move {
            let hdl = ServerBuilder::default()
                .ws_only()
                .build_from_tcp(socket)
                .unwrap()
                .start(pubsub.into_rpc());
            Ok::<_, GeyserPluginError>(hdl)
        })?;

        let inner = GeyserPluginWebsocketInner {
            shutdown,
            sender,
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
            inner
                .shutdown
                .store(true, std::sync::atomic::Ordering::Relaxed);
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
            let message = (account, slot, is_startup).into();
            if let Err(e) = inner
                .sender
                .blocking_send(ChannelMessage::AccountUpdate(Box::new(message)))
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
            if let Err(e) = inner.sender.blocking_send(ChannelMessage::Slot(message)) {
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
        let transaction_message: MessageTransaction = (solana_transaction, slot).into();
        if let Some(inner) = self.inner.as_ref() {
            if let Err(e) = inner
                .sender
                .blocking_send(ChannelMessage::Transaction(Box::new(transaction_message)))
            {
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
