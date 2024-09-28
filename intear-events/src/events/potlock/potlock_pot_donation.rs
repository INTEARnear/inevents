use chrono::{DateTime, Utc};
#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
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

pub struct PotlockPotDonationEvent;

type DonationId = u64;
type PotId = AccountId;

impl PotlockPotDonationEvent {
    pub const ID: &'static str = "potlock_pot_donation";
}

#[cfg(feature = "impl")]
impl Event for PotlockPotDonationEvent {
    const ID: &'static str = Self::ID;
    const CATEGORY: &'static str = "Potlock";

    type EventData = PotlockPotDonationEventData;
    type RealtimeEventFilter = RtPotlockPotDonationFilter;
    type DatabaseAdapter = DbPotlockPotDonationAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PotlockPotDonationEventData {
    pub donation_id: DonationId,
    #[schemars(with = "String")]
    pub pot_id: PotId,
    #[schemars(with = "String")]
    pub donor_id: AccountId,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub total_amount: Balance,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub net_amount: Balance,
    pub message: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    #[schemars(with = "u64")]
    pub donated_at: DateTime<Utc>,
    #[serde(with = "dec_format")]
    #[schemars(with = "String")]
    pub protocol_fee: Balance,
    #[schemars(with = "Option<String>")]
    pub referrer_id: Option<AccountId>,
    #[serde(with = "dec_format")]
    #[schemars(with = "Option<String>")]
    pub referrer_fee: Option<Balance>,
    #[schemars(with = "Option<String>")]
    pub chef_id: Option<AccountId>,
    #[serde(with = "dec_format")]
    #[schemars(with = "Option<String>")]
    pub chef_fee: Option<Balance>,

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
pub struct DbPotlockPotDonationFilter {
    #[schemars(with = "Option<String>")]
    pub pot_id: Option<PotId>,
    #[schemars(with = "Option<String>")]
    pub donor_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub referrer_id: Option<AccountId>,
}

pub struct DbPotlockPotDonationAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbPotlockPotDonationAdapter {
    type Event = PotlockPotDonationEvent;
    type Filter = DbPotlockPotDonationFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO potlock_pot_donation (timestamp, transaction_id, receipt_id, block_height, donation_id, pot_id, donor_id, total_amount, net_amount, message, donated_at, protocol_fee, referrer_id, referrer_fee, chef_id, chef_fee)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
            chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.donation_id as i64,
            event.pot_id.to_string(),
            event.donor_id.to_string(),
            BigDecimal::from(event.total_amount),
            BigDecimal::from(event.net_amount),
            event.message.clone(),
            event.donated_at,
            BigDecimal::from(event.protocol_fee),
            event.referrer_id.as_ref().map(|id| id.to_string()),
            event.referrer_fee.map(|fee| BigDecimal::from(fee)),
            event.chef_id.as_ref().map(|id| id.to_string()),
            event.chef_fee.map(|fee| BigDecimal::from(fee))
        ).execute(pool).await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbPotlockPotDonationFilter {
    type Event = PotlockPotDonationEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlPotlockPotDonationEventData {
            id: i64,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            donation_id: i64,
            pot_id: String,
            donor_id: String,
            total_amount: BigDecimal,
            net_amount: BigDecimal,
            message: Option<String>,
            donated_at: chrono::DateTime<chrono::Utc>,
            protocol_fee: BigDecimal,
            referrer_id: Option<String>,
            referrer_fee: Option<BigDecimal>,
            chef_id: Option<String>,
            chef_fee: Option<BigDecimal>,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let pot_id = self.pot_id.as_ref().map(|id| id.as_str());
        let donor_id = self.donor_id.as_ref().map(|id| id.as_str());
        let referrer_id = self.referrer_id.as_ref().map(|id| id.as_str());

        sqlx_conditional_queries::conditional_query_as!(
            SqlPotlockPotDonationEventData,
            r#"
            SELECT *
            FROM potlock_pot_donation{#testnet}
            WHERE {#time}
                AND ({pot_id}::TEXT IS NULL OR pot_id = {pot_id})
                AND ({donor_id}::TEXT IS NULL OR donor_id = {donor_id})
                AND ({referrer_id}::TEXT IS NULL OR referrer_id = {referrer_id})
            ORDER BY timestamp {#order}
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
                        PotlockPotDonationEventData {
                            donation_id: record.donation_id as u64,
                            pot_id: record.pot_id.parse().unwrap(),
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
                            net_amount: num_traits::ToPrimitive::to_u128(&record.net_amount)
                                .unwrap_or_else(|| {
                                    log::warn!(
                                        "Failed to convert number {} to u128 on {}:{}",
                                        &record.net_amount,
                                        file!(),
                                        line!()
                                    );
                                    Default::default()
                                }),
                            message: record.message,
                            donated_at: record.donated_at,
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
                            chef_id: record.chef_id.as_ref().map(|id| id.parse().unwrap()),
                            chef_fee: record.chef_fee.map(|fee| {
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
pub struct RtPotlockPotDonationFilter {
    pub pot_id: Option<String>,
    pub donor_id: Option<AccountId>,
    pub referrer_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtPotlockPotDonationFilter {
    type Event = PotlockPotDonationEvent;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(pot_id) = &self.pot_id {
            if event.pot_id != *pot_id {
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

        true
    }
}
