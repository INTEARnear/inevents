use inevents::events::event::{EventId, PaginationBy};
use inindexer::near_indexer_primitives::types::{AccountId, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
use inindexer::near_utils::dec_format;
use schemars::JsonSchema;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
#[cfg(feature = "impl")]
use sqlx::postgres::PgQueryResult;
#[cfg(feature = "impl")]
use sqlx::{Pool, Postgres};

#[cfg(feature = "impl")]
use inevents::events::event::{
    DatabaseEventAdapter, DatabaseEventFilter, Event, PaginationParameters, RealtimeEventFilter,
};

pub struct LogNep297Event;

impl LogNep297Event {
    pub const ID: &'static str = "log_nep297";
}

#[cfg(feature = "impl")]
impl Event for LogNep297Event {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("All NEP-297 (https://nomicon.io/Standards/EventsFormat) events produced by smart contracts");
    const CATEGORY: &'static str = "Logs";
    const EXCLUDE_FROM_DATABASE: bool = true;
    const SUPPORTS_TESTNET: bool = true;

    type EventData = LogNep297EventData;
    type RealtimeEventFilter = RtLogNep297Filter;
    type DatabaseAdapter = DbLogNep297Adapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LogNep297EventData {
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
    pub event_standard: String,
    pub event_version: String,
    pub event_event: String,
    pub event_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbLogNep297Filter {
    #[schemars(with = "Option<String>")]
    pub account_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub predecessor_id: Option<AccountId>,
    pub event_standard: Option<String>,
    pub event_version: Option<String>,
    pub event_event: Option<String>,
}

pub struct DbLogNep297Adapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbLogNep297Adapter {
    type Event = LogNep297Event;
    type Filter = DbLogNep297Filter;

    async fn insert(
        _event: &<Self::Event as Event>::EventData,
        _pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        // sqlx::query!(
        //     r#"
        //     INSERT INTO log_nep297 (timestamp, transaction_id, receipt_id, block_height, account_id, predecessor_id, event_standard, event_version, event_event, event_data)
        //     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        //     event.event_standard,
        //     event.event_version,
        //     event.event_event,
        //     event.event_data,
        // )
        // .execute(pool)
        // .await
        // TODO add testnet support
        unimplemented!()
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbLogNep297Filter {
    type Event = LogNep297Event;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlLogNep297EventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            account_id: String,
            predecessor_id: String,
            event_standard: String,
            event_version: String,
            event_event: String,
            event_data: Option<serde_json::Value>,
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

        sqlx_conditional_queries::conditional_query_as!(
            SqlLogNep297EventData,
            r#"
            SELECT *
            FROM log_nep297{#testnet}
            WHERE {#time}
                {#account_id}
                {#predecessor_id}
                {#event_standard}
                {#event_version}
                {#event_event}
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
            #account_id = match self.account_id.as_ref().map(|s| s.as_str()) {
                Some(ref account_id) => "AND account_id = {account_id}",
                None => "",
            },
            #predecessor_id = match self.predecessor_id.as_ref().map(|s| s.as_str()) {
                Some(ref predecessor_id) => "AND predecessor_id = {predecessor_id}",
                None => "",
            },
            #event_standard = match self.event_standard.as_ref().map(|s| s.as_str()) {
                Some(ref event_standard) => "AND event_standard = {event_standard}",
                None => "",
            },
            #event_version = match self.event_version.as_ref().map(|s| s.as_str()) {
                Some(ref event_version) => "AND event_version = {event_version}",
                None => "",
            },
            #event_event = match self.event_event.as_ref().map(|s| s.as_str()) {
                Some(ref event_event) => "AND event_event = {event_event}",
                None => "",
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
                        LogNep297EventData {
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            account_id: record.account_id.parse().unwrap(),
                            predecessor_id: record.predecessor_id.parse().unwrap(),
                            event_standard: record.event_standard,
                            event_version: record.event_version,
                            event_event: record.event_event,
                            event_data: record.event_data,
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtLogNep297Filter {
    pub account_id: Option<AccountId>,
    pub predecessor_id: Option<AccountId>,
    pub standard: Option<String>,
    /// The exact version of the event
    pub version_exact: Option<String>,
    /// The semver version of the event must match the provided string.
    /// For example, "^1.0.0" will match "1.0.0", "1.0.1", "1.1.0", etc., but not "2.0.0".
    #[serde(deserialize_with = "deserialize_version_req")]
    pub version_match: Option<VersionReq>,
    pub event: Option<String>,
}

fn deserialize_version_req<'de, D>(deserializer: D) -> Result<Option<VersionReq>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        Ok(Some(
            VersionReq::parse(&s).map_err(serde::de::Error::custom)?,
        ))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtLogNep297Filter {
    type Event = LogNep297Event;

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

        if let Some(standard) = &self.standard {
            if standard != &event.event_standard {
                return false;
            }
        }

        if let Some(version_exact) = &self.version_exact {
            if version_exact != &event.event_version {
                return false;
            }
        }

        if let Some(version_match) = &self.version_match {
            let Ok(event_version) = Version::parse(&event.event_version) else {
                return false;
            };
            if !version_match.matches(&event_version) {
                return false;
            }
        }

        if let Some(event_event) = &self.event {
            if event_event != &event.event_event {
                return false;
            }
        }

        true
    }
}
