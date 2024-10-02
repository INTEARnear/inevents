use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::BufReader,
};

use crate::events::event::{PaginationBy, PaginationParameters, MAX_EVENTS_PER_REQUEST};

use super::{EventCollection, EventModule, RawEvent};
use actix_cors::Cors;
use actix_web::HttpResponseBuilder;
use actix_web::{
    dev::HttpServiceFactory,
    http::{
        header::{self, CacheDirective},
        StatusCode,
    },
    middleware,
    web::{self, redirect},
    App, HttpRequest, HttpResponse, ResponseError,
};
use async_trait::async_trait;
use schemars::schema::{
    InstanceType, NumberValidation, ObjectValidation, SchemaObject, SingleOrVec,
};
use sqlx::PgPool;
use utoipa::openapi::{
    path::{OperationBuilder, ParameterBuilder, ParameterIn, PathItemBuilder},
    AllOfBuilder, ArrayBuilder, ComponentsBuilder, ContentBuilder, InfoBuilder, ObjectBuilder,
    OpenApi, OpenApiBuilder, PathItemType, PathsBuilder, Ref, RefOr, Required, ResponseBuilder,
    SchemaType,
};
use utoipa_swagger_ui::{Config, SwaggerUi};

pub struct HttpServer {
    redirect_from_homepage: Option<String>,
}

impl HttpServer {
    pub fn new(redirect_from_homepage: Option<String>) -> Self {
        Self {
            redirect_from_homepage,
        }
    }
}

