use crate::server::{ClientStore, WebsocketServer};
use log::info;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use std::fmt::Debug;
use thiserror::Error;

/// This is the main object returned bu our dynamic library in entrypoint.rs
#[derive(Debug)]
pub struct GeyserPluginWebsocket {
    pub client_store: ClientStore,
}

impl GeyserPluginWebsocket {
    pub fn new() -> Self {
        solana_logger::setup_with_default("info");
        info!("creating client");
        Self {
            client_store: Default::default(),
        }
    }

    pub fn slot_update(&self, slot: u64) {
        let clients = self.client_store.lock().unwrap();
        for (_, tx) in clients.iter() {
            let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(
                slot.to_string(),
            ));
        }
    }

    pub fn shutdown(&self) {
        info!("shutting down");
    }
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
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("on_load: config_file: {:#?}", config_file);
        //run socket server in a tokio runtime
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let ws_handle = WebsocketServer::serve("127.0.0.1:9002", self.client_store.clone());
        runtime.spawn(ws_handle);
        //ru
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
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("update_account: account");
        Ok(())
    }

    fn notify_end_of_startup(
        &self,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("notify_end_of_startup");
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        _parent: Option<u64>,
        _status: SlotStatus,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("update_slot_status: slot: {:#?}", slot);
        self.slot_update(slot);
        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("notify_transaction: transaction for {:?}", slot);
        //get validator for this slot
        match transaction {
            ReplicaTransactionInfoVersions::V0_0_2(_transaction_info) => {
                //THE following line cause a SEG_FAULT wtf!
                //info!("sending message to worker {:?}", transaction_info);
                //
                // if let Some(client) = self.client.as_ref() {
                //     let res = client.send(transaction_info.index);
                //     if let Err(e) = res {
                //         return Err(GeyserPluginError::Custom(Box::new(e)));
                //     }
                // } else {
                //     return Err(GeyserPluginError::Custom(Box::new(
                //         GeyserPluginPostgresError::GenericError {
                //             msg: "client not found".to_string(),
                //         },
                //     )));
                // }

                Ok(())
            }
            _ => Err(GeyserPluginError::Custom(Box::new(
                GeyserPluginPostgresError::VersionNotSupported,
            ))),
        }
    }

    fn notify_block_metadata(
        &self,
        _blockinfo: ReplicaBlockInfoVersions,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        false
    }

    fn transaction_notifications_enabled(&self) -> bool {
        true
    }
}
