#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
use inindexer::near_indexer_primitives::types::AccountId;
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

type PoolId = String;

pub struct PricePoolEvent;

impl PricePoolEvent {
    pub const ID: &'static str = "price_pool";
}

#[cfg(feature = "impl")]
impl Event for PricePoolEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired for every trade_pool_change event, with token prices calculated, in the same format for all pools (ref, ref stable swap, ref rated swap, ref dcl, others). Pools with != 2 tokens are not supported.");
    const CATEGORY: &'static str = "Price";

    type EventData = PricePoolEventData;
    type RealtimeEventFilter = RtPricePoolFilter;
    type DatabaseAdapter = DbPricePoolAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PricePoolEventData {
    pub pool_id: PoolId,
    #[schemars(with = "String")]
    pub token0: AccountId,
    #[schemars(with = "String")]
    pub token1: AccountId,
    #[serde(with = "stringified")]
    #[schemars(with = "String")]
    pub token0_in_1_token1: BigDecimal,
    #[serde(with = "stringified")]
    #[schemars(with = "String")]
    pub token1_in_1_token0: BigDecimal,

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
pub struct DbPricePoolFilter {
    pub pool_id: Option<PoolId>,
    /// Comma-separated list of token account IDs
    pub involved_token_account_ids: Option<String>,
}

pub struct DbPricePoolAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbPricePoolAdapter {
    type Event = PricePoolEvent;
    type Filter = DbPricePoolFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"
            INSERT INTO price_pool (timestamp, pool_id, token0, token1, token0_in_1_token1, token1_in_1_token0)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            chrono::DateTime::from_timestamp((event.timestamp_nanosec / 1_000_000_000) as i64, (event.timestamp_nanosec % 1_000_000_000) as u32),
            event.pool_id.as_str(),
            event.token0.as_str(),
            event.token1.as_str(),
            event.token0_in_1_token1,
            event.token1_in_1_token0,
        ).execute(pool).await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbPricePoolFilter {
    type Event = PricePoolEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        if let PaginationBy::BeforeBlockHeight { .. } | PaginationBy::AfterBlockHeight { .. } =
            pagination.pagination_by
        {
            return Ok(Vec::new()); // Doesn't have block height
        }
        #[derive(Debug, sqlx::FromRow)]
        struct SqlPricePoolEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            pool_id: String,
            token0: String,
            token1: String,
            token0_in_1_token1: BigDecimal,
            token1_in_1_token0: BigDecimal,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let involved_token_account_ids = &self
            .involved_token_account_ids
            .as_ref()
            .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();

        let pool_id = self.pool_id.as_deref();
        let involved_token_account_ids = if involved_token_account_ids.is_empty() {
            None
        } else {
            Some(&involved_token_account_ids[..])
        };
        // let involved_token_account_ids = involved_token_account_ids.as_deref();

        sqlx_conditional_queries::conditional_query_as!(
            SqlPricePoolEventData,
            r#"
            SELECT *
            FROM price_pool{#testnet}
            WHERE {#time}
                AND ({pool_id}::TEXT IS NULL OR pool_id = {pool_id})
                AND ({involved_token_account_ids}::TEXT[] IS NULL OR ARRAY[token0, token1] @> {involved_token_account_ids})
            ORDER BY id {#order}
            LIMIT {limit}
            "#,
            #(time, order) = match &pagination.pagination_by {
                PaginationBy::BeforeBlockHeight { ..} | PaginationBy::AfterBlockHeight { .. } => ("true", "DESC"),
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
                    PricePoolEventData {
                        pool_id: record.pool_id,
                        token0: record.token0.parse().unwrap(),
                        token1: record.token1.parse().unwrap(),
                        token0_in_1_token1: record.token0_in_1_token1,
                        token1_in_1_token0: record.token1_in_1_token0,
                        timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                        as u128,
                    },
                )
            })
            .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtPricePoolFilter {
    pub pool_id: Option<PoolId>,
    pub involved_token_account_ids: Option<Vec<AccountId>>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtPricePoolFilter {
    type Event = PricePoolEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(pool_id) = &self.pool_id {
            if pool_id != &event.pool_id {
                return false;
            }
        }

        if let Some(involved_token_account_ids) = &self.involved_token_account_ids {
            for token_account_id in involved_token_account_ids {
                if token_account_id != &event.token0 && token_account_id != &event.token1 {
                    return false;
                }
            }
        }

        true
    }
}
