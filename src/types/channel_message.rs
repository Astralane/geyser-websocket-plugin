use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use serde_json::Value;
use solana_transaction_status::EncodedTransaction;

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    Slot(MessageSlotInfo),
    Transaction(Box<MessageTransaction>),
}

impl TryInto<String> for ChannelMessage {
    type Error = serde_json::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            ChannelMessage::Slot(slot_info) => serde_json::to_string(&slot_info),
            ChannelMessage::Transaction(transaction) => serde_json::to_string(&*transaction),
        }
    }
}

impl TryInto<Value> for ChannelMessage {
    type Error = serde_json::Error;
    fn try_into(self) -> Result<Value, Self::Error> {
        match self {
            ChannelMessage::Slot(slot_info) => serde_json::to_value(slot_info),
            ChannelMessage::Transaction(transaction) => serde_json::to_value(*transaction),
        }
    }
}
