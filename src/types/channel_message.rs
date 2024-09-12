use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use serde_json::Value;
use crate::types::filters::ResponseFilter;

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

impl ChannelMessage{
    pub fn apply_filter(&self, filter: &ResponseFilter) -> bool {
        match self {
            ChannelMessage::Slot(slot_info) => {
                return true
            }
            ChannelMessage::Transaction(transaction) => {
                // check for vote filter
                if !filter.is_vote
            }
        }
    }
}