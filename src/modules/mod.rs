pub mod http_server;
pub mod redis_to_postgres;
pub mod websocket_server;

use async_trait::async_trait;

use crate::events::event::Event;

#[async_trait]
pub trait EventModule {
    async fn start(self, events: Vec<Event>) -> anyhow::Result<()>;
}