struct AppState {
    pg_pool: PgPool,
}

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
    async fn start<E: EventCollection>(self) -> Result<(), anyhow::Error> {
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

            let openapi = create_openapi_spec::<E>(false);
            let openapi_testnet = create_openapi_spec::<E>(true);
            let mut query = web::scope("/query").service(
                web::scope("swagger-ui").service(
                    SwaggerUi::new("/{_:.*}")
                        .url("/swagger.json", openapi)
                        .config(
                            Config::from("/query/swagger-ui/swagger.json")
                                .request_snippets_enabled(true),
                        ),
                ),
            );
            for event in E::events() {
                if event.excluded_from_database {
                    continue;
                }
                query = query.service(create_route(event, false));
            }

            let mut query_testnet = web::scope("/query-testnet").service(
                web::scope("swagger-ui").service(
                    SwaggerUi::new("/{_:.*}")
                        .url("/swagger.json", openapi_testnet)
                        .config(
                            Config::from("/query-testnet/swagger-ui/swagger.json")
                                .request_snippets_enabled(true),
                        ),
                ),
            );
            for event in E::events() {
                if event.excluded_from_database {
                    continue;
                }
                if event.supports_testnet {
                    query_testnet = query_testnet.service(create_route(event, true));
                }
            }

            let mut api = web::scope("/api");
            for event in E::events() {
                if event.excluded_from_database {
                    continue;
                }
                for custom_route in (event.get_custom_endpoints)(pg_pool.clone()) {
                    let custom_route = Rc::new(custom_route);
                    api = api.service(web::resource(custom_route.name()).route(web::get().to(
                        move |query: web::Query<HashMap<String, String>>| {
                            let custom_route = Rc::clone(&custom_route);
                            let query = query.into_inner();
                            async move {
                                let (status, body) = custom_route
                                    .handle(query, false)
                                    .await
                                    .expect("Failed to handle custom route");
                                HttpResponseBuilder::new(status).json(body)
                            }
                        },
                    )));
                }
            }

            let mut api_testnet = web::scope("/api-testnet");
            for event in E::events() {
                if event.excluded_from_database {
                    continue;
                }
                for custom_route in (event.get_custom_endpoints)(pg_pool.clone()) {
                    let custom_route = Rc::new(custom_route);
                    api_testnet = api_testnet.service(web::resource(custom_route.name()).route(
                        web::get().to(move |query: web::Query<HashMap<String, String>>| {
                            let custom_route = Rc::clone(&custom_route);
                            let query = query.into_inner();
                            async move {
                                let (status, body) = custom_route
                                    .handle(query, true)
                                    .await
                                    .expect("Failed to handle custom route");
                                HttpResponseBuilder::new(status).json(body)
                            }
                        }),
                    ));
                }
            }

            let state = AppState {
                pg_pool: pg_pool.clone(),
            };

            let mut app = App::new()
                .app_data(web::Data::new(state))
                .service(query)
                .service(query_testnet)
                .service(api)
                .service(api_testnet);

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

fn create_route(event: RawEvent, testnet: bool) -> impl HttpServiceFactory {
    web::resource(event.event_identifier).route(web::get().to(
        move |state: web::Data<AppState>,
              pagination: web::Query<PaginationParameters>,
              request: HttpRequest| async move {
            let limit = pagination.limit;
            let is_sort_by_newest = matches!(pagination.pagination_by, PaginationBy::Newest);
            if limit > MAX_EVENTS_PER_REQUEST {
                return Err(AppError::TooManyBlocks {
                    requested: limit,
                    max: MAX_EVENTS_PER_REQUEST,
                });
            }
            (event.query_paginated)(pagination.0, state.pg_pool.clone(), request, testnet)
                .await
                .map(|res| {
                    // Cache responses if all `limit` events are returned
                    let cache_control = if res.len() == limit as usize && !is_sort_by_newest {
                        header::CacheControl(vec![
                            CacheDirective::Public,
                            CacheDirective::MaxAge(60 * 10),
                        ])
                    } else {
                        header::CacheControl(vec![CacheDirective::NoCache])
                    };
                    HttpResponse::Ok().append_header(cache_control).json(
                        res.into_iter()
                            .map(|(id, data)| {
                                serde_json::json!({
                                    "id": id,
                                    "event": data,
                                })
                            })
                            .collect::<Vec<_>>(),
                    )
                })
        },
    ))
}

pub fn create_openapi_spec<E: EventCollection>(testnet: bool) -> OpenApi {
    OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .title(env!("CARGO_PKG_NAME"))
                .version(format!("{}-{}", env!("CARGO_PKG_VERSION"), if testnet { "testnet" } else { "mainnet" }))
                .description(Some(concat!(
                    "Automatically generated documentation for ",
                    env!("CARGO_PKG_NAME")
                ))),
        )
        .components(Some({
            let mut builder = ComponentsBuilder::new();
            for event in E::events() {
                if event.excluded_from_database {
                    continue;
                }
                if testnet && !event.supports_testnet {
                    continue;
                }
                let schema = event.event_data_schema;
                for definition in schema.definitions {
                    builder = builder.schema(
                        definition.0,
                        to_openapi_schema(&definition.1).expect("Failed to create schema"),
                    );
                }
            }
            builder.build()
        }))
        .paths({
            let mut builder = PathsBuilder::new();
            for event in E::events() {
                if event.excluded_from_database {
                    continue;
                }
                if testnet && !event.supports_testnet {
                    continue;
                }
                builder = builder.path(
                    format!("/query{}/{}", if testnet { "-testnet" } else { "" }, event.event_identifier),
                    PathItemBuilder::new()
                        .parameters(Some(vec![
                            ParameterBuilder::new()
                                .name("pagination_by")
                                .description(Some("Determines how to paginate the results"))
                                .parameter_in(ParameterIn::Query)
                                .required(Required::True)
                                .schema(Some(utoipa::openapi::schema::Schema::Object(
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::String)
                                        .enum_values(Some(vec![
                                            serde_json::Value::String("BeforeBlockHeight".to_string()),
                                            serde_json::Value::String("AfterBlockHeight".to_string()),
                                            serde_json::Value::String("BeforeTimestamp".to_string()),
                                            serde_json::Value::String("AfterTimestamp".to_string()),
                                            serde_json::Value::String("BeforeId".to_string()),
                                            serde_json::Value::String("AfterId".to_string()),
                                            serde_json::Value::String("Oldest".to_string()),
                                            serde_json::Value::String("Newest".to_string()),
                                        ]))
                                        .build(),
                                )))
                                .build(),
                            ParameterBuilder::new()
                                .name("block_height")
                                .description(Some("Only when `pagination_by` is `BeforeBlockHeight` or `AfterBlockHeight`"))
                                .parameter_in(ParameterIn::Query)
                                .required(Required::False)
                                .schema(Some(utoipa::openapi::schema::Schema::Object(
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::Integer)
                                        .minimum(Some(0.0))
                                        .multiple_of(Some(1.0))
                                        .build(),
                                )))
                                .build(),
                            ParameterBuilder::new()
                                .name("timestamp_nanosec")
                                .description(Some("Only when `pagination_by` is `BeforeTimestamp` or `AfterTimestamp`"))
                                .parameter_in(ParameterIn::Query)
                                .required(Required::False)
                                .schema(Some(utoipa::openapi::schema::Schema::Object(
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::Integer)
                                        .minimum(Some(0.0))
                                        .multiple_of(Some(1.0))
                                        .build(),
                                )))
                                .build(),
                            ParameterBuilder::new()
                                .name("id")
                                .description(Some("Only when `pagination_by` is `BeforeId` or `AfterId`"))
                                .parameter_in(ParameterIn::Query)
                                .required(Required::False)
                                .schema(Some(utoipa::openapi::schema::Schema::Object(
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::Integer)
                                        .minimum(Some(0.0))
                                        .multiple_of(Some(1.0))
                                        .build(),
                                )))
                                .build(),
                            ParameterBuilder::new()
                                .name("limit")
                                .description(Some("How many events you want to get"))
                                .parameter_in(ParameterIn::Query)
                                .required(Required::True)
                                .schema(Some(utoipa::openapi::schema::Schema::Object(
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::Integer)
                                        .minimum(Some(0.0))
                                        .multiple_of(Some(1.0))
                                        .maximum(Some(MAX_EVENTS_PER_REQUEST as f64))
                                        .default(Some(10.into()))
                                        .build(),
                                )))
                                .build(),
                        ]))
                        .operation(
                            PathItemType::Get,
                            OperationBuilder::new()
                                .tag(event.event_category)
                                .description(event.event_description)
                                .operation_id(Some(event.event_identifier))
                                .parameters(Some({
                                    if let Some(object) = &event.db_filter_schema.schema.object {
                                        object
                                            .properties
                                            .iter()
                                            .map(|(name, schema)| {
                                                ParameterBuilder::new()
                                                    .name(name)
                                                    .parameter_in(ParameterIn::Query)
                                                    .required(Required::False)
                                                    .schema(to_openapi_schema(schema))
                                                    .build()
                                            })
                                            .collect::<Vec<_>>()
                                    } else {
                                        log::warn!("Expected database filter to be an object. Ignoring filter.");
                                        vec![]
                                    }
                                }))
                                .response(
                                    "200",
                                    ResponseBuilder::new()
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(utoipa::openapi::schema::Schema::Array(
                                                    ArrayBuilder::new()
                                                        .nullable(false)
                                                        .min_items(Some(0))
                                                        .items(
                                                            to_openapi_schema(
                                                                &schemars::schema::Schema::Object(
                                                                    SchemaObject {
                                                                        object: Some(Box::new(ObjectValidation {
                                                                            properties: BTreeMap::from_iter([
                                                                                ("id".to_string(), schemars::schema::Schema::Object(SchemaObject {
                                                                                    number: Some(Box::new(NumberValidation {
                                                                                        multiple_of: Some(1.0),
                                                                                        minimum: Some(0.0),
                                                                                        ..Default::default()
                                                                                    })),
                                                                                    ..Default::default()
                                                                                })),
                                                                                ("event".to_string(), schemars::schema::Schema::Object(event.event_data_schema.schema)),
                                                                            ]),
                                                                            required: BTreeSet::from_iter(["id".to_string(), "event".to_string()]),
                                                                            ..Default::default()
                                                                        })),
                                                                        ..Default::default()
                                                                    }
                                                                )
                                                            )
                                                            .expect(
                                                                "Failed to create response schema",
                                                            ),
                                                        )
                                                        .build(),
                                                ))
                                                .build(),
                                        )
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                );
            }
            builder.build()
        })
        .build()
}

