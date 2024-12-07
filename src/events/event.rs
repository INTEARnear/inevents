use std::{collections::HashMap, sync::Arc};

use actix_web::http::StatusCode;
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::openapi::PathItem;

#[derive(Clone, Deserialize)]
pub struct Event {
    pub id: String,
    pub description: Option<String>,
    pub category: String,
    pub sql_insert: Option<String>,
    pub endpoints: Vec<QueryEndpoint>,
}

#[derive(Clone)]
pub enum Query {
    Sql(String),
    Custom(Arc<dyn HttpEndpoint>),
}

#[async_trait]
pub trait HttpEndpoint: Send + Sync {
    async fn handle(
        &self,
        pool: PgPool,
        query: HashMap<String, String>,
    ) -> (StatusCode, serde_json::Value);
}

fn deserialize_query<'de, D>(deserializer: D) -> Result<Query, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let query = String::deserialize(deserializer)?;
    Ok(Query::Sql(query))
}

#[derive(Clone, Deserialize)]
pub struct QueryEndpoint {
    /// The name of the endpoint, it will be used in the URL.
    pub route: String,
    /// The OpenAPI schema of the endpoint.
    pub openapi: Option<PathItem>,
    /// The SQL query to query the event from the database.
    /// Accepts $1 as the query parameters in the endpoint URL.
    #[serde(deserialize_with = "deserialize_query")]
    pub query: Query,
}
