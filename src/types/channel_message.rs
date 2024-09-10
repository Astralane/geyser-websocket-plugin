use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    Slot(MessageSlotInfo),
    Transaction(Box<MessageTransaction>),
}
