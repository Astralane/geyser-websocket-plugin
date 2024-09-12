use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ResponseFilter {
    pub is_vote: bool,
    pub include_accounts: Vec<String>,
    pub subscription_type: SubscriptionType,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SubscriptionType {
    None,
    Slot,
    Transaction,
}

impl Default for SubscriptionType {
    fn default() -> Self {
        SubscriptionType::None
    }
}

