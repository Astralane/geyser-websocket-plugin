use solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use crate::plugin::GeyserPluginPostgresError;
use crate::service::{run_service, DBWorker};

#[derive(Clone, Debug)]
pub struct Client {
    sender: crossbeam_channel::Sender<usize>,
}

impl Client {
    pub fn new(url: &str, no_of_workers: u8) -> Self {
        let (sender, recv) = crossbeam_channel::bounded(100);
        let worker = DBWorker::new(url, recv);
        //create a runtime for workers
        for _ in 0..no_of_workers {
            let worker_tmp = worker.clone();
            std::thread::spawn(|| run_service(worker_tmp));
        }
        Self { sender }
    }
    pub fn send(&self, message: usize) -> Result<(), GeyserPluginPostgresError> {
        self.sender
            .send(message)
            .map_err(GeyserPluginPostgresError::ChannelSendError)
    }
}
