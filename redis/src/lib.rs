use std::sync::OnceLock;
use std::time::Duration;
use std::{future::Future, sync::Arc};

use anyhow::Context;
use redis::{
    aio::ConnectionManager,
    streams::{StreamMaxlen, StreamReadOptions},
    AsyncCommands,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;

pub struct RedisEventStream<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> {
    pub connection: ConnectionManager,
    pub stream_name: String,
    queue: Arc<OnceLock<Sender<T>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> RedisEventStream<T> {
    pub fn new(connection: ConnectionManager, stream_name: impl Into<String>) -> Self {
        Self {
            connection,
            stream_name: stream_name.into(),
            queue: Arc::new(OnceLock::new()),
            _marker: std::marker::PhantomData,
        }
    }

    /// Reads events from the stream, runs forever. Last read index will is saved in
    /// Redis by key "{reader_id}-{stream_name}".
    pub async fn start_reading_events<
        R: Future<Output = Result<(), anyhow::Error>>,
        F: FnMut(T) -> R,
    >(
        &mut self,
        reader_id: impl Into<String>,
        mut f: F,
    ) -> Result<(), anyhow::Error> {
        let mut con = self.connection.clone();
        let reader_id = reader_id.into();
        let last_id: Option<String> = con
            .get(format!("{}-{}", &reader_id, self.stream_name))
            .await?;
        let mut last_id = last_id.unwrap_or("0".to_string());
        loop {
            let events = self.read_event(&last_id).await?;
            for (id, event) in events {
                f(event).await.context("Processing event")?;
                last_id = id;
            }
            con.set(format!("{}-{}", &reader_id, self.stream_name), &last_id)
                .await?;
        }
    }

    pub async fn read_event(
        &mut self,
        start_id: impl Into<String>,
    ) -> Result<Vec<(String, T)>, anyhow::Error> {
        let result = self.connection
            .xread_options::<_, _, Option<Vec<Vec<(String, Vec<Vec<(String, Vec<(String, String)>)>>)>>>>(
                &[&self.stream_name],
                &[start_id.into()],
                &StreamReadOptions::default()
                    .count(100)
                    .block(Duration::from_millis(250).as_millis() as usize),
            )
            .await?;
        match result {
            Some(result) => Ok(result
                .into_iter()
                .flatten()
                .flat_map(|(_stream_id, stream)| {
                    stream
                        .into_iter()
                        .flatten()
                        .map(|(id, fields)| {
                            let (key, event) = &fields[0];
                            assert_eq!(key, "event");
                            (id, serde_json::from_str::<T>(event).unwrap())
                        })
                        .collect::<Vec<_>>()
                })
                .collect()),
            None => {
                tokio::task::yield_now().await;
                Ok(vec![])
            }
        }
    }

    /// Index is a value that is used to identify the sequence of events.
    /// Recommended to use block height or a timestamp. The actual index
    /// will be the stream name followed by a hyphen and the index assigned
    /// by Redis.
    pub fn emit_event(
        &mut self,
        index: impl ToString,
        value: T,
        max_len: usize,
    ) -> Result<(), anyhow::Error> {
        let tx = self
            .queue
            .get_or_init(|| {
                let (tx, mut rx) = tokio::sync::mpsc::channel(1000);

                let mut connection = self.connection.clone();
                let index = index.to_string();
                let stream_name = self.stream_name.clone();

                tokio::spawn(async move {
                    while let Some(value) = rx.recv().await {
                        let _: () = connection
                            .xadd_maxlen(
                                &stream_name,
                                StreamMaxlen::Approx(max_len),
                                format!("{index}-*"),
                                &[(
                                    "event",
                                    serde_json::to_string(&value)
                                        .expect("Failed to serialize event"),
                                )],
                            )
                            .await
                            .expect("Failed to emit event");
                    }
                });

                tx
            });
        match tx.try_send(value) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(anyhow::anyhow!("Couldn't send an event because a channel is full (capacity is 1000)")),
            Err(TrySendError::Closed(_)) => Err(anyhow::anyhow!("Couldn't send an event because a channel is closed")),
        }
    }
}
