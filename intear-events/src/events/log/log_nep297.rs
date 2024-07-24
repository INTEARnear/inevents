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
pub struct DbPriceTokenFilter {
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
    type Filter = DbPriceTokenFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO log_nep297 (timestamp, transaction_id, receipt_id, block_height, account_id, predecessor_id, event_standard, event_version, event_event, event_data)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            chrono::DateTime::from_timestamp(
                (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                (event.block_timestamp_nanosec % 1_000_000_000) as u32
            ),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.account_id.to_string(),
            event.predecessor_id.to_string(),
            event.event_standard,
            event.event_version,
            event.event_event,
            event.event_data,
        )
        .execute(pool)
        .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbPriceTokenFilter {
    type Event = LogNep297Event;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        sqlx::query!(
            r#"
            WITH blocks AS (
                SELECT DISTINCT timestamp as t
                FROM log_nep297
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR account_id = $3)
                    AND ($4::TEXT IS NULL OR predecessor_id = $4)
                    AND ($5::TEXT IS NULL OR event_standard = $5)
                    AND ($6::TEXT IS NULL OR event_version = $6)
                    AND ($7::TEXT IS NULL OR event_event = $7)
                ORDER BY t
                LIMIT $2
            )
            SELECT timestamp, transaction_id, receipt_id, block_height, account_id, predecessor_id, event_standard, event_version, event_event, event_data
            FROM log_nep297
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                AND ($3::TEXT IS NULL OR account_id = $3)
                AND ($4::TEXT IS NULL OR predecessor_id = $4)
                AND ($5::TEXT IS NULL OR event_standard = $5)
                AND ($6::TEXT IS NULL OR event_version = $6)
                AND ($7::TEXT IS NULL OR event_event = $7)
            ORDER BY timestamp ASC
            LIMIT $2
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.account_id.as_ref().map(|s| s.to_string()),
            self.predecessor_id.as_ref().map(|s| s.to_string()),
            self.event_standard.as_ref().map(|s| s.to_string()),
            self.event_version.as_ref().map(|s| s.to_string()),
            self.event_event.as_ref().map(|s| s.to_string()),
        )
        .map(|record| LogNep297EventData {
            block_height: record.block_height as u64,
            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
            transaction_id: record.transaction_id.parse().unwrap(),
            receipt_id: record.receipt_id.parse().unwrap(),
            account_id: record.account_id.parse().unwrap(),
            predecessor_id: record.predecessor_id.parse().unwrap(),
            event_standard: record.event_standard,
            event_version: record.event_version,
            event_event: record.event_event,
            event_data: record.event_data,
        })
        .fetch_all(pool)
        .await
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
