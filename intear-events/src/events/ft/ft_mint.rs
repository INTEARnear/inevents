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

pub struct FtMintEvent;

impl FtMintEvent {
    pub const ID: &'static str = "ft_mint";
}

#[cfg(feature = "impl")]
impl Event for FtMintEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when FTs are minted");
    const CATEGORY: &'static str = "FT";

    type EventData = FtMintEventData;
    type RealtimeEventFilter = RtFtMintFilter;
    type DatabaseAdapter = DbFtMintAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FtMintEventData {
    #[schemars(with = "String")]
    pub owner_id: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub amount: Balance,
    pub memo: Option<String>,
    #[schemars(with = "String")]
    pub token_id: AccountId,

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
pub struct DbFtMintFilter {
    #[schemars(with = "Option<String>")]
    pub token_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub owner_id: Option<AccountId>,
    #[serde(with = "dec_format", default)]
    #[schemars(with = "Option<String>")]
    pub amount: Option<Balance>,
    #[serde(with = "dec_format", default)]
    #[schemars(with = "Option<String>")]
    pub min_amount: Option<Balance>,
}

pub struct DbFtMintAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbFtMintAdapter {
    type Event = FtMintEvent;
    type Filter = DbFtMintFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO ft_mint (timestamp, transaction_id, receipt_id, block_height, token_id, owner_id, amount, memo)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.token_id.to_string(),
            event.owner_id.to_string(),
            BigDecimal::from(event.amount),
            event.memo
        )
        .execute(pool)
        .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbFtMintFilter {
    type Event = FtMintEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlFtMintEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            block_height: i64,
            token_id: String,
            owner_id: String,
            amount: BigDecimal,
            memo: Option<String>,
            transaction_id: String,
            receipt_id: String,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let token_id = self.token_id.as_ref().map(|c| c.as_str());
        let owner_id = self.owner_id.as_ref().map(|o| o.as_str());
        let amount = self.amount.as_ref().map(BigDecimal::from);
        let min_amount = self.min_amount.as_ref().map(BigDecimal::from);
        sqlx_conditional_queries::conditional_query_as!(
            SqlFtMintEventData,
            r#"
            SELECT *
            FROM ft_mint{#testnet}
            WHERE {#time}
                AND ({token_id}::TEXT IS NULL OR token_id = {token_id})
                AND ({owner_id}::TEXT IS NULL OR owner_id = {owner_id})
                AND ({amount}::NUMERIC IS NULL OR amount = {amount})
                AND ({min_amount}::NUMERIC IS NULL OR amount >= {min_amount})
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
                        FtMintEventData {
                            owner_id: record.owner_id.parse().unwrap(),
                            amount: num_traits::ToPrimitive::to_u128(&record.amount)
                                .unwrap_or_default(),
                            memo: record.memo,
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            token_id: record.token_id.parse().unwrap(),
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtFtMintFilter {
    pub token_id: Option<AccountId>,
    pub owner_id: Option<AccountId>,
    #[serde(with = "dec_format")]
    pub amount: Option<Balance>,
    #[serde(with = "dec_format")]
    pub min_amount: Option<Balance>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtFtMintFilter {
    type Event = FtMintEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(owner_id) = &self.owner_id {
            if event.owner_id != *owner_id {
                return false;
            }
        }

        if let Some(token_id) = &self.token_id {
            if event.token_id != *token_id {
                return false;
            }
        }

        if let Some(amount) = &self.amount {
            if event.amount != *amount {
                return false;
            }
        }

        if let Some(min_amount) = &self.min_amount {
            if event.amount < *min_amount {
                return false;
            }
        }

        true
    }
}
