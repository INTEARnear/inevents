use std::time::Duration;

use super::{EventCollection, EventModule};
use async_trait::async_trait;
use inevents_redis::RedisEventStream;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

pub struct RedisToPostgres;

#[async_trait]
impl EventModule for RedisToPostgres {
    async fn start<E: EventCollection>(self) -> anyhow::Result<()> {
        let pg_pool = PgPool::connect(
            &std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set"),
        )
        .await
        .expect("Failed to connect to Postgres");

        let mut tasks = Vec::new();
        let cancellation_token = CancellationToken::new();
        for event in E::events() {
            if event.excluded_from_database {
                continue;
            }
            let pg_pool_cloned = pg_pool.clone();
            let cancellation_token_cloned = cancellation_token.clone();
            tasks.push(tokio::spawn(async move {
                let mut stream = RedisEventStream::new(
                    create_connection(
                        &std::env::var("REDIS_URL").expect("REDIS_URL enviroment variable not set"),
                    )
                    .await,
                    event.event_identifier,
                );
                while let Err(err) = stream
                    .start_reading_events(
                        "redis_to_postgres",
                        |value: serde_json::Value| {
                            (event.insert_into_postgres)(pg_pool_cloned.clone(), value, false)
                        },
                        || cancellation_token_cloned.is_cancelled(),
                    )
                    .await
                {
                    log::warn!("Error reading events from Redis: {err:?}\nRetrying in 10 seconds");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }));
            if event.supports_testnet {
                let pg_pool_cloned = pg_pool.clone();
                let cancellation_token_cloned = cancellation_token.clone();
                tasks.push(tokio::spawn(async move {
                    let mut stream = RedisEventStream::new(
                        create_connection(
                            &std::env::var("REDIS_URL")
                                .expect("REDIS_URL enviroment variable not set"),
                        )
                        .await,
                        format!("{}_testnet", event.event_identifier),
                    );
                    while let Err(err) = stream
                        .start_reading_events(
                            "redis_to_postgres",
                            |value: serde_json::Value| {
                                (event.insert_into_postgres)(pg_pool_cloned.clone(), value, true)
                            },
                            || cancellation_token_cloned.is_cancelled(),
                        )
                        .await
                    {
                        log::warn!(
                            "Error reading events from Redis: {err:?}\nRetrying in 10 seconds"
                        );
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }));
            }
        }
        let task = tokio::spawn(futures::future::join_all(tasks));
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        log::info!("Ctrl+C received, stopping redis-to-postgres");
        cancellation_token.cancel();
        let results = task.await.expect("Failed to join redis-to-postgres tasks");
        for result in results {
            if let Err(err) = result {
                log::error!("Error in redis-to-postgres task: {err:?}");
            }
        }
        log::info!("Redis-to-postgres stopped");
        Ok(())
    }
}

async fn create_connection(url: &str) -> ConnectionManager {
    let client = redis::Client::open(url).expect("Failed to connect to Redis");
    ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}
