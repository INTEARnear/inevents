#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
use inindexer::near_indexer_primitives::types::{AccountId, Balance, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
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

pub struct NewMemeCookingMemeEvent;

impl NewMemeCookingMemeEvent {
    pub const ID: &'static str = "newtoken_memecooking";
}

#[cfg(feature = "impl")]
impl Event for NewMemeCookingMemeEvent {
    const ID: &'static str = Self::ID;
    const CATEGORY: &'static str = "New Tokens";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = NewMemeCookingMemeEventData;
    type RealtimeEventFilter = RtNewMemeCookingMemeFilter;
    type DatabaseAdapter = DbNewMemeCookingMemeAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewMemeCookingMemeEventData {
    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,

    pub meme_id: u64,
    #[schemars(with = "String")]
    pub owner: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub end_timestamp_ms: u64,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub total_supply: Balance,
    pub reference: String,
    pub reference_hash: String,
    #[schemars(with = "String")]
    pub deposit_token_id: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub soft_cap: Balance,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub hard_cap: Balance,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NewMemeCookingMemeFilter {
    pub meme_id: Option<u64>,
    #[schemars(with = "Option<String>")]
    pub owner: Option<AccountId>,
}

pub struct DbNewMemeCookingMemeAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbNewMemeCookingMemeAdapter {
    type Event = NewMemeCookingMemeEvent;
    type Filter = NewMemeCookingMemeFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO new_memecooking_meme_testnet (timestamp, transaction_id, receipt_id, block_height, meme_id, owner, end_timestamp_ms, name, symbol, decimals, total_supply, reference, reference_hash, deposit_token_id, soft_cap, hard_cap)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.meme_id as i64,
                event.owner.as_str(),
                event.end_timestamp_ms as i64,
                event.name,
                event.symbol,
                event.decimals as i32,
                BigDecimal::from(event.total_supply),
                event.reference,
                event.reference_hash,
                event.deposit_token_id.as_str(),
                BigDecimal::from(event.soft_cap),
                BigDecimal::from(event.hard_cap),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO new_memecooking_meme_testnet (timestamp, transaction_id, receipt_id, block_height, meme_id, owner, end_timestamp_ms, name, symbol, decimals, total_supply, reference, reference_hash, deposit_token_id, soft_cap, hard_cap)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.meme_id as i64,
                event.owner.as_str(),
                event.end_timestamp_ms as i64,
                event.name,
                event.symbol,
                event.decimals as i32,
                BigDecimal::from(event.total_supply),
                event.reference,
                event.reference_hash,
                event.deposit_token_id.as_str(),
                BigDecimal::from(event.soft_cap),
                BigDecimal::from(event.hard_cap),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for NewMemeCookingMemeFilter {
    type Event = NewMemeCookingMemeEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlNewMemeCookingMemeEventData {
            id: i64,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            meme_id: i64,
            owner: String,
            end_timestamp_ms: i64,
            name: String,
            symbol: String,
            decimals: i32,
            total_supply: BigDecimal,
            reference: String,
            reference_hash: String,
            deposit_token_id: String,
            soft_cap: BigDecimal,
            hard_cap: BigDecimal,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let meme_id = self.meme_id.map(|meme_id| meme_id as i64);
        let owner = self.owner.as_ref().map(|o| o.as_str());
        sqlx_conditional_queries::conditional_query_as!(
            SqlNewMemeCookingMemeEventData,
            r#"
            SELECT *
            FROM new_memecooking_meme{#testnet}
            WHERE {#time}
                AND ({meme_id}::BIGINT IS NULL OR meme_id = {meme_id})
                AND ({owner}::TEXT IS NULL OR owner = {owner})
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
                        NewMemeCookingMemeEventData {
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            meme_id: record.meme_id as u64,
                            owner: record.owner.parse().unwrap(),
                            end_timestamp_ms: record.end_timestamp_ms as u64,
                            name: record.name,
                            symbol: record.symbol,
                            decimals: record.decimals as u32,
                            total_supply: num_traits::ToPrimitive::to_u128(&record.total_supply)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.total_supply,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            reference: record.reference,
                            reference_hash: record.reference_hash,
                            deposit_token_id: record.deposit_token_id.parse().unwrap(),
                            soft_cap: num_traits::ToPrimitive::to_u128(&record.soft_cap)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.soft_cap,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            hard_cap: num_traits::ToPrimitive::to_u128(&record.hard_cap)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.hard_cap,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtNewMemeCookingMemeFilter {
    #[serde(with = "dec_format")]
    pub meme_id: Option<u64>,
    pub owner: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtNewMemeCookingMemeFilter {
    type Event = NewMemeCookingMemeEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(meme_id) = self.meme_id {
            if meme_id != event.meme_id {
                return false;
            }
        }

        if let Some(owner) = &self.owner {
            if owner != &event.owner {
                return false;
            }
        }

        true
    }
}