fn to_openapi_schema(schema: &schemars::schema::Schema) -> Option<RefOr<utoipa::openapi::Schema>> {
    match schema {
        schemars::schema::Schema::Object(schema) => Some({
            let s: RefOr<utoipa::openapi::Schema> = if let Some(object) = &schema.object {
                utoipa::openapi::Schema::Object({
                    let mut builder = ObjectBuilder::new()
                        .schema_type(SchemaType::Object)
                        .min_properties(object.min_properties.map(|x| x as usize))
                        .max_properties(object.max_properties.map(|x| x as usize));
                    for required in &object.required {
                        builder = builder.required(required);
                    }
                    for (name, schema) in object.properties.iter() {
                        if let Some(schema) = to_openapi_schema(schema) {
                            builder = builder.property(name, schema);
                        }
                    }
                    builder.build()
                })
                .into()
            } else if let Some(string) = &schema.string {
                utoipa::openapi::schema::Schema::Object(
                    ObjectBuilder::new()
                        .schema_type(SchemaType::String)
                        .pattern(string.pattern.clone())
                        .min_length(string.min_length.map(|x| x as usize))
                        .max_length(string.max_length.map(|x| x as usize))
                        .build(),
                )
                .into()
            } else if let Some(number) = &schema.number {
                utoipa::openapi::schema::Schema::Object(
                    ObjectBuilder::new()
                        .schema_type(SchemaType::Number)
                        .minimum(number.minimum)
                        .maximum(number.maximum)
                        .exclusive_minimum(number.exclusive_minimum)
                        .exclusive_maximum(number.exclusive_maximum)
                        .multiple_of(number.multiple_of)
                        .build(),
                )
                .into()
            } else if let Some(array) = &schema.array {
                let mut builder = ArrayBuilder::new()
                    .min_items(array.min_items.map(|x| x as usize))
                    .max_items(array.max_items.map(|x| x as usize))
                    .unique_items(array.unique_items.unwrap_or(false));
                if let Some(SingleOrVec::Single(item)) = array.items.as_ref() {
                    if let Some(schema) = to_openapi_schema(item) {
                        builder = builder.items(schema)
                    }
                }
                utoipa::openapi::schema::Schema::Array(builder.build()).into()
            } else if let Some(r#const) = &schema.const_value {
                utoipa::openapi::schema::Schema::Object(
                    ObjectBuilder::new()
                        .schema_type(match r#const {
                            serde_json::Value::String(_) => SchemaType::String,
                            serde_json::Value::Number(_) => SchemaType::Integer,
                            serde_json::Value::Bool(_) => SchemaType::Boolean,
                            serde_json::Value::Array(_) => SchemaType::Array,
                            serde_json::Value::Object(_) => SchemaType::Object,
                            serde_json::Value::Null => SchemaType::Object,
                        })
                        .enum_values(Some(vec![r#const.clone()]))
                        .build(),
                )
                .into()
            } else if let Some(enum_values) = &schema.enum_values {
                utoipa::openapi::schema::Schema::Object({
                    let mut builder = ObjectBuilder::new();
                    if let Some(first_value) = enum_values.first() {
                        builder = builder.schema_type(match first_value {
                            serde_json::Value::String(_) => SchemaType::String,
                            serde_json::Value::Number(_) => SchemaType::Integer,
                            serde_json::Value::Bool(_) => SchemaType::Boolean,
                            serde_json::Value::Array(_) => SchemaType::Array,
                            serde_json::Value::Object(_) => SchemaType::Object,
                            serde_json::Value::Null => SchemaType::Object,
                        });
                    }
                    builder.enum_values(Some(enum_values.clone())).build()
                })
                .into()
            } else if let Some(instance_type) = &schema.instance_type {
                let (r#type, optional) = match instance_type {
                    SingleOrVec::Single(single) => {
                        if **single == InstanceType::Null {
                            (SchemaType::Object, true)
                        } else {
                            (
                                match **single {
                                    InstanceType::String => SchemaType::String,
                                    InstanceType::Number => SchemaType::Number,
                                    InstanceType::Integer => SchemaType::Integer,
                                    InstanceType::Boolean => SchemaType::Boolean,
                                    InstanceType::Array => SchemaType::Array,
                                    InstanceType::Object => SchemaType::Object,
                                    InstanceType::Null => SchemaType::Object,
                                },
                                false,
                            )
                        }
                    }
                    SingleOrVec::Vec(types) => {
                        let types_without_null = types
                            .iter()
                            .filter(|x| **x != InstanceType::Null)
                            .collect::<Vec<_>>();
                        let optional = types.len() != types_without_null.len();
                        let r#type = if let Some(first) = types_without_null.first() {
                            match **first {
                                InstanceType::String => SchemaType::String,
                                InstanceType::Number => SchemaType::Number,
                                InstanceType::Integer => SchemaType::Integer,
                                InstanceType::Boolean => SchemaType::Boolean,
                                InstanceType::Array => SchemaType::Array,
                                InstanceType::Object => SchemaType::Object,
                                InstanceType::Null => SchemaType::Object,
                            }
                        } else {
                            SchemaType::Object
                        };
                        (r#type, optional)
                    }
                };
                utoipa::openapi::schema::Schema::Object(
                    ObjectBuilder::new()
                        .schema_type(r#type)
                        .nullable(optional)
                        .build(),
                )
                .into()
            } else if let Some(reference) = &schema.reference {
                Ref::new(reference.replace("#/definitions/", "#/components/schemas/")).into()
            } else if let Some(subschema) = &schema.subschemas {
                if let Some(all_of) = subschema.all_of.as_ref() {
                    utoipa::openapi::schema::Schema::AllOf({
                        let mut builder = AllOfBuilder::new();
                        for schema in all_of {
                            if let Some(schema) = to_openapi_schema(schema) {
                                builder = builder.item(schema);
                            }
                        }
                        builder.build()
                    })
                } else if let Some(any_of) = subschema.any_of.as_ref() {
                    utoipa::openapi::schema::Schema::AnyOf({
                        let mut builder = utoipa::openapi::schema::AnyOfBuilder::new();
                        for schema in any_of {
                            if let Some(schema) = to_openapi_schema(schema) {
                                builder = builder.item(schema);
                            }
                        }
                        builder.build()
                    })
                } else if let Some(one_of) = subschema.one_of.as_ref() {
                    utoipa::openapi::schema::Schema::OneOf({
                        let mut builder = utoipa::openapi::schema::OneOfBuilder::new();
                        for schema in one_of {
                            if let Some(schema) = to_openapi_schema(schema) {
                                builder = builder.item(schema);
                            }
                        }
                        builder.build()
                    })
                } else {
                    panic!("Unsupported subschema type. Only allOf, anyOf, and oneOf are supported")
                }
                .into()
            } else {
                panic!("Unknown schema type. Please report this bug. Schema: {schema:?}")
            };

            macro_rules! for_any {
                ($var: expr; $( $variant: pat_param )|* => $action: expr) => {
                    match $var {
                        $(
                            $variant => $action,
                        )*
                        _ => {}
                    }
                };
            }

            match (&schema.metadata, s) {
                (Some(metadata), RefOr::T(mut schema)) => {
                    for_any!(&mut schema;
                            utoipa::openapi::Schema::Object(s)
                            | utoipa::openapi::Schema::Array(s)
                            | utoipa::openapi::Schema::AllOf(s)
                            | utoipa::openapi::Schema::AnyOf(s)
                            | utoipa::openapi::Schema::OneOf(s) => {
                        s.example = metadata.examples.first().cloned();
                        s.default = metadata.default.clone();
                        s.description = metadata.description.clone();
                    });
                    schema.into()
                }
                (_, s) => s,
            }
        }),
        schemars::schema::Schema::Bool(_) => None, // This is for open parameters (any / none), is this needed?
    }
}
