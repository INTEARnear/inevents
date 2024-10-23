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

pub struct FtTransferEvent;

impl FtTransferEvent {
    pub const ID: &'static str = "ft_transfer";
}

#[cfg(feature = "impl")]
impl Event for FtTransferEvent {
    const ID: &'static str = Self::ID;
    const DESCRIPTION: Option<&'static str> = Some("Fired when FTs are transferred or sold");
    const CATEGORY: &'static str = "FT";

    type EventData = FtTransferEventData;
    type RealtimeEventFilter = RtFtTransferFilter;
    type DatabaseAdapter = DbFtTransferAdapter;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FtTransferEventData {
    #[schemars(with = "String")]
    pub old_owner_id: AccountId,
    #[schemars(with = "String")]
    pub new_owner_id: AccountId,
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
pub struct DbFtTransferFilter {
    #[schemars(with = "Option<String>")]
    pub token_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub old_owner_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub new_owner_id: Option<AccountId>,
    #[schemars(with = "Option<String>")]
    pub involved_account_ids: Option<String>,
    #[serde(with = "dec_format", default)]
    #[schemars(with = "Option<String>")]
    pub amount: Option<Balance>,
    #[serde(with = "dec_format", default)]
    #[schemars(with = "Option<String>")]
    pub min_amount: Option<Balance>,
}

pub struct DbFtTransferAdapter;

#[cfg(feature = "impl")]
impl DatabaseEventAdapter for DbFtTransferAdapter {
    type Event = FtTransferEvent;
    type Filter = DbFtTransferFilter;

    async fn insert(
        event: &<Self::Event as Event>::EventData,
        pool: &Pool<Postgres>,
        _testnet: bool,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO ft_transfer (timestamp, transaction_id, receipt_id, block_height, token_id, old_owner_id, new_owner_id, amount, memo)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            chrono::DateTime::from_timestamp((event.block_timestamp_nanosec / 1_000_000_000) as i64, (event.block_timestamp_nanosec % 1_000_000_000) as u32),
            event.transaction_id.to_string(),
            event.receipt_id.to_string(),
            event.block_height as i64,
            event.token_id.to_string(),
            event.old_owner_id.to_string(),
            event.new_owner_id.to_string(),
            BigDecimal::from(event.amount),
            event.memo,
        ).execute(pool)
        .await
    }
}

#[cfg(feature = "impl")]
impl DatabaseEventFilter for DbFtTransferFilter {
    type Event = FtTransferEvent;

    async fn query_paginated(
        &self,
        pagination: &PaginationParameters,
        pool: &Pool<Postgres>,
        testnet: bool,
    ) -> Result<Vec<(EventId, <Self::Event as Event>::EventData)>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct SqlFtTransferEventData {
            id: i64,
            timestamp: chrono::DateTime<chrono::Utc>,
            block_height: i64,
            token_id: String,
            old_owner_id: String,
            new_owner_id: String,
            amount: BigDecimal,
            memo: Option<String>,
            transaction_id: String,
            receipt_id: String,
        }
        let (timestamp, id, limit) =
            crate::events::get_pagination_params(pagination, pool, testnet).await;

        let token_id = self.token_id.as_ref().map(|c| c.as_str());
        let old_owner_id = self.old_owner_id.as_ref().map(|o| o.as_str());
        let new_owner_id = self.new_owner_id.as_ref().map(|n| n.as_str());
        let involved_account_ids = self
            .involved_account_ids
            .as_ref()
            .map(|ids| ids.split(',').map(|s| s.to_owned()).collect::<Vec<_>>());
        #[allow(clippy::get_first)]
        let first_involved_account_id = involved_account_ids.as_ref().and_then(|ids| ids.get(0));
        let second_involved_account_id = involved_account_ids.as_ref().and_then(|ids| ids.get(1));
        let amount = self.amount.as_ref().map(|a| BigDecimal::from(*a));
        let min_amount = self.min_amount.as_ref().map(|a| BigDecimal::from(*a));

        sqlx_conditional_queries::conditional_query_as!(
            SqlFtTransferEventData,
            r#"
            SELECT *
            FROM ft_transfer{#testnet}
            WHERE {#time}
                AND ({token_id}::TEXT IS NULL OR token_id = {token_id})
                AND ({old_owner_id}::TEXT IS NULL OR old_owner_id = {old_owner_id})
                AND ({new_owner_id}::TEXT IS NULL OR new_owner_id = {new_owner_id})
                AND ({amount}::NUMERIC IS NULL OR amount = {amount})
                AND {#involved_account_ids}
                AND ({min_amount}::NUMERIC IS NULL OR amount >= {min_amount})
            ORDER BY id {#order}
            LIMIT {limit}
            "#,
            #involved_account_ids = match involved_account_ids.as_ref().map(|ids| ids.len()).unwrap_or_default() {
                0 => "true",
                1 => "(old_owner_id = {first_involved_account_id} OR new_owner_id = {first_involved_account_id})",
                2 => "((old_owner_id = {first_involved_account_id} AND new_owner_id = {second_involved_account_id})
                    OR (old_owner_id = {second_involved_account_id} AND new_owner_id = {first_involved_account_id}))",
                3.. => "false",
            },
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
                true => "", //"_testnet",
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
                        FtTransferEventData {
                            old_owner_id: record.old_owner_id.parse().unwrap(),
                            new_owner_id: record.new_owner_id.parse().unwrap(),
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
pub struct RtFtTransferFilter {
    pub token_id: Option<AccountId>,
    pub old_owner_id: Option<AccountId>,
    pub new_owner_id: Option<AccountId>,
    pub involved_account_ids: Option<Vec<AccountId>>,
    #[serde(with = "dec_format", default)]
    pub amount: Option<Balance>,
    #[serde(with = "dec_format", default)]
    pub min_amount: Option<Balance>,
}

#[cfg(feature = "impl")]
impl RealtimeEventFilter for RtFtTransferFilter {
    type Event = FtTransferEvent;

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
