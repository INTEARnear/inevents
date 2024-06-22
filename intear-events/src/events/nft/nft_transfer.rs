#[cfg(feature = "impl")]
use std::str::FromStr;

use inindexer::near_indexer_primitives::types::{AccountId, Balance, BlockHeight};
use inindexer::near_indexer_primitives::CryptoHash;
use inindexer::near_utils::{dec_format, dec_format_vec};
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

pub struct NftTransferEvent;

impl NftTransferEvent {
    pub const ID: &'static str = "nft_transfer";
}

#[cfg(feature = "impl")]
impl Event for NftTransferEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when NFTs are transferred or sold");
    const CATEGORY: &'static str = "NFT";

    type EventData = NftTransferEventData;
    type RealtimeEventFilter = RtNftTransferFilter;
    type DatabaseAdapter = DbNftTransferAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NftTransferEventData {
    #[schemars(with = "String")]
    pub old_owner_id: AccountId,
    #[schemars(with = "String")]
    pub new_owner_id: AccountId,
    pub token_ids: Vec<String>,
    pub memo: Option<String>,

    #[serde(with = "dec_format_vec")]
    #[schemars(with = "Vec<Option<String>>")]
    pub token_prices_near: Vec<Option<Balance>>,

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
pub struct DbNftTransferFilter {
    #[schemars(with = "Option<String>")]
    pub contract_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub old_owner_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub new_owner_id: Option<AccountId>,
    /// Comma-separated list of account IDs
    pub involved_account_ids: Option<String>,
}

pub struct DbNftTransferAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbNftTransferAdapter {
    type Event = NftTransferEvent;
    type Filter = DbNftTransferFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"INSERT INTO nft_transfer (timestamp, transaction_id, receipt_id, block_height, contract_id, old_owner_id, new_owner_id, token_ids, memo, token_prices_near)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.contract_id.to_string(),
            event.old_owner_id.to_string(),
            event.new_owner_id.to_string(),
            &event.token_ids,
            event.memo,
            &event.token_prices_near.iter().map(|price| price.unwrap_or_default()).map(|price| BigDecimal::from_str(&price.to_string()).unwrap()).collect::<Vec<_>>()
        ).execute(pool)
        .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbNftTransferFilter {
    type Event = NftTransferEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<<Self::Event as Event>::EventData>, sqlx::Error> {
        let involved_account_ids = self
            .involved_account_ids
            .as_ref()
            .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        sqlx::query!(r#"
            WITH blocks AS (
                SELECT DISTINCT timestamp as t
                FROM nft_transfer
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR contract_id = $3)
                    AND ($4::TEXT IS NULL OR old_owner_id = $4)
                    AND ($5::TEXT IS NULL OR new_owner_id = $5)
                    AND ($6::TEXT[] IS NULL OR ARRAY[old_owner_id, new_owner_id] @> $6)
                ORDER BY t
                LIMIT $2
            )
            SELECT old_owner_id, new_owner_id, token_ids, memo, token_prices_near, transaction_id, receipt_id, block_height, timestamp, contract_id
            FROM nft_transfer
            INNER JOIN blocks ON timestamp = blocks.t
            WHERE ($3::TEXT IS NULL OR contract_id = $3)
                AND ($4::TEXT IS NULL OR old_owner_id = $4)
                AND ($5::TEXT IS NULL OR new_owner_id = $5)
                AND ($6::TEXT IS NULL OR ARRAY[old_owner_id, new_owner_id] @> $6)
            ORDER BY timestamp ASC
            "#,
            pagination.start_block_timestamp_nanosec as i64,
            pagination.blocks as i64,
            self.contract_id.as_deref().map(|s| s.as_str()),
            self.old_owner_id.as_deref().map(|s| s.as_str()),
            self.new_owner_id.as_deref().map(|s| s.as_str()),
            involved_account_ids.as_slice(),
        ).map(|record| NftTransferEventData {
            old_owner_id: record.old_owner_id.parse().unwrap(),
            new_owner_id: record.new_owner_id.parse().unwrap(),
            token_ids: record.token_ids,
            memo: record.memo,
            token_prices_near: record.token_prices_near.iter().map(|price| if price.to_string() == "0" { None } else { Some(price.to_string().parse().unwrap()) }).collect(),
            transaction_id: record.transaction_id.parse().unwrap(),
            receipt_id: record.receipt_id.parse().unwrap(),
            block_height: record.block_height as u64,
            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
            contract_id: record.contract_id.parse().unwrap(),
        }).fetch_all(pool).await
    }
}

#[derive(Debug, Deserialize)]
pub struct RtNftTransferFilter {
    pub contract_id: Option<AccountId>,
    pub old_owner_id: Option<AccountId>,
    pub new_owner_id: Option<AccountId>,
    pub involved_account_ids: Option<Vec<AccountId>>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtNftTransferFilter {
    type Event = NftTransferEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(old_owner_id) = &self.old_owner_id {
            if event.old_owner_id != *old_owner_id {
                return false;
            }
        }

        if let Some(new_owner_id) = &self.new_owner_id {
            if event.new_owner_id != *new_owner_id {
                return false;
            }
        }

        if let Some(involved_account_ids) = &self.involved_account_ids {
            if !involved_account_ids.contains(&event.old_owner_id)
                && !involved_account_ids.contains(&event.new_owner_id)
            {
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
