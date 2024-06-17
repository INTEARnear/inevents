use inindexer::near_indexer_primitives::types::{AccountId, BlockHeight};
use inindexer::near_utils::dec_format;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Pool, Postgres};

use inevents::events::event::{
    DatabaseEventAdapter, DatabaseEventFilter, Event, PaginationParameters, RealtimeEventFilter,
};
use inevents::events::types::{ReceiptId, TransactionId};

pub struct NftMintEvent;

impl Event for NftMintEvent {
    const ID: &'static str = "nft_mint";
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
    pub transaction_id: TransactionId,
    #[schemars(with = "String")]
    pub receipt_id: ReceiptId,
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
    contract_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    owner_id: Option<AccountId>,
}

pub struct DbNftMintAdapter;

impl DatabaseEventAdapter for DbNftMintAdapter {
    type Event = NftMintEvent;
    type Filter = DbNftMintFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
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

impl DatabaseEventFilter for DbNftMintFilter {
    type Event = NftMintEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        sqlx::query!(r#"
            WITH blocks AS (
                SELECT DISTINCT timestamp as t
                FROM nft_mint
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR contract_id = $3)
                    AND ($4::TEXT IS NULL OR owner_id = $4)
                ORDER BY t
                LIMIT $2
            )
            SELECT owner_id, token_ids, memo, transaction_id, receipt_id, block_height, timestamp, contract_id
            FROM nft_mint
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE ($3::TEXT IS NULL OR contract_id = $3)
                AND ($4::TEXT IS NULL OR owner_id = $4)
            ORDER BY timestamp ASC
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.contract_id.as_deref().map(|s| s.as_str()),
            self.owner_id.as_deref().map(|s| s.as_str()),
        ).map(|record| NftMintEventData {
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
pub struct RtNftMintFilter {
    contract_id: Option<AccountId>,
    owner_id: Option<AccountId>,
}

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
