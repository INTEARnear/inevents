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

pub struct MemeCookingWithdrawEvent;

impl MemeCookingWithdrawEvent {
    pub const ID: &'static str = "memecooking_withdraw";
}

#[cfg(feature = "impl")]
impl Event for MemeCookingWithdrawEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> =
        Some("Fired when a withdraw is made in the MemeCooking protocol.");
    const CATEGORY: &'static str = "Trade";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = MemeCookingWithdrawEventData;
    type RealtimeEventFilter = RtMemeCookingWithdrawFilter;
    type DatabaseAdapter = DbMemeCookingWithdrawAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MemeCookingWithdrawEventData {
    pub meme_id: u64,
    #[schemars(with = "String")]
    pub trader: AccountId,
    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub amount: Balance,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub fee: Balance,
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbMemeCookingWithdrawFilter {
    pub meme_id: Option<u64>,
    #[schemars(with = "Option<String>")]
    pub trader_account_id: Option<AccountId>,
}

pub struct DbMemeCookingWithdrawAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbMemeCookingWithdrawAdapter {
    type Event = MemeCookingWithdrawEvent;
    type Filter = DbMemeCookingWithdrawFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO memecooking_withdraw_testnet (timestamp, transaction_id, receipt_id, block_height, trader, meme_id, amount, fee)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                event.meme_id as i64,
                BigDecimal::from(event.amount),
                BigDecimal::from(event.fee),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO memecooking_withdraw (timestamp, transaction_id, receipt_id, block_height, trader, meme_id, amount, fee)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                event.meme_id as i64,
                BigDecimal::from(event.amount),
                BigDecimal::from(event.fee),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbMemeCookingWithdrawFilter {
    type Event = MemeCookingWithdrawEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlMemeCookingWithdrawEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            trader: String,
            meme_id: i64,
            amount: BigDecimal,
            fee: BigDecimal,
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
            SqlMemeCookingWithdrawEventData,
            r#"
            SELECT *
            FROM memecooking_withdraw{#testnet}
            WHERE {#time}
                {#trader}
                {#meme_id}
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
            #trader = match self.trader_account_id.as_ref().map(|id| id.as_str()) {
                Some(ref trader) => "AND trader = {trader}",
                None => "",
            },
            #meme_id = match self.meme_id {
                Some(meme_id) => "AND meme_id = {meme_id}",
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
                        record.id as EventId,
                        MemeCookingWithdrawEventData {
                            meme_id: record.meme_id as u64,
                            trader: record.trader.parse().unwrap(),
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            amount: num_traits::ToPrimitive::to_u128(&record.amount)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.amount,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            fee: num_traits::ToPrimitive::to_u128(&record.fee).unwrap_or_else(
                                || {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.fee,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                },
                            ),
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
pub struct RtMemeCookingWithdrawFilter {
    pub meme_id: Option<u64>,
    pub trader_account_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtMemeCookingWithdrawFilter {
    type Event = MemeCookingWithdrawEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(trader_account_id) = &self.trader_account_id {
            if trader_account_id != &event.trader {
                return false;
            }
        }

        if let Some(meme_id) = &self.meme_id {
            if event.meme_id != *meme_id {
                return false;
            }
        }

        true
    }
}
