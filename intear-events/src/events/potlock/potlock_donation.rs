use inevents::events::event::{EventId, PaginationBy};
use std::collections::HashMap;

use chrono::{DateTime, Utc};
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

pub struct PotlockDonationEvent;

type DonationId = u64;
type ProjectId = AccountId;

impl PotlockDonationEvent {
    pub const ID: &'static str = "potlock_donation";
}

#[cfg(feature = "impl")]
impl Event for PotlockDonationEvent {
    const ID: &'static str = Self::ID;
    const CATEGORY: &'static str = "Potlock";

    type EventData = PotlockDonationEventData;
    type RealtimeEventFilter = RtPotlockDonationFilter;
    type DatabaseAdapter = DbPotlockDonationAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PotlockDonationEventData {
    pub donation_id: DonationId,
    #[schemars(with = "String")]
    pub donor_id: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub total_amount: Balance,
    #[schemars(with = "String")]
    pub ft_id: AccountId,
    pub message: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    #[schemars(with = "u64")]
    pub donated_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub project_id: ProjectId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub protocol_fee: Balance,
    #[schemars(with = "Option<String>")]
    pub referrer_id: Option<AccountId>,
    #[serde(with = "dec_format")]
    #[schemars(with = "Option<String>")]
    pub referrer_fee: Option<Balance>,

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
pub struct DbPotlockDonationFilter {
    #[schemars(with = "Option<String>")]
    pub project_id: Option<ProjectId>,
    #[schemars(with = "Option<String>")]
    pub donor_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub referrer_id: Option<AccountId>,
}

pub struct DbPotlockDonationAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbPotlockDonationAdapter {
    type Event = PotlockDonationEvent;
    type Filter = DbPotlockDonationFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO potlock_donation (timestamp, transaction_id, receipt_id, block_height, donation_id, donor_id, total_amount, ft_id, message, donated_at, project_id, protocol_fee, referrer_id, referrer_fee)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.donation_id as i64,
            event.donor_id.to_string(),
            BigDecimal::from(event.total_amount),
            event.ft_id.to_string(),
            event.message,
            event.donated_at,
            event.project_id.to_string(),
            BigDecimal::from(event.protocol_fee),
            event.referrer_id.as_ref().map(|id| id.to_string()),
            event.referrer_fee.map(|fee| BigDecimal::from(fee))
        ).execute(pool).await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbPotlockDonationFilter {
    type Event = PotlockDonationEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        let limit = pagination.limit as i64;

        #[derive(Debug, sqlx::FromRow)]
        struct SqlPotlockDonationEventData {
            id: i64,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            donation_id: i64,
            donor_id: String,
            total_amount: BigDecimal,
            ft_id: String,
            message: Option<String>,
            donated_at: chrono::DateTime<chrono::Utc>,
            project_id: String,
            protocol_fee: BigDecimal,
            referrer_id: Option<String>,
            referrer_fee: Option<BigDecimal>,
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

        let project_id = self.project_id.as_ref().map(|id| id.as_str());
        let donor_id = self.donor_id.as_ref().map(|id| id.as_str());
        let referrer_id = self.referrer_id.as_ref().map(|id| id.as_str());

        sqlx_conditional_queries::conditional_query_as!(
            SqlPotlockDonationEventData,
            r#"
            SELECT *
            FROM potlock_donation{#testnet}
            WHERE {#time}
                AND ({project_id}::TEXT IS NULL OR project_id = {project_id})
                AND ({donor_id}::TEXT IS NULL OR donor_id = {donor_id})
                AND ({referrer_id}::TEXT IS NULL OR referrer_id = {referrer_id})
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
                        PotlockDonationEventData {
                            donation_id: record.donation_id as u64,
                            donor_id: record.donor_id.parse().unwrap(),
                            total_amount: num_traits::ToPrimitive::to_u128(&record.total_amount)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.total_amount,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            ft_id: record.ft_id.parse().unwrap(),
                            message: record.message,
                            donated_at: record.donated_at,
                            project_id: record.project_id.parse().unwrap(),
                            protocol_fee: num_traits::ToPrimitive::to_u128(&record.protocol_fee)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.protocol_fee,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            referrer_id: record.referrer_id.as_ref().map(|id| id.parse().unwrap()),
                            referrer_fee: record.referrer_fee.map(|fee| {
                                num_traits::ToPrimitive::to_u128(&fee).unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &fee,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                })
                            }),
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtPotlockDonationFilter {
    pub project_id: Option<ProjectId>,
    pub donor_id: Option<AccountId>,
    pub referrer_id: Option<AccountId>,
    pub min_amounts: Option<HashMap<AccountId, Balance>>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtPotlockDonationFilter {
    type Event = PotlockDonationEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(project_id) = &self.project_id {
            if event.project_id != *project_id {
                return false;
            }
        }

        if let Some(donor_id) = &self.donor_id {
            if event.donor_id != *donor_id {
                return false;
            }
        }

        if let Some(referrer_id) = &self.referrer_id {
            if event.referrer_id.as_ref() != Some(referrer_id) {
                return false;
            }
        }

        if let Some(min_amounts) = &self.min_amounts {
            if let Some(min_amount) = min_amounts.get(&event.donor_id) {
                if event.total_amount < *min_amount {
                    return false;
                }
            }
        }

        true
    }
}
