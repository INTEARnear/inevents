use std::str::FromStr;

#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
use inindexer::near_indexer_primitives::types::{AccountId, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
use inindexer::near_utils::dec_format;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "impl")]
use sqlx::postgres::PgQueryResult;
#[cfg(feature = "impl")]
use sqlx::{Pool, Postgres};

#[cfg(feature = "impl")]
use inevents::events::event::{
    DatabaseEventAdapter, DatabaseEventFilter, Event, PaginationParameters, RealtimeEventFilter,
};

pub struct TxTransactionEvent;

impl TxTransactionEvent {
    pub const ID: &'static str = "tx_transaction";
}

#[cfg(feature = "impl")]
impl Event for TxTransactionEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("High-level transaction information");
    const CATEGORY: &'static str = "Transactions";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = TxTransactionEventData;
    type RealtimeEventFilter = RtTxTransactionFilter;
    type DatabaseAdapter = DbTxTransactionAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TxTransactionEventData {
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    pub block_height: BlockHeight,
    #[schemars(with = "String")]
    pub signer_id: AccountId,
    #[schemars(with = "String")]
    pub public_key: String,
    pub nonce: u64,
    #[schemars(with = "String")]
    pub receiver_id: AccountId,
    pub actions: Vec<serde_json::Value>,
    pub priority_fee: Option<u64>,
    pub signature: String,
    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbTxTransactionFilter {
    #[schemars(with = "Option<String>")]
    pub signer_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub receiver_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub transaction_id: Option<CryptoHash>,
}

pub struct DbTxTransactionAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbTxTransactionAdapter {
    type Event = TxTransactionEvent;
    type Filter = DbTxTransactionFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO tx_transactions_testnet (timestamp, block_height, signer_id, receiver_id, transaction_id, actions, priority_fee, signature)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_height as i64,
                event.signer_id.to_string(),
                event.receiver_id.to_string(),
                event.transaction_id.to_string(),
                &event.actions,
                event.priority_fee.map(|v| v as i64),
                event.signature,
            )
            .execute(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO tx_transactions (timestamp, block_height, signer_id, public_key, nonce, receiver_id, transaction_id, actions, priority_fee, signature)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_height as i64,
                event.signer_id.to_string(),
                event.public_key,
                event.nonce as i64,
                event.receiver_id.to_string(),
                event.transaction_id.to_string(),
                &event.actions,
                event.priority_fee.map(|v| v as i64),
                event.signature,
            )
            .execute(pool)
            .await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbTxTransactionFilter {
    type Event = TxTransactionEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;
        #[derive(Debug, sqlx::FromRow)]
        struct SqlTxTransactionEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            block_height: i64,
            signer_id: String,
            public_key: String,
            nonce: i64,
            receiver_id: String,
            actions: Vec<serde_json::Value>,
            priority_fee: Option<i64>,
            signature: String,
            transaction_id: String,
        }
        let block_height = if let PaginationBy::BeforeBlockHeight { block_height }
        | PaginationBy::AfterBlockHeight { block_height } =
            pagination.pagination_by
        {
            block_height as i64
        } else {
            -1
        };
        let timestamp = if let PaginationBy::BeforeTimestamp { timestamp_nanosec }
        | PaginationBy::AfterTimestamp { timestamp_nanosec } =
            pagination.pagination_by
        {
            chrono::DateTime::from_timestamp(
                (timestamp_nanosec / 1_000_000_000) as i64,
                (timestamp_nanosec % 1_000_000_000) as u32,
            )
        } else {
            chrono::DateTime::from_timestamp(0, 0)
        };
        let id = if let PaginationBy::BeforeId { id } | PaginationBy::AfterId { id } =
            pagination.pagination_by
        {
            id as i32
        } else {
            -1
        };

        let signer_id = self.signer_id.as_ref().map(|t| t.as_str());
        let receiver_id = self.receiver_id.as_ref().map(|t| t.as_str());
        let transaction_id = self.transaction_id.as_ref().map(|t| t.to_string());
        sqlx_conditional_queries::conditional_query_as!(
            SqlTxTransactionEventData,
            r#"
            SELECT *
            FROM tx_transactions{#testnet}
            WHERE {#time}
                AND ({signer_id}::TEXT IS NULL OR signer_id = {signer_id})
                AND ({receiver_id}::TEXT IS NULL OR receiver_id = {receiver_id})
                AND ({transaction_id}::TEXT IS NULL OR transaction_id = {transaction_id})
            ORDER BY id {#order}
            LIMIT {limit}
            "#,
            #(time, order) = match &pagination.pagination_by {
                PaginationBy::BeforeBlockHeight { .. } => ("block_height < {block_height}", "DESC"),
                PaginationBy::AfterBlockHeight { .. } => ("block_height > {block_height}", "ASC"),
                PaginationBy::BeforeTimestamp { .. } => ("timestamp < {timestamp}", "DESC"),
                PaginationBy::AfterTimestamp { .. } => ("timestamp > {timestamp}", "ASC"),
                PaginationBy::BeforeId { .. } => ("id < {id}", "DESC"),
                PaginationBy::AfterId { .. } => ("id > {id}", "ASC"),
                PaginationBy::Oldest => ("true", "ASC"),
                PaginationBy::Newest => ("true", "DESC"),
            },
            #testnet = match testnet {
                true => "_testnet",
                false => "",
            },
        )
        .fetch_all(pool)
        .await
        .map(|records| {
            records
                .into_iter()
                .map(|record| {
                    (
                        record.id as EventId,
                        TxTransactionEventData {
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            block_height: record.block_height as u64,
                            signer_id: AccountId::from_str(&record.signer_id).unwrap(),
                            public_key: record.public_key,
                            nonce: record.nonce as u64,
                            receiver_id: AccountId::from_str(&record.receiver_id).unwrap(),
                            transaction_id: record.transaction_id.parse().unwrap(),
                            actions: record.actions,
                            priority_fee: record.priority_fee.map(|v| v as u64),
                            signature: record.signature,
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtTxTransactionFilter {
    pub signer_id: Option<AccountId>,
    pub receiver_id: Option<AccountId>,
    pub transaction_id: Option<CryptoHash>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtTxTransactionFilter {
    type Event = TxTransactionEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(signer_id) = &self.signer_id {
            if signer_id != &event.signer_id {
                return false;
            }
        }
        if let Some(receiver_id) = &self.receiver_id {
            if receiver_id != &event.receiver_id {
                return false;
            }
        }
        if let Some(transaction_id) = &self.transaction_id {
            if transaction_id != &event.transaction_id {
                return false;
            }
        }
        true
    }
}
