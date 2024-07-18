use crate::plugin::GeyserPluginPostgresError;
use crate::service::{run_service, DBWorker, DBWorkerMessage};

#[derive(Clone, Debug)]
pub struct Client {
    sender: crossbeam_channel::Sender<DBWorkerMessage>,
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
    pub fn send(&self, message: DBWorkerMessage) -> Result<(), GeyserPluginPostgresError> {
        self.sender
            .send(message)
            .map_err(|e| GeyserPluginPostgresError::ChannelSendError(e))
    }
}
