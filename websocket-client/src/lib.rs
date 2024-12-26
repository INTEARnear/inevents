use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
pub struct WebSocketServer(reqwest::Url);

impl Default for WebSocketServer {
    fn default() -> Self {
        Self(reqwest::Url::parse("wss://ws-events-v3.intear.tech").unwrap())
    }
}

impl WebSocketServer {
    pub fn new(url: reqwest::Url) -> Self {
        Self(url)
    }

    pub async fn stream_v3_events<E, F, Fut>(&self, event_id: &'static str, mut callback: F)
    where
        E: DeserializeOwned + Send + 'static,
        F: FnMut(E) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        'outer: loop {
            let url = self
                .0
                .join(&format!("events/{event_id}"))
                .expect("Failed to join URL paths");

            let (mut stream, _) = tokio_tungstenite::connect_async(url.as_str())
                .await
                .expect("Failed to connect to event stream");

            stream
                .send(Message::Text(r#"{"And":[]}"#.to_string()))
                .await
                .expect("Failed to send empty filter");

            while let Some(message) = stream.next().await {
                let Ok(msg) = message else {
                    continue 'outer;
                };
                match msg {
                    Message::Close(_) => {
                        log::warn!("Event stream events/{event_id} closed");
                        break 'outer;
                    }
                    Message::Ping(data) => {
                        stream
                            .send(Message::Pong(data))
                            .await
                            .expect("Failed to pong");
                    }
                    Message::Pong(_) => {}
                    Message::Text(text) => {
                        let events: Vec<E> =
                            serde_json::from_str(&text).expect("Failed to parse message as json");
                        for event in events {
                            callback(event).await;
                        }
                    }
                    Message::Binary(_) => {}
                    Message::Frame(_) => unreachable!(),
                }
            }
            log::warn!("Reconnecting to event stream events/{event_id}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_log_text_stream() {
        let server = WebSocketServer::default();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&received_events);

        let handle = tokio::spawn(async move {
            server
                .stream_v3_events::<serde_json::Value, _, _>("log_text", move |event| {
                    let events = Arc::clone(&events_clone);
                    async move {
                        println!("Received event: {}", serde_json::to_string(&event).unwrap());
                        events.lock().unwrap().push(event);
                    }
                })
                .await;
        });

        // Wait for some events
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        handle.abort();

        let final_events = received_events.lock().unwrap();
        assert!(!final_events.is_empty(), "Should have received some events");
    }
}
