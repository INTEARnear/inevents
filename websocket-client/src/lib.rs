use futures_util::{SinkExt, StreamExt};
use json_filter::Operator;
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
pub struct EventStreamClient(reqwest::Url);

impl Default for EventStreamClient {
    fn default() -> Self {
        Self(reqwest::Url::parse("wss://ws-events-v3.intear.tech").unwrap())
    }
}

impl EventStreamClient {
    pub fn new(url: reqwest::Url) -> Self {
        Self(url)
    }

    pub async fn stream_events<E, F, Fut>(
        &self,
        event_id: &'static str,
        filter: Option<Operator>,
        mut callback: F,
    ) where
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

            let filter_json = if let Some(filter) = &filter {
                serde_json::to_string(filter).expect("Failed to serialize filter")
            } else {
                r#"{"And":[]}"#.to_string()
            };

            stream
                .send(Message::Text(filter_json))
                .await
                .expect("Failed to send filter");

            while let Some(message) = stream.next().await {
                let Ok(msg) = message else {
                    log::warn!("Connection error, reconnecting in 1 second");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue 'outer;
                };
                match msg {
                    Message::Close(_) => {
                        log::warn!(
                            "Event stream events/{event_id} closed, reconnecting in 1 second"
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue 'outer;
                    }
                    Message::Ping(data) => {
                        stream
                            .send(Message::Pong(data))
                            .await
                            .expect("Failed to pong");
                    }
                    Message::Pong(_) => {}
                    Message::Text(text) => {
                        let events: Vec<E> = serde_json::from_str(&text).unwrap_or_else(|err| {
                            panic!("Failed to parse message: {text}, error: {err}")
                        });
                        for event in events {
                            callback(event).await;
                        }
                    }
                    Message::Binary(_) => {}
                    Message::Frame(_) => unreachable!(),
                }
            }
            log::warn!("Connection lost, reconnecting in 1 second");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use json_filter::{Filter, Operator};

    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_log_text_stream() {
        let client = EventStreamClient::default();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&received_events);

        let handle = tokio::spawn(async move {
            client
                .stream_events::<serde_json::Value, _, _>("log_text", None, move |event| {
                    let events = Arc::clone(&events_clone);
                    async move {
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
        println!("\nTest Summary:");
        println!("Total events received: {}", final_events.len());
        println!(
            "First event: {}",
            serde_json::to_string(&final_events[0]).unwrap()
        );
        println!(
            "Last event: {}",
            serde_json::to_string(&final_events.last().unwrap()).unwrap()
        );
    }

    #[tokio::test]
    async fn test_filter_events() {
        let client = EventStreamClient::default();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&received_events);
        let filter = Operator::And(vec![Filter::new(
            "event_standard",
            Operator::Equals("nep141".into()),
        )]);

        let handle = tokio::spawn(async move {
            client
                .stream_events::<serde_json::Value, _, _>(
                    "log_nep297",
                    Some(filter),
                    move |event| {
                        let events = Arc::clone(&events_clone);
                        async move {
                            events.lock().unwrap().push(event);
                        }
                    },
                )
                .await;
        });

        // Wait for some events
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        handle.abort();

        let final_events = received_events.lock().unwrap();
        assert!(!final_events.is_empty(), "Should have received some events");
        println!("\nTest Summary:");
        println!("Total events received: {}", final_events.len());
        println!(
            "First event: {}",
            serde_json::to_string(&final_events[0]).unwrap()
        );
        println!(
            "Last event: {}",
            serde_json::to_string(&final_events.last().unwrap()).unwrap()
        );
    }
}
