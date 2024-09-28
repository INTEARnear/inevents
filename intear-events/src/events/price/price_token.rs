use std::collections::HashMap;
use std::str::FromStr;

#[cfg(feature = "impl")]
use inevents::actix_web::http::StatusCode;
#[cfg(feature = "impl")]
use inevents::events::event::{CustomHttpEndpoint, EventId, PaginationBy};
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

    #[cfg(feature = "impl")]
    fn custom_http_endpoints(pool: Pool<Postgres>) -> Vec<Box<dyn CustomHttpEndpoint>> {
        vec![Box::new(OhlcEndpoint { pool })]
    }
}

#[cfg(feature = "impl")]
pub struct OhlcEndpoint {
    pool: Pool<Postgres>,
}

#[cfg(feature = "impl")]
impl CustomHttpEndpoint for OhlcEndpoint {
    fn name(&self) -> &'static str {
        "ohlc"
    }

    fn handle(
        &self,
        query: HashMap<String, String>,
        _testnet: bool,
    ) -> tokio::task::JoinHandle<(StatusCode, serde_json::Value)> {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let Some(token) = query.get("token") else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({"error": "token is required"}),
                );
            };
            let Ok(token) = AccountId::from_str(token) else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "invalid token"
                    }),
                );
            };
            let Some(resolution) = query.get("resolution") else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "resolution is required"
                    }),
                );
            };
            let Ok(resolution) = OhlcResolution::from_str(resolution) else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "invalid resolution"
                    }),
                );
            };
            let Some(count_back) = query.get("count_back") else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "count_back is required"
                    }),
                );
            };
            let Ok(count_back) = count_back.parse::<usize>() else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "invalid count_back"
                    }),
                );
            };
            const MAX_COUNT_BACK: usize = 15000;
            if count_back > MAX_COUNT_BACK {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": format!("count_back must be less than or equal to {MAX_COUNT_BACK}")
                    }),
                );
            }
            let Some(to) = query.get("to") else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "to is required"
                    }),
                );
            };
            let Ok(to) = to.parse::<i64>() else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "invalid to"
                    }),
                );
            };
            let Some(to) = chrono::DateTime::from_timestamp_millis(to) else {
                return (
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({
                        "error": "invalid to"
                    }),
                );
            };

            macro_rules! query_materialized_view {
                ($q: literal) => {
                    sqlx::query!(
                        $q,
                        token.to_string(),
                        count_back as i64 + 1,
                        to,
                    )
                    .fetch_all(&pool)
                    .await
                    .map(|records| {
                        let mut bars = Vec::new();
                        let mut records = records.into_iter().rev();
                        let Some(first_bar) = records.next() else {
                            return bars;
                        };
                        let mut prev_close = first_bar.close.unwrap_or_default();
                        for record in records {
                            let high = record.high.unwrap_or_default().max(prev_close.clone());
                            let low = record.low.unwrap_or_default().min(prev_close.clone());
                            bars.push(serde_json::json!({
                                "time": record.bucket.unwrap_or_default().timestamp_millis(),
                                "open": prev_close.with_prec(42).to_string(),
                                "high": high.with_prec(42).to_string(),
                                "low": low.with_prec(42).to_string(),
                                "close": record.close.clone().unwrap_or_default().with_prec(42).to_string(),
                            }));
                            prev_close = record.close.unwrap_or_default();
                        }
                        bars
                    })
                    .map_err(|err| {
                        log::error!("Error querying OHLC data: {:?}", err);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            serde_json::json!({"error": "internal server error"}),
                        )
                    })
                };
            }

            let results = match resolution {
                OhlcResolution::OneMinute => query_materialized_view!(
                    r#"
                    SELECT bucket, open, high, low, close
                    FROM price_token_1min_ohlc
                    WHERE token = $1
                    AND bucket < $3
                    ORDER BY bucket DESC
                    LIMIT $2;
                    "#
                ),
                OhlcResolution::OneHour => query_materialized_view!(
                    r#"
                    SELECT bucket, open, high, low, close
                    FROM price_token_1hour_ohlc
                    WHERE token = $1
                    AND bucket < $3
                    ORDER BY bucket DESC
                    LIMIT $2;
                    "#
                ),
                OhlcResolution::OneDay => query_materialized_view!(
                    r#"
                    SELECT bucket, open, high, low, close
                    FROM price_token_1day_ohlc
                    WHERE token = $1
                    AND bucket < $3
                    ORDER BY bucket DESC
                    LIMIT $2;
                    "#
                ),
            };
            let results = match results {
                Ok(results) => results,
                Err((status, error)) => return (status, error),
            };

            (
                StatusCode::OK,
                serde_json::to_value(&results).expect("Error serializing OHLC response"),
            )
        })
    }
}

#[allow(clippy::enum_variant_names)]
enum OhlcResolution {
    OneMinute,
    OneHour,
    OneDay,
}

impl FromStr for OhlcResolution {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::OneMinute),
            "60" => Ok(Self::OneHour),
            "1D" => Ok(Self::OneDay),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PriceTokenEventData {
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
            INSERT INTO price_token (timestamp, token, price_usd)
            VALUES ($1, $2, $3)
            "#,
            chrono::DateTime::from_timestamp(
                (event.timestamp_nanosec / 1_000_000_000) as i64,
                (event.timestamp_nanosec % 1_000_000_000) as u32
            ),
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
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        if let PaginationBy::BeforeBlockHeight { .. } | PaginationBy::AfterBlockHeight { .. } =
            pagination.pagination_by
        {
            return Ok(Vec::new()); // Doesn't have block height
        }
        #[derive(Debug, sqlx::FromRow)]
        struct SqlPriceTokenEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            token: String,
            price_usd: BigDecimal,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let token = self.token.as_ref().map(|t| t.as_str());
        sqlx_conditional_queries::conditional_query_as!(
            SqlPriceTokenEventData,
            r#"
            SELECT *
            FROM price_token{#testnet}
            WHERE {#time}
                AND ({token}::TEXT IS NULL OR token = {token})
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
                        PriceTokenEventData {
                            token: record.token.parse().unwrap(),
                            price_usd: record.price_usd,
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
