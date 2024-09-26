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

pub struct NewMemeCookingTokenEvent;

impl NewMemeCookingTokenEvent {
    pub const ID: &'static str = "newtoken_memecooking_token";
}

#[cfg(feature = "impl")]
impl Event for NewMemeCookingTokenEvent {
    const ID: &'static str = Self::ID;
    const CATEGORY: &'static str = "New Tokens";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = NewMemeCookingTokenEventData;
    type RealtimeEventFilter = RtNewMemeCookingMemeFilter;
    type DatabaseAdapter = DbNewMemeCookingMemeAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewMemeCookingTokenEventData {
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
    pub token_id: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub total_supply: Balance,
    pub pool_id: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NewMemeCookingMemeFilter {
    pub meme_id: Option<u64>,
    #[schemars(with = "Option<String>")]
    pub token_id: Option<AccountId>,
}

pub struct DbNewMemeCookingMemeAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbNewMemeCookingMemeAdapter {
    type Event = NewMemeCookingTokenEvent;
    type Filter = NewMemeCookingMemeFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO memecooking_create_token_testnet (timestamp, transaction_id, receipt_id, block_height, meme_id, token_id, total_supply, pool_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.meme_id as i64,
                event.token_id.to_string(),
                BigDecimal::from(event.total_supply),
                event.pool_id as i64,
            )
            .execute(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO memecooking_create_token (timestamp, transaction_id, receipt_id, block_height, meme_id, token_id, total_supply, pool_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.meme_id as i64,
                event.token_id.to_string(),
                BigDecimal::from(event.total_supply),
                event.pool_id as i64,
            )
            .execute(pool)
            .await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for NewMemeCookingMemeFilter {
    type Event = NewMemeCookingTokenEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlNewMemeCookingTokenEventData {
            id: i64,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            meme_id: i64,
            token_id: String,
            total_supply: BigDecimal,
            pool_id: i64,
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

        let meme_id = self.meme_id.map(|meme_id| meme_id as i64);
        let token_id = self.token_id.as_ref().map(|id| id.as_str());
        sqlx_conditional_queries::conditional_query_as!(
            SqlNewMemeCookingTokenEventData,
            r#"
            SELECT *
            FROM memecooking_create_token{#testnet}
            WHERE {#time}
            AND ({meme_id}::BIGINT IS NULL OR meme_id = {meme_id})
            AND ({token_id}::TEXT IS NULL OR token_id = {token_id})
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
                        NewMemeCookingTokenEventData {
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            meme_id: record.meme_id as u64,
                            token_id: record.token_id.parse().unwrap(),
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
                            pool_id: record.pool_id as u64,
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
    pub token_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtNewMemeCookingMemeFilter {
    type Event = NewMemeCookingTokenEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(meme_id) = self.meme_id {
            if meme_id != event.meme_id {
                return false;
            }
        }

        if let Some(token_id) = &self.token_id {
            if token_id != &event.token_id {
                return false;
            }
        }

        true
    }
}
