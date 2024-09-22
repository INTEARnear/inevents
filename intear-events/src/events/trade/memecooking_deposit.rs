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

type MemeId = i64;

pub struct MemeCookingDepositEvent;

impl MemeCookingDepositEvent {
    pub const ID: &'static str = "memecooking_deposit";
}

#[cfg(feature = "impl")]
impl Event for MemeCookingDepositEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> =
        Some("Fired when a deposit is made in the MemeCooking protocol.");
    const CATEGORY: &'static str = "Trade";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = MemeCookingDepositEventData;
    type RealtimeEventFilter = RtMemeCookingDepositFilter;
    type DatabaseAdapter = DbMemeCookingDepositAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MemeCookingDepositEventData {
    pub meme_id: MemeId,
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
    pub protocol_fee: Balance,
    #[schemars(with = "Option<String>")]
    pub referrer: Option<AccountId>,
    #[serde(with = "dec_format")]
    #[schemars(with = "Option<String>")]
    pub referrer_fee: Option<Balance>,
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbMemeCookingDepositFilter {
    pub meme_id: Option<MemeId>,
    #[schemars(with = "Option<String>")]
    pub trader_account_id: Option<AccountId>,
}

pub struct DbMemeCookingDepositAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbMemeCookingDepositAdapter {
    type Event = MemeCookingDepositEvent;
    type Filter = DbMemeCookingDepositFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO memecooking_deposit_testnet (timestamp, transaction_id, receipt_id, block_height, trader, meme_id, amount, protocol_fee, referrer, referrer_fee)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                event.meme_id,
                BigDecimal::from(event.amount),
                BigDecimal::from(event.protocol_fee),
                event.referrer.as_ref().map(|id| id.to_string()),
                event.referrer_fee.map(BigDecimal::from),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO memecooking_deposit (timestamp, transaction_id, receipt_id, block_height, trader, meme_id, amount, protocol_fee, referrer, referrer_fee)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.trader.to_string(),
                event.meme_id,
                BigDecimal::from(event.amount),
                BigDecimal::from(event.protocol_fee),
                event.referrer.as_ref().map(|id| id.to_string()),
                event.referrer_fee.map(BigDecimal::from),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbMemeCookingDepositFilter {
    type Event = MemeCookingDepositEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                WITH blocks AS (
                    SELECT DISTINCT timestamp as t
                    FROM memecooking_deposit_testnet
                    WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                        AND ($3::TEXT IS NULL OR trader = $3)
                        AND ($4::BIGINT IS NULL OR meme_id = $4)
                    ORDER BY t
                    LIMIT $2
                )
                SELECT transaction_id, receipt_id, block_height, timestamp, trader, meme_id, amount, protocol_fee, referrer, referrer_fee
                FROM memecooking_deposit_testnet
                INNER JOIN blocks ON timestamp = blocks.t
                WHERE ($3::TEXT IS NULL OR trader = $3)
                    AND ($4::BIGINT IS NULL OR meme_id = $4)
                ORDER BY timestamp ASC
                "#,
                pagination.start_block_timestamp_nanosec as i64,
                pagination.blocks as i64,
                self.trader_account_id.as_ref().map(|id| id.to_string()),
                self.meme_id,
            )
            .map(|record| MemeCookingDepositEventData {
                trader: record.trader.parse().unwrap(),
                transaction_id: record.transaction_id.parse().unwrap(),
                receipt_id: record.receipt_id.parse().unwrap(),
                block_height: record.block_height as u64,
                block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                meme_id: record.meme_id,
                amount: num_traits::ToPrimitive::to_u128(&record.amount).unwrap_or_else(|| {
                    log::warn!("Failed to convert number {} to u128 on {}:{}", &record.amount, file!(), line!());
                    Default::default()
                }),
                protocol_fee: num_traits::ToPrimitive::to_u128(&record.protocol_fee).unwrap_or_else(|| {
                    log::warn!("Failed to convert number {} to u128 on {}:{}", &record.protocol_fee, file!(), line!());
                    Default::default()
                }),
                referrer: record.referrer.map(|r| r.parse().unwrap()),
                referrer_fee: record.referrer_fee.map(|f| num_traits::ToPrimitive::to_u128(&f).unwrap_or_else(|| {
                    log::warn!("Failed to convert number {} to u128 on {}:{}", &f, file!(), line!());
                    Default::default()
                })),
            })
            .fetch_all(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                WITH blocks AS (
                    SELECT DISTINCT timestamp as t
                    FROM memecooking_deposit
                    WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                        AND ($3::TEXT IS NULL OR trader = $3)
                        AND ($4::BIGINT IS NULL OR meme_id = $4)
                    ORDER BY t
                    LIMIT $2
                )
                SELECT transaction_id, receipt_id, block_height, timestamp, trader, meme_id, amount, protocol_fee, referrer, referrer_fee
                FROM memecooking_deposit
                INNER JOIN blocks ON timestamp = blocks.t
                WHERE ($3::TEXT IS NULL OR trader = $3)
                    AND ($4::BIGINT IS NULL OR meme_id = $4)
                ORDER BY timestamp ASC
                "#,
                pagination.start_block_timestamp_nanosec as i64,
                pagination.blocks as i64,
                self.trader_account_id.as_ref().map(|id| id.to_string()),
                self.meme_id,
            )
            .map(|record| MemeCookingDepositEventData {
                trader: record.trader.parse().unwrap(),
                transaction_id: record.transaction_id.parse().unwrap(),
                receipt_id: record.receipt_id.parse().unwrap(),
                block_height: record.block_height as u64,
                block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                meme_id: record.meme_id,
                amount: num_traits::ToPrimitive::to_u128(&record.amount).unwrap_or_else(|| {
                    log::warn!("Failed to convert number {} to u128 on {}:{}", &record.amount, file!(), line!());
                    Default::default()
                }),
                protocol_fee: num_traits::ToPrimitive::to_u128(&record.protocol_fee).unwrap_or_else(|| {
                    log::warn!("Failed to convert number {} to u128 on {}:{}", &record.protocol_fee, file!(), line!());
                    Default::default()
                }),
                referrer: record.referrer.map(|r| r.parse().unwrap()),
                referrer_fee: record.referrer_fee.map(|f| num_traits::ToPrimitive::to_u128(&f).unwrap_or_else(|| {
                    log::warn!("Failed to convert number {} to u128 on {}:{}", &f, file!(), line!());
                    Default::default()
                })),
            })
            .fetch_all(pool)
            .await
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RtMemeCookingDepositFilter {
    pub meme_id: Option<MemeId>,
    pub trader_account_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtMemeCookingDepositFilter {
    type Event = MemeCookingDepositEvent;

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
