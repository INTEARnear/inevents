use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use redis::{
    aio::ConnectionManager,
    streams::{StreamMaxlen, StreamReadOptions},
    AsyncCommands,
};
use serde::{Deserialize, Serialize};

pub struct RedisEventStream<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> {
    pub connection: ConnectionManager,
    pub stream_name: String,
    to_be_published: Vec<T>,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum EventStreamError<E> {
    #[error("Can't get last id, redis error: {0}")]
    CantGetLastId(redis::RedisError),
    #[error("Error in event handler: {0}")]
    EventHandlerError(E),
}

#[derive(Debug, thiserror::Error)]
pub enum EventEmitError<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> {
    #[error("Channel is closed, event not emitted")]
    ChannelClosed(#[source] tokio::sync::mpsc::error::SendError<(String, T)>),
}

impl<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> RedisEventStream<T> {
    pub fn new(connection: ConnectionManager, stream_name: impl Into<String>) -> Self {
        Self {
            connection,
            stream_name: stream_name.into(),
            to_be_published: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Reads events from the stream, runs forever. Last read index will is saved in
    /// Redis by key "{reader_id}-{stream_name}".
    pub async fn start_reading_events<E, R: Future<Output = Result<(), E>>, F: FnMut(T) -> R>(
        &mut self,
        reader_id: impl Into<String>,
        mut f: F,
        should_stop: impl Fn() -> bool,
    ) -> Result<(), EventStreamError<E>> {
        let mut con = self.connection.clone();
        let reader_id = reader_id.into();
        let key = format!("{}-{}", &reader_id, self.stream_name);
        let last_id: Option<String> = con
            .get(&key)
            .await
            .map_err(EventStreamError::CantGetLastId)?;
        let mut last_id = last_id.unwrap_or("0".to_string());
        loop {
            if should_stop() {
                break Ok(());
            }
            let events = self.read_events(&last_id).await;
            if events.is_empty() {
                tokio::time::sleep(Duration::from_millis(25)).await;
                continue;
            }
            for (id, events) in events {
                for event in events {
                    f(event)
                        .await
                        .map_err(|e| EventStreamError::EventHandlerError(e))?;
                }
                last_id = id;
            }
            loop {
                match con.set::<_, _, ()>(&key, &last_id).await {
                    Ok(_) => {
                        log::info!("Set last id for {key}: {last_id}");
                        break;
                    }
                    Err(err) => {
                        log::warn!("Failed to set last id for {key}: {err:?}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    /// Reads vecs of events from the stream, runs forever. Last read index will is
    /// saved in Redis by key "{reader_id}-{stream_name}".
    pub async fn start_reading_event_vecs<
        E,
        R: Future<Output = Result<(), E>>,
        F: FnMut(Vec<T>) -> R,
    >(
        &mut self,
        reader_id: impl Into<String>,
        mut f: F,
        should_stop: impl Fn() -> bool,
    ) -> Result<(), EventStreamError<E>> {
        let mut con = self.connection.clone();
        let reader_id = reader_id.into();
        let key = format!("{}-{}", &reader_id, self.stream_name);
        let last_id: Option<String> = con
            .get(&key)
            .await
            .map_err(EventStreamError::CantGetLastId)?;
        let mut last_id = last_id.unwrap_or("0".to_string());
        loop {
            if should_stop() {
                break Ok(());
            }
            let events = self.read_events(&last_id).await;
            if events.is_empty() {
                tokio::time::sleep(Duration::from_millis(25)).await;
                continue;
            }
            for (id, events) in events {
                f(events)
                    .await
                    .map_err(|e| EventStreamError::EventHandlerError(e))?;
                last_id = id;
            }
            loop {
                match con.set::<_, _, ()>(&key, &last_id).await {
                    Ok(_) => {
                        log::info!("Set last id for {key}: {last_id}");
                        break;
                    }
                    Err(err) => {
                        log::warn!("Failed to set last id for {key}: {err:?}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    pub async fn read_events(&mut self, start_id: impl Into<String>) -> Vec<(String, Vec<T>)> {
        let start_id = start_id.into();
        let result = loop {
            match self.connection
                .xread_options::<_, _, Option<Vec<Vec<(String, Vec<Vec<(String, Vec<(String, String)>)>>)>>>>(
                    &[&self.stream_name],
                    &[&start_id],
                    &StreamReadOptions::default()
                        .count(100)
                        .block(Duration::from_millis(25).as_millis() as usize),
                )
                .await {
                    Ok(result) => break result,
                    Err(err) => {
                        log::warn!("Failed to read an event from {}: {err:?}", self.stream_name);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
            }
        };
        match result {
            Some(result) => result
                .into_iter()
                .flatten()
                .flat_map(|(_stream_id, stream)| {
                    stream
                        .into_iter()
                        .flatten()
                        .map(|(id, fields)| {
                            let (key, event) = &fields[0];
                            assert_eq!(key, "event");
                            (id, serde_json::from_str::<Vec<T>>(event).unwrap())
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            None => vec![],
        }
    }

    /// Add an event to be published. After adding enough events, use `flush_events` to
    /// publish them to a Redis stream.
    pub fn add_event(&mut self, value: T) {
        self.to_be_published.push(value);
    }

    /// Index is a value that is used to identify the sequence of events.
    /// Recommended to use block height or a timestamp. The actual index
    /// will be the stream name followed by a hyphen and the index assigned
    /// by Redis.
    pub async fn flush_events(
        &mut self,
        index: impl ToString,
        max_len: usize,
    ) -> Result<(), EventEmitError<T>> {
        if self.to_be_published.is_empty() {
            return Ok(());
        }
        log::info!(
            "Flushing {stream_name} events at index {index}",
            stream_name = self.stream_name,
            index = index.to_string()
        );
        while let Err(err) = self
            .connection
            .xadd_maxlen::<_, _, _, _, ()>(
                &self.stream_name,
                StreamMaxlen::Approx(max_len),
                format!("{index}-*", index = index.to_string()),
                &[(
                    "event",
                    serde_json::to_string(&self.to_be_published).unwrap_or_else(|err| {
                        panic!(
                            "Failed to serialize {stream_name} event: {err:?}",
                            stream_name = self.stream_name
                        )
                    }),
                )],
            )
            .await
        {
            log::warn!(
                "Failed to emit {stream_name} event at index {index}: {err:?}",
                stream_name = self.stream_name,
                index = index.to_string()
            );
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        self.to_be_published.clear();
        Ok(())
    }
}
