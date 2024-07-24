pub mod http_server;
pub mod redis_to_postgres;
pub mod websocket_server;

use std::pin::Pin;

use actix_web::HttpRequest;
use async_trait::async_trait;
use http_server::AppError;
use schemars::schema::RootSchema;
use sqlx::{PgPool, Pool, Postgres};

use crate::events::event::PaginationParameters;

#[async_trait]
pub trait EventModule {
    async fn start<E: EventCollection>(&self) -> anyhow::Result<()>;
}

pub trait EventCollection {
    fn events() -> Vec<RawEvent>;
}

pub(in crate::modules) type FilterFn = Box<dyn Fn(&serde_json::Value) -> bool>;
type PaginatedResponse = Result<Vec<serde_json::Value>, AppError>;

type BoxFutureWithLifetime<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T>>>;

pub struct RawEvent {
    pub event_identifier: &'static str,
    pub event_description: Option<&'static str>,
    pub event_category: &'static str,
    pub query_paginated:
        fn(PaginationParameters, Pool<Postgres>, HttpRequest) -> BoxFuture<PaginatedResponse>,
    pub realtime_filter_constructor: fn(&str) -> Result<FilterFn, anyhow::Error>,
    pub insert_into_postgres:
        fn(PgPool, serde_json::Value) -> BoxFutureWithLifetime<'static, Result<(), anyhow::Error>>,
    pub event_data_schema: RootSchema,
    pub db_filter_schema: RootSchema,
    pub excluded_from_database: bool,
}
