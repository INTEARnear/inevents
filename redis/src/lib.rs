use std::future::Future;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::Context;
use redis::{
    aio::ConnectionManager,
    streams::{StreamMaxlen, StreamReadOptions},
    AsyncCommands,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

type Queue<T> = (JoinHandle<()>, Sender<(String, T)>);

pub struct RedisEventStream<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> {
    pub connection: ConnectionManager,
    pub stream_name: String,
    queue: OnceLock<Queue<T>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> RedisEventStream<T> {
    pub fn new(connection: ConnectionManager, stream_name: impl Into<String>) -> Self {
        Self {
            connection,
            stream_name: stream_name.into(),
            queue: OnceLock::new(),
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
        should_stop: impl Fn() -> bool,
    ) -> Result<(), anyhow::Error> {
        let mut con = self.connection.clone();
        let reader_id = reader_id.into();
        let last_id: Option<String> = con
            .get(format!("{}-{}", &reader_id, self.stream_name))
            .await?;
        let mut last_id = last_id.unwrap_or("0".to_string());
        loop {
            if should_stop() {
                break Ok(());
            }
            let events = self.read_event(&last_id).await?;
            if events.is_empty() {
                continue;
            }
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
                    .count(10000)
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
    pub async fn emit_event(
        &self,
        index: impl ToString,
        value: T,
        max_len: usize,
    ) -> Result<(), anyhow::Error> {
        let (_, tx) = self.queue.get_or_init(|| {
            let (tx, mut rx) = tokio::sync::mpsc::channel(max_len);

            let mut connection = self.connection.clone();
            let stream_name = self.stream_name.clone();

            let join_handle = tokio::spawn(async move {
                while let Some((index, value)) = rx.recv().await {
                    connection
                        .xadd_maxlen::<_, _, _, _, ()>(
                            &stream_name,
                            StreamMaxlen::Approx(max_len),
                            format!("{index}-*"),
                            &[(
                                "event",
                                serde_json::to_string(&value).unwrap_or_else(|_| {
                                    panic!("Failed to serialize {stream_name} event")
                                }),
                            )],
                        )
                        .await
                        .unwrap_or_else(|_| {
                            panic!("Failed to emit {stream_name} event at index {index}")
                        });
                }
            });

            (join_handle, tx)
        });
        match tx.send((index.to_string(), value)).await {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(
                "Couldn't send event {e:?} because a channel is closed"
            )),
        }
    }

    pub async fn stop(self) {
        if let Some((join_handle, tx)) = self.queue.into_inner() {
            drop(tx); // Receiving end of the channel will close when all senders are dropped
            join_handle.await.expect("Failed to stop the event stream");
        }
    }
}
