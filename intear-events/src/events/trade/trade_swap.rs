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
    ) -> Result<PgQueryResult, sqlx::Error> {
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

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbTradeSwapFilter {
    type Event = TradeSwapEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        let involved_token_account_ids = self
            .involved_token_account_ids
            .as_ref()
            .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        sqlx::query!(
            r#"
            WITH blocks AS (
                SELECT DISTINCT timestamp as t
                FROM trade_swap
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR trader = $3)
                    AND ($4::TEXT[] IS NULL OR balance_changes ?& $4)
                ORDER BY t
                LIMIT $2
            )
            SELECT transaction_id, receipt_id, block_height, timestamp, trader, balance_changes
            FROM trade_swap
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE ($3::TEXT IS NULL OR trader = $3)
                AND ($4::TEXT[] IS NULL OR balance_changes ?& $4)
            ORDER BY timestamp ASC
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.trader_account_id.as_ref().map(|id| id.to_string()),
            involved_token_account_ids.as_slice(),
        )
        .map(|record| TradeSwapEventData {
            trader: record.trader.parse().unwrap(),
            transaction_id: record.transaction_id.parse().unwrap(),
            receipt_id: record.receipt_id.parse().unwrap(),
            block_height: record.block_height as u64,
            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
            balance_changes: {
                let balance_changes: HashMap<AccountId, String> =
                    serde_json::from_value(record.balance_changes).unwrap();
                balance_changes
                    .into_iter()
                    .map(|(k, v)| (k, v.parse().unwrap()))
                    .collect()
            },
        })
        .fetch_all(pool)
        .await
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
