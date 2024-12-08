// Actix needs a single-threaded context, but we don't want that, so we're running 2 tokio runtimes in parallel.

use std::convert::Infallible;
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::events::event::Event;

use super::EventModule;
use actix::prelude::{
    dev::Message, Actor, ActorContext, Addr, AsyncContext, Handler, Running, StreamHandler,
};
use actix_cors::Cors;
use actix_web::{
    dev::HttpServiceFactory,
    middleware,
    web::{self, redirect},
    App, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws::{self, CloseReason, WsResponseBuilder};
use async_trait::async_trait;
use dashmap::DashSet;
use filter::{Filter, Operator};
use inevents_redis::RedisEventStream;
use redis::aio::ConnectionManager;
use tokio::{sync::OnceCell, task::JoinHandle};
use tokio_util::sync::CancellationToken;

// EventWebSocket is the client, Server is the server.
// Typical flow:
// 1. EventWebSocket -> Server: SubscribeToEvents
// 2. Server: Adds the client to the list of subscribers
// 3. Server: Deserializes the Redis event using FromRedis trait
// 4. Server -> EventWebSocket: Event, Event, Event, ...
// 5. EventWebSocket: Checks if the event matches the filter using EventFilter trait and sends JSON-serialized event to the client
// 6. EventWebSocket -> Server: UnsubscribeFromEvents
// 7. Server: Removes the client from the list of subscribers
pub struct WebsocketServer {
    redirect_from_homepage: Option<String>,
}

impl WebsocketServer {
    pub fn new(redirect_from_homepage: Option<String>) -> Self {
        Self {
            redirect_from_homepage,
        }
    }
}

#[async_trait]
impl EventModule for WebsocketServer {
    async fn start(self, events: Vec<Event>) -> anyhow::Result<()> {
        let cancellation_token = CancellationToken::new();
        let server_addr = Arc::new(OnceCell::new());

        let server_addr_clone = Arc::clone(&server_addr);
        let cancellation_token_clone = cancellation_token.clone();
        let sockets = HashMap::from_iter(
            events
                .iter()
                .map(|event| (event.id.clone(), Arc::new(DashSet::new()))),
        );
        tokio::task::spawn_blocking(|| {
            actix::run(async move {
                let server = Server { sockets };
                server_addr_clone.set(server.start()).unwrap();
                cancellation_token_clone.cancelled().await;
            })
            .expect("Failed to run Websocket server")
        });

        while server_addr.get().is_none() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let server_addr = server_addr.get().unwrap().clone();

        let mut join_handles = Vec::new();
        for event in events.iter() {
            join_handles.push(launch_event_stream(
                event,
                server_addr.clone(),
                cancellation_token.clone(),
            ));
        }

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
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET"])
                .max_age(3600)
                .supports_credentials();

            let mut api = web::scope("/events");
            for event in events.iter() {
                api = api.service(create_route(event));
            }

            let mut app = App::new()
                .app_data(web::Data::new(server_addr.clone()))
                .service(api);

            if let Some(redirect_from_homepage) = &redirect_from_homepage {
                app = app.service(redirect("/", redirect_from_homepage.clone()));
            }

            app.wrap(cors).wrap(middleware::Logger::new(
                "[WS] %{r}a %a \"%r\"        Code: %s \"%{Referer}i\" \"%{User-Agent}i\" %T",
            ))
        });

        let server = if let Some(tls_config) = tls_config {
            server
                .bind_rustls_0_22(
                    std::env::var("WEBSOCKET_BIND_ADDRESS").unwrap_or("0.0.0.0:3000".to_string()),
                    tls_config,
                )
                .expect("Failed to bind to address with TLS")
        } else {
            server
                .bind(std::env::var("WEBSOCKET_BIND_ADDRESS").unwrap_or("0.0.0.0:3000".to_string()))
                .expect("Failed to bind to address")
        };

        if let Err(err) = server.run().await {
            log::error!("Failed to start websocket server: {err:?}");
        }

        log::info!("Websocket server stopped. Stopping event readers");

        cancellation_token.cancel();
        for handle in join_handles {
            handle.await.expect("Failed to join event stream task");
        }

        log::info!("Websocket event readers stopped");

        Ok(())
    }
}

