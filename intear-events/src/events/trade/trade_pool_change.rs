use inevents::events::event::{EventId, PaginationBy};
use inindexer::near_indexer_primitives::types::{AccountId, Balance, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
use inindexer::near_utils::{dec_format, dec_format_vec};
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

type PoolId = String;

pub struct TradePoolChangeEvent;

impl TradePoolChangeEvent {
    pub const ID: &'static str = "trade_pool_change";
}

#[cfg(feature = "impl")]
impl Event for TradePoolChangeEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when a DEX pool changes. For example, when someone exchanges tokens, adds or removes liquidity, or when fee is changed. The behavior is different for each pool, but it's pretty much guaranteed that 2 consecutive events will have different data.");
    const CATEGORY: &'static str = "Trade";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = TradePoolChangeEventData;
    type RealtimeEventFilter = RtTradePoolChangeilter;
    type DatabaseAdapter = DbTradePoolChangeAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TradePoolChangeEventData {
    pub pool_id: PoolId,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    pub block_height: BlockHeight,
    pub pool: PoolType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPoolSwap {
    pub pool: PoolId,
    pub token_in: AccountId,
    pub token_out: AccountId,
    #[serde(with = "dec_format")]
    pub amount_in: Balance,
    #[serde(with = "dec_format")]
    pub amount_out: Balance,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbTradePoolChangeFilter {
    pub pool_id: Option<PoolId>,
}

pub struct DbTradePoolChangeAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbTradePoolChangeAdapter {
    type Event = TradePoolChangeEvent;
    type Filter = DbTradePoolChangeFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO trade_pool_change_testnet (timestamp, receipt_id, block_height, pool_id, pool)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.pool_id,
                serde_json::to_value(&event.pool).unwrap(),
            )
            .execute(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO trade_pool_change (timestamp, receipt_id, block_height, pool_id, pool)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.pool_id,
                serde_json::to_value(&event.pool).unwrap(),
            )
            .execute(pool)
            .await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbTradePoolChangeFilter {
    type Event = TradePoolChangeEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlTradePoolChangeEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            receipt_id: String,
            block_height: i64,
            pool_id: String,
            pool: serde_json::Value,
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

        let pool_id = self.pool_id.as_deref();
        sqlx_conditional_queries::conditional_query_as!(
            SqlTradePoolChangeEventData,
            r#"
            SELECT *
            FROM trade_pool_change{#testnet}
            WHERE {#time}
                AND ({pool_id}::TEXT IS NULL OR pool_id = {pool_id})
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
                        TradePoolChangeEventData {
                            pool_id: record.pool_id,
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            block_height: record.block_height as BlockHeight,
                            pool: serde_json::from_value(record.pool).unwrap(),
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtTradePoolChangeilter {
    pub pool_id: Option<PoolId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtTradePoolChangeilter {
    type Event = TradePoolChangeEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(pool_id) = &self.pool_id {
            if pool_id != &event.pool_id {
                return false;
            }
        }

        true
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub enum PoolType {
    Ref(RefPool),
}

#[allow(clippy::enum_variant_names)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub enum RefPool {
    SimplePool(RefSimplePool),
    StableSwapPool(RefStableSwapPool),
    RatedSwapPool(RefRatedSwapPool),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RefSimplePool {
    /// List of tokens in the pool.
    #[schemars(with = "Vec<String>")]
    pub token_account_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    #[serde(with = "dec_format_vec")]
    #[schemars(with = "Vec<String>")]
    pub amounts: Vec<Balance>,
    /// Volumes accumulated by this pool.
    pub volumes: Vec<RefSwapVolume>,
    /// Fee charged for swap (gets divided by FEE_DIVISOR).
    pub total_fee: u32,
    /// Obsolete, reserve to simplify upgrade.
    pub exchange_fee: u32,
    /// Obsolete, reserve to simplify upgrade.
    pub referral_fee: u32,
    // /// Shares of the pool by liquidity providers.
    // pub shares: LookupMap<SdkAccountId, Balance>,
    /// Total number of shares.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub shares_total_supply: Balance,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RefSwapVolume {
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub input: Balance,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub output: Balance,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RefStableSwapPool {
    /// List of tokens in the pool.
    #[schemars(with = "Vec<String>")]
    pub token_account_ids: Vec<AccountId>,
    /// Each decimals for tokens in the pool
    pub token_decimals: Vec<u8>,
    /// token amounts in comparable decimal.
    #[serde(with = "dec_format_vec")]
    #[schemars(with = "Vec<String>")]
    pub c_amounts: Vec<Balance>,
    /// Volumes accumulated by this pool.
    pub volumes: Vec<RefSwapVolume>,
    /// Fee charged for swap (gets divided by FEE_DIVISOR).
    pub total_fee: u32,
    // /// Shares of the pool by liquidity providers.
    // pub shares: LookupMap<SdkAccountId, Balance>,
    /// Total number of shares.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub shares_total_supply: Balance,
    /// Initial amplification coefficient.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub init_amp_factor: u128,
    /// Target for ramping up amplification coefficient.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub target_amp_factor: u128,
    /// Initial amplification time.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub init_amp_time: u64,
    /// Stop ramp up amplification time.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub stop_amp_time: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RefRatedSwapPool {
    /// List of tokens in the pool.
    #[schemars(with = "Vec<String>")]
    pub token_account_ids: Vec<AccountId>,
    /// Each decimals for tokens in the pool
    pub token_decimals: Vec<u8>,
    /// token amounts in comparable decimal.
    #[serde(with = "dec_format_vec")]
    #[schemars(with = "Vec<String>")]
    pub c_amounts: Vec<Balance>,
    /// Volumes accumulated by this pool.
    pub volumes: Vec<RefSwapVolume>,
    /// Fee charged for swap (gets divided by FEE_DIVISOR).
    pub total_fee: u32,
    // /// Shares of the pool by liquidity providers.
    // actual type: pub shares: LookupMap<SdkAccountId, Balance>,
    /// Total number of shares.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub shares_total_supply: Balance,
    /// Initial amplification coefficient.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub init_amp_factor: u128,
    /// Target for ramping up amplification coefficient.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub target_amp_factor: u128,
    /// Initial amplification time.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub init_amp_time: u64,
    /// Stop ramp up amplification time.
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub stop_amp_time: u64,
}
