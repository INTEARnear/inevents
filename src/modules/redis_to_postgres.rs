use super::{EventCollection, EventModule};
use async_trait::async_trait;
use inevents_redis::RedisEventStream;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

pub struct RedisToPostgres;

#[async_trait]
impl EventModule for RedisToPostgres {
    async fn start<E: EventCollection>(&self) -> anyhow::Result<()> {
        let redis_connection = create_connection(
            &std::env::var("REDIS_URL").expect("REDIS_URL enviroment variable not set"),
        )
        .await;
        let pg_pool = PgPool::connect(
            &std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set"),
        )
        .await
        .expect("Failed to connect to Postgres");

        for event in E::events() {
            let pg_pool = pg_pool.clone();
            let redis_connection = redis_connection.clone();
            tokio::spawn(async move {
                let mut stream = RedisEventStream::new(redis_connection, event.event_identifier);
                stream
                    .start_reading_events("redis_to_postgres", move |value: serde_json::Value| {
                        (event.insert_into_postgres)(pg_pool.clone(), value)
                    })
                    .await
            });
        }
        Ok(())
    }
}

async fn create_connection(url: &str) -> ConnectionManager {
    let client = redis::Client::open(url).expect("Failed to connect to Redis");
    ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}
