use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use redis::{
    aio::ConnectionManager,
    streams::{StreamMaxlen, StreamReadOptions},
    AsyncCommands,
};
use serde::{Deserialize, Serialize};

pub struct RedisEventStream<T: Serialize + for<'de> Deserialize<'de>> {
    pub connection: ConnectionManager,
    pub stream_name: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de>> RedisEventStream<T> {
    pub async fn new(connection: ConnectionManager, stream_name: impl Into<String>) -> Self {
        Self {
            connection,
            stream_name: stream_name.into(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Reads events from the stream, runs forever. Last read index will is saved in
    /// Redis by key "{reader_id}-{stream_name}".
    pub async fn start_reading_events<
        R: Future<Output = Result<(), anyhow::Error>>,
        F: Fn(T) -> R,
    >(
        &mut self,
        reader_id: impl Into<String>,
        f: F,
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
    pub async fn emit_event(
        &mut self,
        index: impl ToString,
        value: T,
        max_len: usize,
    ) -> Result<(), anyhow::Error> {
        self.connection
            .xadd_maxlen(
                &self.stream_name,
                StreamMaxlen::Approx(max_len),
                format!("{}-*", index.to_string()),
                &[("event", serde_json::to_string(&value)?)],
            )
            .await?;
        Ok(())
    }
}
