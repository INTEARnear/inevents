use std::time::Duration;

use crate::events::event::Event;

use super::EventModule;
use async_trait::async_trait;
use inevents_redis::RedisEventStream;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

pub struct RedisToPostgres;

#[async_trait]
impl EventModule for RedisToPostgres {
    async fn start(self, events: Vec<Event>) -> anyhow::Result<()> {
        let pg_pool = PgPool::connect(
            &std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set"),
        )
        .await
        .expect("Failed to connect to Postgres");

        let mut tasks = Vec::new();
        let cancellation_token = CancellationToken::new();
        for event in events {
            if let Some(sql_insert) = event.sql_insert {
                let pg_pool = pg_pool.clone();
                let cancellation_token_cloned = cancellation_token.clone();
                tasks.push(tokio::spawn(async move {
                    let mut stream = RedisEventStream::new(
                        create_connection(
                            &std::env::var("REDIS_URL")
                                .expect("REDIS_URL enviroment variable not set"),
                        )
                        .await,
                        event.id.clone(),
                    );
                    while let Err(err) = stream
                        .start_reading_event_vecs(
                            "redis_to_postgres",
                            |values: Vec<serde_json::Value>| {
                                let pg_pool = pg_pool.clone();
                                let sql_insert = sql_insert.clone();
                                let event_id = event.id.clone();
                                async move {
                                    let time = tokio::time::Instant::now();
                                    match sqlx::query(&sql_insert)
                                        .bind(serde_json::to_value(values).unwrap())
                                        .execute(&pg_pool)
                                        .await
                                    {
                                        Ok(_) => {
                                            let duration = time.elapsed();
                                            if duration > Duration::from_millis(25) {
                                                log::warn!(
                                                    "Insertion of {event_id} took {duration:?}"
                                                );
                                            }
                                            Ok(())
                                        },
                                        Err(err) => {
                                            log::error!(
                                                "Error inserting {event_id} event into database: {err:?}",
                                            );
                                            Err(anyhow::anyhow!("Failed to insert event"))
                                        }
                                    }
                                }
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
