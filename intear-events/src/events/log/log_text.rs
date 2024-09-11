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

pub struct LogTextEvent;

impl LogTextEvent {
    pub const ID: &'static str = "log_text";
}

#[cfg(feature = "impl")]
impl Event for LogTextEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("All logs produced by smart contracts");
    const CATEGORY: &'static str = "Logs";
    const EXCLUDE_FROM_DATABASE: bool = true;
    const SUPPORTS_TESTNET: bool = true;

    type EventData = LogTextEventData;
    type RealtimeEventFilter = RtLogTextFilter;
    type DatabaseAdapter = DbLogTextAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LogTextEventData {
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,

    #[schemars(with = "String")]
    pub account_id: AccountId,
    #[schemars(with = "String")]
    pub predecessor_id: AccountId,
    pub log_text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbLogTextFilter {
    #[schemars(with = "Option<String>")]
    pub account_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub predecessor_id: Option<AccountId>,
}

pub struct DbLogTextAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbLogTextAdapter {
    type Event = LogTextEvent;
    type Filter = DbLogTextFilter;

    async fn insert(
        _event: &<Self::Event as Event>::EventData,
        _pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        // sqlx::query!(
        //     r#"
        //     INSERT INTO log_text (timestamp, transaction_id, receipt_id, block_height, account_id, predecessor_id, log_text)
        //     VALUES ($1, $2, $3, $4, $5, $6, $7)
        //     "#,
        //     chrono::DateTime::from_timestamp(
        //         (event.block_timestamp_nanosec / 1_000_000_000) as i64,
        //         (event.block_timestamp_nanosec % 1_000_000_000) as u32
        //     ),
        //     event.transaction_id.to_string(),
        //     event.receipt_id.to_string(),
        //     event.block_height as i64,
        //     event.account_id.to_string(),
        //     event.predecessor_id.to_string(),
        //     event.log_text,
        // )
        // .execute(pool)
        // .await
        // TODO add testnet support
        unimplemented!()
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbLogTextFilter {
    type Event = LogTextEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        sqlx::query!(
            r#"
            WITH blocks AS (
                SELECT DISTINCT timestamp as t
                FROM log_text
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR account_id = $3)
                    AND ($4::TEXT IS NULL OR predecessor_id = $4)
                ORDER BY t
                LIMIT $2
            )
            SELECT timestamp, transaction_id, receipt_id, block_height, account_id, predecessor_id, log_text
            FROM log_text
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                AND ($3::TEXT IS NULL OR account_id = $3)
                AND ($4::TEXT IS NULL OR predecessor_id = $4)
            ORDER BY timestamp ASC
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.account_id.as_ref().map(|s| s.to_string()),
            self.predecessor_id.as_ref().map(|s| s.to_string())
        )
        .map(|record| LogTextEventData {
            block_height: record.block_height as u64,
            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
            transaction_id: record.transaction_id.parse().unwrap(),
            receipt_id: record.receipt_id.parse().unwrap(),
            account_id: record.account_id.parse().unwrap(),
            predecessor_id: record.predecessor_id.parse().unwrap(),
            log_text: record.log_text,
        })
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, Deserialize)]
pub struct RtLogTextFilter {
    pub account_id: Option<AccountId>,
    pub predecessor_id: Option<AccountId>,
    pub text: Option<String>,
    pub text_starts_with: Option<String>,
    pub text_ends_with: Option<String>,
    pub text_contains: Option<String>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtLogTextFilter {
    type Event = LogTextEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(account_id) = &self.account_id {
            if account_id != &event.account_id {
                return false;
            }
        }

        if let Some(predecessor_id) = &self.predecessor_id {
            if predecessor_id != &event.predecessor_id {
                return false;
            }
        }

        if let Some(text) = &self.text {
            if text != &event.log_text {
                return false;
            }
        }

        if let Some(text_starts_with) = &self.text_starts_with {
            if !event.log_text.starts_with(text_starts_with) {
                return false;
            }
        }

        if let Some(text_ends_with) = &self.text_ends_with {
            if !event.log_text.ends_with(text_ends_with) {
                return false;
            }
        }

        if let Some(text_contains) = &self.text_contains {
            if !event.log_text.contains(text_contains) {
                return false;
            }
        }

        true
    }
}
