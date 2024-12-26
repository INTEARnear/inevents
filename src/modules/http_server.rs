use std::collections::HashMap;
use std::rc::Rc;
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::BufReader,
};

use crate::events::event::{Event, Query, QueryEndpoint};

use super::EventModule;
use actix_cors::Cors;
use actix_web::HttpResponseBuilder;
use actix_web::{
    http::StatusCode,
    middleware,
    web::{self, redirect},
    App, HttpResponse, ResponseError,
};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct HttpServer {
    redirect_from_homepage: Option<String>,
    custom_endpoints: HashMap<String, Vec<QueryEndpoint>>,
}

impl HttpServer {
    pub fn new(
        redirect_from_homepage: Option<String>,
        custom_endpoints: HashMap<String, Vec<QueryEndpoint>>,
    ) -> Self {
        Self {
            redirect_from_homepage,
            custom_endpoints,
        }
    }
}

struct AppState;

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    Actix(actix_web::Error),
    TooManyBlocks { requested: u64, max: u64 },
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Actix(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TooManyBlocks { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Db(err) => HttpResponse::InternalServerError().body(err.to_string()),
            AppError::Actix(err) => HttpResponse::InternalServerError().body(err.to_string()),
            AppError::TooManyBlocks { requested, max } => HttpResponse::BadRequest().body(format!(
                "Too many blocks requested. Requested: {requested}, Max: {max}"
            )),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Db(err) => write!(f, "Database error: {err}"),
            AppError::Actix(err) => write!(f, "Actix error: {err}"),
            AppError::TooManyBlocks { requested, max } => write!(
                f,
                "Too many blocks requested. Requested: {requested}, Max: {max}"
            ),
        }
    }
}

#[async_trait]
impl EventModule for HttpServer {
    async fn start(self, events: Vec<Event>) -> Result<(), anyhow::Error> {
        let pg_pool = PgPool::connect(
            &std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set"),
        )
        .await
        .expect("Failed to connect to Postgres");

        let tls_config = if let Ok(files) = std::env::var("SSL") {
            #[allow(clippy::iter_nth_zero)]
            let mut certs_file =
                BufReader::new(File::open(files.split(',').nth(0).unwrap()).unwrap());
            let mut key_file =
                BufReader::new(File::open(files.split(',').nth(1).unwrap()).unwrap());
            let tls_certs = rustls_pemfile::certs(&mut certs_file)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
                .next()
                .unwrap()
                .unwrap();
            Some(
                rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
                    .unwrap(),
            )
        } else {
            None
        };

        let redirect_from_homepage = self.redirect_from_homepage.clone();
        let server = actix_web::HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET"])
                .max_age(3600)
                .supports_credentials();

            let mut api = web::scope("/v3");
            for event in events.clone() {
                for endpoint in event.endpoints.into_iter().chain(
                    self.custom_endpoints
                        .get(&event.id)
                        .unwrap_or(&Vec::new())
                        .clone()
                        .into_iter(),
                ) {
                    let endpoint = Rc::new(endpoint);
                    let pg_pool = pg_pool.clone();
                    log::info!("Registering endpoint /v3/{}", endpoint.route);
                    api = api.service(web::resource(&endpoint.route).route(web::get().to(
                        move |query: web::Query<HashMap<String, String>>| {
                            let pg_pool = pg_pool.clone();
                            let endpoint = Rc::clone(&endpoint);
                            let query = query.into_inner();
                            async move {
                                let sql_query = &endpoint.query;
                                match sql_query {
                                    Query::Sql(sql_query) => {
                                        match sqlx::query_as::<_, (serde_json::Value,)>(sql_query)
                                            .bind(serde_json::to_value(&query).unwrap())
                                            .fetch_all(&pg_pool)
                                            .await
                                        {
                                            Ok(records) => HttpResponseBuilder::new(StatusCode::OK)
                                                .json(serde_json::json!({
                                                    "result": records,
                                                })),
                                            Err(err) => {
                                                log::error!("Error querying database: {err:?}");
                                                HttpResponseBuilder::new(
                                                    StatusCode::INTERNAL_SERVER_ERROR,
                                                )
                                                .json(
                                                    serde_json::json!({"error": format!("{err}")}),
                                                )
                                            }
                                        }
                                    }
                                    Query::Custom(endpoint) => {
                                        let (status, body) = endpoint.handle(pg_pool, query).await;
                                        HttpResponseBuilder::new(status).json(body)
                                    }
                                }
                            }
                        },
                    )));
                }
            }

            let state = AppState;

            let mut app = App::new().app_data(web::Data::new(state)).service(api);

            if let Some(redirect_from_homepage) = &redirect_from_homepage {
                app = app.service(redirect("/", redirect_from_homepage.clone()));
            }

            app
                .wrap(cors)
                .wrap(middleware::Logger::new(
                    "[HTTP] %{r}a %a \"%r\"        Code: %s Size: %b bytes \"%{Referer}i\" \"%{User-Agent}i\" %T",
                ))
        });

        let server = if let Some(tls_config) = tls_config {
            server.bind_rustls_0_22(
                std::env::var("HTTP_BIND_ADDRESS").unwrap_or("0.0.0.0:8080".to_string()),
                tls_config,
            )?
        } else {
            server.bind(std::env::var("HTTP_BIND_ADDRESS").unwrap_or("0.0.0.0:8080".to_string()))?
        };

        server.run().await?;

        Ok(())
    }
}
