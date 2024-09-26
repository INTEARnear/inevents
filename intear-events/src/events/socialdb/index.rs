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

pub struct SocialDBIndexEvent;

impl SocialDBIndexEvent {
    pub const ID: &'static str = "socialdb_index";
}

#[cfg(feature = "impl")]
impl Event for SocialDBIndexEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("https://github.com/NearSocial/standards/blob/8713aed325226db5cf97ab9744ba78b561cc377b/types/index/Index.md");
    const CATEGORY: &'static str = "SocialDB";

    type EventData = SocialDBIndexEventData;
    type RealtimeEventFilter = RtSocialDBIndexFilter;
    type DatabaseAdapter = DbSocialDBIndexAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SocialDBIndexEventData {
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
    pub index_type: String,
    pub index_key: serde_json::Value,
    pub index_value: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbSocialDBIndexFilter {
    #[schemars(with = "Option<String>")]
    pub account_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub index_type: Option<String>,
    pub index_key: Option<serde_json::Value>,
}

pub struct DbSocialDBIndexAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbSocialDBIndexAdapter {
    type Event = SocialDBIndexEvent;
    type Filter = DbSocialDBIndexFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO socialdb_index (timestamp, transaction_id, receipt_id, block_height, account_id, index_type, index_key, index_value)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            chrono::DateTime::from_timestamp(
                (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                (event.block_timestamp_nanosec % 1_000_000_000) as u32
            ),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.account_id.to_string(),
            event.index_type,
            event.index_key,
            event.index_value,
        )
        .execute(pool)
        .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbSocialDBIndexFilter {
    type Event = SocialDBIndexEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlSocialDBIndexEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            account_id: String,
            index_type: String,
            index_key: serde_json::Value,
            index_value: serde_json::Value,
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

        let account_id = self.account_id.as_ref().map(|a| a.as_str());
        let index_type = self.index_type.as_ref();
        let index_key = self.index_key.as_ref();

        sqlx_conditional_queries::conditional_query_as!(
            SqlSocialDBIndexEventData,
            r#"
            SELECT *
            FROM socialdb_index{#testnet}
            WHERE {#time}
                AND ({account_id}::TEXT IS NULL OR account_id = {account_id})
                AND ({index_type}::TEXT IS NULL OR index_type = {index_type})
                AND ({index_key}::JSONB IS NULL OR index_key = {index_key})
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
                true => "", // "_testnet",
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
                        SocialDBIndexEventData {
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            account_id: record.account_id.parse().unwrap(),
                            index_type: record.index_type,
                            index_key: record.index_key,
                            index_value: record.index_value,
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtSocialDBIndexFilter {
    pub account_id: Option<AccountId>,
    pub index_type: Option<String>,
    pub index_key: Option<serde_json::Value>,
    pub index_value: Option<serde_json::Value>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtSocialDBIndexFilter {
    type Event = SocialDBIndexEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(account_id) = &self.account_id {
            if account_id != &event.account_id {
                return false;
            }
        }

        if let Some(index_type) = &self.index_type {
            if index_type != &event.index_type {
                return false;
            }
        }

        if let Some(index_key) = &self.index_key {
            if index_key != &event.index_key {
                return false;
            }
        }

        if let Some(index_value) = &self.index_value {
            if index_value != &event.index_value {
                return false;
            }
        }

        true
    }
}
