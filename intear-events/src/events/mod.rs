#[cfg(feature = "impl")]
use inevents::events::event::{PaginationBy, PaginationParameters};

#[cfg(feature = "impl")]
pub async fn get_pagination_params(
    pagination: &PaginationParameters,
    pool: &sqlx::Pool<sqlx::Postgres>,
    testnet: bool,
) -> (Option<chrono::DateTime<chrono::Utc>>, Option<i32>, i64) {
    {
        let limit = pagination.limit as i64;
        let timestamp = if let PaginationBy::BeforeTimestamp { timestamp_nanosec }
        | PaginationBy::AfterTimestamp { timestamp_nanosec } =
            pagination.pagination_by
        {
            chrono::DateTime::from_timestamp(
                (timestamp_nanosec / 1_000_000_000) as i64,
                (timestamp_nanosec % 1_000_000_000) as u32,
            )
        } else if let PaginationBy::BeforeBlockHeight { block_height }
        | PaginationBy::AfterBlockHeight { block_height } = pagination.pagination_by
        {
            #[derive(Debug, sqlx::FromRow)]
            struct SqlBlockInfo {
                timestamp: chrono::DateTime<chrono::Utc>,
            }

            let block_height = block_height as i64;
            if let Ok(block_info) = sqlx_conditional_queries::conditional_query_as!(
                SqlBlockInfo,
                r#"
                SELECT timestamp
                FROM block_info{#testnet}
                WHERE block_height = {block_height}
                "#,
                #testnet = match testnet {
                    true => "_testnet",
                    false => "",
                },
            )
            .fetch_one(pool)
            .await
            {
                Some(block_info.timestamp)
            } else {
                ::log::warn!("Block height {block_height} not found");
                None
            }
        } else {
            None
        };
        let id = if let PaginationBy::BeforeId { id } | PaginationBy::AfterId { id } =
            pagination.pagination_by
        {
            Some(id as i32)
        } else {
            None
        };
        (timestamp, id, limit)
    }
}

pub mod log;
pub mod newcontract;
pub mod nft;
pub mod potlock;
pub mod price;
pub mod socialdb;
pub mod tps;
pub mod trade;
pub mod transactions;
