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

pub struct TradeSwapEvent;

impl TradeSwapEvent {
    pub const ID: &'static str = "trade_swap";
}

#[cfg(feature = "impl")]
impl Event for TradeSwapEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when someone exchanges tokens. 1 trade = 1 event, even if it goes through multiple pools. This event is a net result of all sub-trades, and only includes the net balance changes of different tokens. If a trade involves a token but net change is 0 (for example, USDT -> USDC -> NEAR, all received USDC is exchanged for NEAR, so it's not included in the event). That means trades made by arbitrage bots will mostly have positive NEAR balance and no other tokens.");
    const CATEGORY: &'static str = "Trade";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = TradeSwapEventData;
    type RealtimeEventFilter = RtTradeSwapilter;
    type DatabaseAdapter = DbTradeSwapAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TradeSwapEventData {
    #[serde(with = "stringified_map")]
    #[schemars(with = "HashMap<String, String>")]
    pub balance_changes: HashMap<AccountId, i128>,

    #[schemars(with = "String")]
    pub trader: AccountId,
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
pub struct DbTradeSwapFilter {
    #[schemars(with = "Option<String>")]
    pub trader_account_id: Option<AccountId>,
    /// Comma-separated list of token account IDs
    pub involved_token_account_ids: Option<String>,
}

pub struct DbTradeSwapAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbTradeSwapAdapter {
    type Event = TradeSwapEvent;
    type Filter = DbTradeSwapFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO trade_swap_testnet (timestamp, transaction_id, receipt_id, block_height, trader, balance_changes)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                serde_json::to_value(&event.balance_changes.iter().map(|(k, v)| (k, v.to_string())).collect::<HashMap<_, _>>()).unwrap(),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO trade_swap (timestamp, transaction_id, receipt_id, block_height, trader, balance_changes)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                serde_json::to_value(&event.balance_changes.iter().map(|(k, v)| (k, v.to_string())).collect::<HashMap<_, _>>()).unwrap(),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbTradeSwapFilter {
    type Event = TradeSwapEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlTradeSwapEventData {
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            trader: String,
            balance_changes: serde_json::Value,
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
            SqlTradeSwapEventData,
            r#"
            SELECT *
            FROM trade_swap{#testnet}
            WHERE {#time}
                {#trader}
                {#balance_changes}
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
            #trader = match self.trader_account_id.as_ref().map(|t| t.as_str()) {
                Some(ref trader) => "AND trader = {trader}",
                None => "",
            },
            #balance_changes = match self.involved_token_account_ids.as_ref() {
                Some(ref tokens) => "AND balance_changes ?& {tokens}",
                None => "",
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
                .map(|record| {
                    (
                        record.transaction_id.parse().unwrap(),
                        TradeSwapEventData {
                            trader: record.trader.parse().unwrap(),
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            balance_changes: {
                                let balance_changes: HashMap<AccountId, String> =
                                    serde_json::from_value(record.balance_changes).unwrap();
                                balance_changes
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
pub struct RtTradeSwapilter {
    pub trader_account_id: Option<AccountId>,
    pub involved_token_account_ids: Option<Vec<AccountId>>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtTradeSwapilter {
    type Event = TradeSwapEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(trader_account_id) = &self.trader_account_id {
            if event.trader != *trader_account_id {
                return false;
            }
        }

        if let Some(involved_token_account_ids) = &self.involved_token_account_ids {
            for token_account_id in involved_token_account_ids {
                if !event.balance_changes.contains_key(token_account_id) {
                    return false;
                }
            }
        }

        true
    }
}
