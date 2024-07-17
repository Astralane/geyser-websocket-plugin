use crate::database::{Database, TransactionDTO};
use tokio::sync::mpsc;

pub enum DBMessage {
    //start and end indeces of the chunk
    Transaction(TransactionDTO),
}

pub struct DBWorkerMessage {
    pub message: DBMessage,
}

pub struct DBWorker {
    pub receiver: mpsc::Receiver<DBWorkerMessage>,
    pub db: Database,
}

impl DBWorker {
    pub fn new(url: &str, recv: mpsc::Receiver<DBWorkerMessage>) -> Self {
        let db = Database::new(url);
        Self { receiver: recv, db }
    }
    pub fn handle_message(&mut self, message: DBWorkerMessage) {
        match message.message {
            DBMessage::Transaction(transaction) => {
                if let Err(e) = self.db.add_transaction(transaction) {
                    log::error!("Error adding transaction: {:?}", e);
                }
            }
        }
    }
}

pub async fn run_service(mut actor: DBWorker) {
    while let Some(message) = actor.receiver.recv().await {
        actor.handle_message(message);
    }
}
