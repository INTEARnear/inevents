use inindexer::near_indexer_primitives::types::{AccountId, Balance, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
use inindexer::near_utils::dec_format;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TxTransactionEvent {
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    pub block_height: BlockHeight,
    #[schemars(with = "String")]
    pub transaction_hash: CryptoHash,
    #[schemars(with = "String")]
    pub included_in_block_hash: CryptoHash,
    #[schemars(with = "String")]
    pub included_in_chunk_hash: CryptoHash,
    pub index_in_chunk: u64,
    #[schemars(with = "String")]
    pub signer_account_id: AccountId,
    #[schemars(with = "String")]
    pub receiver_account_id: AccountId,
    pub status: &'static str,
    #[schemars(with = "String")]
    pub converted_into_receipt_id: CryptoHash,
    pub receipt_conversion_gas_burnt: u64,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub receipt_conversion_tokens_burnt: Balance,
}

impl TxTransactionEvent {
    pub const ID: &'static str = "tx_transaction";

    pub const STATUS_SUCCESS_VALUE: &'static str = "SUCCESS_VALUE";
    pub const STATUS_SUCCESS_RECEIPT_ID: &'static str = "SUCCESS_RECEIPT_ID";
    pub const STATUS_FAILURE: &'static str = "FAILURE";
    pub const STATUS_UNKNOWN: &'static str = "UNKNOWN";
}
