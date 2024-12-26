# inevents

`inevents` is a tool for building APIs for blockchain events.

## Features

- Redis streams: in [redis/](./redis/) there is a crate that you can use to `emit_event`, `read_event` for a single event, and `start_reading_events` for a continuous stream of events with continuation on interruption.
- Redis-to-database: Transform a Redis stream of events into a persistent PostgreSQL-like table.
- HTTP API: `GET /query/<event_id>?start_block_timestamp_nanosec=XXXXXXXX&blocks=XX&<custom_filters>`. Store events in a PostgreSQL-like database, define your own schema, insert/select queries, custom filters, custom endpoints (soon). Check `/query/swagger-ui/` for more documentation on endpoints, or check the example [intear-events](./intear-events/) (note that it doesn't actually emit events, you would need [all-indexers](https://github.com/INTEARnear/all-indexers) to actually get the data).
- WebSocket API: `/events/<event_id>`, optionally send a JSON message containing the custom filter object.

## Getting started

Just add your own `Event` and run one of the modules (HttpServer, WebsocketServer, RedisToPostgres) or build your own one. Check the examples in [intear-events/](./intear-events/), they're used on `events.intear.tech` and are always up to date.
