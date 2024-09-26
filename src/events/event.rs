use std::collections::HashMap;
use std::future::Future;

use actix_web::http::StatusCode;
use near_indexer_primitives::near_primitives::serialize::dec_format;
use near_indexer_primitives::types::BlockHeight;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, Pool, Postgres};
use utoipa::openapi::PathItem;

pub trait Event {
    const ID: &'static str;
    const DESCRIPTION: Option<&'static str> = None;
    const CATEGORY: &'static str = "Unspecified";
    const EXCLUDE_FROM_DATABASE: bool = false;
    const SUPPORTS_TESTNET: bool = false;

    type EventData: Serialize + for<'de> Deserialize<'de> + JsonSchema;
    type RealtimeEventFilter: RealtimeEventFilter<Event = Self>;
    type DatabaseAdapter: DatabaseEventAdapter<Event = Self>;

    fn custom_http_endpoints(_pool: Pool<Postgres>) -> Vec<Box<dyn CustomHttpEndpoint>> {
        vec![]
    }
}

pub trait CustomHttpEndpoint {
    fn name(&self) -> &'static str;

    /// Return none if not supported
    fn openapi(&self) -> Option<PathItem> {
        None
    }

    fn handle(
        &self,
        query: HashMap<String, String>,
        testnet: bool,
    ) -> tokio::task::JoinHandle<(StatusCode, serde_json::Value)>;
}

pub trait RealtimeEventFilter: for<'de> Deserialize<'de> {
    type Event: Event;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool;
}

pub trait DatabaseEventAdapter {
    type Event: Event;
    type Filter: DatabaseEventFilter<Event = Self::Event> + for<'de> Deserialize<'de> + JsonSchema;

    fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> impl Future<Output = Result<PgQueryResult, sqlx::Error>>;
}

pub trait DatabaseEventFilter {
    type Event: Event;

    fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> impl Future<Output = Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error>>;
}

pub const MAX_EVENTS_PER_REQUEST: u64 = 200;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PaginationParameters {
    pub limit: u64,
    #[serde(flatten)]
    pub pagination_by: PaginationBy,
}

#[derive(Debug, Deserialize, JsonSchema, utoipa::ToSchema)]
#[serde(tag = "pagination_by")]
pub enum PaginationBy {
    Oldest,
    Newest,
    BeforeBlockHeight {
        #[serde(deserialize_with = "dec_format::deserialize")]
        block_height: BlockHeight,
    },
    AfterBlockHeight {
        #[serde(deserialize_with = "dec_format::deserialize")]
        block_height: BlockHeight,
    },
    BeforeTimestamp {
        #[serde(deserialize_with = "dec_format::deserialize")]
        timestamp_nanosec: u128,
    },
    AfterTimestamp {
        #[serde(deserialize_with = "dec_format::deserialize")]
        timestamp_nanosec: u128,
    },
    BeforeId {
        #[serde(deserialize_with = "dec_format::deserialize")]
        id: EventId,
    },
    AfterId {
        #[serde(deserialize_with = "dec_format::deserialize")]
        id: EventId,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
pub enum Order {
    Ascending,
    Descending,
}

pub type EventId = u64;
