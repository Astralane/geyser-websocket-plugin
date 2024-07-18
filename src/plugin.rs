use crate::client::Client;
use crate::database::TransactionDTO;
use crate::service::{DBMessage, DBWorkerMessage};
use log::info;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, GeyserPluginError, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, SlotStatus,
};
use std::fmt::{Debug};
use thiserror::Error;

/// This is the main object returned bu our dynamic library in entrypoint.rs
#[derive(Debug)]
pub struct GeyserPluginPostgres {
    // client: Client,
}

impl GeyserPluginPostgres {
    pub fn new(url: &str) -> Self {
        solana_logger::setup_with_default("info");
        // let client = Client::new(url);
        // Self { client }
        Self {
        }
    }
}

#[derive(Error, Debug)]
pub enum GeyserPluginPostgresError {
    #[error("Generic Error message: ({msg})")]
    GenericError { msg: String },

    #[error("channel send error")]
    ChannelSendError(#[from] crossbeam_channel::SendError<DBWorkerMessage>),

    #[error("Database internal error message")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("version  not supported anymore")]
    VersionNotSupported,
}

impl GeyserPlugin for GeyserPluginPostgres {
    fn name(&self) -> &'static str {
        "GeyserPluginPostgres"
    }

    fn on_load(
        &mut self,
        config_file: &str,
        _is_reload: bool,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("on_load: config_file: {:#?}", config_file);
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
        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> solana_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        info!("notify_transaction: transaction for {:?}", slot);
        //get validator for this slot
        // match transaction {
        //     ReplicaTransactionInfoVersions::V0_0_2(transaction_info) => {
        //         info!(
        //             "notify_transaction: transaction_info: {:#?}",
        //             transaction_info
        //         );
        //
        //         let tx = TransactionDTO {
        //             signature: transaction_info.signature.to_string(),
        //             fee: transaction_info
        //                 .transaction_status_meta
        //                 .fee
        //                 .try_into()
        //                 .unwrap(),
        //             slot: slot.try_into().unwrap(),
        //         };
        //
        //         let msg = DBWorkerMessage {
        //             message: DBMessage::Transaction(tx),
        //         };
        //
        //         let res = self.client.send(msg);
        //         if let Err(e) = res {
        //             return Err(GeyserPluginError::Custom(Box::new(e)));
        //         }
        //
        //         Ok(())
        //     }
        //     _ => Err(GeyserPluginError::Custom(Box::new(
        //         GeyserPluginPostgresError::GenericError {
        //             msg: "version not supported".to_string(),
        //         },
        //     ))),
        // }
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
