use crate::database::{Database, TransactionDTO};

pub enum DBMessage {
    //start and end indeces of the chunk
    Transaction(TransactionDTO),
}

pub struct DBWorkerMessage {
    pub message: DBMessage,
}

#[derive(Clone)]
pub struct DBWorker {
    pub receiver: crossbeam_channel::Receiver<DBWorkerMessage>,
    pub db: Database,
}

impl DBWorker {
    pub fn new(url: &str, recv: crossbeam_channel::Receiver<DBWorkerMessage>) -> Self {
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
    while let Ok(message) = actor.receiver.recv() {
        actor.handle_message(message);
    }
}
