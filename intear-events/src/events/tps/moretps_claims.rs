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
    pub transaction_id: CryptoHash,
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
                INSERT INTO moretps_testnet (timestamp, block_height, receipt_id, transaction_id, claimed_account_id, claimed_parent_account_id, round_account_id, round_parent_account_id, is_success)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_height as i64,
                event.receipt_id.to_string(),
                event.transaction_id.to_string(),
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
                INSERT INTO moretps (timestamp, block_height, receipt_id, transaction_id, claimed_account_id, claimed_parent_account_id, round_account_id, round_parent_account_id, is_success)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                chrono::DateTime::from_timestamp(
                    (event.block_timestamp_nanosec / 1_000_000_000) as i64,
                    (event.block_timestamp_nanosec % 1_000_000_000) as u32
                ),
                event.block_height as i64,
                event.receipt_id.to_string(),
                event.transaction_id.to_string(),
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
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlMoreTpsClaimEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            block_height: i64,
            receipt_id: String,
            transaction_id: String,
            claimed_account_id: String,
            claimed_parent_account_id: String,
            round_account_id: String,
            round_parent_account_id: String,
            is_success: bool,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let claim_account_id = self.claim_account_id.as_ref().map(|id| id.as_str());
        let claim_parent_account_id = self.claim_parent_account_id.as_ref().map(|id| id.as_str());
        let round_account_id = self.round_account_id.as_ref().map(|id| id.as_str());
        let round_parent_account_id = self.round_parent_account_id.as_ref().map(|id| id.as_str());

        sqlx_conditional_queries::conditional_query_as!(
            SqlMoreTpsClaimEventData,
            r#"
            SELECT *
            FROM moretps{#testnet}
            WHERE {#time}
                AND ({claim_account_id}::TEXT IS NULL OR claimed_account_id = {claim_account_id})
                AND ({claim_parent_account_id}::TEXT IS NULL OR claimed_parent_account_id = {claim_parent_account_id})
                AND ({round_account_id}::TEXT IS NULL OR round_account_id = {round_account_id})
                AND ({round_parent_account_id}::TEXT IS NULL OR round_parent_account_id = {round_parent_account_id})
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
                    MoreTpsClaimEventData {
                        block_height: record.block_height as u64,
                        block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap() as u128,
                        receipt_id: record.receipt_id.parse().unwrap(),
                        transaction_id: record.transaction_id.parse().unwrap(),
                        claimed_account_id: record.claimed_account_id.parse().unwrap(),
                        claimed_parent_account_id: record.claimed_parent_account_id.parse().unwrap(),
                        round_account_id: record.round_account_id.parse().unwrap(),
                        round_parent_account_id: record.round_parent_account_id.parse().unwrap(),
                        is_success: record.is_success,
                    },
                )
            })
            .collect()
        })
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
