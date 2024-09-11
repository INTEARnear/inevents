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

pub struct BlockInfoEvent;

impl BlockInfoEvent {
    pub const ID: &'static str = "block_info";
}

#[cfg(feature = "impl")]
impl Event for BlockInfoEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Information about blocks");
    const CATEGORY: &'static str = "TPS";
    const EXCLUDE_FROM_DATABASE: bool = false;
    const SUPPORTS_TESTNET: bool = true;

    type EventData = BlockInfoEventData;
    type RealtimeEventFilter = RtBlockInfoFilter;
    type DatabaseAdapter = DbBlockInfoAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BlockInfoEventData {
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    #[schemars(with = "String")]
    pub block_hash: CryptoHash,
    #[schemars(with = "String")]
    pub block_producer: AccountId,
    pub transaction_count: u64,
    pub receipt_count: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbBlockInfoFilter {
    pub block_height: Option<BlockHeight>,
    #[schemars(with = "Option<String>")]
    pub block_hash: Option<CryptoHash>,
    #[schemars(with = "Option<String>")]
    pub block_producer: Option<AccountId>,
}

pub struct DbBlockInfoAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbBlockInfoAdapter {
    type Event = BlockInfoEvent;
    type Filter = DbBlockInfoFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO block_info_testnet (timestamp, block_hash, block_producer, transaction_count, receipt_count)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_hash.to_string(),
                event.block_producer.to_string(),
                event.transaction_count as i64,
                event.receipt_count as i64
            )
            .execute(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO block_info (timestamp, block_hash, block_producer, transaction_count, receipt_count)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_hash.to_string(),
                event.block_producer.to_string(),
                event.transaction_count as i64,
                event.receipt_count as i64
            )
            .execute(pool)
            .await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbBlockInfoFilter {
    type Event = BlockInfoEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                SELECT timestamp, block_height, block_hash, block_producer, transaction_count, receipt_count
                FROM block_info_testnet
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::BIGINT IS NULL OR block_height = $3)
                    AND ($4::TEXT IS NULL OR block_hash = $4)
                    AND ($5::TEXT IS NULL OR block_producer = $5)
                ORDER BY timestamp ASC
                LIMIT $2
                "#,
                pagination.start_block_timestamp_nanosec as i64,
                pagination.blocks as i64,
                self.block_height.map(|h| h as i64),
                self.block_hash.as_ref().map(|s| s.to_string()),
                self.block_producer.as_ref().map(|s| s.as_str())
            )
            .map(|record| BlockInfoEventData {
                block_height: record.block_height as u64,
                block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                block_hash: record.block_hash.parse().unwrap(),
                block_producer: record.block_producer.parse().unwrap(),
                transaction_count: record.transaction_count as u64,
                receipt_count: record.receipt_count as u64,
            })
            .fetch_all(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT timestamp, block_height, block_hash, block_producer, transaction_count, receipt_count
                FROM block_info
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::BIGINT IS NULL OR block_height = $3)
                    AND ($4::TEXT IS NULL OR block_hash = $4)
                    AND ($5::TEXT IS NULL OR block_producer = $5)
                ORDER BY timestamp ASC
                LIMIT $2
                "#,
                pagination.start_block_timestamp_nanosec as i64,
                pagination.blocks as i64,
                self.block_height.map(|h| h as i64),
                self.block_hash.as_ref().map(|s| s.to_string()),
                self.block_producer.as_ref().map(|s| s.as_str())
            )
            .map(|record| BlockInfoEventData {
                block_height: record.block_height as u64,
                block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                block_hash: record.block_hash.parse().unwrap(),
                block_producer: record.block_producer.parse().unwrap(),
                transaction_count: record.transaction_count as u64,
                receipt_count: record.receipt_count as u64,
            })
            .fetch_all(pool)
            .await
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RtBlockInfoFilter {
    pub block_height_equal: Option<BlockHeight>,
    pub block_height_until: Option<BlockHeight>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtBlockInfoFilter {
    type Event = BlockInfoEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(block_height_equal) = self.block_height_equal {
            if event.block_height != block_height_equal {
                return false;
            }
        }

        if let Some(block_height_until) = self.block_height_until {
            if event.block_height > block_height_until {
                return false;
            }
        }

        true
    }
}
