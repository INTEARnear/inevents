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

pub struct TxReceiptEvent;

impl TxReceiptEvent {
    pub const ID: &'static str = "tx_receipt";
}

#[cfg(feature = "impl")]
impl Event for TxReceiptEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("High-level transaction information");
    const CATEGORY: &'static str = "Transactions";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = TxReceiptEventData;
    type RealtimeEventFilter = RtTxReceiptFilter;
    type DatabaseAdapter = DbTxReceiptAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TxReceiptEventData {
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    pub block_height: BlockHeight,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
    #[schemars(with = "String")]
    pub executor_id: AccountId,
    pub success: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbTxReceiptFilter {
    #[schemars(with = "Option<String>")]
    pub receipt_id: Option<CryptoHash>,
    #[schemars(with = "Option<String>")]
    pub transaction_id: Option<CryptoHash>,
    #[schemars(with = "Option<String>")]
    pub executor_id: Option<AccountId>,
}

pub struct DbTxReceiptAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbTxReceiptAdapter {
    type Event = TxReceiptEvent;
    type Filter = DbTxReceiptFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO tx_receipts_testnet (timestamp, block_height, receipt_id, transaction_id, executor_id, success)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_height as i64,
                event.receipt_id.to_string(),
                event.transaction_id.to_string(),
                event.executor_id.to_string(),
                event.success,
            )
            .execute(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO tx_receipts (timestamp, block_height, receipt_id, transaction_id, executor_id, success)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_height as i64,
                event.receipt_id.to_string(),
                event.transaction_id.to_string(),
                event.executor_id.to_string(),
                event.success,
            )
            .execute(pool)
            .await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbTxReceiptFilter {
    type Event = TxReceiptEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlTxReceiptEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            block_height: i64,
            receipt_id: String,
            transaction_id: String,
            executor_id: String,
            success: Option<bool>,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let receipt_id = self.receipt_id.as_ref().map(|v| v.to_string());
        let transaction_id = self.transaction_id.as_ref().map(|v| v.to_string());
        let executor_id = self.executor_id.as_ref().map(|v| v.as_str());
        sqlx_conditional_queries::conditional_query_as!(
            SqlTxReceiptEventData,
            r#"
            SELECT *
            FROM tx_receipts{#testnet}
            WHERE {#time}
                AND ({receipt_id}::TEXT IS NULL OR receipt_id = {receipt_id})
                AND ({transaction_id}::TEXT IS NULL OR transaction_id = {transaction_id})
                AND ({executor_id}::TEXT IS NULL OR executor_id = {executor_id})
            ORDER BY id {#order}
            LIMIT {limit}
            "#,
            #(time, order) = match &pagination.pagination_by {
                PaginationBy::BeforeTimestamp { .. }
                    | PaginationBy::BeforeBlockHeight { .. } => ("timestamp < {timestamp}", "DESC"),
                PaginationBy::AfterTimestamp { .. }
                    |  PaginationBy::AfterBlockHeight { .. } => ("timestamp > {timestamp}", "ASC"),
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
                        TxReceiptEventData {
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            block_height: record.block_height as BlockHeight,
                            receipt_id: CryptoHash::from_str(&record.receipt_id).unwrap(),
                            transaction_id: CryptoHash::from_str(&record.transaction_id).unwrap(),
                            executor_id: AccountId::from_str(&record.executor_id).unwrap(),
                            success: record.success,
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtTxReceiptFilter {
    pub executor_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtTxReceiptFilter {
    type Event = TxReceiptEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(executor_id) = &self.executor_id {
            if executor_id != &event.executor_id {
                return false;
            }
        }
        true
    }
}
