use crate::plugin::GeyserPluginPostgresError;
use crate::service::{run_service, DBWorker, DBWorkerMessage};

#[derive(Clone, Debug)]
pub struct Client {
    sender: crossbeam_channel::Sender<DBWorkerMessage>,
}

impl Client {
    pub fn new(url: &str) -> Self {
        let (sender, recv) = crossbeam_channel::bounded(100);
        let service = DBWorker::new(url, recv);
        // aribitrary number of of workers can be spawned
        tokio::task::spawn(run_service(service.clone()));
        tokio::task::spawn(run_service(service.clone()));
        tokio::task::spawn(run_service(service));
        Self { sender }
    }
    pub fn send(&self, message: DBWorkerMessage) -> Result<(), GeyserPluginPostgresError> {
        self.sender
            .send(message)
            .map_err(|e| GeyserPluginPostgresError::ChannelSendError(e))
    }
}
