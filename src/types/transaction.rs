use agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use serde::{Deserialize, Serialize};
use solana_transaction_status::{EncodableWithMeta, EncodedTransaction, UiTransactionEncoding};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MessageTransaction {
    pub transaction: EncodedTransaction,
}
impl<'a> From<&'a ReplicaTransactionInfoV2<'a>> for MessageTransaction {
    fn from(tx: &'a ReplicaTransactionInfoV2<'a>) -> Self {
        Self {
            transaction: tx.transaction.to_versioned_transaction().encode_with_meta(
                UiTransactionEncoding::JsonParsed,
                tx.transaction_status_meta,
            ),
        }
    }
}
