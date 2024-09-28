#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
use inindexer::near_indexer_primitives::types::{AccountId, Balance, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
use inindexer::near_utils::dec_format;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(feature = "impl")]
use sqlx::postgres::PgQueryResult;
#[cfg(feature = "impl")]
use sqlx::types::BigDecimal;
#[cfg(feature = "impl")]
use sqlx::{Pool, Postgres};

#[cfg(feature = "impl")]
use inevents::events::event::{
    DatabaseEventAdapter, DatabaseEventFilter, Event, PaginationParameters, RealtimeEventFilter,
};

type PoolId = String;

pub struct TradePoolEvent;

impl TradePoolEvent {
    pub const ID: &'static str = "trade_pool";
}

#[cfg(feature = "impl")]
impl Event for TradePoolEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired for each pool a trade goes through. For example, if someone exchanges USDT -> USDC -> NEAR, this event will be fired twice, for USDT -> USDC and USDC -> NEAR");
    const CATEGORY: &'static str = "Trade";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = TradePoolEventData;
    type RealtimeEventFilter = RtTradePoolilter;
    type DatabaseAdapter = DbTradePoolAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TradePoolEventData {
    pub pool: PoolId,
    #[schemars(with = "String")]
    pub token_in: AccountId,
    #[schemars(with = "String")]
    pub token_out: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub amount_in: Balance,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub amount_out: Balance,

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbTradePoolFilter {
    pub pool_id: Option<PoolId>,
    #[schemars(with = "Option<String>")]
    pub trader_account_id: Option<AccountId>,
}

pub struct DbTradePoolAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbTradePoolAdapter {
    type Event = TradePoolEvent;
    type Filter = DbTradePoolFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO trade_pool_testnet (timestamp, transaction_id, receipt_id, block_height, trader, pool, token_in, token_out, amount_in, amount_out)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                event.pool.to_string(),
                event.token_in.to_string(),
                event.token_out.to_string(),
                BigDecimal::from(event.amount_in),
                BigDecimal::from(event.amount_out),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO trade_pool (timestamp, transaction_id, receipt_id, block_height, trader, pool, token_in, token_out, amount_in, amount_out)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                event.pool.to_string(),
                event.token_in.to_string(),
                event.token_out.to_string(),
                BigDecimal::from(event.amount_in),
                BigDecimal::from(event.amount_out),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbTradePoolFilter {
    type Event = TradePoolEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlTradePoolEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            trader: String,
            pool: String,
            token_in: String,
            token_out: String,
            amount_in: BigDecimal,
            amount_out: BigDecimal,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let pool_id = self.pool_id.as_deref();
        let trader_account_id = self.trader_account_id.as_ref().map(|id| id.as_str());

        sqlx_conditional_queries::conditional_query_as!(
            SqlTradePoolEventData,
            r#"
            SELECT *
            FROM trade_pool{#testnet}
            WHERE {#time}
                AND ({trader_account_id}::TEXT IS NULL OR trader = {trader_account_id})
                AND ({pool_id}::TEXT IS NULL OR pool = {pool_id})
            ORDER BY id {#order}
            LIMIT {limit}
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
                .map(|record| {
                    (
                        record.id as EventId,
                        TradePoolEventData {
                            pool: record.pool,
                            token_in: record.token_in.parse().unwrap(),
                            token_out: record.token_out.parse().unwrap(),
                            amount_in: num_traits::ToPrimitive::to_u128(&record.amount_in)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.amount_in,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            amount_out: num_traits::ToPrimitive::to_u128(&record.amount_out)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.amount_out,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            trader: record.trader.parse().unwrap(),
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtTradePoolilter {
    pub pool_id: Option<PoolId>,
    pub trader_account_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtTradePoolilter {
    type Event = TradePoolEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(trader_account_id) = &self.trader_account_id {
            if trader_account_id != &event.trader {
                return false;
            }
        }

        if let Some(pool_id) = &self.pool_id {
            if event.pool != *pool_id {
                return false;
            }
        }

        true
    }
}
