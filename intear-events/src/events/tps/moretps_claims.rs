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

pub struct MoreTpsClaimEvent;

impl MoreTpsClaimEvent {
    pub const ID: &'static str = "moretps";
}

#[cfg(feature = "impl")]
impl Event for MoreTpsClaimEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("MoreTPS claims");
    const CATEGORY: &'static str = "TPS";
    const EXCLUDE_FROM_DATABASE: bool = false;
    const SUPPORTS_TESTNET: bool = true;

    type EventData = MoreTpsClaimEventData;
    type RealtimeEventFilter = RtMoreTpsClaimFilter;
    type DatabaseAdapter = DbMoreTpsClaimAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MoreTpsClaimEventData {
    pub block_height: BlockHeight,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub block_timestamp_nanosec: u128,
    #[schemars(with = "String")]
    pub receipt_id: CryptoHash,
    #[schemars(with = "String")]
    pub claimed_account_id: AccountId,
    #[schemars(with = "String")]
    pub claimed_parent_account_id: AccountId,
    #[schemars(with = "String")]
    pub round_account_id: AccountId,
    #[schemars(with = "String")]
    pub round_parent_account_id: AccountId,
    pub is_success: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DbMoreTpsClaimFilter {
    #[schemars(with = "Option<String>")]
    pub claim_account_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub claim_parent_account_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub round_account_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub round_parent_account_id: Option<AccountId>,
}

pub struct DbMoreTpsClaimAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbMoreTpsClaimAdapter {
    type Event = MoreTpsClaimEvent;
    type Filter = DbMoreTpsClaimFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO moretps_testnet (timestamp, receipt_id, claimed_account_id, claimed_parent_account_id, round_account_id, round_parent_account_id, is_success)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.receipt_id.to_string(),
                event.claimed_account_id.to_string(),
                event.claimed_parent_account_id.to_string(),
                event.round_account_id.to_string(),
                event.round_parent_account_id.to_string(),
                event.is_success
            )
            .execute(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO moretps (timestamp, receipt_id, claimed_account_id, claimed_parent_account_id, round_account_id, round_parent_account_id, is_success)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.receipt_id.to_string(),
                event.claimed_account_id.to_string(),
                event.claimed_parent_account_id.to_string(),
                event.round_account_id.to_string(),
                event.round_parent_account_id.to_string(),
                event.is_success
            )
            .execute(pool)
            .await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbMoreTpsClaimFilter {
    type Event = MoreTpsClaimEvent;

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
                    FROM moretps_testnet
                    WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                        AND ($3::TEXT IS NULL OR claimed_account_id = $3)
                        AND ($4::TEXT IS NULL OR claimed_parent_account_id = $4)
                        AND ($5::TEXT IS NULL OR round_account_id = $5)
                        AND ($6::TEXT IS NULL OR round_parent_account_id = $6)
                    ORDER BY t
                    LIMIT $2
                )
                SELECT timestamp, block_height, receipt_id, claimed_account_id, claimed_parent_account_id, round_account_id, round_parent_account_id, is_success
                FROM moretps_testnet
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR claimed_account_id = $3)
                    AND ($4::TEXT IS NULL OR claimed_parent_account_id = $4)
                    AND ($5::TEXT IS NULL OR round_account_id = $5)
                    AND ($6::TEXT IS NULL OR round_parent_account_id = $6)
                ORDER BY timestamp ASC
                "#,
                pagination.start_block_timestamp_nanosec as i64,
                pagination.blocks as i64,
                self.claim_account_id.as_ref().map(|s| s.as_str()),
                self.claim_parent_account_id.as_ref().map(|s| s.as_str()),
                self.round_account_id.as_ref().map(|s| s.as_str()),
                self.round_parent_account_id.as_ref().map(|s| s.as_str())
            )
            .map(|record| MoreTpsClaimEventData {
                block_height: record.block_height as u64,
                block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                receipt_id: record.receipt_id.parse().unwrap(),
                claimed_account_id: record.claimed_account_id.parse().unwrap(),
                claimed_parent_account_id: record.claimed_parent_account_id.parse().unwrap(),
                round_account_id: record.round_account_id.parse().unwrap(),
                round_parent_account_id: record.round_parent_account_id.parse().unwrap(),
                is_success: record.is_success
            })
            .fetch_all(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                WITH blocks AS (
                    SELECT DISTINCT timestamp as t
                    FROM moretps
                    WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                        AND ($3::TEXT IS NULL OR claimed_account_id = $3)
                        AND ($4::TEXT IS NULL OR claimed_parent_account_id = $4)
                        AND ($5::TEXT IS NULL OR round_account_id = $5)
                        AND ($6::TEXT IS NULL OR round_parent_account_id = $6)
                    ORDER BY t
                    LIMIT $2
                )
                SELECT timestamp, block_height, receipt_id, claimed_account_id, claimed_parent_account_id, round_account_id, round_parent_account_id, is_success
                FROM moretps
                WHERE extract(epoch from timestamp) * 1_000_000_000 >= $1
                    AND ($3::TEXT IS NULL OR claimed_account_id = $3)
                    AND ($4::TEXT IS NULL OR claimed_parent_account_id = $4)
                    AND ($5::TEXT IS NULL OR round_account_id = $5)
                    AND ($6::TEXT IS NULL OR round_parent_account_id = $6)
                ORDER BY timestamp ASC
                "#,
                pagination.start_block_timestamp_nanosec as i64,
                pagination.blocks as i64,
                self.claim_account_id.as_ref().map(|s| s.as_str()),
                self.claim_parent_account_id.as_ref().map(|s| s.as_str()),
                self.round_account_id.as_ref().map(|s| s.as_str()),
                self.round_parent_account_id.as_ref().map(|s| s.as_str())
            )
            .map(|record| MoreTpsClaimEventData {
                block_height: record.block_height as u64,
                block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                receipt_id: record.receipt_id.parse().unwrap(),
                claimed_account_id: record.claimed_account_id.parse().unwrap(),
                claimed_parent_account_id: record.claimed_parent_account_id.parse().unwrap(),
                round_account_id: record.round_account_id.parse().unwrap(),
                round_parent_account_id: record.round_parent_account_id.parse().unwrap(),
                is_success: record.is_success
            })
            .fetch_all(pool)
            .await
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RtMoreTpsClaimFilter {
    pub block_height_equal: Option<BlockHeight>,
    pub block_height_until: Option<BlockHeight>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtMoreTpsClaimFilter {
    type Event = MoreTpsClaimEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(block_height_equal) = self.block_height_equal {
            if event.block_height != block_height_equal {
                return false;
            }
        }

        if let Some(block_height_until) = self.block_height_until {
            if event.block_height > block_height_until {
                return false;
            }
        }

        true
    }
}
