use std::future::Future;

use near_indexer_primitives::types::BlockHeightDelta;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, Pool, Postgres};

pub trait Event {
    const ID: &'static str;
    const DESCRIPTION: Option<&'static str> = None;
    const CATEGORY: &'static str = "Unspecified";

    type EventData: Serialize + for<'de> Deserialize<'de> + JsonSchema;
    type RealtimeEventFilter: RealtimeEventFilter<Event = Self>;
    type DatabaseAdapter: DatabaseEventAdapter<Event = Self>;
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
    ) -> impl Future<Output = Result<PgQueryResult, sqlx::Error>>;
}

pub trait DatabaseEventFilter {
    type Event: Event;

    fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
    ) -> impl Future<Output = Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error>>;
}

pub const MAX_BLOCKS_PER_REQUEST: u64 = 100;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PaginationParameters {
    #[serde(default)]
    pub start_block_timestamp_nanosec: u64,
    #[serde(default = "default_blocks_per_request")]
    #[schemars(range(max = 100))]
    pub blocks: BlockHeightDelta,
}

fn default_blocks_per_request() -> u64 {
    10
}
