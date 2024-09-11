use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Filters {
    pub is_vote: bool,
    pub include_accounts: Vec<String>,
    pub subscription_type: SubscriptionType,
}

#[derive(Clone, Serialize, Deserialize)]
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
