use inindexer::near_indexer_primitives::types::{AccountId, BlockHeight};
use inindexer::near_utils::dec_format;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "impl")]
use sqlx::postgres::PgQueryResult;
use sqlx::types::BigDecimal;
#[cfg(feature = "impl")]
use sqlx::{Pool, Postgres};

#[cfg(feature = "impl")]
use inevents::events::event::{
    DatabaseEventAdapter, DatabaseEventFilter, Event, PaginationParameters, RealtimeEventFilter,
};

pub struct PriceTokenEvent;

impl PriceTokenEvent {
    pub const ID: &'static str = "price_token";
}

#[cfg(feature = "impl")]
impl Event for PriceTokenEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired approximately every 1-15 seconds for each token if its price has changed (even if the quote asset has changed its price but no transaction with the token itself), and for every transaction if a token price has been directly changed in the transaction (e.g. swap with a token). Contains the price in USD");
    const CATEGORY: &'static str = "Price";

    type EventData = PriceTokenEventData;
    type RealtimeEventFilter = RtPriceTokenFilter;
    type DatabaseAdapter = DbPriceTokenAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PriceTokenEventData {
    pub block_height: BlockHeight,
    #[schemars(with = "String")]
    pub token: AccountId,
    #[serde(with = "stringified")]
    #[schemars(with = "String")]
    pub price_usd: BigDecimal,

    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub timestamp_nanosec: u128,
}

mod stringified {
    use serde::Deserialize;

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: ToString,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbPriceTokenFilter {
    #[schemars(with = "Option<String>")]
    pub token: Option<AccountId>,
}

pub struct DbPriceTokenAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbPriceTokenAdapter {
    type Event = PriceTokenEvent;
    type Filter = DbPriceTokenFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO price_token (timestamp, block_height, token, price_usd)
            VALUES ($1, $2, $3, $4)
            "#,
            chrono::DateTime::from_timestamp(
                (event.timestamp_nanosec / 1_000_000_000) as i64,
                (event.timestamp_nanosec % 1_000_000_000) as u32
            ),
            event.block_height as i64,
            event.token.to_string(),
            event.price_usd,
        )
        .execute(pool)
        .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbPriceTokenFilter {
    type Event = PriceTokenEvent;

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
                FROM price_token
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR token = $3)
                ORDER BY t
                LIMIT $2
            )
            SELECT timestamp, block_height, token, price_usd
            FROM price_token
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                AND ($3::TEXT IS NULL OR token = $3)
            ORDER BY timestamp ASC
            LIMIT $2
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.token.as_ref().map(|s| s.to_string()),
        )
        .map(|record| PriceTokenEventData {
            block_height: record.block_height as u64,
            token: record.token.parse().unwrap(),
            price_usd: record.price_usd,
            timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
        })
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, Deserialize)]
pub struct RtPriceTokenFilter {
    pub token: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtPriceTokenFilter {
    type Event = PriceTokenEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(token) = &self.token {
            if token != &event.token {
                return false;
            }
        }

        true
    }
}