struct Server {
    sockets: HashMap<String, Arc<DashSet<Addr<EventWebSocket>>>>,
}

impl Handler<UnsubscribeFromEvents> for Server {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeFromEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.sockets[&msg.1].remove(&msg.0);
    }
}

impl Handler<SubscribeToEvents> for Server {
    type Result = ();

    fn handle(&mut self, msg: SubscribeToEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.sockets[&msg.1].insert(msg.0);
    }
}

pub struct EventWebSocket {
    last_heartbeat: Instant,
    filter: Option<Filter>,
    server: Addr<Server>,
    event_identifier: String,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

impl Actor for EventWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.last_heartbeat = Instant::now();

        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                ctx.stop();
            }

            ctx.ping(b"");
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.server.do_send(UnsubscribeFromEvents(
            ctx.address(),
            self.event_identifier.clone(),
        ));
        Running::Stop
    }
}

impl Actor for Server {
    type Context = actix::Context<Server>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(10), |server, _ctx| {
            log::info!(
                target: "websocket-status",
                "Websocket connections: {}",
                server.sockets.values().map(|sockets| sockets.len()).sum::<usize>()
            );
        });
    }
}

impl Handler<UntypedEvents> for Server {
    type Result = ();

    fn handle(&mut self, msg: UntypedEvents, _ctx: &mut Self::Context) -> Self::Result {
        let msg = Arc::new(msg);
        for socket in self.sockets.get(&msg.0).unwrap().iter() {
            socket.do_send(Arc::clone(&msg));
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for EventWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let text = text.to_string();
                match serde_json::from_str::<Operator>(&text) {
                    Ok(filter) => {
                        self.filter = Some(Filter::new(".", filter));
                    }
                    Err(err) => {
                        ctx.text(format!("Invalid filter: {err}"));
                        ctx.close(Some(CloseReason {
                            code: ws::CloseCode::Invalid,
                            description: Some("Invalid filter".to_string()),
                        }));
                    }
                }
            }
            _ => ctx.stop(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct UntypedEvents(String, Vec<(String, serde_json::Value)>);

impl Handler<Arc<UntypedEvents>> for EventWebSocket {
    type Result = ();

    fn handle(&mut self, msg: Arc<UntypedEvents>, ctx: &mut Self::Context) -> Self::Result {
        if self.filter.is_none() {
            return;
        }

        let mut s = "[".to_string();
        for (text, obj) in msg.1.iter() {
            match self.filter.as_ref().unwrap().check(obj) {
                Ok(true) => {
                    s.push_str(text);
                    s.push(',');
                }
                Ok(false) => {}
                Err(err) => {
                    ctx.text(format!("Error evaluating filter: {err}"));
                    ctx.close(Some(CloseReason {
                        code: ws::CloseCode::Invalid,
                        description: Some(format!("Error evaluating filter: {err:?}")),
                    }));
                    return;
                }
            }
        }
        if s == "[" {
            return;
        }
        ctx.text(s.trim_end_matches(',').to_owned() + "]");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct SubscribeToEvents(Addr<EventWebSocket>, String);

#[derive(Message)]
#[rtype(result = "()")]
struct UnsubscribeFromEvents(Addr<EventWebSocket>, String);

async fn create_connection(url: &str) -> ConnectionManager {
    let client = redis::Client::open(url).expect("Failed to connect to Redis");
    ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}

fn create_route(event: &Event) -> impl HttpServiceFactory {
    let event_id = event.id.clone();
    web::resource(&event.id).route(web::get().to(
        move |req: HttpRequest, stream: web::Payload, server: web::Data<Addr<Server>>| {
            let event_id = event_id.clone();
            async move {
                let (addr, res) = WsResponseBuilder::new(
                    EventWebSocket {
                        last_heartbeat: Instant::now(),
                        filter: None,
                        server: server.get_ref().clone(),
                        event_identifier: event_id.clone(),
                    },
                    &req,
                    stream,
                )
                .start_with_addr()?;
                server
                    .send(SubscribeToEvents(addr, event_id.clone()))
                    .await
                    .unwrap();
                Result::<HttpResponse, actix_web::Error>::Ok(res)
            }
        },
    ))
}

fn launch_event_stream(
    event_type: &Event,
    server: Addr<Server>,
    cancellation_token: CancellationToken,
) -> JoinHandle<()> {
    let event_id = event_type.id.clone();
    tokio::spawn(async move {
        let mut stream = RedisEventStream::new(
            create_connection(
                &std::env::var("REDIS_URL").expect("REDIS_URL enviroment variable not set"),
            )
            .await,
            event_id.to_string(),
        );
        stream
            .start_reading_event_vecs(
                "websocket",
                |events: Vec<serde_json::Value>| {
                    server.do_send(UntypedEvents(
                        event_id.clone(),
                        events
                            .into_iter()
                            .map(|event| (serde_json::to_string(&event).unwrap(), event))
                            .collect(),
                    ));
                    async { Ok::<(), Infallible>(()) }
                },
                || cancellation_token.is_cancelled(),
            )
            .await
            .unwrap();
    })
}

mod filter {
    use serde::Deserialize;
    use serde_json::Value;
    use thiserror::Error;

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub enum Operator {
        // Numeric operators
        GreaterThan(f64),
        LessThan(f64),
        GreaterOrEqual(f64),
        LessOrEqual(f64),

        // General equality
        Equals(Value),
        NotEqual(Value),

        // String operators
        StartsWith(String),
        EndsWith(String),
        Contains(String),

        // Array operators
        ArrayContains(Value),

        // Object operators
        HasKey(String),

        // Logical operators
        And(Vec<Filter>),
        Or(Vec<Filter>),
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Filter {
        pub path: String,
        pub operator: Operator,
    }

    #[derive(Error, Debug)]
    pub enum FilterError {
        #[error("Path not found: {0}")]
        PathNotFound(String),

        #[error("Type mismatch: expected {expected}, got {got}")]
        TypeMismatch { expected: String, got: String },

        #[error("Invalid array index in path: {0}")]
        InvalidArrayIndex(String),

        #[error("Invalid path format: {0}")]
        InvalidPath(String),
    }

    impl Filter {
        pub fn new(path: impl Into<String>, operator: Operator) -> Self {
            Self {
                path: path.into(),
                operator,
            }
        }

        pub fn check(&self, value: &Value) -> Result<bool, FilterError> {
            let target = self.resolve_path(value)?;
            self.check_operator(target)
        }

        fn resolve_path<'a>(&self, value: &'a Value) -> Result<&'a Value, FilterError> {
            let mut current = value;

            if self.path == "." {
                return Ok(current);
            }

            for segment in self.path.split('.') {
                if segment.contains('[') && segment.ends_with(']') {
                    let (field, index) = self.parse_array_segment(segment)?;

                    if !field.is_empty() {
                        current = current
                            .get(&field)
                            .ok_or_else(|| FilterError::PathNotFound(field.to_string()))?;
                    }

                    current = match current {
                        Value::Array(arr) => arr
                            .get(index)
                            .ok_or_else(|| FilterError::InvalidArrayIndex(index.to_string()))?,
                        _ => {
                            return Err(FilterError::TypeMismatch {
                                expected: "array".to_string(),
                                got: format!("{:?}", current),
                            })
                        }
                    };
                } else {
                    current = current
                        .get(segment)
                        .ok_or_else(|| FilterError::PathNotFound(segment.to_string()))?;
                }
            }

            Ok(current)
        }

        fn parse_array_segment(&self, segment: &str) -> Result<(String, usize), FilterError> {
            let bracket_idx = segment
                .find('[')
                .ok_or_else(|| FilterError::InvalidPath(segment.to_string()))?;

            let field = segment[..bracket_idx].to_string();
            let index_str = &segment[bracket_idx + 1..segment.len() - 1];

            let index = index_str
                .parse::<usize>()
                .map_err(|_| FilterError::InvalidArrayIndex(index_str.to_string()))?;

            Ok((field, index))
        }

        fn check_operator(&self, value: &Value) -> Result<bool, FilterError> {
            match &self.operator {
                Operator::GreaterThan(n) => {
                    if let Value::Number(num) = value {
                        Ok(num.as_f64().unwrap() > *n)
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "number".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::LessThan(n) => {
                    if let Value::Number(num) = value {
                        Ok(num.as_f64().unwrap() < *n)
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "number".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::GreaterOrEqual(n) => {
                    if let Value::Number(num) = value {
                        Ok(num.as_f64().unwrap() >= *n)
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "number".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::LessOrEqual(n) => {
                    if let Value::Number(num) = value {
                        Ok(num.as_f64().unwrap() <= *n)
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "number".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::Equals(target) => Ok(value == target),

                Operator::NotEqual(target) => Ok(value != target),

                Operator::StartsWith(s) => {
                    if let Value::String(str) = value {
                        Ok(str.starts_with(s))
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "string".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::EndsWith(s) => {
                    if let Value::String(str) = value {
                        Ok(str.ends_with(s))
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "string".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::Contains(s) => {
                    if let Value::String(str) = value {
                        Ok(str.contains(s))
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "string".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::ArrayContains(target) => {
                    if let Value::Array(arr) = value {
                        Ok(arr.contains(target))
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "array".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::HasKey(key) => {
                    if let Value::Object(obj) = value {
                        Ok(obj.contains_key(key))
                    } else {
                        Err(FilterError::TypeMismatch {
                            expected: "object".to_string(),
                            got: format!("{:?}", value),
                        })
                    }
                }

                Operator::And(filters) => {
                    let mut results = Vec::new();
                    for filter in filters {
                        results.push(filter.check(value)?);
                    }
                    Ok(results.iter().all(|&x| x))
                }

                Operator::Or(filters) => {
                    let mut results = Vec::new();
                    for filter in filters {
                        results.push(filter.check(value)?);
                    }
                    Ok(results.iter().any(|&x| x))
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn test_numeric_operators() {
            let value = json!({ "age": 25 });

            let filter = Filter::new("age", Operator::GreaterThan(20.0));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new("age", Operator::LessThan(30.0));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new("age", Operator::GreaterOrEqual(25.0));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new("age", Operator::LessOrEqual(25.0));
            assert!(filter.check(&value).unwrap());
        }

        #[test]
        fn test_string_operators() {
            let value = json!({ "name": "John Doe" });

            let filter = Filter::new("name", Operator::StartsWith("John".to_string()));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new("name", Operator::EndsWith("Doe".to_string()));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new("name", Operator::Contains("hn D".to_string()));
            assert!(filter.check(&value).unwrap());
        }

        #[test]
        fn test_array_operators() {
            let value = json!({ "tags": ["rust", "coding", "json"] });

            let filter = Filter::new("tags", Operator::ArrayContains(json!("rust")));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new("tags[1]", Operator::Equals(json!("coding")));
            assert!(filter.check(&value).unwrap());
        }

        #[test]
        fn test_object_operators() {
            let value = json!({
                "user": {
                    "id": 123,
                    "details": {
                        "email": "john@example.com"
                    }
                }
            });

            let filter = Filter::new("user", Operator::HasKey("id".to_string()));
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new(
                "user.details.email",
                Operator::EndsWith("@example.com".to_string()),
            );
            assert!(filter.check(&value).unwrap());
        }

        #[test]
        fn test_logical_operators() {
            let value = json!({
                "age": 25,
                "name": "John Doe"
            });

            let filter = Filter::new(
                ".",
                Operator::And(vec![
                    Filter::new("age", Operator::GreaterThan(20.0)),
                    Filter::new("name", Operator::StartsWith("John".to_string())),
                ]),
            );
            assert!(filter.check(&value).unwrap());

            let filter = Filter::new(
                ".",
                Operator::Or(vec![
                    Filter::new("age", Operator::GreaterThan(30.0)),
                    Filter::new("name", Operator::Contains("John".to_string())),
                ]),
            );
            assert!(filter.check(&value).unwrap());
        }

        #[test]
        fn test_type_mismatch() {
            let value = json!({ "age": "25" }); // age is a string, not a number

            let filter = Filter::new("age", Operator::GreaterThan(20.0));
            assert!(matches!(
                filter.check(&value),
                Err(FilterError::TypeMismatch { .. })
            ));
        }

        #[test]
        fn test_path_not_found() {
            let value = json!({ "name": "John" });

            let filter = Filter::new("age", Operator::GreaterThan(20.0));
            assert!(matches!(
                filter.check(&value),
                Err(FilterError::PathNotFound(..))
            ));
        }
    }
}
