use crate::plugin::GeyserPluginPostgresError;
use crate::service::{run_service, DBWorker, DBWorkerMessage};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct Client {
    sender: mpsc::Sender<DBWorkerMessage>,
}

impl Client {
    pub fn new(url: &str) -> Self {
        let (sender, recv) = mpsc::channel(100);
        let service = DBWorker::new(url, recv);
        tokio::task::spawn(run_service(service));
        Self { sender }
    }
    pub fn send(&self, message: DBWorkerMessage) -> Result<(), GeyserPluginPostgresError> {
        self.sender
            .blocking_send(message)
            .map_err(|e| GeyserPluginPostgresError::ChannelSendError(e))
    }
}
