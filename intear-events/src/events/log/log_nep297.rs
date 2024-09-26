use inevents::events::event::EventId;
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
        unimplemented!()
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbLogNep297Filter {
    type Event = LogNep297Event;

    async fn query_paginated(
        &self,
        _pagination: &PaginationParameters,
        _pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        unimplemented!()
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
