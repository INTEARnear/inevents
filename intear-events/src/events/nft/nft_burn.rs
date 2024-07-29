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

pub struct NftBurnEvent;

impl NftBurnEvent {
    pub const ID: &'static str = "nft_burn";
}

#[cfg(feature = "impl")]
impl Event for NftBurnEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when NFTs are burned");
    const CATEGORY: &'static str = "NFT";

    type EventData = NftBurnEventData;
    type RealtimeEventFilter = RtNftBurnFilter;
    type DatabaseAdapter = DbNftBurnAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NftBurnEventData {
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
pub struct DbNftBurnFilter {
    #[schemars(with = "Option<String>")]
    pub contract_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub owner_id: Option<AccountId>,
}

pub struct DbNftBurnAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbNftBurnAdapter {
    type Event = NftBurnEvent;
    type Filter = DbNftBurnFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"INSERT INTO nft_burn (timestamp, transaction_id, receipt_id, block_height, contract_id, owner_id, token_ids, memo)
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
impl DatabaseEventFilter for DbNftBurnFilter {
    type Event = NftBurnEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        sqlx::query!(r#"
            WITH blocks AS (
                SELECT DISTINCT timestamp as t
                FROM nft_burn
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR contract_id = $3)
                    AND ($4::TEXT IS NULL OR owner_id = $4)
                ORDER BY t
                LIMIT $2
            )
            SELECT owner_id, token_ids, memo, transaction_id, receipt_id, block_height, timestamp, contract_id
            FROM nft_burn
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE ($3::TEXT IS NULL OR contract_id = $3)
                AND ($4::TEXT IS NULL OR owner_id = $4)
            ORDER BY timestamp ASC
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.contract_id.as_deref().map(|s| s.as_str()),
            self.owner_id.as_deref().map(|s| s.as_str()),
        ).map(|record| NftBurnEventData {
            owner_id: record.owner_id.parse().unwrap(),
            token_ids: record.token_ids,
            memo: record.memo,
            transaction_id: record.transaction_id.parse().unwrap(),
            receipt_id: record.receipt_id.parse().unwrap(),
            block_height: record.block_height as u64,
            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
            contract_id: record.contract_id.parse().unwrap(),
        }).fetch_all(pool).await
    }
}

#[derive(Debug, Deserialize)]
pub struct RtNftBurnFilter {
    pub contract_id: Option<AccountId>,
    pub owner_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtNftBurnFilter {
    type Event = NftBurnEvent;

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
