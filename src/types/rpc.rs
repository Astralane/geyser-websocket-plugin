use crate::types::slot_info::MessageSlotInfo;
use crate::types::transaction::MessageTransaction;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerRequest {
    pub method: String,
    pub params: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerResponse {
    pub result: Value,
}

impl TryFrom<MessageTransaction> for ServerResponse {
    type Error = serde_json::Error;
    fn try_from(value: MessageTransaction) -> Result<Self, Self::Error> {
        Ok(Self {
            result: serde_json::to_value(value)?,
        })
    }
}

impl TryFrom<MessageSlotInfo> for ServerResponse {
    type Error = serde_json::Error;
    fn try_from(value: MessageSlotInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            result: serde_json::to_value(value)?,
        })
    }
}
