use crate::server::{ClientStore, WebsocketServer};
use log::info;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use std::fmt::Debug;
use solana_program::message::v0::Message;
use solana_sdk::commitment_config::CommitmentConfig;
use thiserror::Error;
use tokio::runtime::Runtime;
use crate::types::channel_message::ChannelMessage;
use crate::types::transaction::{Transaction, TransactionMeta};

/// This is the main object returned bu our dynamic library in entrypoint.rs
#[derive(Debug)]
pub struct GeyserPluginWebsocket {
    pub client_store: ClientStore,
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
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        solana_logger::setup_with_default("info");
        info!("on_load: config_file: {:?}", config_file);
        //run socket server in a tokio runtime
        info!("starting runtime of ws server");
        self.runtime.spawn(WebsocketServer::serve(
            "127.0.0.1:9002",
            self.client_store.clone(),
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
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!("update_account: account");
        Ok(())
    }

    fn notify_end_of_startup(
        &self,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!("notify_end_of_startup");
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!("update_slot_status: slot: {:?}", slot);
        let commitment_level = match status {
            SlotStatus::Processed => CommitmentConfig::processed(),
            SlotStatus::Rooted => CommitmentConfig::finalized(),
            SlotStatus::Confirmed => CommitmentConfig::confirmed(),
        };
        let message = ChannelMessage::Slot(slot, parent.unwrap_or_default(), commitment_level);
        self.notify_clients(message);
        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        // info!("notify_transaction: transaction for {:?}", slot);
        //get validator for this slot
        let ReplicaTransactionInfoVersions::V0_0_2(solana_transaction) = transaction else {
            return Err(GeyserPluginError::TransactionUpdateError {
                msg: "Unsupported transaction version".to_string(),
            });
        };

        let message = solana_transaction.transaction.message();
        let mut account_keys = vec![];
        //
        for index in 0.. {
            let account = message.account_keys().get(index);
            match account {
                Some(account) => account_keys.push(*account),
                None => break,
            }
        }
        //
        let v0_message = Message {
            header: *message.header(),
            account_keys,
            recent_blockhash: *message.recent_blockhash(),
            instructions: message.instructions().to_vec(),
            address_table_lookups: message.message_address_table_lookups().to_vec(),
        };
        //
        // let status_meta = solana_transaction.transaction_status_meta;

        // let transaction = Transaction {
        //     slot,
        //     signatures: solana_transaction.transaction.signatures().to_vec(),
        //     message: v0_message,
        //     is_vote: solana_transaction.is_vote,
        //     transasction_meta: TransactionMeta {
        //         error: match &status_meta.status {
        //             Ok(_) => None,
        //             Err(e) => Some(e.clone()),
        //         },
        //         fee: status_meta.fee,
        //         pre_balances: status_meta.pre_balances.clone(),
        //         post_balances: status_meta.post_balances.clone(),
        //         inner_instructions: status_meta.inner_instructions.clone(),
        //         log_messages: status_meta.log_messages.clone(),
        //         rewards: status_meta.rewards.clone(),
        //         loaded_addresses: status_meta.loaded_addresses.clone(),
        //         return_data: status_meta.return_data.clone(),
        //         compute_units_consumed: status_meta.compute_units_consumed,
        //     },
        //     index: solana_transaction.index as u64,
        // };
       // let message = ChannelMessage::Transaction(Box::new(transaction));
        //self.notify_clients(message);
        Ok(())
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

impl GeyserPluginWebsocket {
    pub fn new() -> Self {
        solana_logger::setup_with_default("info");
        info!("creating client");
        Self {
            client_store: Default::default(),
            runtime: Runtime::new().unwrap(),
        }
    }

    pub fn notify_clients(&self, message: ChannelMessage) {
        info!("sending slot update to clients");
        let clients = self.client_store.lock().unwrap();
        match message {
            ChannelMessage::Slot(slot, parent, commit) => {
                for (_, tx) in clients.iter() {
                    let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(
                        slot.to_string(),
                    ));
                }
            }
            ChannelMessage::Transaction(transaction_update) => {
                for (_, tx) in clients.iter() {
                    let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(
                        serde_json::to_string(&transaction_update).unwrap(),
                    ));
                }
            }
        }
    }

    pub fn shutdown(&self) {
        info!("shutting down");
    }
}
