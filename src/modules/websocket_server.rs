// Actix needs a single-threaded context, but we don't want that, so we're running 2 tokio runtimes in parallel.

use std::convert::Infallible;
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    sync::Arc,
    time::{Duration, Instant},
};

use super::{EventCollection, EventModule, FilterFn, RawEvent};
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
use actix_web_actors::ws::{self, WsResponseBuilder};
use async_trait::async_trait;
use dashmap::DashSet;
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
    async fn start<E: EventCollection>(self) -> anyhow::Result<()> {
        let cancellation_token = CancellationToken::new();
        let server_addr = Arc::new(OnceCell::new());

        let server_addr_clone = Arc::clone(&server_addr);
        let cancellation_token_clone = cancellation_token.clone();
        tokio::task::spawn_blocking(|| {
            actix::run(async move {
                let server = Server {
                    sockets: HashMap::from_iter(
                        E::events()
                            .into_iter()
                            .map(|event| {
                                ((event.event_identifier, false), Arc::new(DashSet::new()))
                            })
                            .chain(
                                E::events()
                                    .into_iter()
                                    .filter(|event| event.supports_testnet)
                                    .map(|event| {
                                        ((event.event_identifier, true), Arc::new(DashSet::new()))
                                    }),
                            ),
                    ),
                };
                server_addr_clone.set(server.start()).unwrap();
                while !cancellation_token_clone.is_cancelled() {
                    tokio::task::yield_now().await;
                }
            })
            .expect("Failed to run Websocket server")
        });

        while server_addr.get().is_none() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let server_addr = server_addr.get().unwrap().clone();

        let mut join_handles = Vec::new();
        for event in E::events() {
            join_handles.push(launch_event_stream(
                event,
                server_addr.clone(),
                cancellation_token.clone(),
                false,
            ));
        }
        for event in E::events()
            .into_iter()
            .filter(|event| event.supports_testnet)
        {
            join_handles.push(launch_event_stream(
                event,
                server_addr.clone(),
                cancellation_token.clone(),
                true,
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
            for event in E::events() {
                api = api.service(create_route(event, false));
            }

            let mut api_testnet = web::scope("/events-testnet");
            for event in E::events() {
                if event.supports_testnet {
                    api_testnet = api_testnet.service(create_route(event, true));
                }
            }

            let mut app = App::new()
                .app_data(web::Data::new(server_addr.clone()))
                .service(api)
                .service(api_testnet);

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
    sockets: HashMap<(&'static str, bool), Arc<DashSet<Addr<EventWebSocket>>>>,
}

impl Handler<UnsubscribeFromEvents> for Server {
    type Result = ();

    fn handle(&mut self, msg: UnsubscribeFromEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.sockets[&(msg.1, msg.2)].remove(&msg.0);
    }
}

impl Handler<SubscribeToEvents> for Server {
    type Result = ();

    fn handle(&mut self, msg: SubscribeToEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.sockets[&(msg.1, msg.2)].insert(msg.0);
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct UntypedEvent(&'static str, String, serde_json::Value, bool);

pub struct EventWebSocket {
    last_heartbeat: Instant,
    filter_constructor: fn(&str) -> Result<FilterFn, anyhow::Error>,
    filter: Option<FilterFn>,
    server: Addr<Server>,
    event_identifier: &'static str,
    testnet: bool,
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
            self.event_identifier,
            self.testnet,
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

impl Handler<UntypedEvent> for Server {
    type Result = ();

    fn handle(&mut self, msg: UntypedEvent, _ctx: &mut Self::Context) -> Self::Result {
        let msg = Arc::new(msg);
        for socket in self.sockets.get(&(msg.0, msg.3)).unwrap().iter() {
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
                if let Ok(filter) = (self.filter_constructor)(&text) {
                    self.filter = Some(filter);
                }
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<Arc<UntypedEvent>> for EventWebSocket {
    type Result = ();

    fn handle(&mut self, msg: Arc<UntypedEvent>, ctx: &mut Self::Context) -> Self::Result {
        if self.filter.is_none() || !(self.filter.as_ref().unwrap())(&msg.2) {
            return;
        }

        ctx.text(msg.1.as_str());
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct SubscribeToEvents(Addr<EventWebSocket>, &'static str, bool);

#[derive(Message)]
#[rtype(result = "()")]
struct UnsubscribeFromEvents(Addr<EventWebSocket>, &'static str, bool);

async fn create_connection(url: &str) -> ConnectionManager {
    let client = redis::Client::open(url).expect("Failed to connect to Redis");
    ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}

fn create_route(event: RawEvent, testnet: bool) -> impl HttpServiceFactory {
    web::resource(event.event_identifier).route(web::get().to(
        move |req: HttpRequest, stream: web::Payload, server: web::Data<Addr<Server>>| async move {
            let (addr, res) = WsResponseBuilder::new(
                EventWebSocket {
                    last_heartbeat: Instant::now(),
                    filter_constructor: event.realtime_filter_constructor,
                    filter: None,
                    server: server.get_ref().clone(),
                    event_identifier: event.event_identifier,
                    testnet,
                },
                &req,
                stream,
            )
            .start_with_addr()?;
            server
                .send(SubscribeToEvents(addr, event.event_identifier, testnet))
                .await
                .unwrap();
            Result::<HttpResponse, actix_web::Error>::Ok(res)
        },
    ))
}

fn launch_event_stream(
    raw_event: RawEvent,
    server: Addr<Server>,
    cancellation_token: CancellationToken,
    testnet: bool,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut stream = RedisEventStream::new(
            create_connection(
                &std::env::var("REDIS_URL").expect("REDIS_URL enviroment variable not set"),
            )
            .await,
            if testnet {
                format!("{}_testnet", raw_event.event_identifier)
            } else {
                raw_event.event_identifier.to_string()
            },
        );
        stream
            .start_reading_events(
                "websocket",
                |event: serde_json::Value| {
                    server.do_send(UntypedEvent(
                        raw_event.event_identifier,
                        serde_json::to_string(&event).unwrap(),
                        event,
                        testnet,
                    ));
                    async { Ok::<(), Infallible>(()) }
                },
                || cancellation_token.is_cancelled(),
            )
            .await
            .unwrap();
    })
}
