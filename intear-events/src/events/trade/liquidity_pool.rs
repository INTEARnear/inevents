#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
use std::collections::HashMap;

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

pub struct LiquidityPoolEvent;

impl LiquidityPoolEvent {
    pub const ID: &'static str = "liquidity_pool";
}

#[cfg(feature = "impl")]
impl Event for LiquidityPoolEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when someone adds or removes LP.");
    const CATEGORY: &'static str = "Trade";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = LiquidityPoolEventData;
    type RealtimeEventFilter = RtLiquidityPoolilter;
    type DatabaseAdapter = DbLiquidityPoolAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LiquidityPoolEventData {
    pub pool: String,
    #[serde(with = "stringified_map")]
    #[schemars(with = "HashMap<String, String>")]
    pub tokens: HashMap<AccountId, i128>,
    #[schemars(with = "String")]
    pub provider_account_id: AccountId,
    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
}

mod stringified_map {
    use std::collections::HashMap;
    use std::str::FromStr;

    use serde::de::MapAccess;
    use serde::ser::SerializeMap;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Serialize,
        V: ToString,
    {
        let mut map = serializer.serialize_map(Some(value.len()))?;
        for (k, v) in value {
            map.serialize_entry(k, &v.to_string())?;
        }
        map.end()
    }

    struct Visitor<T, K, V>(
        std::marker::PhantomData<T>,
        std::marker::PhantomData<K>,
        std::marker::PhantomData<V>,
    );

    impl<'de, T, K, V> de::Visitor<'de> for Visitor<T, K, V>
    where
        K: Deserialize<'de>,
        V: FromStr,
        V::Err: std::fmt::Display,
        T: FromIterator<(K, V)>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut items = Vec::with_capacity(map.size_hint().unwrap_or(0));
            while let Some((key, value)) = map.next_entry::<K, String>()? {
                items.push((key, V::from_str(&value).map_err(serde::de::Error::custom)?));
            }
            Ok(items.into_iter().collect())
        }
    }

    pub fn deserialize<'de, T, K, V, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: FromStr,
        V::Err: std::fmt::Display,
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(Visitor(
            std::marker::PhantomData,
            std::marker::PhantomData,
            std::marker::PhantomData,
        ))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbLiquidityPoolFilter {
    #[schemars(with = "Option<String>")]
    pub provider_account_id: Option<AccountId>,
    /// Comma-separated list of token account IDs
    pub involved_token_account_ids: Option<String>,
}

pub struct DbLiquidityPoolAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbLiquidityPoolAdapter {
    type Event = LiquidityPoolEvent;
    type Filter = DbLiquidityPoolFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO liquidity_pool_testnet (timestamp, transaction_id, receipt_id, block_height, provider_account_id, pool_id, tokens)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.provider_account_id.to_string(),
                event.pool,
                serde_json::to_value(&event.tokens.iter().map(|(k, v)| (k, v.to_string())).collect::<HashMap<_, _>>()).unwrap(),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO liquidity_pool (timestamp, transaction_id, receipt_id, block_height, provider_account_id, pool_id, tokens)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.provider_account_id.to_string(),
                event.pool,
                serde_json::to_value(&event.tokens.iter().map(|(k, v)| (k, v.to_string())).collect::<HashMap<_, _>>()).unwrap(),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbLiquidityPoolFilter {
    type Event = LiquidityPoolEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlLiquidityPoolEventData {
            id: i64,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            provider_account_id: String,
            pool_id: String,
            tokens: serde_json::Value,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let provider_account_id = self.provider_account_id.as_ref().map(|t| t.as_str());
        let involved_token_account_ids = self
            .involved_token_account_ids
            .as_ref()
            .map(|ids| ids.split(',').map(|s| s.to_string()).collect::<Vec<_>>());
        let involved_token_account_ids = involved_token_account_ids.as_deref();

        sqlx_conditional_queries::conditional_query_as!(
            SqlLiquidityPoolEventData,
            r#"
            SELECT *
            FROM liquidity_pool{#testnet}
            WHERE {#time}
                AND ({provider_account_id}::TEXT IS NULL OR provider_account_id = {provider_account_id})
                AND ({involved_token_account_ids}::TEXT[] IS NULL OR tokens ?& {involved_token_account_ids})
            ORDER BY id {#order}
            LIMIT greatest({limit}, 16::bigint) -- db gives better strategy for 16+ limit, so limiting it on server side
            "#,
            #(time, order) = match &pagination.pagination_by {
                PaginationBy::BeforeTimestamp { .. }
                    | PaginationBy::BeforeBlockHeight { .. } => ("timestamp < {timestamp}", "DESC"),
                PaginationBy::AfterTimestamp { .. }
                    |  PaginationBy::AfterBlockHeight { .. } => ("timestamp > {timestamp}", "ASC"),
                PaginationBy::BeforeId { .. } => ("id < {id}", "DESC"),
                PaginationBy::AfterId { .. } => ("id > {id}", "ASC"),
                PaginationBy::Oldest => ("true", "ASC"),
                PaginationBy::Newest => ("true", "DESC"),
            },
            #testnet = match testnet {
                true => "_testnet",
                false => "",
            },
        )
        .fetch_all(pool)
        .await
        .map(|records| {
            records
            .into_iter()
            .take(limit as usize) // we added greatest(limit, 16) in query
            .map(|record| {
                (
                    record.id as u64,
                    LiquidityPoolEventData {
                        pool: record.pool_id,
                        provider_account_id: record.provider_account_id.parse().unwrap(),
                        transaction_id: record.transaction_id.parse().unwrap(),
                        receipt_id: record.receipt_id.parse().unwrap(),
                        block_height: record.block_height as u64,
                        block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                        tokens: {
                            let tokens: HashMap<AccountId, String> = serde_json::from_value(record.tokens).unwrap();
                            tokens
                                .into_iter()
                                .map(|(k, v)| (k, v.parse().unwrap()))
                                .collect()
                        },
                    },
                )
            })
            .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtLiquidityPoolilter {
    pub provider_account_id: Option<AccountId>,
    pub involved_token_account_ids: Option<Vec<AccountId>>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtLiquidityPoolilter {
    type Event = LiquidityPoolEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(provider_account_id) = &self.provider_account_id {
            if event.provider_account_id != *provider_account_id {
                return false;
            }
        }

        if let Some(involved_token_account_ids) = &self.involved_token_account_ids {
            for token_account_id in involved_token_account_ids {
                if !event.tokens.contains_key(token_account_id) {
                    return false;
                }
            }
        }

        true
    }
}
