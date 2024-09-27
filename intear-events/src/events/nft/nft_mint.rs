#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
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

pub struct NftMintEvent;

impl NftMintEvent {
    pub const ID: &'static str = "nft_mint";
}

#[cfg(feature = "impl")]
impl Event for NftMintEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when NFTs are minted");
    const CATEGORY: &'static str = "NFT";

    type EventData = NftMintEventData;
    type RealtimeEventFilter = RtNftMintFilter;
    type DatabaseAdapter = DbNftMintAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NftMintEventData {
    #[schemars(with = "String")]
    pub owner_id: AccountId,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,

    #[schemars(with = "String")]
    pub transaction_id: CryptoHash,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    #[schemars(with = "String")]
    pub contract_id: AccountId,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbNftMintFilter {
    #[schemars(with = "Option<String>")]
    pub contract_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub owner_id: Option<AccountId>,
}

pub struct DbNftMintAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbNftMintAdapter {
    type Event = NftMintEvent;
    type Filter = DbNftMintFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"INSERT INTO nft_mint (timestamp, transaction_id, receipt_id, block_height, contract_id, owner_id, token_ids, memo)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.contract_id.to_string(),
                event.owner_id.to_string(),
                &event.token_ids,
                event.memo
            )
            .execute(pool)
            .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbNftMintFilter {
    type Event = NftMintEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlNftMintEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            block_height: i64,
            contract_id: String,
            owner_id: String,
            token_ids: Vec<String>,
            memo: Option<String>,
            transaction_id: String,
            receipt_id: String,
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

        let contract_id = self.contract_id.as_ref().map(|c| c.as_str());
        let owner_id = self.owner_id.as_ref().map(|o| o.as_str());
        sqlx_conditional_queries::conditional_query_as!(
            SqlNftMintEventData,
            r#"
            SELECT *
            FROM nft_mint{#testnet}
            WHERE {#time}
                AND ({contract_id}::TEXT IS NULL OR contract_id = {contract_id})
                AND ({owner_id}::TEXT IS NULL OR owner_id = {owner_id})
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
                        NftMintEventData {
                            owner_id: record.owner_id.parse().unwrap(),
                            token_ids: record.token_ids,
                            memo: record.memo,
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            contract_id: record.contract_id.parse().unwrap(),
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtNftMintFilter {
    pub contract_id: Option<AccountId>,
    pub owner_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtNftMintFilter {
    type Event = NftMintEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(owner_id) = &self.owner_id {
            if event.owner_id != *owner_id {
                return false;
            }
        }

        if let Some(contract_id) = &self.contract_id {
            if event.contract_id != *contract_id {
                return false;
            }
        }

        true
    }
}
