#[cfg(feature = "impl")]
use inevents::events::event::{EventId, PaginationBy};
#[cfg(feature = "impl")]
use std::str::FromStr;

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

pub struct NewContractNep141Event;

impl NewContractNep141Event {
    pub const ID: &'static str = "newtoken_nep141";
}

#[cfg(feature = "impl")]
impl Event for NewContractNep141Event {
    const ID: &'static str = Self::ID;
    const CATEGORY: &'static str = "New Tokens";
    const SUPPORTS_TESTNET: bool = true;

    type EventData = NewContractNep141EventData;
    type RealtimeEventFilter = RtNewContractNep141Filter;
    type DatabaseAdapter = DbNewContractNep141Adapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewContractNep141EventData {
    #[schemars(with = "String")]
    pub account_id: AccountId,

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
pub struct NewContractNep141Filter {
    #[schemars(with = "Option<String>")]
    pub account_id: Option<AccountId>,
}

pub struct DbNewContractNep141Adapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbNewContractNep141Adapter {
    type Event = NewContractNep141Event;
    type Filter = NewContractNep141Filter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        if testnet {
            sqlx::query!(
                r#"
                INSERT INTO newcontract_nep141_testnet (timestamp, transaction_id, receipt_id, block_height, account_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.account_id.as_str(),
            ).execute(pool).await
        } else {
            sqlx::query!(
                r#"
                INSERT INTO newcontract_nep141 (timestamp, transaction_id, receipt_id, block_height, account_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
                event.transaction_id.to_string(),
                event.receipt_id.to_string(),
                event.block_height as i64,
                event.account_id.as_str(),
            ).execute(pool).await
        }
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for NewContractNep141Filter {
    type Event = NewContractNep141Event;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlNewContractNep141EventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            transaction_id: String,
            receipt_id: String,
            block_height: i64,
            account_id: String,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let account_id = self.account_id.as_ref().map(|id| id.as_str());
        sqlx_conditional_queries::conditional_query_as!(
            SqlNewContractNep141EventData,
            r#"
            SELECT *
            FROM newcontract_nep141{#testnet}
            WHERE {#time}
                AND ({account_id}::TEXT IS NULL OR account_id = {account_id})
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
                        NewContractNep141EventData {
                            transaction_id: record.transaction_id.parse().unwrap(),
                            receipt_id: record.receipt_id.parse().unwrap(),
                            block_height: record.block_height as u64,
                            block_timestamp_nanosec: record.timestamp.timestamp_nanos_opt().unwrap()
                                as u128,
                            account_id: AccountId::from_str(&record.account_id).unwrap(),
                        },
                    )
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RtNewContractNep141Filter {
    pub account_id: Option<AccountId>,
    pub parent_account_id: Option<AccountId>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtNewContractNep141Filter {
    type Event = NewContractNep141Event;

    fn matches(&self, event: &<Self::Event as Event>::EventData) -> bool {
        if let Some(account_id) = &self.account_id {
            if account_id != &event.account_id {
                return false;
            }
        }

        if let Some(parent_account_id) = &self.parent_account_id {
            let Some(subaccount) = event
                .account_id
                .as_str()
                .strip_suffix(parent_account_id.as_str())
            else {
                return false;
            };
            let Some(subaccount) = subaccount.strip_suffix('.') else {
                return false;
            };
            if subaccount.contains('.') {
                // Only direct subaccounts are allowed
                return false;
            }
        }

        true
    }
}
